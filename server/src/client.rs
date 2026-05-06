use crate::AudioRestartConfig;
use crate::message::ServerMessage;
use serde::{Deserialize, Serialize};
use socket2::SockRef;
use sova_core::protocol::DeviceInfo;
use sova_core::schedule::ActionTiming;
use sova_core::schedule::SchedulerMessage;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub const PROTOCOL_VERSION: u8 = 0x04;
pub const MAX_MESSAGE_SIZE: u32 = 10 * 1024 * 1024;

pub fn serialize_to_wire_frame(msg: &ServerMessage) -> io::Result<Vec<u8>> {
    let payload = rmp_serde::to_vec_named(msg).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize ServerMessage: {}", e),
        )
    })?;
    Ok(build_frame_raw(&payload))
}

fn build_frame_raw(payload: &[u8]) -> Vec<u8> {
    let crc = crc32fast::hash(payload);
    let len_bytes = (payload.len() as u32).to_be_bytes();
    let mut frame = Vec::with_capacity(8 + payload.len());
    frame.push(PROTOCOL_VERSION);
    frame.extend_from_slice(&len_bytes[1..4]);
    frame.extend_from_slice(&crc.to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

fn validate_length(length: u32) -> io::Result<()> {
    if length == 0 || length > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid message length: {} bytes", length),
        ));
    }
    Ok(())
}

/// Reads one wire frame and deserializes a ServerMessage.
pub async fn read_server_message<R: AsyncReadExt + Unpin>(
    reader: &mut R,
) -> io::Result<ServerMessage> {
    let payload = read_wire_frame(reader).await?;
    rmp_serde::from_slice::<ServerMessage>(&payload).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Deserialization failed: {}", e),
        )
    })
}

/// Reads one wire frame, returning CRC-verified payload bytes.
pub async fn read_wire_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut first = [0u8; 1];
    reader.read_exact(&mut first).await?;

    if first[0] != PROTOCOL_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsupported protocol version: 0x{:02x}", first[0]),
        ));
    }

    let mut header = [0u8; 7];
    reader.read_exact(&mut header).await?;
    let length = u32::from_be_bytes([0x00, header[0], header[1], header[2]]);
    let expected_crc = u32::from_be_bytes([header[3], header[4], header[5], header[6]]);
    validate_length(length)?;
    let mut buf = vec![0u8; length as usize];
    reader.read_exact(&mut buf).await?;
    let actual_crc = crc32fast::hash(&buf);
    if actual_crc != expected_crc {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("CRC mismatch (expected 0x{expected_crc:08x}, got 0x{actual_crc:08x})"),
        ));
    }
    Ok(buf)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    SetName {
        name: String,
        password: Option<String>,
    },
    GetScene,
    GetLine(usize),
    GetFrame(usize, usize),
    GetClock,
    GetPeers,
    Chat(String),
    GetSnapshot,
    StartedEditingFrame(usize, usize),
    StoppedEditingFrame(usize, usize),
    RequestDeviceList,
    ConnectMidiDeviceByName(String),
    DisconnectMidiDeviceByName(String),
    CreateVirtualMidiOutput(String),
    AssignDeviceToSlot(usize, String),
    UnassignDeviceFromSlot(usize),
    CreateOscDevice(String, String, u16),
    CreateOscInputDevice(String, u16),
    RemoveOscDevice(String),
    SetDeviceLatency(String, f64),
    RestoreDevices(Vec<DeviceInfo>),
    GetAudioEngineState,
    RestartAudioEngine(AudioRestartConfig),
    PreviewSample {
        folder: String,
        index: usize,
        begin: f64,
    },
    EnableFeedback,
    RestartCore,
    ResetScene(ActionTiming),
    SetMasterVolume(f32),
    Hush,
    Panic,
    ScriptEdit {
        frame_text_id: crate::FrameTextId,
        update: Vec<u8>,
    },
    Presence {
        update: Vec<u8>,
    },
    SetLinkEnabled(bool),
    SetStartStopSync(bool),
    #[serde(untagged)]
    SchedulerControl(SchedulerMessage),
}

impl ClientMessage {
    pub fn deserialize(bytes: &[u8]) -> io::Result<Option<Self>> {
        match rmp_serde::from_slice::<ClientMessage>(bytes) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("MessagePack deserialization error: {}", e),
            )),
        }
    }
}

impl From<SchedulerMessage> for ClientMessage {
    fn from(value: SchedulerMessage) -> Self {
        ClientMessage::SchedulerControl(value)
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
        let payload = rmp_serde::to_vec_named(&message).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize ClientMessage: {}", e),
            )
        })?;

        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Client not connected"))?;

        let frame = build_frame_raw(&payload);

        if let Err(e) = writer.write_all(&frame).await {
            self.connected = false;
            return Err(e);
        }

        if let Err(e) = writer.flush().await {
            self.connected = false;
            return Err(e);
        }

        Ok(())
    }

    pub fn take_reader(&mut self) -> Option<BufReader<OwnedReadHalf>> {
        self.reader.take()
    }

    pub async fn disconnect(&mut self) -> io::Result<()> {
        self.connected = false;
        if let Some(mut writer) = self.writer.take() {
            let _ = writer.shutdown().await;
        }
        self.reader.take();
        Ok(())
    }

    pub async fn read(&mut self) -> io::Result<Option<ServerMessage>> {
        if !self.connected {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client not connected",
            ));
        }
        let reader = self
            .reader
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Client not connected"))?;

        let payload = match read_wire_frame(reader).await {
            Ok(buf) => buf,
            Err(e) => {
                self.connected = false;
                return Err(e);
            }
        };

        match rmp_serde::from_slice::<ServerMessage>(&payload) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Deserialization failed: {}", e),
            )),
        }
    }
}

