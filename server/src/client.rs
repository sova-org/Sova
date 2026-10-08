use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::ErrorKind;
use std::sync::Arc;
use std::time::Duration;

use crate::message::ServerMessage;
use crate::server::apply_notification;
use doux_sova::AudioEngineState;
use socket2::SockRef;
use sova_core::LogMessage;
use sova_core::Scene;
use sova_core::clock::ClockSnapshot;
use sova_core::clock::LinkState;
use sova_core::error::SovaError;
use sova_core::protocol::DeviceInfo;
use sova_core::schedule::ActionTiming;
use sova_core::schedule::SchedulerMessage;
use sova_core::schedule::SovaNotification;
use sova_core::schedule::playback::PlaybackState;
use sova_core::vm::interpreter::Annotation;
use sova_core::vm::language::LanguageDefinition;
use sova_core::vm::variable::VariableValue;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

mod message;
pub use message::*;
use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

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

#[derive(Default, Clone)]
pub struct ClientState {
    pub connected: bool,
    pub peer_id: u64,
    pub playback_state: PlaybackState,
    pub scene: Scene,
    pub positions: Vec<Vec<(usize, usize)>>,
    pub devices: Vec<DeviceInfo>,
    pub clock: ClockSnapshot,
    pub audio_state: AudioEngineState,
    pub scope_data: Vec<f32>,
    pub scope_generation: u64,
    pub peak_data: Vec<f32>,
    pub peers: Vec<String>,
    pub languages: Vec<LanguageDefinition>,
    pub peer_editing: HashMap<(usize, usize), Vec<String>>,
    pub chat_messages: VecDeque<(String, String)>,
    pub errors: HashMap<(usize, usize), SovaError>,
    pub annotations: Vec<Vec<Vec<Annotation>>>,
    pub global_vars: HashMap<String, VariableValue>,
    pub logs: VecDeque<LogMessage>,
    pub server_errors: VecDeque<String>,
    pub link_state: LinkState,
}

async fn read_client(
    reader: &mut BufReader<OwnedReadHalf>, 
    state: &Mutex<ClientState>
) -> io::Result<ServerMessage> {
    let payload = match read_wire_frame(reader).await {
        Ok(buf) => buf,
        Err(e) => {
            state.lock().await.connected = false;
            return Err(e);
        }
    };

    match rmp_serde::from_slice::<ServerMessage>(&payload) {
        Ok(msg) => Ok(msg),
        Err(e) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Deserialization failed: {}", e),
        )),
    }
}

#[derive(Default)]
pub struct SovaClient {
    pub ip: String,
    pub port: u16,
    pub name: String,
    pub password: Option<String>,  
    pub feedback: bool,
    reader: Option<BufReader<OwnedReadHalf>>,
    writer: Option<BufWriter<OwnedWriteHalf>>,
    
    cancel_token: CancellationToken,
    pub client_state: Arc<Mutex<ClientState>>,
    task: Option<JoinHandle<io::Result<BufReader<OwnedReadHalf>>>>,
    pub sched_iface: Option<crossbeam_channel::Sender<SchedulerMessage>>
}

