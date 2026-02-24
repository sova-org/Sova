use crate::message::ServerMessage;
use serde::{Deserialize, Serialize};
use socket2::SockRef;
use sova_core::log_eprintln;
use sova_core::protocol::DeviceInfo;
use sova_core::scene::{ExecutionMode, Frame, Line, Scene};
use sova_core::schedule::ActionTiming;
use sova_core::schedule::SchedulerMessage;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

pub(crate) const COMPRESSION_MIN_SIZE: usize = 64;
pub(crate) const COMPRESSION_ADAPTIVE_THRESHOLD: usize = 256;
pub(crate) const HIGH_COMPRESSION_CUTOFF: usize = 1024;
pub(crate) const COMPRESSION_FLAG: u32 = 0x80000000;
pub(crate) const LENGTH_MASK: u32 = 0x7FFFFFFF;

#[derive(Debug, Clone, Copy)]
pub enum CompressionStrategy {
    Never,
    Always,
    Adaptive,
}

impl CompressionStrategy {
    pub fn compress(&self, msgpack_bytes: &[u8]) -> io::Result<(Vec<u8>, bool)> {
        match self {
            CompressionStrategy::Never => Ok((msgpack_bytes.to_vec(), false)),
            CompressionStrategy::Always => {
                if msgpack_bytes.len() > COMPRESSION_MIN_SIZE {
                    let level = if msgpack_bytes.len() < HIGH_COMPRESSION_CUTOFF {
                        1
                    } else {
                        3
                    };
                    let compressed = zstd::encode_all(msgpack_bytes, level)
                        .map_err(|e| io::Error::other(format!("Compression failed: {}", e)))?;
                    if compressed.len() < msgpack_bytes.len() {
                        Ok((compressed, true))
                    } else {
                        Ok((msgpack_bytes.to_vec(), false))
                    }
                } else {
                    Ok((msgpack_bytes.to_vec(), false))
                }
            }
            CompressionStrategy::Adaptive => {
                if msgpack_bytes.len() < COMPRESSION_ADAPTIVE_THRESHOLD {
                    Ok((msgpack_bytes.to_vec(), false))
                } else {
                    let level = if msgpack_bytes.len() < HIGH_COMPRESSION_CUTOFF {
                        1
                    } else {
                        3
                    };
                    let compressed = zstd::encode_all(msgpack_bytes, level)
                        .map_err(|e| io::Error::other(format!("Compression failed: {}", e)))?;
                    Ok((compressed, true))
                }
            }
        }
    }
}

pub fn serialize_to_wire_frame(msg: &ServerMessage) -> io::Result<Vec<u8>> {
    let msgpack_bytes = rmp_serde::to_vec_named(msg).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize ServerMessage: {}", e),
        )
    })?;
    let (payload, is_compressed) = msg.compression_strategy().compress(&msgpack_bytes)?;
    let mut length = payload.len() as u32;
    if is_compressed {
        length |= COMPRESSION_FLAG;
    }
    let mut frame = Vec::with_capacity(4 + payload.len());
    frame.extend_from_slice(&length.to_be_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    SchedulerControl(SchedulerMessage),
    SetTempo(f64, ActionTiming),
    SetName(String),
    GetScene,
    SetScene(Scene, ActionTiming),
    GetLine(usize),
    SetLines(Vec<(usize, Line)>, ActionTiming),
    ConfigureLines(Vec<(usize, Line)>, ActionTiming),
    AddLine(usize, Line, ActionTiming),
    RemoveLine(usize, ActionTiming),
    GetFrame(usize, usize),
    SetFrames(Vec<(usize, usize, Frame)>, ActionTiming),
    AddFrame(usize, usize, Frame, ActionTiming),
    RemoveFrame(usize, usize, ActionTiming),
    GetClock,
    GetPeers,
    Chat(String),
    GetSnapshot,
    StartedEditingFrame(usize, usize),
    StoppedEditingFrame(usize, usize),
    CursorPosition(usize, usize),
    TransportStart(ActionTiming),
    TransportStop(ActionTiming),
    SetSceneMode(ExecutionMode, ActionTiming),
    RequestDeviceList,
    ConnectMidiDeviceByName(String),
    DisconnectMidiDeviceByName(String),
    CreateVirtualMidiOutput(String),
    AssignDeviceToSlot(usize, String),
    UnassignDeviceFromSlot(usize),
    CreateOscDevice(String, String, u16),
    RemoveOscDevice(String),
    RestoreDevices(Vec<DeviceInfo>),
    GetAudioEngineState,
    RestartAudioEngine {
        device: Option<String>,
        input_device: Option<String>,
        channels: u16,
        buffer_size: Option<u32>,
        sample_paths: Vec<String>,
    },
    PreviewSample {
        folder: String,
        index: usize,
        begin: f64,
    },
    EnableFeedback,
}

