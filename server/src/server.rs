use crate::audio::AudioEngineState;
use crate::client::{ClientMessage, serialize_to_wire_frame};
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use socket2::SockRef;
use sova_core::{Scene, schedule::playback::PlaybackState, vm::LanguageCenter};
use std::{
    io::ErrorKind,
    path::PathBuf,
    sync::{
        Arc, Mutex as StdMutex, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};
use tokio::time::{Duration, timeout};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    select, signal,
    sync::{Mutex, broadcast, mpsc},
};

use sova_core::{
    clock::{Clock, ClockServer, SyncTime},
    device_map::DeviceMap,
    schedule::{SchedulerMessage, SovaNotification},
};

use crate::message::ServerMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioRestartConfig {
    pub device: Option<String>,
    pub input_device: Option<String>,
    pub channels: u16,
    pub buffer_size: Option<u32>,
    pub sample_paths: Vec<PathBuf>,
    pub max_voices: usize,
}

pub struct AudioRestartRequest {
    pub config: AudioRestartConfig,
    pub response_tx: crossbeam_channel::Sender<Result<AudioEngineState, String>>,
}

pub struct CoreRestartRequest {
    pub response_tx: crossbeam_channel::Sender<Result<(), String>>,
}

pub const DEFAULT_CLIENT_NAME: &str = "Unknown musician";
const POSITION_BROADCAST_INTERVAL_MS: u64 = 33;
const CLIENT_CHANNEL_CAPACITY: usize = 512;
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub enum BroadcastItem {
    Raw {
        bytes: Arc<Vec<u8>>,
        droppable: bool,
    },
    Filtered(SovaNotification),
    Feedback(SchedulerMessage),
}

impl BroadcastItem {
    fn is_droppable(&self) -> bool {
        match self {
            BroadcastItem::Raw { droppable, .. } => *droppable,
            BroadcastItem::Feedback(_) => false,
            BroadcastItem::Filtered(notif) => matches!(
                notif,
                SovaNotification::PeerCursorMoved(..)
                    | SovaNotification::TempoChanged(_)
                    | SovaNotification::QuantumChanged(_)
                    | SovaNotification::Tick
            ),
        }
    }
}

struct ClientHandle {
    tx: mpsc::Sender<BroadcastItem>,
    needs_resync: Arc<AtomicBool>,
}

#[derive(Clone, Default)]
pub struct ClientRegistry {
    handles: Arc<StdMutex<Vec<ClientHandle>>>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        ClientRegistry {
            handles: Arc::new(StdMutex::new(Vec::new())),
        }
    }

    pub fn register(&self) -> (mpsc::Receiver<BroadcastItem>, Arc<AtomicBool>) {
        let (tx, rx) = mpsc::channel(CLIENT_CHANNEL_CAPACITY);
        let needs_resync = Arc::new(AtomicBool::new(false));
        self.handles.lock().unwrap().push(ClientHandle {
            tx,
            needs_resync: Arc::clone(&needs_resync),
        });
        (rx, needs_resync)
    }

    pub fn broadcast(&self, item: BroadcastItem) {
        let mut handles = self.handles.lock().unwrap();
        handles.retain(|handle| match handle.tx.try_send(item.clone()) {
            Ok(()) => true,
            Err(mpsc::error::TrySendError::Full(_)) => {
                if !item.is_droppable() {
                    handle.needs_resync.store(true, Ordering::Relaxed);
                }
                true
            }
            Err(mpsc::error::TrySendError::Closed(_)) => false,
        });
    }
}

#[derive(Clone)]
pub struct ServerState {
    pub clock_server: Arc<ClockServer>,
    pub devices: Arc<DeviceMap>,
    pub sched_iface: Arc<RwLock<Sender<SchedulerMessage>>>,
    pub update_sender: broadcast::Sender<SovaNotification>,
    pub client_registry: ClientRegistry,
    pub clients: Arc<Mutex<Vec<String>>>,
    pub scene_image: Arc<Mutex<Scene>>,
    pub languages: Arc<LanguageCenter>,
    pub is_playing: Arc<AtomicBool>,
    pub audio_engine_state: Arc<StdMutex<AudioEngineState>>,
    pub audio_restart_tx: Option<Sender<AudioRestartRequest>>,
    pub core_restart_tx: Option<Sender<CoreRestartRequest>>,
    pub password: Option<String>,
}

impl ServerState {
    pub fn new(
        scene_image: Arc<Mutex<Scene>>,
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        sched_iface: Sender<SchedulerMessage>,
        update_sender: broadcast::Sender<SovaNotification>,
        client_registry: ClientRegistry,
        languages: Arc<LanguageCenter>,
        audio_engine_state: Arc<StdMutex<AudioEngineState>>,
        audio_restart_tx: Option<Sender<AudioRestartRequest>>,
        core_restart_tx: Option<Sender<CoreRestartRequest>>,
        password: Option<String>,
    ) -> Self {
        ServerState {
            clock_server,
            devices,
            sched_iface: Arc::new(RwLock::new(sched_iface)),
            update_sender,
            client_registry,
            clients: Arc::new(Mutex::new(Vec::new())),
            scene_image,
            languages,
            is_playing: Arc::new(AtomicBool::new(false)),
            audio_engine_state,
            audio_restart_tx,
            core_restart_tx,
            password,
        }
    }

