use crate::audio::AudioEngineState;
use crate::client::ClientMessage;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use sova_core::{Scene, schedule::playback::PlaybackState, vm::LanguageCenter};
use std::{
    io::ErrorKind,
    sync::{
        Arc, Mutex as StdMutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};
use tokio::time::Duration;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    select, signal,
    sync::{Mutex, broadcast},
};

use sova_core::{
    clock::{Clock, ClockServer, SyncTime},
    device_map::DeviceMap,
    schedule::{SchedulerMessage, SovaNotification},
};

use crate::message::ServerMessage;

pub const DEFAULT_CLIENT_NAME: &str = "Unknown musician";

const COMPRESSION_MIN_SIZE: usize = 64;
const COMPRESSION_ADAPTIVE_THRESHOLD: usize = 256;
const HIGH_COMPRESSION_CUTOFF: usize = 1024;
const COMPRESSION_FLAG: u32 = 0x80000000;
const LENGTH_MASK: u32 = 0x7FFFFFFF;
const POSITION_BROADCAST_INTERVAL_MS: u64 = 33;

#[derive(Clone)]
pub struct ServerState {
    pub clock_server: Arc<ClockServer>,
    pub devices: Arc<DeviceMap>,
    pub sched_iface: Sender<SchedulerMessage>,
    pub update_sender: broadcast::Sender<SovaNotification>,
    pub clients: Arc<Mutex<Vec<String>>>,
    pub scene_image: Arc<Mutex<Scene>>,
    pub languages: Arc<LanguageCenter>,
    pub is_playing: Arc<AtomicBool>,
    pub audio_engine_state: Arc<StdMutex<AudioEngineState>>,
}