impl ClientMessage {
    pub fn compression_strategy(&self) -> CompressionStrategy {
        match self {
            ClientMessage::StartedEditingFrame(_, _)
            | ClientMessage::StoppedEditingFrame(_, _)
            | ClientMessage::CursorPosition(_, _)
            | ClientMessage::GetClock
            | ClientMessage::GetPeers
            | ClientMessage::GetScene
            | ClientMessage::GetSnapshot
            | ClientMessage::RequestDeviceList
            | ClientMessage::GetAudioEngineState
            | ClientMessage::RestartAudioEngine { .. }
            | ClientMessage::PreviewSample { .. }
            | ClientMessage::EnableFeedback => CompressionStrategy::Never,

            ClientMessage::SetScene(_, _) | ClientMessage::SetLines(_, _) => {
                CompressionStrategy::Always
            }

            _ => CompressionStrategy::Adaptive,
        }
    }

    pub fn deserialize(final_bytes: &[u8]) -> io::Result<Option<Self>> {
        match rmp_serde::from_slice::<ClientMessage>(final_bytes) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("MessagePack deserialization error: {}", e),
            )),
        }
    }
}

pub struct SovaClient {
    pub ip: String,
    pub port: u16,
    reader: Option<BufReader<OwnedReadHalf>>,
    writer: Option<BufWriter<OwnedWriteHalf>>,
    pub connected: bool,
}

impl SovaClient {
    pub fn new(ip: String, port: u16) -> Self {
        SovaClient {
            ip,
            port,
            reader: None,
            writer: None,
            connected: false,
        }
    }

    pub async fn connect(&mut self) -> io::Result<()> {
        let addr = format!("{}:{}", self.ip, self.port);
        let stream = TcpStream::connect(&addr).await?;
        stream.set_nodelay(true)?;
        let keepalive = socket2::TcpKeepalive::new()
            .with_time(std::time::Duration::from_secs(60))
            .with_interval(std::time::Duration::from_secs(10));
        let _ = SockRef::from(&stream).set_tcp_keepalive(&keepalive);
        let (read_half, write_half) = stream.into_split();
        self.reader = Some(BufReader::with_capacity(32 * 1024, read_half));
        self.writer = Some(BufWriter::with_capacity(32 * 1024, write_half));
        self.connected = true;
        Ok(())
    }

    pub async fn send(&mut self, message: ClientMessage) -> io::Result<()> {
        let msgpack_bytes = rmp_serde::to_vec_named(&message).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize ClientMessage to MessagePack: {}", e),
            )
        })?;

        let (final_bytes, is_compressed) =
            message.compression_strategy().compress(&msgpack_bytes)?;

        let mut length = final_bytes.len() as u32;
        if is_compressed {
            length |= COMPRESSION_FLAG;
        }

        let writer = self.writer.as_mut().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotConnected, "Client not connected")
        })?;

        if let Err(e) = writer.write_all(&length.to_be_bytes()).await {
            self.connected = false;
            return Err(e);
        }

        if let Err(e) = writer.write_all(&final_bytes).await {
            self.connected = false;
            return Err(e);
        }

        if let Err(e) = writer.flush().await {
            self.connected = false;
            return Err(e);
        }

        Ok(())
    }

    pub async fn disconnect(&mut self) -> io::Result<()> {
        self.connected = false;
        if let Some(mut writer) = self.writer.take() {
            let _ = writer.shutdown().await;
        }
        self.reader.take();
        Ok(())
    }

    /// Read the next server message from the TCP stream.
    ///
    /// Uses the dedicated read half, so this is safe to race against
    /// sends in a `tokio::select!` — reads and writes are independent.
    pub async fn read(&mut self) -> io::Result<Option<ServerMessage>> {
        if !self.connected {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client not connected",
            ));
        }
        let reader = self.reader.as_mut().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotConnected, "Client not connected")
        })?;

        let mut len_buf = [0u8; 4];
        if let Err(e) = reader.read_exact(&mut len_buf).await {
            self.connected = false;
            return Err(e);
        }

        let len_with_flag = u32::from_be_bytes(len_buf);
        let is_compressed = (len_with_flag & COMPRESSION_FLAG) != 0;
        let length = len_with_flag & LENGTH_MASK;

        if length == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Received zero-length message",
            ));
        }

        let mut message_buf = vec![0u8; length as usize];
        if let Err(e) = reader.read_exact(&mut message_buf).await {
            self.connected = false;
            return Err(e);
        }

        let final_bytes = if is_compressed {
            match zstd::decode_all(message_buf.as_slice()) {
                Ok(bytes) => bytes,
                Err(e) => {
                    log_eprintln!(
                        "Zstd decompression failed (len={}, first 32 bytes: {:02x?}): {}",
                        message_buf.len(),
                        &message_buf[..message_buf.len().min(32)],
                        e
                    );
                    return Ok(None);
                }
            }
        } else {
            message_buf
        };

        match rmp_serde::from_slice::<ServerMessage>(&final_bytes) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => {
                log_eprintln!(
                    "MessagePack deserialization failed (compressed={}, payload_len={}, first 32 bytes: {:02x?}): {}",
                    is_compressed,
                    final_bytes.len(),
                    &final_bytes[..final_bytes.len().min(32)],
                    e
                );
                Ok(None)
            }
        }
    }
}