    pub fn get_audio_engine_state(&self) -> AudioEngineState {
        self.audio_engine_state
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }
}

pub struct SovaCoreServer {
    pub ip: String,
    pub port: u16,
    pub state: ServerState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub scene: Scene,
    pub tempo: f64,
    pub beat: f64,
    pub micros: SyncTime,
    pub quantum: f64,
    #[serde(default)]
    pub devices: Vec<sova_core::protocol::DeviceInfo>,
}

fn send_and_relay(state: &ServerState, msg: SchedulerMessage) -> ServerMessage {
    let iface = state.sched_iface.read().unwrap();
    if iface.send(msg.clone()).is_err() {
        return ServerMessage::InternalError("Scheduler communication error.".into());
    }
    drop(iface);
    state
        .client_registry
        .broadcast(BroadcastItem::Feedback(msg));
    ServerMessage::Success
}

async fn on_message(
    msg: ClientMessage,
    state: &ServerState,
    client_name: &mut String,
) -> ServerMessage {
    println!("[➡️ ] Client '{}' sent: {:?}", client_name, msg);

    match msg {
        ClientMessage::Chat(chat_msg) => {
            state.client_registry.broadcast(BroadcastItem::Filtered(
                SovaNotification::ChatReceived(client_name.clone(), chat_msg),
            ));
            ServerMessage::Success
        }
        ClientMessage::SetName { name: new_name, .. } => {
            let mut clients_guard = state.clients.lock().await;
            let old_name = client_name.clone();
            let is_new_client = *client_name == DEFAULT_CLIENT_NAME;

            if is_new_client {
                println!("Client identified as: {}", new_name);
                clients_guard.push(new_name.clone());
            } else if let Some(i) = clients_guard.iter().position(|x| *x == old_name) {
                println!("Client {} changed name to {}", clients_guard[i], new_name);
                clients_guard[i] = new_name.clone();
            } else {
                eprintln!(
                    "Error: Could not find old name '{}' to replace. Adding '{}'.",
                    old_name, new_name
                );
                clients_guard.push(new_name.clone());
            }
            *client_name = new_name;

            let updated_clients = clients_guard.clone();
            drop(clients_guard);

            broadcast_raw(
                &state.client_registry,
                &ServerMessage::PeersUpdated(updated_clients),
                false,
            );

            ServerMessage::Success
        }
        ClientMessage::SchedulerControl(sched_msg) => send_and_relay(state, sched_msg),
        ClientMessage::SetTempo(tempo, timing) => {
            send_and_relay(state, SchedulerMessage::SetTempo(tempo, timing))
        }
        ClientMessage::GetClock => {
            let clock = Clock::from(&state.clock_server);
            ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum())
        }
        ClientMessage::GetScene => {
            ServerMessage::SceneValue(state.scene_image.lock().await.clone())
        }
        ClientMessage::GetPeers => ServerMessage::PeersUpdated(state.clients.lock().await.clone()),
        ClientMessage::SetScene(scene, timing) => {
            send_and_relay(state, SchedulerMessage::SetScene(scene, timing))
        }
        ClientMessage::RemoveFrame(line_id, position, timing) => send_and_relay(
            state,
            SchedulerMessage::RemoveFrame(line_id, position, timing),
        ),
        ClientMessage::GetSnapshot => {
            let scene = state.scene_image.lock().await.clone();
            let clock = Clock::from(&state.clock_server);
            let devices = state.devices.create_device_snapshot();
            let snapshot = Snapshot {
                scene,
                tempo: clock.tempo(),
                beat: clock.beat(),
                micros: clock.micros(),
                quantum: clock.quantum(),
                devices,
            };
            ServerMessage::Snapshot(snapshot)
        }
        ClientMessage::StartedEditingFrame(line_idx, frame_idx) => {
            state.client_registry.broadcast(BroadcastItem::Filtered(
                SovaNotification::PeerStartedEditingFrame(client_name.clone(), line_idx, frame_idx),
            ));
            ServerMessage::Success
        }
        ClientMessage::StoppedEditingFrame(line_idx, frame_idx) => {
            state.client_registry.broadcast(BroadcastItem::Filtered(
                SovaNotification::PeerStoppedEditingFrame(client_name.clone(), line_idx, frame_idx),
            ));
            ServerMessage::Success
        }
        ClientMessage::CursorPosition(line_idx, frame_idx) => {
            state.client_registry.broadcast(BroadcastItem::Filtered(
                SovaNotification::PeerCursorMoved(client_name.clone(), line_idx, frame_idx),
            ));
            ServerMessage::Success
        }
        ClientMessage::TransportStart(timing) => {
            send_and_relay(state, SchedulerMessage::TransportStart(timing))
        }
        ClientMessage::TransportStop(timing) => {
            send_and_relay(state, SchedulerMessage::TransportStop(timing))
        }
        ClientMessage::SetSceneMode(mode, timing) => {
            send_and_relay(state, SchedulerMessage::SetSceneMode(mode, timing))
        }
        ClientMessage::RequestDeviceList => {
            println!("[ info ] Client '{}' requested device list.", client_name);
            ServerMessage::DeviceList(state.devices.device_list())
        }
        ClientMessage::ConnectMidiDeviceByName(device_name) => {
            match state.devices.connect_midi_by_name(&device_name) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    broadcast_raw(
                        &state.client_registry,
                        &ServerMessage::DeviceList(updated_list.clone()),
                        false,
                    );
                    ServerMessage::DeviceList(updated_list)
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to connect device '{}': {}",
                    device_name, e
                )),
            }
        }
        ClientMessage::DisconnectMidiDeviceByName(device_name) => {
            match state.devices.disconnect_midi_by_name(&device_name) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    broadcast_raw(
                        &state.client_registry,
                        &ServerMessage::DeviceList(updated_list.clone()),
                        false,
                    );
                    ServerMessage::DeviceList(updated_list)
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to disconnect device '{}': {}",
                    device_name, e
                )),
            }
        }
        ClientMessage::CreateVirtualMidiOutput(device_name) => {
            match state.devices.create_virtual_midi_port(&device_name) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    broadcast_raw(
                        &state.client_registry,
                        &ServerMessage::DeviceList(updated_list.clone()),
                        false,
                    );
                    ServerMessage::DeviceList(updated_list)
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to create virtual device '{}': {}",
                    device_name, e
                )),
            }
        }
        ClientMessage::AssignDeviceToSlot(slot_id, device_name) => {
            match state.devices.assign_slot(slot_id, &device_name) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    broadcast_raw(
                        &state.client_registry,
                        &ServerMessage::DeviceList(updated_list.clone()),
                        false,
                    );
                    ServerMessage::DeviceList(updated_list)
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to assign slot {}: {}",
                    slot_id, e
                )),
            }
        }
        ClientMessage::UnassignDeviceFromSlot(slot_id) => {
            match state.devices.unassign_slot(slot_id) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    broadcast_raw(
                        &state.client_registry,
                        &ServerMessage::DeviceList(updated_list.clone()),
                        false,
                    );
                    ServerMessage::DeviceList(updated_list)
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to unassign slot {}: {}",
                    slot_id, e
                )),
            }
        }
        ClientMessage::CreateOscDevice(name, ip, port) => {
            match state.devices.create_osc_output_device(&name, &ip, port) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    broadcast_raw(
                        &state.client_registry,
                        &ServerMessage::DeviceList(updated_list.clone()),
                        false,
                    );
                    ServerMessage::DeviceList(updated_list)
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to create OSC device '{}': {}",
                    name, e
                )),
            }
        }
        ClientMessage::RemoveOscDevice(name) => match state.devices.remove_output_device(&name) {
            Ok(_) => {
                let updated_list = state.devices.device_list();
                broadcast_raw(
                    &state.client_registry,
                    &ServerMessage::DeviceList(updated_list.clone()),
                    false,
                );
                ServerMessage::DeviceList(updated_list)
            }
            Err(e) => ServerMessage::InternalError(format!(
                "Failed to remove OSC device '{}': {}",
                name, e
            )),
        },
        ClientMessage::SetDeviceLatency(name, latency) => {
            state.devices.set_latency(name, latency);
            let updated_list = state.devices.device_list();
            broadcast_raw(
                &state.client_registry,
                &ServerMessage::DeviceList(updated_list.clone()),
                false,
            );
            ServerMessage::DeviceList(updated_list)
        }
        ClientMessage::GetLine(line_id) => {
            let scene = state.scene_image.lock().await;
            if let Some(line) = scene.line(line_id) {
                ServerMessage::LineValues(vec![(line_id, line.clone())])
            } else {
                ServerMessage::InternalError(format!("No line at index {}", line_id))
            }
        }
        ClientMessage::SetLines(lines, timing) => {
            send_and_relay(state, SchedulerMessage::SetLines(lines, timing))
        }
        ClientMessage::ConfigureLines(lines, timing) => {
            send_and_relay(state, SchedulerMessage::ConfigureLines(lines, timing))
        }
        ClientMessage::AddLine(line_id, line, timing) => {
            send_and_relay(state, SchedulerMessage::AddLine(line_id, line, timing))
        }
        ClientMessage::RemoveLine(line_id, timing) => {
            send_and_relay(state, SchedulerMessage::RemoveLine(line_id, timing))
        }
        ClientMessage::GetFrame(line_id, frame_id) => {
            let scene = state.scene_image.lock().await;
            if let Some(frame) = scene.get_frame(line_id, frame_id) {
                ServerMessage::FrameValues(vec![(line_id, frame_id, frame.clone())])
            } else {
                ServerMessage::InternalError(format!(
                    "Unable to get frame {} at line {}",
                    frame_id, line_id
                ))
            }
        }
        ClientMessage::SetFrames(frames, timing) => {
            send_and_relay(state, SchedulerMessage::SetFrames(frames, timing))
        }
        ClientMessage::AddFrame(line_id, frame_id, frame, timing) => send_and_relay(
            state,
            SchedulerMessage::AddFrame(line_id, frame_id, frame, timing),
        ),
        ClientMessage::RestoreDevices(devices) => {
            let missing_devices = state.devices.restore_from_snapshot(devices);
            let updated_list = state.devices.device_list();
            broadcast_raw(
                &state.client_registry,
                &ServerMessage::DeviceList(updated_list),
                false,
            );
            ServerMessage::DevicesRestored { missing_devices }
        }
        ClientMessage::PreviewSample {
            folder,
            index,
            begin,
        } => {
            use sova_core::vm::event::ConcreteEvent;
            use sova_core::vm::variable::VariableValue;

            let mut args = std::collections::HashMap::new();
            args.insert("s".to_string(), VariableValue::Str(folder));
            args.insert("n".to_string(), VariableValue::Integer(index as i64));
            args.insert("gain".to_string(), VariableValue::Float(1.0));
            args.insert("dur".to_string(), VariableValue::Float(2.0));
            args.insert("begin".to_string(), VariableValue::Float(begin));

            let event = ConcreteEvent::Dirt { args, device_id: 0 };

            let clock = Clock::from(&state.clock_server);
            let time = clock.micros();
            let messages = state
                .devices
                .map_event_for_device_name("Doux", event, time, &clock);

            for timed in messages {
                let _ = timed.message.send();
            }

            ServerMessage::Success
        }
        ClientMessage::GetAudioEngineState => {
            ServerMessage::AudioEngineState(state.get_audio_engine_state())
        }
        ClientMessage::RestartAudioEngine(config) => {
            let Some(ref restart_tx) = state.audio_restart_tx else {
                return ServerMessage::InternalError("Audio engine not available".to_string());
            };

            let (response_tx, response_rx) = crossbeam_channel::bounded(1);
            let request = AudioRestartRequest {
                config,
                response_tx,
            };

            if restart_tx.send(request).is_err() {
                return ServerMessage::InternalError("Failed to send restart request".to_string());
            }

            match response_rx.recv() {
                Ok(Ok(new_state)) => ServerMessage::AudioEngineState(new_state),
                Ok(Err(e)) => ServerMessage::InternalError(format!("Audio restart failed: {}", e)),
                Err(_) => ServerMessage::InternalError("Audio restart channel closed".to_string()),
            }
        }
        ClientMessage::ResetScene(timing) => {
            send_and_relay(state, SchedulerMessage::SetScene(Scene::default(), timing))
        }
        ClientMessage::RestartCore => {
            let Some(ref restart_tx) = state.core_restart_tx else {
                return ServerMessage::InternalError("Core restart not available".into());
            };
            let restart_tx = restart_tx.clone();
            match tokio::task::spawn_blocking(move || {
                let (response_tx, response_rx) = crossbeam_channel::bounded(1);
                if restart_tx.send(CoreRestartRequest { response_tx }).is_err() {
                    return ServerMessage::InternalError("Core restart channel closed".into());
                }
                match response_rx.recv() {
                    Ok(Ok(())) => ServerMessage::Success,
                    Ok(Err(e)) => ServerMessage::InternalError(format!("Core restart failed: {e}")),
                    Err(_) => ServerMessage::InternalError("Core restart channel closed".into()),
                }
            }).await {
                Ok(msg) => msg,
                Err(e) => ServerMessage::InternalError(format!("Restart task panicked: {e}")),
            }
        }
        ClientMessage::HydraCode(code) => {
            broadcast_raw(
                &state.client_registry,
                &ServerMessage::HydraCode(client_name.clone(), code),
                false,
            );
            ServerMessage::Success
        }
        ClientMessage::EnableFeedback => {
            let scene = state.scene_image.lock().await.clone();
            let clock = Clock::from(&state.clock_server);
            ServerMessage::FeedbackEnabled {
                scene,
                tempo: clock.tempo(),
                quantum: clock.quantum(),
                is_playing: state.is_playing.load(Ordering::Relaxed),
            }
        }
    }
}

