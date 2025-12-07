use crate::{Scene, lang::LanguageCenter, schedule::playback::PlaybackState};
use client::ClientMessage;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    io::ErrorKind,
    sync::{
        atomic::{AtomicBool, Ordering}, Arc
    }, thread,
};
use tokio::time::Duration;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    select, signal,
    sync::{Mutex, watch},
};

use crate::{
    clock::{Clock, ClockServer, SyncTime},
    device_map::DeviceMap,
    schedule::{SchedulerMessage, SovaNotification},
    {log_eprintln, log_println},
};

pub mod client;

mod message;
pub use message::ServerMessage;

/// Byte delimiter used to separate JSON messages in the TCP stream.
pub const ENDING_BYTE: u8 = 0x07;
/// Default name assigned to clients before they identify themselves.
pub const DEFAULT_CLIENT_NAME: &str = "Unknown musician";

/// Shared server state accessible by all connection handlers.
///
/// Contains references to core components like the clock, device map,
/// scheduler interfaces, and shared data like the client list and scene image.
#[derive(Clone)]
pub struct ServerState {
    /// Provides access to the shared Ableton Link-enabled clock.
    pub clock_server: Arc<ClockServer>,
    /// Manages connections to output devices (e.g., MIDI, OSC).
    pub devices: Arc<DeviceMap>,
    /// Sender for sending control messages to the `Scheduler` task.
    pub sched_iface: Sender<SchedulerMessage>,
    /// Watch channel sender used to broadcast server-wide notifications
    /// (e.g., scene updates, client list changes) to all connected clients.
    pub update_sender: watch::Sender<SovaNotification>,
    /// Watch channel receiver used by each client task to receive broadcasts
    /// sent via the `update_sender`.
    pub update_receiver: watch::Receiver<SovaNotification>,
    /// List of names of currently connected clients.
    /// Protected by a Mutex for safe concurrent access.
    pub clients: Arc<Mutex<Vec<String>>>,
    /// A snapshot of the current scene state, shared across threads.
    /// Updated by a dedicated maintenance thread listening to scheduler notifications.
    pub scene_image: Arc<Mutex<Scene>>,
    /// Handles compilers and interpreters
    pub languages: Arc<LanguageCenter>,
    pub is_playing: Arc<AtomicBool>,
}

impl ServerState {
    /// Creates a new `ServerState`.
    ///
    /// # Arguments
    ///
    /// * `scene_image` - The initial shared scene image.
    /// * `clock_server` - The shared clock server instance.
    /// * `devices` - The shared device map.
    /// * `sched_iface` - Sender channel to the `Scheduler` task.
    /// * `update_sender` - Sender part of the broadcast channel.
    /// * `update_receiver` - Receiver template for the broadcast channel.
    /// * `transcoder` - The shared script transcoder.
    /// * `shared_atomic_is_playing` - Shared flag indicating current transport status.
    pub fn new(
        scene_image: Arc<Mutex<Scene>>,
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        sched_iface: Sender<SchedulerMessage>,
        update_sender: watch::Sender<SovaNotification>,
        update_receiver: watch::Receiver<SovaNotification>,
        languages: Arc<LanguageCenter>,
    ) -> Self {
        ServerState {
            clock_server,
            devices,
            sched_iface,
            update_sender,
            update_receiver,
            clients: Arc::new(Mutex::new(Vec::new())),
            scene_image,
            languages,
            is_playing: Arc::new(AtomicBool::new(false))
        }
    }

}

/// Represents the main Sova TCP server application.
///
/// Responsible for binding to an address and port, accepting client connections,
/// and spawning tasks to handle each connection.
pub struct SovaCoreServer {
    /// The IP address the server will listen on (e.g., "127.0.0.1" or "0.0.0.0").
    pub ip: String,
    /// The TCP port number the server will listen on (e.g., 8080).
    pub port: u16,
    pub state: ServerState,
}

/// Represents a complete snapshot of the server's current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// The current scene state, including lines and scripts.
    pub scene: Scene,
    /// Tempo in beats per minute (BPM).
    pub tempo: f64,
    /// Current beat time within the Ableton Link session.
    pub beat: f64,
    /// Current microsecond time within the Ableton Link session.
    pub micros: SyncTime,
    /// The musical quantum (e.g., 4.0 for 4/4 time).
    pub quantum: f64,
}

