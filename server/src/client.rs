use crate::message::ServerMessage;
use sova_core::log_eprintln;
use sova_core::protocol::DeviceInfo;
use sova_core::scene::{Frame, Line, Scene};
use sova_core::schedule::ActionTiming;
use sova_core::schedule::SchedulerMessage;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug, Clone, Copy)]
pub enum CompressionStrategy {
    Never,
    Always,
    Adaptive,
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
    TransportStart(ActionTiming),
    TransportStop(ActionTiming),
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
}

impl ClientMessage {
    pub fn compression_strategy(&self) -> CompressionStrategy {
        match self {
            ClientMessage::StartedEditingFrame(_, _)
            | ClientMessage::StoppedEditingFrame(_, _)
            | ClientMessage::GetClock
            | ClientMessage::GetPeers
            | ClientMessage::GetScene
            | ClientMessage::GetSnapshot
            | ClientMessage::RequestDeviceList
            | ClientMessage::GetAudioEngineState => CompressionStrategy::Never,

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

struct BufferPool {
    small_buffers: Vec<Vec<u8>>,
    large_buffers: Vec<Vec<u8>>,
}

impl BufferPool {
    fn new() -> Self {
        BufferPool {
            small_buffers: Vec::new(),
            large_buffers: Vec::new(),
        }
    }

    fn get_buffer(&mut self, size: usize) -> Vec<u8> {
        if size < 1024 {
            self.small_buffers
                .pop()
                .map(|mut buf| {
                    buf.clear();
                    buf.reserve(size);
                    buf
                })
                .unwrap_or_else(|| Vec::with_capacity(size.max(512)))
        } else {
            self.large_buffers
                .pop()
                .map(|mut buf| {
                    buf.clear();
                    buf.reserve(size);
                    buf
                })
                .unwrap_or_else(|| Vec::with_capacity(size.max(2048)))
        }
    }

    #[allow(dead_code)]
    fn return_buffer(&mut self, mut buffer: Vec<u8>) {
        if buffer.capacity() < 1024 && self.small_buffers.len() < 8 {
            buffer.clear();
            self.small_buffers.push(buffer);
        } else if buffer.capacity() >= 1024 && self.large_buffers.len() < 4 {
            buffer.clear();
            self.large_buffers.push(buffer);
        }
    }
}

pub struct SovaClient {
    pub ip: String,
    pub port: u16,
    pub stream: Option<TcpStream>,
    pub connected: bool,
    buffer_pool: BufferPool,
}

impl SovaClient {
    pub fn new(ip: String, port: u16) -> Self {
        SovaClient {
            ip,
            port,
            stream: None,
            connected: false,
            buffer_pool: BufferPool::new(),
        }
    }

    pub async fn connect(&mut self) -> io::Result<()> {
        let addr = format!("{}:{}", self.ip, self.port);
        let stream = TcpStream::connect(&addr).await?;
        stream.set_nodelay(true)?;
        self.stream = Some(stream);
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

        let (final_bytes, is_compressed) = self.compress_intelligently(&message, &msgpack_bytes)?;

        let mut length = final_bytes.len() as u32;
        if is_compressed {
            length |= 0x80000000;
        }

        let socket = self.mut_socket()?;

        if let Err(e) = socket.write_all(&length.to_be_bytes()).await {
            self.connected = false;
            return Err(e);
        }

        if let Err(e) = socket.write_all(&final_bytes).await {
            self.connected = false;
            return Err(e);
        }

        Ok(())
    }

    fn compress_intelligently(
        &mut self,
        message: &ClientMessage,
        msgpack_bytes: &[u8],
    ) -> io::Result<(Vec<u8>, bool)> {
        match message.compression_strategy() {
            CompressionStrategy::Never => {
                let mut buffer = self.buffer_pool.get_buffer(msgpack_bytes.len());
                buffer.extend_from_slice(msgpack_bytes);
                Ok((buffer, false))
            }
            CompressionStrategy::Always => {
                if msgpack_bytes.len() > 64 {
                    let compression_level = if msgpack_bytes.len() < 1024 { 1 } else { 3 };
                    let compressed = zstd::encode_all(msgpack_bytes, compression_level)
                        .map_err(|e| io::Error::other(format!("Compression failed: {}", e)))?;
                    if compressed.len() < msgpack_bytes.len() {
                        Ok((compressed, true))
                    } else {
                        let mut buffer = self.buffer_pool.get_buffer(msgpack_bytes.len());
                        buffer.extend_from_slice(msgpack_bytes);
                        Ok((buffer, false))
                    }
                } else {
                    let mut buffer = self.buffer_pool.get_buffer(msgpack_bytes.len());
                    buffer.extend_from_slice(msgpack_bytes);
                    Ok((buffer, false))
                }
            }
            CompressionStrategy::Adaptive => {
                if msgpack_bytes.len() < 256 {
                    let mut buffer = self.buffer_pool.get_buffer(msgpack_bytes.len());
                    buffer.extend_from_slice(msgpack_bytes);
                    Ok((buffer, false))
                } else {
                    let compression_level = if msgpack_bytes.len() < 1024 { 1 } else { 3 };
                    let compressed = zstd::encode_all(msgpack_bytes, compression_level)
                        .map_err(|e| io::Error::other(format!("Compression failed: {}", e)))?;
                    Ok((compressed, true))
                }
            }
        }
    }

    pub fn mut_socket(&mut self) -> io::Result<&mut TcpStream> {
        match &mut self.stream {
            Some(x) => Ok(x),
            None => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client not connected",
            )),
        }
    }

    pub fn socket(&self) -> io::Result<&TcpStream> {
        match &self.stream {
            Some(x) => Ok(x),
            None => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client not connected",
            )),
        }
    }

    pub async fn ready(&mut self) -> bool {
        let mut buf = [0];
        let Ok(socket) = self.socket() else {
            return false;
        };
        match socket.peek(&mut buf).await {
            Ok(0) => {
                self.connected = false;
                false
            }
            Ok(_) => true,
            Err(_) => {
                self.connected = false;
                false
            }
        }
    }

    pub async fn disconnect(&mut self) -> io::Result<()> {
        self.connected = false;
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
        }
        Ok(())
    }

    pub async fn read(&mut self) -> io::Result<ServerMessage> {
        if !self.connected {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client not connected",
            ));
        }
        let socket = self.mut_socket()?;

        let mut len_buf = [0u8; 4];
        if let Err(e) = socket.read_exact(&mut len_buf).await {
            self.connected = false;
            return Err(e);
        }

        let len_with_flag = u32::from_be_bytes(len_buf);
        let is_compressed = (len_with_flag & 0x80000000) != 0;
        let length = len_with_flag & 0x7FFFFFFF;

        if length == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Received zero-length message",
            ));
        }

        let mut message_buf = vec![0u8; length as usize];
        if let Err(e) = socket.read_exact(&mut message_buf).await {
            self.connected = false;
            return Err(e);
        }

        let final_bytes = if is_compressed {
            zstd::decode_all(message_buf.as_slice()).map_err(|e| {
                log_eprintln!("[!] Failed to decompress Zstd data from server: {}", e);
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Zstd decompression failed: {}", e),
                )
            })?
        } else {
            message_buf
        };

        rmp_serde::from_slice::<ServerMessage>(&final_bytes).map_err(|e| {
            log_eprintln!("[!] Failed to deserialize MessagePack from server: {}", e);
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("MessagePack deserialization failed: {}", e),
            )
        })
    }
}