impl ServerState {
    pub fn new(
        scene_image: Arc<Mutex<Scene>>,
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        sched_iface: Sender<SchedulerMessage>,
        update_sender: broadcast::Sender<SovaNotification>,
        languages: Arc<LanguageCenter>,
        audio_engine_state: Arc<StdMutex<AudioEngineState>>,
    ) -> Self {
        ServerState {
            clock_server,
            devices,
            sched_iface,
            update_sender,
            clients: Arc::new(Mutex::new(Vec::new())),
            scene_image,
            languages,
            is_playing: Arc::new(AtomicBool::new(false)),
            audio_engine_state,
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
    pub devices: Option<Vec<sova_core::protocol::DeviceInfo>>,
}

async fn on_message(
    msg: ClientMessage,
    state: &ServerState,
    client_name: &mut String,
) -> ServerMessage {
    println!("[➡️ ] Client '{}' sent: {:?}", client_name, msg);

    match msg {
        ClientMessage::Chat(chat_msg) => {
            let _ = state.update_sender.send(SovaNotification::ChatReceived(
                client_name.clone(),
                chat_msg,
            ));
            ServerMessage::Success
        }
        ClientMessage::SetName(new_name) => {
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

            let _ = state
                .update_sender
                .send(SovaNotification::ClientListChanged(updated_clients));

            ServerMessage::Success
        }
        ClientMessage::SchedulerControl(sched_msg) => {
            if state.sched_iface.send(sched_msg).is_ok() {
                ServerMessage::Success
            } else {
                eprintln!("Failed to send SchedulerControl message.");
                ServerMessage::InternalError("Failed to send command to scheduler.".to_string())
            }
        }
        ClientMessage::SetTempo(tempo, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::SetTempo(tempo, timing))
                .is_err()
            {
                eprintln!("Failed to send SetTempo to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            ServerMessage::Success
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
            if state
                .sched_iface
                .send(SchedulerMessage::SetScene(scene, timing))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("Failed to send Setscene to scheduler.");
                ServerMessage::InternalError(
                    "Failed to apply scene update to scheduler.".to_string(),
                )
            }
        }
        ClientMessage::RemoveFrame(line_id, position, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::RemoveFrame(line_id, position, timing))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("Failed to send RemoveLine to scheduler.");
                ServerMessage::InternalError(
                    "Failed to send remove line update to scheduler.".to_string(),
                )
            }
        }
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
                devices: Some(devices),
            };
            ServerMessage::Snapshot(snapshot)
        }
        ClientMessage::StartedEditingFrame(line_idx, frame_idx) => {
            let _ = state
                .update_sender
                .send(SovaNotification::PeerStartedEditingFrame(
                    client_name.clone(),
                    line_idx,
                    frame_idx,
                ));
            ServerMessage::Success
        }
        ClientMessage::StoppedEditingFrame(line_idx, frame_idx) => {
            let _ = state
                .update_sender
                .send(SovaNotification::PeerStoppedEditingFrame(
                    client_name.clone(),
                    line_idx,
                    frame_idx,
                ));
            ServerMessage::Success
        }
        ClientMessage::TransportStart(timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::TransportStart(timing))
                .is_err()
            {
                eprintln!("Failed to send TransportStart to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            ServerMessage::Success
        }
        ClientMessage::TransportStop(timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::TransportStop(timing))
                .is_err()
            {
                eprintln!("Failed to send TransportStop to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            ServerMessage::Success
        }
        ClientMessage::RequestDeviceList => {
            println!("[ info ] Client '{}' requested device list.", client_name);
            ServerMessage::DeviceList(state.devices.device_list())
        }
        ClientMessage::ConnectMidiDeviceByName(device_name) => {
            match state.devices.connect_midi_by_name(&device_name) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    let _ = state
                        .update_sender
                        .send(SovaNotification::DeviceListChanged(updated_list.clone()));
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
                    let _ = state
                        .update_sender
                        .send(SovaNotification::DeviceListChanged(updated_list.clone()));
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
                    let _ = state
                        .update_sender
                        .send(SovaNotification::DeviceListChanged(updated_list.clone()));
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
                    let _ = state
                        .update_sender
                        .send(SovaNotification::DeviceListChanged(updated_list.clone()));
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
                    let _ = state
                        .update_sender
                        .send(SovaNotification::DeviceListChanged(updated_list.clone()));
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
                    let _ = state
                        .update_sender
                        .send(SovaNotification::DeviceListChanged(updated_list.clone()));
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
                let _ = state
                    .update_sender
                    .send(SovaNotification::DeviceListChanged(updated_list.clone()));
                ServerMessage::DeviceList(updated_list)
            }
            Err(e) => ServerMessage::InternalError(format!(
                "Failed to remove OSC device '{}': {}",
                name, e
            )),
        },
        ClientMessage::GetLine(line_id) => {
            let scene = state.scene_image.lock().await;
            if let Some(line) = scene.line(line_id) {
                ServerMessage::LineValues(vec![(line_id, line.clone())])
            } else {
                ServerMessage::InternalError(format!("No line at index {}", line_id))
            }
        }
        ClientMessage::SetLines(lines, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::SetLines(lines, timing))
                .is_err()
            {
                eprintln!("Failed to send SetLines to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            ServerMessage::Success
        }
        ClientMessage::ConfigureLines(lines, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::ConfigureLines(lines, timing))
                .is_err()
            {
                eprintln!("Failed to send ConfigureLines to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            ServerMessage::Success
        }
        ClientMessage::AddLine(line_id, line, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::AddLine(line_id, line, timing))
                .is_err()
            {
                eprintln!("Failed to send AddLine to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            ServerMessage::Success
        }
        ClientMessage::RemoveLine(line_id, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::RemoveLine(line_id, timing))
                .is_err()
            {
                eprintln!("Failed to send RemoveLine to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            ServerMessage::Success
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
            if state
                .sched_iface
                .send(SchedulerMessage::SetFrames(frames, timing))
                .is_err()
            {
                eprintln!("Failed to send SetFrames to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            ServerMessage::Success
        }
        ClientMessage::AddFrame(line_id, frame_id, frame, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::AddFrame(line_id, frame_id, frame, timing))
                .is_err()
            {
                eprintln!("Failed to send AddFrame to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            ServerMessage::Success
        }
        ClientMessage::RestoreDevices(devices) => {
            let missing_devices = state.devices.restore_from_snapshot(devices);
            let updated_list = state.devices.device_list();
            let _ = state
                .update_sender
                .send(SovaNotification::DeviceListChanged(updated_list));
            ServerMessage::DevicesRestored { missing_devices }
        }
        ClientMessage::GetAudioEngineState => {
            ServerMessage::AudioEngineState(state.get_audio_engine_state())
        }
    }
}

async fn send_msg<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: ServerMessage) -> io::Result<()> {
    let msgpack_bytes = rmp_serde::to_vec_named(&msg).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize ServerMessage to MessagePack: {}", e),
        )
    })?;

    let (final_bytes, is_compressed) = compress_message_intelligently(&msg, &msgpack_bytes)?;

    let mut len = final_bytes.len() as u32;
    if is_compressed {
        len |= COMPRESSION_FLAG;
    }

    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(&final_bytes).await?;
    writer.flush().await?;

    Ok(())
}

fn compress_message_intelligently(
    msg: &ServerMessage,
    msgpack_bytes: &[u8],
) -> io::Result<(Vec<u8>, bool)> {
    use crate::client::CompressionStrategy;

    match msg.compression_strategy() {
        CompressionStrategy::Never => Ok((msgpack_bytes.to_vec(), false)),
        CompressionStrategy::Always => {
            if msgpack_bytes.len() > COMPRESSION_MIN_SIZE {
                let compression_level = if msgpack_bytes.len() < HIGH_COMPRESSION_CUTOFF {
                    1
                } else {
                    3
                };
                let compressed = zstd::encode_all(msgpack_bytes, compression_level)
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
                let compression_level = if msgpack_bytes.len() < HIGH_COMPRESSION_CUTOFF {
                    1
                } else {
                    3
                };
                let compressed = zstd::encode_all(msgpack_bytes, compression_level)
                    .map_err(|e| io::Error::other(format!("Compression failed: {}", e)))?;
                Ok((compressed, true))
            }
        }
    }
}

impl SovaCoreServer {
    pub fn new(ip: String, port: u16, state: ServerState) -> Self {
        SovaCoreServer { ip, port, state }
    }

    pub async fn start(
        &self,
        scheduler_notifications: Receiver<SovaNotification>,
    ) -> io::Result<()> {
        let addr = format!("{}:{}", self.ip, self.port);
        let listener = TcpListener::bind(&addr).await?;
        println!("Server listening on {}", addr);
        self.start_image_maintainer(scheduler_notifications);
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
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    let _ = self.state.update_sender.send(SovaNotification::Tick);
                }
            }
        }

        Ok(())
    }

    pub fn start_image_maintainer(&self, scheduler_notifications: Receiver<SovaNotification>) {
        let scene_image = self.state.scene_image.clone();
        let update_sender = self.state.update_sender.clone();
        let is_playing = self.state.is_playing.clone();
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
                            let _ = update_sender.send(p);
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

async fn process_client(socket: TcpStream, state: ServerState) -> io::Result<String> {
    socket.set_nodelay(true)?;
    let client_addr = socket.peer_addr()?;
    let client_addr_str = client_addr.to_string();
    let (reader, writer) = socket.into_split();
    let mut reader = BufReader::with_capacity(32 * 1024, reader);
    let mut writer = BufWriter::with_capacity(32 * 1024, writer);
    let mut client_name = DEFAULT_CLIENT_NAME.to_string();

    let mut clock = Clock::from(&state.clock_server);

    let hello_msg: ServerMessage;

    match read_message_internal(&mut reader, &client_addr_str).await {
        Ok(Some(ClientMessage::SetName(new_name))) => {
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

            let _ = state
                .update_sender
                .send(SovaNotification::ClientListChanged(
                    updated_peers_for_broadcast,
                ));

            let initial_link_state = (
                clock.tempo(),
                clock.beat(),
                clock.beat() % clock.quantum(),
                state.clock_server.link.num_peers() as u32,
                state.clock_server.link.is_start_stop_sync_enabled(),
            );
            let initial_is_playing = state.is_playing.load(Ordering::Relaxed);

            let available_languages: Vec<String> =
                state.languages.languages().map(str::to_owned).collect();

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
                available_languages,
                audio_engine_state: state.get_audio_engine_state(),
            };

            if send_msg(&mut writer, hello_msg).await.is_err() {
                eprintln!("Failed to send Hello to {}", client_name);
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

    let mut update_receiver = state.update_sender.subscribe();

    loop {
        select! {
            biased;

            read_result = read_message_internal(&mut reader, &client_name) => {
                match read_result {
                    Ok(Some(msg)) => {
                        let response = on_message(msg, &state, &mut client_name).await;

                        if send_msg(&mut writer, response).await.is_err() {
                            eprintln!("Failed write direct response to {}", client_name);
                            break;
                        }
                    },
                    Ok(None) => {
                        println!("Connection closed cleanly by {}.", client_name);
                        break;
                    },
                    Err(_e) => {
                        eprintln!("Read error for client {}. Closing connection.", client_name);
                        break;
                    }
                }
            }

            update_result = update_receiver.recv() => {
                let notification = match update_result {
                    Ok(notif) => notif,
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        eprintln!("Client {} lagged {} notifications", client_name, count);
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                };
                let broadcast_msg_opt: Option<ServerMessage> = match notification {
                    SovaNotification::UpdatedScene(p) => {
                        Some(ServerMessage::SceneValue(p))
                    }
                    SovaNotification::UpdatedLines(lines) => {
                        Some(ServerMessage::LineValues(lines))
                    }
                    SovaNotification::UpdatedLineConfigurations(lines) => {
                        Some(ServerMessage::LineConfigurations(lines))
                    }
                    SovaNotification::AddedLine(line_id, line) => {
                        Some(ServerMessage::AddLine(line_id, line))
                    }
                    SovaNotification::RemovedLine(line_id) => {
                        Some(ServerMessage::RemoveLine(line_id))
                    }
                    SovaNotification::UpdatedFrames(frames) => {
                        Some(ServerMessage::FrameValues(frames))
                    }
                    SovaNotification::AddedFrame(line_id, frame_id, frame) => {
                        Some(ServerMessage::AddFrame(line_id, frame_id, frame))
                    }
                    SovaNotification::RemovedFrame(line_id, frame_id) => {
                        Some(ServerMessage::RemoveFrame(line_id, frame_id))
                    }
                    SovaNotification::PlaybackStateChanged(state) => {
                        Some(ServerMessage::PlaybackStateChanged(state))
                    }
                    SovaNotification::FramePositionChanged(pos) => {
                        Some(ServerMessage::FramePosition(pos))
                    }
                    SovaNotification::Log(log_message) => {
                        Some(ServerMessage::Log(log_message))
                    }
                    SovaNotification::TempoChanged(_) => {
                        let clock = Clock::from(&state.clock_server);
                        Some(ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum()))
                    }
                    SovaNotification::QuantumChanged(_) => {
                        let clock = Clock::from(&state.clock_server);
                        Some(ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum()))
                    }
                    SovaNotification::ClientListChanged(clients) => {
                        Some(ServerMessage::PeersUpdated(clients))
                    }
                    SovaNotification::ChatReceived(sender_name, chat_msg) => {
                        if sender_name != *client_name {
                           Some(ServerMessage::Chat(sender_name, chat_msg))
                        } else {
                            None
                        }
                    }
                    SovaNotification::PeerStartedEditingFrame(sender_name, line_idx, frame_idx) => {
                        if sender_name != *client_name {
                            Some(ServerMessage::PeerStartedEditing(sender_name, line_idx, frame_idx))
                        } else {
                            None
                        }
                    }
                    SovaNotification::PeerStoppedEditingFrame(sender_name, line_idx, frame_idx) => {
                        if sender_name != *client_name {
                            Some(ServerMessage::PeerStoppedEditing(sender_name, line_idx, frame_idx))
                        } else {
                            None
                        }
                    }
                    SovaNotification::DeviceListChanged(devices) => {
                        println!("[ broadcast ] Sending updated device list ({} devices) to {}", devices.len(), client_name);
                        Some(ServerMessage::DeviceList(devices))
                    }
                    SovaNotification::GlobalVariablesChanged(vars) => {
                        Some(ServerMessage::GlobalVariablesUpdate(vars))
                    }
                    SovaNotification::CompilationUpdated(line_id, frame_id, script_id, state) => {
                        Some(ServerMessage::CompilationUpdate(line_id, frame_id, script_id, state))
                    }
                    SovaNotification::Tick => {
                        clock.capture_app_state();
                        Some(ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum()))
                    }
                };

                if let Some(broadcast_msg) = broadcast_msg_opt {
                    let send_res = send_msg(&mut writer, broadcast_msg).await;
                    if send_res.is_err() {
                        break;
                    }
                }
            }
        }
    }

    println!("Cleaning up connection for client: {}", client_name);
    if client_name != DEFAULT_CLIENT_NAME {
        let mut clients_guard = state.clients.lock().await;
        if let Some(i) = clients_guard.iter().position(|x| *x == client_name) {
            clients_guard.remove(i);
            println!("Removed {} from client list.", client_name);
            let updated_clients = clients_guard.clone();
            drop(clients_guard);
            let _ = state
                .update_sender
                .send(SovaNotification::ClientListChanged(updated_clients));
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
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {
            let len_with_flag = u32::from_be_bytes(len_buf);
            let is_compressed = (len_with_flag & COMPRESSION_FLAG) != 0;
            let length = len_with_flag & LENGTH_MASK;

            if length == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Received zero-length message header",
                ));
            }

            let mut message_buf = vec![0u8; length as usize];
            reader.read_exact(&mut message_buf).await?;

            let final_bytes = if is_compressed {
                decompress_message(&message_buf, client_id_for_logging)?
            } else {
                message_buf
            };

            let msg = ClientMessage::deserialize(&final_bytes);
            if msg.is_err() {
                eprintln!(
                    "Failed to deserialize MessagePack from {}",
                    client_id_for_logging
                );
            }
            msg
        }
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
            println!(
                "Connection closed by {} (EOF before header).",
                client_id_for_logging
            );
            Ok(None)
        }
        Err(e) => {
            eprintln!(
                "Error reading message header from {}: {}",
                client_id_for_logging, e
            );
            Err(e)
        }
    }
}

fn decompress_message(message_buf: &[u8], client_id: &str) -> io::Result<Vec<u8>> {
    zstd::decode_all(message_buf).map_err(|e| {
        eprintln!("Failed to decompress Zstd data from {}: {}", client_id, e);
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Zstd decompression error: {}", e),
        )
    })
}