/// Processes a received `ClientMessage` and returns a direct `ServerMessage` response.
///
/// This function handles the logic for each type of message a client can send.
/// It interacts with the scheduler, clock, or client list as needed and
/// sends appropriate notifications via the `update_sender`.
///
/// **Note:** This function only returns messages intended *directly* for the requesting
/// client (e.g., `Success`, `InternalError`, `ClockState` for `GetClock`).
/// Broadcast updates resulting from the message (e.g., `SceneValue`, `PeersUpdated`)
/// are handled separately via the `SovaNotification` mechanism.
///
/// # Arguments
/// * `msg` - The `ClientMessage` received from the client.
/// * `state` - A reference to the shared `ServerState`.
/// * `client_name` - A mutable reference to the name associated with this client connection.
///   This will be updated if the client sends `SetName`.
///
/// # Returns
/// The `ServerMessage` to be sent back directly to the requesting client.
async fn on_message(
    msg: ClientMessage,
    state: &ServerState,
    client_name: &mut String,
) -> ServerMessage {
    // Log the incoming request
    log_println!("[➡️ ] Client '{}' sent: {:?}", client_name, msg);

    match msg {
        ClientMessage::Chat(chat_msg) => {
            // Broadcast user chat message
            let _ = state
                .update_sender
                .send(SovaNotification::ChatReceived(
                    client_name.clone(),
                    chat_msg,
                ));
            ServerMessage::Success
        }
        ClientMessage::SetName(new_name) => {
            // Update client name in shared list and broadcast the change
            let mut clients_guard = state.clients.lock().await;
            let old_name = client_name.clone();
            let is_new_client = *client_name == DEFAULT_CLIENT_NAME;

            if is_new_client {
                log_println!("[👤] Client identified as: {}", new_name);
                clients_guard.push(new_name.clone());
            } else if let Some(i) = clients_guard.iter().position(|x| *x == old_name) {
                log_println!(
                    "[👤] Client {} changed name to {}",
                    clients_guard[i],
                    new_name
                );
                clients_guard[i] = new_name.clone();
            } else {
                // Should not happen if client is not new, but handle defensively
                log_eprintln!(
                    "[!] Error: Could not find old name '{}' to replace. Adding '{}'.",
                    old_name,
                    new_name
                );
                clients_guard.push(new_name.clone());
            }
            *client_name = new_name; // Update local name for this connection task

            let updated_clients = clients_guard.clone();
            drop(clients_guard); // Release lock before sending notification

            // Broadcast the updated client list
            let _ = state
                .update_sender
                .send(SovaNotification::ClientListChanged(updated_clients));

            ServerMessage::Success
        }
        ClientMessage::SchedulerControl(sched_msg) => {
            // Forward control message directly to scheduler
            if state.sched_iface.send(sched_msg).is_ok() {
                ServerMessage::Success
            } else {
                log_eprintln!("[!] Failed to send SchedulerControl message.");
                ServerMessage::InternalError("Failed to send command to scheduler.".to_string())
            }
        }
        ClientMessage::SetTempo(tempo, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::SetTempo(tempo, timing))
                .is_err()
            {
                log_eprintln!("[!] Failed to send SetTempo to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            // Tempo changes might need immediate feedback even if deferred in scheduler?
            // If so, we *could* send a TempoChanged notification here, but let's stick
            // to the scheduler handling notifications for consistency for now.
            ServerMessage::Success
        }
        ClientMessage::GetClock => {
            // Return current clock state directly
            let clock = Clock::from(&state.clock_server);
            ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum())
        }
        ClientMessage::GetScene => {
            // Return current scene snapshot directly
            ServerMessage::SceneValue(state.scene_image.lock().await.clone())
        }
        ClientMessage::GetPeers => {
            // Return current client list directly
            ServerMessage::PeersUpdated(state.clients.lock().await.clone())
        }
        ClientMessage::SetScene(scene, timing) => {
            // Forward the processed scene to the scheduler
            if state
                .sched_iface
                .send(SchedulerMessage::SetScene(scene, timing))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                log_eprintln!("[!] Failed to send Setscene to scheduler.");
                ServerMessage::InternalError(
                    "Failed to apply scene update to scheduler.".to_string(),
                )
            }
        }
        ClientMessage::RemoveFrame(line_id, position, timing) => {
            // Forward to scheduler
            if state
                .sched_iface
                .send(SchedulerMessage::RemoveFrame(line_id, position, timing))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                log_eprintln!("[!] Failed to send RemoveLine to scheduler.");
                ServerMessage::InternalError(
                    "Failed to send remove line update to scheduler.".to_string(),
                )
            }
        }
        ClientMessage::GetSnapshot => {
            // Get scene and clock state to build the snapshot
            let scene = state.scene_image.lock().await.clone();
            let clock = Clock::from(&state.clock_server);
            let snapshot = Snapshot {
                scene,
                tempo: clock.tempo(),
                beat: clock.beat(),
                micros: clock.micros(),
                quantum: clock.quantum(),
            };
            ServerMessage::Snapshot(snapshot)
        }
        ClientMessage::StartedEditingFrame(line_idx, frame_idx) => {
            // Broadcast notification that this client started editing
            let _ = state
                .update_sender
                .send(SovaNotification::PeerStartedEditingFrame(
                    client_name.clone(),
                    line_idx,
                    frame_idx,
                ));
            ServerMessage::Success // Acknowledge receipt
        }
        ClientMessage::StoppedEditingFrame(line_idx, frame_idx) => {
            // Broadcast notification that this client stopped editing
            let _ = state
                .update_sender
                .send(SovaNotification::PeerStoppedEditingFrame(
                    client_name.clone(),
                    line_idx,
                    frame_idx,
                ));
            ServerMessage::Success // Acknowledge receipt
        }
        ClientMessage::TransportStart(timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::TransportStart(timing))
                .is_err()
            {
                log_eprintln!("[!] Failed to send TransportStart to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            // Revert: No longer send immediate status based on atomic
            ServerMessage::Success
        }
        ClientMessage::TransportStop(timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::TransportStop(timing))
                .is_err()
            {
                log_eprintln!("[!] Failed to send TransportStop to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            // Revert: No longer send immediate status based on atomic
            ServerMessage::Success
        }
        ClientMessage::RequestDeviceList => {
            log_println!("[ info ] Client '{}' requested device list.", client_name);
            // Send back the current list obtained from device_map
            ServerMessage::DeviceList(state.devices.device_list())
        }
        ClientMessage::ConnectMidiDeviceByName(device_name) => {
            // Use the new bidirectional connect method
            match state.devices.connect_midi_by_name(&device_name) {
                Ok(_) => {
                    // Trigger broadcast update first
                    let updated_list = state.devices.device_list();
                    let _ = state
                        .update_sender
                        .send(SovaNotification::DeviceListChanged(
                            updated_list.clone(),
                        ));
                    // Send the updated list directly back to the requester
                    ServerMessage::DeviceList(updated_list)
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to connect device '{}': {}",
                    device_name, e
                )),
            }
        }
        ClientMessage::DisconnectMidiDeviceByName(device_name) => {
            // Use the new bidirectional disconnect method
            match state.devices.disconnect_midi_by_name(&device_name) {
                Ok(_) => {
                    // Trigger broadcast update first
                    let updated_list = state.devices.device_list();
                    let _ = state
                        .update_sender
                        .send(SovaNotification::DeviceListChanged(
                            updated_list.clone(),
                        ));
                    // Send the updated list directly back to the requester
                    ServerMessage::DeviceList(updated_list)
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to disconnect device '{}': {}",
                    device_name, e
                )),
            }
        }
        ClientMessage::CreateVirtualMidiOutput(device_name) => {
            // Use the new bidirectional virtual port creation method
            match state.devices.create_virtual_midi_port(&device_name) {
                Ok(_) => {
                    // Trigger broadcast update first
                    let updated_list = state.devices.device_list();
                    let _ = state
                        .update_sender
                        .send(SovaNotification::DeviceListChanged(
                            updated_list.clone(),
                        ));
                    // Send the updated list directly back to the requester
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
                        .send(SovaNotification::DeviceListChanged(
                            updated_list.clone(),
                        ));
                    ServerMessage::DeviceList(updated_list) // Send updated list confirming assignment
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
                        .send(SovaNotification::DeviceListChanged(
                            updated_list.clone(),
                        ));
                    ServerMessage::DeviceList(updated_list) // Send updated list confirming unassignment
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
                        .send(SovaNotification::DeviceListChanged(
                            updated_list.clone(),
                        ));
                    ServerMessage::DeviceList(updated_list)
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to create OSC device '{}': {}",
                    name, e
                )),
            }
        }
        ClientMessage::RemoveOscDevice(name) => {
            match state.devices.remove_output_device(&name) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    let _ = state
                        .update_sender
                        .send(SovaNotification::DeviceListChanged(
                            updated_list.clone(),
                        ));
                    ServerMessage::DeviceList(updated_list)
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to remove OSC device '{}': {}",
                    name, e
                )),
            }
        }
        ClientMessage::GetLine(line_id) => {
            let scene = state.scene_image.lock().await;
            if let Some(line) = scene.line(line_id) {
                ServerMessage::LineValues(vec![(line_id, line.clone())])
            } else {
                ServerMessage::InternalError(format!(
                    "No line at index {}",
                    line_id
                ))
            }
        },
        ClientMessage::SetLines(lines, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::SetLines(lines, timing))
                .is_err()
            {
                log_eprintln!("[!] Failed to send SetLines to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            // Revert: No longer send immediate status based on atomic
            ServerMessage::Success
        },
        ClientMessage::ConfigureLines(lines, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::ConfigureLines(lines, timing))
                .is_err()
            {
                log_eprintln!("[!] Failed to send ConfigureLines to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            // Revert: No longer send immediate status based on atomic
            ServerMessage::Success
        },
        ClientMessage::AddLine(line_id, line, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::AddLine(line_id, line, timing))
                .is_err()
            {
                log_eprintln!("[!] Failed to send AddLine to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            // Revert: No longer send immediate status based on atomic
            ServerMessage::Success
        },
        ClientMessage::RemoveLine(line_id, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::RemoveLine(line_id, timing))
                .is_err()
            {
                log_eprintln!("[!] Failed to send RemoveLine to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            // Revert: No longer send immediate status based on atomic
            ServerMessage::Success
        },
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
        },
        ClientMessage::SetFrames(frames, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::SetFrames(frames, timing))
                .is_err()
            {
                log_eprintln!("[!] Failed to send SetFrames to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            // Revert: No longer send immediate status based on atomic
            ServerMessage::Success
        },
        ClientMessage::AddFrame(line_id, frame_id, frame, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::AddFrame(line_id, frame_id, frame, timing))
                .is_err()
            {
                log_eprintln!("[!] Failed to send AddFrame to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            // Revert: No longer send immediate status based on atomic
            ServerMessage::Success
        },
    }
}

/// Serializes a `ServerMessage` to MessagePack, optionally compresses it using Zstd,
/// and sends it with a 4-byte length prefix with compression flag to the client's output stream.
async fn send_msg<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: ServerMessage) -> io::Result<()> {
    // Serialize to MessagePack
    let msgpack_bytes = rmp_serde::to_vec_named(&msg).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize ServerMessage to MessagePack: {}", e),
        )
    })?;

    // Determine compression strategy based on message semantics
    let (final_bytes, is_compressed) = compress_message_intelligently(&msg, &msgpack_bytes)?;

    // Prepare length prefix with compression flag
    let mut len = final_bytes.len() as u32;
    if is_compressed {
        len |= 0x80000000; // Set high bit to indicate compression
    }

    // Send length prefix and data
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(&final_bytes).await?;
    writer.flush().await?;

    Ok(())
}