impl SovaClient {
    pub fn new(ip: String, port: u16) -> Self {
        SovaClient {
            ip,
            port,
            ..Default::default()
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
        self.login().await?;
        self.state().await.connected = true;
        Ok(())
    }

    pub async fn connect_with_timeout(&mut self, dur: Duration) -> io::Result<()> {
        timeout(dur, self.connect())
            .await
            .unwrap_or(Err(io::Error::new(ErrorKind::TimedOut, "Connection timed out!")))
    }

    pub async fn state(&'_ self) -> MutexGuard<'_, ClientState> {
        self.client_state.lock().await
    }

    pub async fn login(&mut self) -> io::Result<()> {
        let msg = ClientMessage::Login { 
            name: self.name.clone(), 
            password: self.password.clone(), 
        };
        self.send(msg).await?;
        match self.read().await? {
            ServerMessage::Hello {
                username,
                peer_id,
                scene,
                devices,
                peers,
                clock_state,
                is_playing,
                languages,
                audio_engine_state,
                link_state,
                frame_text_layout,
                frame_doc_snapshots,
                presence,
            } => {
                self.name = username;
                let mut state = self.state().await;
                state.peer_id = peer_id;
                state.scene = scene;
                state.devices = devices;
                state.peers = peers;
                state.languages = languages;
                state.audio_state = audio_engine_state;
                state.clock = clock_state;
                state.link_state = link_state;
                state.playback_state = if is_playing {
                    PlaybackState::Playing
                } else {
                    PlaybackState::Stopped
                };
                Ok(())
            }
            ServerMessage::ConnectionRefused(reason) => {
                self.disconnect().await?;
                Err(io::Error::new(ErrorKind::ConnectionRefused, reason))
            }
            other => {
                let kind = format!("{:?}", std::mem::discriminant(&other));
                self.disconnect().await?;
                Err(io::Error::new(ErrorKind::InvalidData, format!(
                    "Unexpected first server message (variant {kind}); expected Hello — \
                        is the server running an older protocol?")))
            }
        }
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
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Unable to send: client not connected"))?;

        let frame = build_frame_raw(&payload);

        if let Err(e) = writer.write_all(&frame).await {
            self.state().await.connected = false;
            return Err(e);
        }

        if let Err(e) = writer.flush().await {
            self.state().await.connected = false;
            return Err(e);
        }

        Ok(())
    }

    pub fn take_reader(&mut self) -> Option<BufReader<OwnedReadHalf>> {
        self.reader.take()
    }

    pub async fn disconnect(&mut self) -> io::Result<()> {
        self.state().await.connected = false;
        if let Some(mut writer) = self.writer.take() {
            let _ = writer.shutdown().await;
        }
        self.reader.take();
        Ok(())
    }

    pub async fn read(&mut self) -> io::Result<ServerMessage> {
        let mut reader = self
            .take_reader()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Unable to read: client not connected"))?;

        let res = read_client(&mut reader, &self.client_state).await;

        self.reader = Some(reader);

        res
    }

    /// Spawns a tokio task waiting for server messages and updating the client state
    /// This method consumes the reader, call `stop_task` in order to restore it.
    pub fn run(&mut self, relay: Option<UnboundedSender<ServerMessage>>) -> io::Result<()> {
        self.cancel_token = CancellationToken::new();
        let child_token = self.cancel_token.child_token();
        let state = self.client_state.clone();
        let mut reader = self.take_reader()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Unable to take reader: client not connected"))?;
        let sched_iface = self.sched_iface.clone();
        let handle = tokio::spawn(async move {
            while state.lock().await.connected {
                select! {
                    _ = child_token.cancelled() => {
                        break;
                    }
                    msg = read_client(&mut reader, &state) => {
                        if let Err(e) = msg {
                            state.lock().await.server_errors.push_back(e.to_string());
                            continue;
                        }
                        let msg = msg.unwrap();
                        let mut guard = state.lock().await;
                        match &relay {
                            Some(relay) => {
                                handle_server_message(&mut guard, msg.clone(), &sched_iface);
                                let _ = relay.send(msg);
                            }
                            None => {
                                handle_server_message(&mut guard, msg, &sched_iface);
                            }
                        }
                    }
                }
            }
            Ok(reader)
        });
        self.task = Some(handle);
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        match &self.task {
            Some(handle) => !handle.is_finished(),
            None => false
        }
    }

    pub fn has_task(&self) -> bool {
        self.task.is_some()
    }

    pub async fn stop_task(&mut self) -> io::Result<()>{
        let task = self.task.take();
        match task {
            Some(handle) => {
                self.cancel_token.cancel();
                let reader = handle.await.unwrap_or(
                    Err(io::Error::new(ErrorKind::Other, "Unable to join client task thread"))
                )?;
                self.reader = Some(reader);
                Ok(())
            }
            None => Ok(()),
        }
    }

}

fn handle_server_message(
    state: &mut ClientState, 
    msg: ServerMessage, 
    sched_iface: &Option<crossbeam_channel::Sender<SchedulerMessage>>
) {
    match msg {
        ServerMessage::Hello { .. } => (),
        ServerMessage::Success => (),
        ServerMessage::PeersUpdated(peers) => {
            state.peers = peers;
        }
        ServerMessage::PeerStartedEditing(_, _, _) => todo!(),
        ServerMessage::PeerStoppedEditing(_, _, _) => todo!(),
        ServerMessage::Chat(from, text) => {
            state.chat_messages.push_back((from, text));
        }
        ServerMessage::InternalError(e) => {
            state.server_errors.push_back(e);
        }
        ServerMessage::ConnectionRefused(e) => {
            state.server_errors.push_back(e);
            state.connected = false;
        }
        ServerMessage::Snapshot(snapshot) => {
            state.scene = snapshot.scene;
            state.clock.micros = snapshot.micros;
            state.clock.beat = snapshot.beat;
            state.clock.quantum = snapshot.quantum;
            state.clock.tempo = snapshot.tempo;
            state.devices = snapshot.devices;
        }
        ServerMessage::ClockState(clock_snapshot) => {
            state.clock = clock_snapshot;
        }
        ServerMessage::DevicesRestored { missing_devices } => todo!(),
        ServerMessage::AudioEngineState(audio_engine_state) => {
            state.audio_state = audio_engine_state
        }
        ServerMessage::ScopeData(items) => todo!(),
        ServerMessage::PeakData(items) => todo!(),
        ServerMessage::FeedbackEnabled => {
            let Some(iface) = sched_iface else {
                state.server_errors.push_back("Unable to enable feedback: no scheduler interface !".to_owned());
                return;
            };
            let _ = iface.send(SchedulerMessage::SetScene(state.scene.clone(), ActionTiming::Immediate));
            let _ = iface.send(SchedulerMessage::SetTempo(state.clock.tempo, ActionTiming::Immediate));
            let _ = iface.send(SchedulerMessage::SetQuantum(state.clock.quantum, ActionTiming::Immediate));
            if state.playback_state.is_playing() {
                let _ = iface.send(SchedulerMessage::TransportStart(ActionTiming::Immediate));
            }
        }
        ServerMessage::Feedback(scheduler_message) => {
            let Some(iface) = sched_iface else {
                state.server_errors.push_back("Unable to relay feedback: no scheduler interface !".to_owned());
                return;
            };
            let _ = iface.send(scheduler_message);
        }
        ServerMessage::CoreRestarted => (),
        ServerMessage::LinkState(link_state) => {
            state.link_state = link_state
        }
        ServerMessage::ScriptEdit { sender, frame_text_id, update } => todo!(),
        ServerMessage::Presence { update } => todo!(),
        ServerMessage::FrameTextLayout { mapping, new_doc_snapshots } => todo!(),
        ServerMessage::Notification(notif) => {
            match notif {
                SovaNotification::Tick => (),
                SovaNotification::CompilationUpdated(i, j, id, compilation_state) => {
                    state.scene.frame_mut(i, j).update_compilation_state(id, compilation_state);
                }
                SovaNotification::TempoChanged(tempo) => {
                    state.clock.tempo = tempo;
                }
                SovaNotification::QuantumChanged(quantum) => {
                    state.clock.quantum = quantum;
                }
                SovaNotification::Log(log_message) => {
                    state.logs.push_back(log_message);
                }
                SovaNotification::PlaybackStateChanged(playback_state) => {
                    state.playback_state = playback_state;
                }
                SovaNotification::FramePositionChanged(pos) => {
                    state.positions = pos;
                }
                SovaNotification::DeviceListChanged(device_infos) => {
                    state.devices = device_infos;
                }
                SovaNotification::GlobalVariablesChanged(hash_map) => {
                    state.global_vars = hash_map;
                }
                SovaNotification::Annotations(annotations) => {
                    state.annotations = annotations
                }
                SovaNotification::Error(error) => {
                    state.errors.insert((error.line, error.frame), error);
                }
                notif => {
                    apply_notification(&mut state.scene, &notif)
                }
            }
        }
    }
}