async fn send_msg<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: ServerMessage) -> io::Result<()> {
    let frame = serialize_to_wire_frame(&msg)?;
    writer.write_all(&frame).await?;
    writer.flush().await?;
    Ok(())
}

fn broadcast_raw(registry: &ClientRegistry, msg: &ServerMessage, droppable: bool) {
    if let Ok(bytes) = serialize_to_wire_frame(msg) {
        registry.broadcast(BroadcastItem::Raw {
            bytes: Arc::new(bytes),
            droppable,
        });
    }
}

fn notification_to_server_message(
    notif: SovaNotification,
) -> Result<ServerMessage, SovaNotification> {
    match notif {
        SovaNotification::UpdatedScene(p) => Ok(ServerMessage::SceneValue(p)),
        SovaNotification::UpdatedSceneMode(m) => Ok(ServerMessage::SceneMode(m)),
        SovaNotification::UpdatedLines(lines) => Ok(ServerMessage::LineValues(lines)),
        SovaNotification::UpdatedLineConfigurations(lines) => {
            Ok(ServerMessage::LineConfigurations(lines))
        }
        SovaNotification::AddedLine(id, line) => Ok(ServerMessage::AddLine(id, line)),
        SovaNotification::RemovedLine(id) => Ok(ServerMessage::RemoveLine(id)),
        SovaNotification::UpdatedFrames(frames) => Ok(ServerMessage::FrameValues(frames)),
        SovaNotification::AddedFrame(li, fi, frame) => Ok(ServerMessage::AddFrame(li, fi, frame)),
        SovaNotification::RemovedFrame(li, fi) => Ok(ServerMessage::RemoveFrame(li, fi)),
        SovaNotification::PlaybackStateChanged(state) => {
            Ok(ServerMessage::PlaybackStateChanged(state))
        }
        SovaNotification::FramePositionChanged(pos) => Ok(ServerMessage::FramePosition(pos)),
        SovaNotification::Log(msg) => Ok(ServerMessage::Log(msg)),
        SovaNotification::ClientListChanged(clients) => Ok(ServerMessage::PeersUpdated(clients)),
        SovaNotification::DeviceListChanged(devices) => Ok(ServerMessage::DeviceList(devices)),
        SovaNotification::ScopeData(peaks) => Ok(ServerMessage::ScopeData(peaks)),
        SovaNotification::GlobalVariablesChanged(vars) => {
            Ok(ServerMessage::GlobalVariablesUpdate(vars))
        }
        SovaNotification::CompilationUpdated(li, fi, sid, state) => {
            Ok(ServerMessage::CompilationUpdate(li, fi, sid, state))
        }
        SovaNotification::Error(e) => Ok(ServerMessage::Error(e)),
        other => Err(other),
    }
}