/// Intelligently compresses message based on type and content
fn compress_message_intelligently(
    msg: &ServerMessage,
    msgpack_bytes: &[u8],
) -> io::Result<(Vec<u8>, bool)> {
    use crate::server::client::CompressionStrategy;

    match msg.compression_strategy() {
        CompressionStrategy::Never => {
            // Never compress frequent/small messages
            Ok((msgpack_bytes.to_vec(), false))
        }
        CompressionStrategy::Always => {
            // Always compress large content, but only if beneficial
            if msgpack_bytes.len() > 64 {
                let compression_level = if msgpack_bytes.len() < 1024 { 1 } else { 3 };
                let compressed = zstd::encode_all(msgpack_bytes, compression_level)
                    .map_err(|e| io::Error::other(format!("Compression failed: {}", e)))?;
                // Only use compressed if it's actually smaller
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
            // Original size-based logic
            if msgpack_bytes.len() < 256 {
                Ok((msgpack_bytes.to_vec(), false))
            } else {
                let compression_level = if msgpack_bytes.len() < 1024 { 1 } else { 3 };
                let compressed = zstd::encode_all(msgpack_bytes, compression_level)
                    .map_err(|e| io::Error::other(format!("Compression failed: {}", e)))?;
                Ok((compressed, true))
            }
        }
    }
}

impl SovaCoreServer {
    /// Creates a new `SovaCoreServer` instance with the specified address and port.
    pub fn new(ip: String, port: u16, state: ServerState) -> Self {
        SovaCoreServer { ip, port, state }
    }

    /// Starts the TCP server, listens for connections, and handles graceful shutdown.
    ///
    /// This function enters the main server loop, accepting new connections and
    /// spawning `process_client` tasks. It also listens for a Ctrl+C signal
    /// to initiate a shutdown.
    ///
    /// # Arguments
    /// * `state` - The shared `ServerState` to be cloned for each client task.
    pub async fn start(&self, scheduler_notifications: Receiver<SovaNotification>) -> io::Result<()> {
        let addr = format!("{}:{}", self.ip, self.port);
        let listener = TcpListener::bind(&addr).await?;
        log_println!("[+] Server listening on {}", addr);
        self.start_image_maintainer(scheduler_notifications);
        loop {
            select! {
                // Accept new TCP connections
                Ok((socket, client_addr)) = listener.accept() => {
                    log_println!("[🔌] New connection from {}", client_addr);
                    let client_state = self.state.clone(); // Clone state for the new task
                    // Spawn a task to handle this client independently
                    tokio::spawn(async move {
                        match process_client(socket, client_state).await {
                            Ok(client_name) => {
                            // Log graceful disconnection
                            log_println!("[🔌] Client '{}' disconnected.", client_name);
                            },
                            Err(e) => {
                                // Log errors during client processing
                                log_eprintln!("[!] Error handling client {}: {}", client_addr, e);
                            }
                        }
                    });
                }
                // Handle Ctrl+C for graceful shutdown
                _ = signal::ctrl_c() => {
                    log_println!("\n[!] Ctrl+C received, shutting down server...");
                    break; // Exit the main loop
                }
                // Avoid 100% CPU usage if no events occur
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    if self.state.update_sender.send(SovaNotification::Tick).is_err() {
                        break;
                    }
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
                                guard.line_mut(*line_id).insert_frame(*frame_id, frame.clone());
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
                        let _ = update_sender.send(p);
                    }
                    Err(_) => break,
                }
            }
        });
    }

}

