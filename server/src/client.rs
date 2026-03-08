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

pub fn serialize_to_wire_frame(msg: &ServerMessage) -> io::Result<Vec<u8>> {
    let payload = postcard::to_allocvec(msg).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize ServerMessage: {}", e),
        )
    })?;
    let length = payload.len() as u32;
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
    pub fn deserialize(bytes: &[u8]) -> io::Result<Option<Self>> {
        match postcard::from_bytes::<ClientMessage>(bytes) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Postcard deserialization error: {}", e),
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
        let payload = postcard::to_allocvec(&message).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize ClientMessage: {}", e),
            )
        })?;

        let length = payload.len() as u32;

        let writer = self.writer.as_mut().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotConnected, "Client not connected")
        })?;

        if let Err(e) = writer.write_all(&length.to_be_bytes()).await {
            self.connected = false;
            return Err(e);
        }

        if let Err(e) = writer.write_all(&payload).await {
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

        let length = u32::from_be_bytes(len_buf);

        if length == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Received zero-length message",
            ));
        }

        const MAX_MESSAGE_SIZE: u32 = 10 * 1024 * 1024;
        if length > MAX_MESSAGE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Message too large: {} bytes", length),
            ));
        }

        let mut message_buf = vec![0u8; length as usize];
        if let Err(e) = reader.read_exact(&mut message_buf).await {
            self.connected = false;
            return Err(e);
        }

        match postcard::from_bytes::<ServerMessage>(&message_buf) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => {
                log_eprintln!(
                    "Deserialization failed (payload_len={}, first 64 bytes: {:02x?}): {}",
                    message_buf.len(),
                    &message_buf[..message_buf.len().min(64)],
                    e
                );
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use sova_core::{
        clock::TimeSpan,
        scene::{Frame, Line, Scene, script::Script},
        schedule::ActionTiming,
        vm::variable::{VariableStore, VariableValue},
    };

    fn roundtrip(msg: &ClientMessage) {
        let bytes = postcard::to_allocvec(msg)
            .unwrap_or_else(|e| panic!("serialize failed for {:?}: {e}", std::mem::discriminant(msg)));
        postcard::from_bytes::<ClientMessage>(&bytes)
            .unwrap_or_else(|e| {
                panic!(
                    "deserialize failed for {:?} (len={}, first 32 bytes: {:02x?}): {e}",
                    std::mem::discriminant(msg),
                    bytes.len(),
                    &bytes[..bytes.len().min(32)],
                )
            });
    }

    fn frame_with_vars() -> Frame {
        let mut f = Frame::default();
        f.vars = VariableStore::from(HashMap::from([
            ("i".into(), VariableValue::Integer(10)),
            ("f".into(), VariableValue::Float(0.5)),
            ("d".into(), VariableValue::Dur(TimeSpan::Beats(2.0))),
            ("m".into(), VariableValue::Map(HashMap::from([
                ("k".into(), VariableValue::Str("v".into())),
            ]))),
            ("v".into(), VariableValue::Vec(vec![
                VariableValue::Bool(true),
                VariableValue::Integer(99),
            ])),
        ]));
        f.set_script(Script::new("note 60".into(), "bob".into()));
        f
    }

    #[test]
    fn client_message_roundtrip_with_variable_values() {
        let frame = frame_with_vars();
        let mut scene = Scene::default();
        scene.lines.push(Line::default());
        scene.lines[0].frames.push(frame.clone());

        let variants: Vec<ClientMessage> = vec![
            ClientMessage::SetScene(scene, ActionTiming::Immediate),
            ClientMessage::SetFrames(
                vec![(0, 0, frame.clone())],
                ActionTiming::AtNextBeat,
            ),
            ClientMessage::AddFrame(0, 1, frame, ActionTiming::Immediate),
        ];

        for msg in &variants {
            roundtrip(msg);
        }
    }
}