impl SovaCoreServer {
    pub fn new(ip: String, port: u16, state: ServerState) -> Self {
        SovaCoreServer { ip, port, state }
    }

    pub async fn start(
        &self,
        scheduler_notifications: Option<Receiver<SovaNotification>>,
    ) -> io::Result<()> {
        let addr = format!("{}:{}", self.ip, self.port);
        let listener = TcpListener::bind(&addr).await?;
        println!("Server listening on {}", addr);
        if let Some(rx) = scheduler_notifications {
            self.start_image_maintainer(rx);
        }

        // Bridge logger notifications (from core) to per-client channels
        let mut log_rx = self.state.update_sender.subscribe();
        let bridge_registry = self.state.client_registry.clone();
        tokio::spawn(async move {
            loop {
                match log_rx.recv().await {
                    Ok(notif) => match notification_to_server_message(notif) {
                        Ok(msg) => {
                            if let Ok(bytes) = serialize_to_wire_frame(&msg) {
                                bridge_registry.broadcast(BroadcastItem::Raw {
                                    bytes: Arc::new(bytes),
                                    droppable: false,
                                });
                            }
                        }
                        Err(notif) => {
                            bridge_registry.broadcast(BroadcastItem::Filtered(notif));
                        }
                    },
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        loop {
            select! {
                Ok((socket, client_addr)) = listener.accept() => {
                    println!("New connection from {}", client_addr);
                    let client_state = self.state.clone();
                    tokio::spawn(async move {
                        match process_client(socket, client_state).await {
                            Ok(client_name) => {
                            println!("Client '{}' disconnected.", client_name);
                            },
                            Err(e) => {
                                eprintln!("Error handling client {}: {}", client_addr, e);
                            }
                        }
                    });
                }
                _ = signal::ctrl_c() => {
                    println!("\n[!] Ctrl+C received, shutting down server...");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(20)) => {
                    self.state
                        .client_registry
                        .broadcast(BroadcastItem::Filtered(SovaNotification::Tick));
                }
            }
        }

        Ok(())
    }

    pub fn start_image_maintainer(&self, scheduler_notifications: Receiver<SovaNotification>) {
        start_image_maintainer(
            scheduler_notifications,
            self.state.scene_image.clone(),
            self.state.client_registry.clone(),
            self.state.is_playing.clone(),
        );
    }
}

pub fn start_image_maintainer(
    scheduler_notifications: Receiver<SovaNotification>,
    scene_image: Arc<Mutex<Scene>>,
    client_registry: ClientRegistry,
    is_playing: Arc<AtomicBool>,
) {
    thread::spawn(move || {
            let position_broadcast_interval =
                std::time::Duration::from_millis(POSITION_BROADCAST_INTERVAL_MS);
            let mut last_position_broadcast = std::time::Instant::now();

            loop {
                match scheduler_notifications.recv() {
                    Ok(p) => {
                        let mut guard = scene_image.blocking_lock();
                        match &p {
                            SovaNotification::UpdatedScene(scene) => {
                                *guard = scene.clone();
                            }
                            SovaNotification::UpdatedSceneMode(mode) => {
                                guard.mode = *mode;
                            }
                            SovaNotification::UpdatedScenePrelude(prelude) => {
                                guard.prelude = prelude.clone();
                            }
                            SovaNotification::UpdatedLines(lines) => {
                                for (i, line) in lines {
                                    guard.set_line(*i, line.clone());
                                }
                            }
                            SovaNotification::AddedLine(i, line) => {
                                guard.insert_line(*i, line.clone());
                            }
                            SovaNotification::RemovedLine(index) => {
                                guard.remove_line(*index);
                            }
                            SovaNotification::UpdatedFrames(frames) => {
                                for (line_id, frame_id, frame) in frames.iter() {
                                    guard.line_mut(*line_id).set_frame(*frame_id, frame.clone());
                                }
                            }
                            SovaNotification::AddedFrame(line_id, frame_id, frame) => {
                                guard
                                    .line_mut(*line_id)
                                    .insert_frame(*frame_id, frame.clone());
                            }
                            SovaNotification::RemovedFrame(line_id, frame_id) => {
                                guard.line_mut(*line_id).remove_frame(*frame_id);
                            }
                            SovaNotification::PlaybackStateChanged(state) => {
                                let playing = match state {
                                    PlaybackState::Stopped => false,
                                    PlaybackState::Starting(_) => false,
                                    PlaybackState::Playing => true,
                                };
                                is_playing.store(playing, Ordering::Relaxed);
                            }
                            _ => (),
                        };
                        drop(guard);

                        let should_broadcast = match &p {
                            SovaNotification::FramePositionChanged(_) => {
                                let now = std::time::Instant::now();
                                if now.duration_since(last_position_broadcast)
                                    >= position_broadcast_interval
                                {
                                    last_position_broadcast = now;
                                    true
                                } else {
                                    false
                                }
                            }
                            _ => true,
                        };

                        if should_broadcast {
                            let droppable = matches!(&p, SovaNotification::FramePositionChanged(_));
                            match notification_to_server_message(p) {
                                Ok(msg) => {
                                    if let Ok(bytes) = serialize_to_wire_frame(&msg) {
                                        client_registry.broadcast(BroadcastItem::Raw {
                                            bytes: Arc::new(bytes),
                                            droppable,
                                        });
                                    }
                                }
                                Err(notif) => {
                                    client_registry.broadcast(BroadcastItem::Filtered(notif));
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });
}

async fn process_client(socket: TcpStream, state: ServerState) -> io::Result<String> {
    socket.set_nodelay(true)?;
    let keepalive = socket2::TcpKeepalive::new()
        .with_time(std::time::Duration::from_secs(60))
        .with_interval(std::time::Duration::from_secs(10));
    let _ = SockRef::from(&socket).set_tcp_keepalive(&keepalive);
    let client_addr = socket.peer_addr()?;
    let client_addr_str = client_addr.to_string();
    let (reader, writer) = socket.into_split();
    let mut reader = BufReader::with_capacity(32 * 1024, reader);
    let mut writer = BufWriter::with_capacity(32 * 1024, writer);
    let mut client_name = DEFAULT_CLIENT_NAME.to_string();

    let mut clock = Clock::from(&state.clock_server);

    let hello_msg: ServerMessage;

    match read_message_internal(&mut reader, &client_addr_str).await {
        Ok(Some(ClientMessage::SetName { name: new_name, password })) => {
            if let Some(required) = &state.password {
                if password.as_deref() != Some(required.as_str()) {
                    eprintln!(
                        "Connection rejected: Invalid password from {}",
                        client_addr_str
                    );
                    let refuse_msg =
                        ServerMessage::ConnectionRefused("Invalid password.".to_string());
                    let _ = send_msg(&mut writer, refuse_msg).await;
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "Invalid password",
                    ));
                }
            }

            if new_name.is_empty() || new_name == DEFAULT_CLIENT_NAME {
                eprintln!(
                    "Connection rejected: Invalid username '{}' from {}",
                    new_name, client_addr_str
                );
                let refuse_msg = ServerMessage::ConnectionRefused(
                    "Invalid username (empty or reserved).".to_string(),
                );
                let _ = send_msg(&mut writer, refuse_msg).await;
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid username",
                ));
            }

            let mut clients_guard = state.clients.lock().await;
            if clients_guard.iter().any(|name| name == &new_name) {
                eprintln!(
                    "Connection rejected: Username '{}' already taken by {}",
                    new_name, client_addr_str
                );
                let refuse_msg = ServerMessage::ConnectionRefused(format!(
                    "Username '{}' is already taken.",
                    new_name
                ));
                let _ = send_msg(&mut writer, refuse_msg).await;
                drop(clients_guard);
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Username taken",
                ));
            }

            client_name = new_name;
            println!("Client {} identified as: {}", client_addr_str, client_name);
            clients_guard.push(client_name.clone());

            let initial_scene = state.scene_image.lock().await.clone();
            let initial_devices = state.devices.device_list();
            let initial_peers = clients_guard.clone();
            let updated_peers_for_broadcast = initial_peers.clone();

            drop(clients_guard);

            broadcast_raw(
                &state.client_registry,
                &ServerMessage::PeersUpdated(updated_peers_for_broadcast),
                false,
            );

            let initial_link_state = (
                clock.tempo(),
                clock.beat(),
                clock.beat() % clock.quantum(),
                state.clock_server.link.num_peers() as u32,
                state.clock_server.link.is_start_stop_sync_enabled(),
            );
            let initial_is_playing = state.is_playing.load(Ordering::Relaxed);

            let available_languages = state.languages.definitions().collect();

            println!(
                "[ handshake ] Sending Hello to {} ({}). Initial is_playing state: {}",
                client_addr_str, client_name, initial_is_playing
            );
            hello_msg = ServerMessage::Hello {
                username: client_name.clone(),
                scene: initial_scene,
                devices: initial_devices,
                peers: initial_peers,
                link_state: initial_link_state,
                is_playing: initial_is_playing,
                languages: available_languages,
                audio_engine_state: state.get_audio_engine_state(),
            };

            if !matches!(
                timeout(WRITE_TIMEOUT, send_msg(&mut writer, hello_msg)).await,
                Ok(Ok(()))
            ) {
                eprintln!("Failed to send Hello to {}", client_name);
                let mut clients_guard = state.clients.lock().await;
                if let Some(i) = clients_guard.iter().position(|x| *x == client_name) {
                    clients_guard.remove(i);
                }
                let updated = clients_guard.clone();
                drop(clients_guard);
                broadcast_raw(
                    &state.client_registry,
                    &ServerMessage::PeersUpdated(updated),
                    false,
                );
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "Failed to send Hello message",
                ));
            }
        }
        Ok(Some(other_msg)) => {
            eprintln!(
                "Connection rejected: Expected SetName, received {:?} from {}",
                other_msg, client_addr_str
            );
            let refuse_msg =
                ServerMessage::ConnectionRefused("Invalid handshake sequence.".to_string());
            let _ = send_msg(&mut writer, refuse_msg).await;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid handshake sequence",
            ));
        }
        Ok(None) => {
            println!("Connection closed by {} during handshake.", client_addr_str);
            return Ok(client_name);
        }
        Err(e) => {
            eprintln!(
                "Read error during handshake with {}: {}",
                client_addr_str, e
            );
            return Err(e);
        }
    }