/// Handles the lifecycle of a single client connection.
///
/// This function manages reading messages from the client, processing them via `on_message`,
/// sending direct responses, listening for broadcast notifications, and handling disconnection.
///
/// # Arguments
/// * `socket` - The `TcpStream` for the connected client.
/// * `state` - A clone of the shared `ServerState`.
///
/// # Returns
/// An `io::Result` containing the final name of the client upon disconnection, or an `io::Error`.
async fn process_client(socket: TcpStream, state: ServerState) -> io::Result<String> {
    let client_addr = socket.peer_addr()?;
    let client_addr_str = client_addr.to_string(); // For logging before name is set
    let (reader, writer) = socket.into_split(); // Split into read/write halves
    let mut reader = BufReader::with_capacity(32 * 1024, reader);
    let mut writer = BufWriter::with_capacity(32 * 1024, writer);
    let mut client_name = DEFAULT_CLIENT_NAME.to_string(); // Start with default name

    let mut clock = Clock::from(&state.clock_server);

    // --- Handshake: Expect SetName first ---
    let hello_msg: ServerMessage; // Declare hello_msg variable

    match read_message_internal(&mut reader, &client_addr_str).await {
        Ok(Some(ClientMessage::SetName(new_name))) => {
            // Validate name (e.g., non-empty, allowed characters, uniqueness)
            if new_name.is_empty() || new_name == DEFAULT_CLIENT_NAME {
                log_eprintln!(
                    "[!] Connection rejected: Invalid username '{}' from {}",
                    new_name,
                    client_addr_str
                );
                let refuse_msg = ServerMessage::ConnectionRefused(
                    "Invalid username (empty or reserved).".to_string(),
                );
                let _ = send_msg(&mut writer, refuse_msg).await; // Attempt to notify client
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid username",
                ));
            }

            // Check for uniqueness
            let mut clients_guard = state.clients.lock().await;
            if clients_guard.iter().any(|name| name == &new_name) {
                log_eprintln!(
                    "[!] Connection rejected: Username '{}' already taken by {}",
                    new_name,
                    client_addr_str
                );
                let refuse_msg = ServerMessage::ConnectionRefused(format!(
                    "Username '{}' is already taken.",
                    new_name
                ));
                let _ = send_msg(&mut writer, refuse_msg).await; // Attempt to notify client
                drop(clients_guard); // Release lock
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Username taken",
                ));
            }

            // Name is valid and unique, accept connection
            client_name = new_name; // Assign the validated name
            log_println!(
                "[👤] Client {} identified as: {}",
                client_addr_str,
                client_name
            );
            clients_guard.push(client_name.clone());

            // --- Get initial data AFTER adding client ---
            let initial_scene = state.scene_image.lock().await.clone();
            let initial_devices = state.devices.device_list();
            let initial_peers = clients_guard.clone(); // Get updated list including new client
            let updated_peers_for_broadcast = initial_peers.clone(); // Clone for broadcast

            drop(clients_guard); // Release lock

            // Broadcast the updated client list
            let _ = state
                .update_sender
                .send(SovaNotification::ClientListChanged(
                    updated_peers_for_broadcast,
                ));

            // --- THEN fetch dynamic state like clock/playing status ---
            
            let initial_link_state = (
                clock.tempo(),
                clock.beat(),
                clock.beat() % clock.quantum(),
                state.clock_server.link.num_peers() as u32, // Cast u64 to u32
                state.clock_server.link.is_start_stop_sync_enabled(),
            );
            let initial_is_playing = state.is_playing.load(Ordering::Relaxed);

            // --- Get available compilers and their syntax definitions ---
            let available_languages : Vec<String> = state.languages.languages().map(str::to_owned).collect();
            let mut syntax_definitions = std::collections::HashMap::new();
            for lang in available_languages.iter() {
                if let Some(compiler) = state.languages.transcoder.compilers.get(lang) {
                    if let Some(Cow::Borrowed(content)) = compiler.syntax() {
                        syntax_definitions.insert(lang.clone(), content.to_string());
                    }
                }
            }

            // --- Construct the Hello message ---
            log_println!(
                "[ handshake ] Sending Hello to {} ({}). Initial is_playing state: {}",
                client_addr_str,
                client_name,
                initial_is_playing
            );
            hello_msg = ServerMessage::Hello {
                username: client_name.clone(), // Send the *accepted* name
                scene: initial_scene,
                devices: initial_devices,
                peers: initial_peers, // Send the updated list
                link_state: initial_link_state,
                is_playing: initial_is_playing,
                available_languages,
                syntax_definitions,
            };

            // Send Hello
            if send_msg(&mut writer, hello_msg).await.is_err() {
                log_eprintln!("[!] Failed to send Hello to {}", client_name);
                // Don't remove from list yet, cleanup will handle it
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero, // Or other appropriate error
                    "Failed to send Hello message",
                ));
            }
        }
        Ok(Some(other_msg)) => {
            // First message was not SetName
            log_eprintln!(
                "[!] Connection rejected: Expected SetName, received {:?} from {}",
                other_msg,
                client_addr_str
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
            // Connection closed during handshake before sending SetName
            log_println!(
                "[🔌] Connection closed by {} during handshake.",
                client_addr_str
            );
            return Ok(client_name); // Return default name as it wasn't set
        }
        Err(e) => {
            // Read error during handshake
            log_eprintln!(
                "[!] Read error during handshake with {}: {}",
                client_addr_str,
                e
            );
            return Err(e);
        }
    }

    // --- Main Loop: Read client messages and listen for broadcasts ---
    let mut update_receiver = state.update_receiver.clone(); // Clone receiver for this task

    loop {
        select! {
            // Prioritize reading client messages
            biased;

            // Branch for reading subsequent client data
            read_result = read_message_internal(&mut reader, &client_name) => {
                match read_result {
                    Ok(Some(msg)) => {
                        // Handle SetName again? Or disallow after handshake?
                        // For now, let's allow name changes via the main handler.
                        let response = on_message(msg, &state, &mut client_name).await;

                        // Avoid sending Success for SetName handled during handshake?
                        // The `on_message` for SetName already handles broadcasting.
                        // Let's check if the response is just a placeholder Success from SetName
                        // If we modify on_message SetName to return something else (like NoResponse),
                        // we could skip sending here. For now, we send Success.
                        if send_msg(&mut writer, response).await.is_err() {
                            log_eprintln!("[!] Failed write direct response to {}", client_name);
                            break; // Assume connection broken
                        }
                    },
                    Ok(None) => {
                        // Clean disconnect (EOF)
                        log_println!("[🔌] Connection closed cleanly by {}.", client_name);
                        break;
                    },
                    Err(_e) => {
                        // Read error occurred and was logged by read_message_internal
                        log_eprintln!("[!] Read error for client {}. Closing connection.", client_name);
                        break; // Break the loop on error
                    }
                }
            }

            // Listen for broadcast notifications from the server
            update_result = update_receiver.changed() => {
                if update_result.is_err() {
                    break;
                }
                let notification = update_receiver.borrow().clone();
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
                        // Don't send the update back to the originator
                        if sender_name != *client_name {
                            Some(ServerMessage::PeerStartedEditing(sender_name, line_idx, frame_idx))
                        } else {
                            None
                        }
                    }
                    SovaNotification::PeerStoppedEditingFrame(sender_name, line_idx, frame_idx) => {
                        // Don't send the update back to the originator
                        if sender_name != *client_name {
                            Some(ServerMessage::PeerStoppedEditing(sender_name, line_idx, frame_idx))
                        } else {
                            None
                        }
                    }
                    // Add handler for DeviceListChanged
                    SovaNotification::DeviceListChanged(devices) => {
                        log_println!("[ broadcast ] Sending updated device list ({} devices) to {}", devices.len(), client_name);
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

    // --- Cleanup after loop breaks ---
    log_println!("[🔌] Cleaning up connection for client: {}", client_name);
    // Only remove the client if they successfully completed the handshake (i.e., name is not default)
    if client_name != DEFAULT_CLIENT_NAME {
        let mut clients_guard = state.clients.lock().await;
        if let Some(i) = clients_guard.iter().position(|x| *x == client_name) {
            clients_guard.remove(i);
            log_println!("[👤] Removed {} from client list.", client_name);
            // Broadcast the updated client list after removal
            let updated_clients = clients_guard.clone();
            drop(clients_guard); // Drop lock before sending notification
            let _ = state
                .update_sender
                .send(SovaNotification::ClientListChanged(updated_clients));
        } else {
            // This case might happen if the client disconnected right after handshake
            // before the main loop really started, or if there's a race condition.
            log_eprintln!(
                "[!] Client '{}' not found in list during cleanup, though name was set.",
                client_name
            );
        }
    } else {
        log_println!(
            "[🔌] Client disconnected before setting a name (still '{}'). No list removal needed.",
            DEFAULT_CLIENT_NAME
        );
    }

    Ok(client_name) // Return the final name for logging by the caller
}

/// Helper function to read a single message with support for both old and new header formats.
/// Returns Ok(None) if the connection is closed cleanly (EOF on length read).
/// Returns Err for other IO errors or deserialization failures.
async fn read_message_internal<R: AsyncReadExt + Unpin>(
    reader: &mut R,
    client_id_for_logging: &str,
) -> io::Result<Option<ClientMessage>> {
    // Read old 4-byte header format: [length_with_compression_flag: u32]
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {
            let len_with_flag = u32::from_be_bytes(len_buf);
            let is_compressed = (len_with_flag & 0x80000000) != 0;
            let length = len_with_flag & 0x7FFFFFFF;

            if length == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Received zero-length message header",
                ));
            }

            // Read message body
            let mut message_buf = vec![0u8; length as usize];
            reader.read_exact(&mut message_buf).await?;

            // Decompress if needed
            let final_bytes = if is_compressed {
                decompress_message(&message_buf, client_id_for_logging)?
            } else {
                message_buf
            };

            // Deserialize MessagePack
            let msg = ClientMessage::deserialize(&final_bytes);
            if msg.is_err() {
                log_eprintln!(
                    "[!] Failed to deserialize MessagePack from {}",
                    client_id_for_logging
                );
            }
            msg
        }
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
            log_println!(
                "[🔌] Connection closed by {} (EOF before header).",
                client_id_for_logging
            );
            Ok(None) // Indicate clean closure
        }
        Err(e) => {
            log_eprintln!(
                "[!] Error reading message header from {}: {}",
                client_id_for_logging,
                e
            );
            Err(e)
        }
    }
}

/// Decompresses a message buffer using Zstd
fn decompress_message(message_buf: &[u8], client_id: &str) -> io::Result<Vec<u8>> {
    zstd::decode_all(message_buf).map_err(|e| {
        log_eprintln!(
            "[!] Failed to decompress Zstd data from {}: {}",
            client_id,
            e
        );
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Zstd decompression error: {}", e),
        )
    })
}