    let (mut update_receiver, needs_resync) = state.client_registry.register();
    let mut feedback_enabled = false;

    // Dedicated reader task — never cancelled, so read_wire_frame
    // can't lose partial reads (which would desync the TCP stream).
    enum ClientRead {
        Message(Box<ClientMessage>),
        Closed,
        Error(io::Error),
    }
    let (client_msg_tx, mut client_msg_rx) = mpsc::unbounded_channel::<ClientRead>();
    let reader_client_name = client_name.clone();
    let reader_task = tokio::spawn(async move {
        loop {
            match read_message_internal(&mut reader, &reader_client_name).await {
                Ok(Some(msg)) => {
                    if client_msg_tx
                        .send(ClientRead::Message(Box::new(msg)))
                        .is_err()
                    {
                        break;
                    }
                }
                Ok(None) => {
                    let _ = client_msg_tx.send(ClientRead::Closed);
                    break;
                }
                Err(e) if e.kind() == ErrorKind::InvalidData => {
                    eprintln!("Bad frame from {}: {}. Skipping.", reader_client_name, e);
                }
                Err(e) => {
                    let _ = client_msg_tx.send(ClientRead::Error(e));
                    break;
                }
            }
        }
    });

    loop {
        select! {
            biased;

            read_result = client_msg_rx.recv() => {
                match read_result {
                    Some(ClientRead::Message(msg)) => {
                        if matches!(*msg, ClientMessage::EnableFeedback) {
                            feedback_enabled = true;
                        }
                        let response = on_message(*msg, &state, &mut client_name).await;

                        if !matches!(
                            timeout(WRITE_TIMEOUT, send_msg(&mut writer, response)).await,
                            Ok(Ok(()))
                        ) {
                            eprintln!("Failed write direct response to {}", client_name);
                            break;
                        }
                    },
                    Some(ClientRead::Closed) => {
                        println!("Connection closed cleanly by {}.", client_name);
                        break;
                    },
                    Some(ClientRead::Error(_e)) => {
                        eprintln!("Read error for client {}. Closing connection.", client_name);
                        break;
                    }
                    None => {
                        eprintln!("Reader task ended for {}. Closing connection.", client_name);
                        break;
                    }
                }
            }

            update_result = update_receiver.recv() => {
                if needs_resync.swap(false, Ordering::Relaxed) {
                    let scene = state.scene_image.lock().await.clone();
                    let c = Clock::from(&state.clock_server);
                    let devices = state.devices.create_device_snapshot();
                    let snapshot = Snapshot {
                        scene,
                        tempo: c.tempo(),
                        beat: c.beat(),
                        micros: c.micros(),
                        quantum: c.quantum(),
                        devices,
                    };
                    if !matches!(
                        timeout(WRITE_TIMEOUT, send_msg(&mut writer, ServerMessage::Snapshot(snapshot))).await,
                        Ok(Ok(()))
                    ) {
                        eprintln!("Resync write failed for {}, evicting", client_name);
                        break;
                    }
                    while update_receiver.try_recv().is_ok() {}
                    continue;
                }

                let Some(item) = update_result else { break; };

                match item {
                    BroadcastItem::Raw { bytes, .. } => {
                        if !matches!(
                            timeout(WRITE_TIMEOUT, async {
                                writer.write_all(&bytes).await?;
                                writer.flush().await
                            }).await,
                            Ok(Ok(()))
                        ) {
                            break;
                        }
                    }
                    BroadcastItem::Feedback(msg) => {
                        if feedback_enabled
                            && !matches!(
                                timeout(WRITE_TIMEOUT, send_msg(&mut writer, ServerMessage::Feedback(msg))).await,
                                Ok(Ok(()))
                            )
                        {
                            break;
                        }
                    }
                    BroadcastItem::Filtered(notification) => {
                        let msg_opt: Option<ServerMessage> = match notification {
                            SovaNotification::ChatReceived(sender_name, chat_msg) => {
                                if sender_name != *client_name {
                                    Some(ServerMessage::Chat(sender_name, chat_msg))
                                } else {
                                    None
                                }
                            }
                            SovaNotification::PeerStartedEditingFrame(sender_name, li, fi) => {
                                if sender_name != *client_name {
                                    Some(ServerMessage::PeerStartedEditing(sender_name, li, fi))
                                } else {
                                    None
                                }
                            }
                            SovaNotification::PeerStoppedEditingFrame(sender_name, li, fi) => {
                                if sender_name != *client_name {
                                    Some(ServerMessage::PeerStoppedEditing(sender_name, li, fi))
                                } else {
                                    None
                                }
                            }
                            SovaNotification::PeerCursorMoved(sender_name, li, fi) => {
                                if sender_name != *client_name {
                                    Some(ServerMessage::PeerCursorMoved(sender_name, li, fi))
                                } else {
                                    None
                                }
                            }
                            SovaNotification::TempoChanged(_) => {
                                let c = Clock::from(&state.clock_server);
                                Some(ServerMessage::ClockState(c.tempo(), c.beat(), c.micros(), c.quantum()))
                            }
                            SovaNotification::QuantumChanged(_) => {
                                let c = Clock::from(&state.clock_server);
                                Some(ServerMessage::ClockState(c.tempo(), c.beat(), c.micros(), c.quantum()))
                            }
                            SovaNotification::Tick => {
                                clock.capture_app_state();
                                Some(ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum()))
                            }
                            _ => None,
                        };

                        if let Some(msg) = msg_opt
                            && !matches!(
                                timeout(WRITE_TIMEOUT, send_msg(&mut writer, msg)).await,
                                Ok(Ok(()))
                            )
                        {
                            break;
                        }
                    }
                }
            }
        }
    }

    reader_task.abort();

    println!("Cleaning up connection for client: {}", client_name);
    if client_name != DEFAULT_CLIENT_NAME {
        let mut clients_guard = state.clients.lock().await;
        if let Some(i) = clients_guard.iter().position(|x| *x == client_name) {
            clients_guard.remove(i);
            println!("Removed {} from client list.", client_name);
            let updated_clients = clients_guard.clone();
            drop(clients_guard);
            broadcast_raw(
                &state.client_registry,
                &ServerMessage::PeersUpdated(updated_clients),
                false,
            );
        } else {
            eprintln!(
                "Client '{}' not found in list during cleanup, though name was set.",
                client_name
            );
        }
    } else {
        println!(
            "Client disconnected before setting a name (still '{}'). No list removal needed.",
            DEFAULT_CLIENT_NAME
        );
    }

    Ok(client_name)
}

async fn read_message_internal<R: AsyncReadExt + Unpin>(
    reader: &mut R,
    client_id_for_logging: &str,
) -> io::Result<Option<ClientMessage>> {
    use crate::client::read_wire_frame;

    let payload = match read_wire_frame(reader).await {
        Ok(buf) => buf,
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
            println!("Connection closed by {} (EOF).", client_id_for_logging);
            return Ok(None);
        }
        Err(e) => return Err(e),
    };

    ClientMessage::deserialize(&payload)
}
