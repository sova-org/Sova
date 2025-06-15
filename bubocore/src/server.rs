use crate::scene::script::Script;
use client::ClientMessage;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    io::ErrorKind,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
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
    compiler::CompilationError,
    device_map::DeviceMap,
    protocol::message::TimedMessage,
    scene::Scene,
    schedule::{
        message::SchedulerMessage,
        notification::SchedulerNotification
    },
    shared_types::{DeviceInfo, GridSelection},
    transcoder::Transcoder,
};

pub mod client;

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
    /// Sender for transmitting timed messages (often OSC) to the `World` task.
    pub world_iface: Sender<TimedMessage>,
    /// Sender for sending control messages to the `Scheduler` task.
    pub sched_iface: Sender<SchedulerMessage>,
    /// Watch channel sender used to broadcast server-wide notifications
    /// (e.g., scene updates, client list changes) to all connected clients.
    pub update_sender: watch::Sender<SchedulerNotification>,
    /// Watch channel receiver used by each client task to receive broadcasts
    /// sent via the `update_sender`.
    pub update_receiver: watch::Receiver<SchedulerNotification>,
    /// List of names of currently connected clients.
    /// Protected by a Mutex for safe concurrent access.
    pub clients: Arc<Mutex<Vec<String>>>,
    /// A snapshot of the current scene state, shared across threads.
    /// Updated by a dedicated maintenance thread listening to scheduler notifications.
    pub scene_image: Arc<Mutex<Scene>>,
    /// Handles script compilation (e.g., Baliscript).
    pub transcoder: Arc<Mutex<Transcoder>>,
    /// Shared flag indicating current transport status, updated by the Scheduler.
    pub shared_atomic_is_playing: Arc<AtomicBool>,
}

impl ServerState {
    /// Creates a new `ServerState`.
    ///
    /// # Arguments
    ///
    /// * `scene_image` - The initial shared scene image.
    /// * `clock_server` - The shared clock server instance.
    /// * `devices` - The shared device map.
    /// * `world_iface` - Sender channel to the `World` task.
    /// * `sched_iface` - Sender channel to the `Scheduler` task.
    /// * `update_sender` - Sender part of the broadcast channel.
    /// * `update_receiver` - Receiver template for the broadcast channel.
    /// * `transcoder` - The shared script transcoder.
    /// * `shared_atomic_is_playing` - Shared flag indicating current transport status.
    pub fn new(
        scene_image: Arc<Mutex<Scene>>,
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        world_iface: Sender<TimedMessage>,
        sched_iface: Sender<SchedulerMessage>,
        update_sender: watch::Sender<SchedulerNotification>,
        update_receiver: watch::Receiver<SchedulerNotification>,
        transcoder: Arc<Mutex<Transcoder>>,
        shared_atomic_is_playing: Arc<AtomicBool>,
    ) -> Self {
        ServerState {
            clock_server,
            devices,
            world_iface,
            sched_iface,
            update_sender,
            update_receiver,
            clients: Arc::new(Mutex::new(Vec::new())),
            scene_image,
            transcoder,
            shared_atomic_is_playing,
        }
    }
}

/// Represents the main BuboCore TCP server application.
///
/// Responsible for binding to an address and port, accepting client connections,
/// and spawning tasks to handle each connection.
pub struct BuboCoreServer {
    /// The IP address the server will listen on (e.g., "127.0.0.1" or "0.0.0.0").
    pub ip: String,
    /// The TCP port number the server will listen on (e.g., 8080).
    pub port: u16,
}

/// Represents messages sent FROM the server TO a client.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    /// Initial greeting message sent upon successful connection.
    /// Includes necessary state for the client to initialize.
    Hello {
        /// The client's assigned username.
        username: String,
        /// The current scene state.
        scene: Scene,
        /// List of available/connected devices.
        devices: Vec<DeviceInfo>,
        /// List of names of other currently connected clients.
        peers: Vec<String>,
        /// Current Link state (tempo, beat, phase, peers, is_enabled).
        link_state: (f64, f64, f64, u32, bool),
        /// Current transport playing state.
        is_playing: bool,
        /// List of available compiler names.
        available_compilers: Vec<String>,
        /// Map of compiler name to its .sublime-syntax content.
        syntax_definitions: std::collections::HashMap<String, String>,
    },
    /// Broadcast containing the updated list of connected client names.
    PeersUpdated(Vec<String>),
    /// Broadcasts an update to a specific peer's grid selection.
    PeerGridSelectionUpdate(String, GridSelection),
    /// Broadcasts that a peer started editing a specific frame.
    PeerStartedEditing(String, usize, usize),
    /// Broadcasts that a peer stopped editing a specific frame.
    PeerStoppedEditing(String, usize, usize),
    /// Sends the requested script content to the client.
    ScriptContent {
        line_idx: usize,
        frame_idx: usize,
        content: String,
    },
    /// Confirms a script was successfully compiled and uploaded.
    ScriptCompiled { line_idx: usize, frame_idx: usize },
    /// Sends compilation error details back to the client.
    CompilationErrorOccurred(CompilationError),
    /// Indicates the transport playback has started.
    TransportStarted,
    /// Indicates the transport playback has stopped.
    TransportStopped,
    /// A log message originating from the server or scheduler.
    LogString(String),
    /// A chat message broadcast from another client or the server itself.
    Chat(String),
    /// Generic success response, indicating a requested action was accepted.
    Success,
    /// Indicates an internal server error occurred while processing a request.
    InternalError(String),
    /// Indicate connection refused (e.g., username taken).
    ConnectionRefused(String),
    /// A complete snapshot of the current server state (used for save/load?).
    Snapshot(Snapshot),
    /// Sends the full list of available/connected devices (can be requested).
    DeviceList(Vec<DeviceInfo>),
    /// tempo, beat, micros, quantum
    ClockState(f64, f64, SyncTime, f64),
    /// Broadcast containing the complete current state of the scene.
    SceneValue(Scene),
    /// The current length of the scene.
    SceneLength(usize),
    /// The current frame positions within each line (line_idx, frame_idx, repetition_idx)
    FramePosition(Vec<(usize, usize, usize)>),
}

impl ServerMessage {
    /// Get the compression strategy for this message type based on semantics
    pub fn compression_strategy(&self) -> crate::server::client::CompressionStrategy {
        use crate::server::client::CompressionStrategy;
        match self {
            // Real-time/frequent messages that should never be compressed
            ServerMessage::PeerGridSelectionUpdate(_, _) |
            ServerMessage::PeerStartedEditing(_, _, _) |
            ServerMessage::PeerStoppedEditing(_, _, _) |
            ServerMessage::ClockState(_, _, _, _) |
            ServerMessage::SceneLength(_) |
            ServerMessage::FramePosition(_) |
            ServerMessage::TransportStarted |
            ServerMessage::TransportStopped => CompressionStrategy::Never,
            
            // Large content messages that should always be compressed if beneficial
            ServerMessage::Hello { .. } |
            ServerMessage::SceneValue(_) |
            ServerMessage::Snapshot(_) |
            ServerMessage::DeviceList(_) => CompressionStrategy::Always,
            
            // Everything else uses adaptive compression
            _ => CompressionStrategy::Adaptive,
        }
    }
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
/// are handled separately via the `SchedulerNotification` mechanism.
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
    println!("[➡️ ] Client '{}' sent: {:?}", client_name, msg);

    match msg {
        ClientMessage::EnableFrames(line_id, frames, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::EnableFrames(line_id, frames, timing))
                .is_err()
            {
                eprintln!("[!] Failed to send EnableFrames to scheduler.");
            }
            ServerMessage::Success
        }
        ClientMessage::DisableFrames(line_id, frames, timing) => {
            // Forward to scheduler with the vector of frames
            if state
                .sched_iface
                .send(SchedulerMessage::DisableFrames(line_id, frames, timing))
                .is_err()
            {
                eprintln!("[!] Failed to send DisableFrames to scheduler.");
            }
            ServerMessage::Success
        }
        ClientMessage::SetScript(line_id, frame_id, script_content, timing) => {
            // 1. Determine the correct language for this frame
            let lang_to_use: String; // Declare variable to hold the final language
            {
                // Scope for scene_image lock
                let scene_image = state.scene_image.lock().await;
                let lang_opt = scene_image
                    .lines
                    .get(line_id)
                    .and_then(|l| l.scripts.iter().find(|s| s.index == frame_id))
                    .map(|s| s.lang.clone()); // Get Option<String>

                if let Some(lang) = lang_opt {
                    lang_to_use = lang;
                } else {
                    // Drop the scene_image lock before the async fallback
                    drop(scene_image);
                    // Asynchronous fallback logic
                    eprintln!(
                        "[!] SetScript: Could not find script for ({}, {}) to determine language. Using default.",
                        line_id, frame_id
                    );
                    // Fallback to the transcoder's active compiler or a hardcoded default
                    lang_to_use = state
                        .transcoder
                        .lock()
                        .await
                        .active_compiler
                        .clone()
                        .unwrap_or_else(|| "bali".to_string()); // Final fallback
                }
            } // scene_image lock is implicitly dropped here if not already dropped

            // 2. Compile using the determined language
            match state
                .transcoder
                .lock()
                .await
                .compile(&script_content, &lang_to_use)
            {
                Ok(compiled_script) => {
                    // 3. Create the Script object with the correct language
                    let script = Script::new(
                        script_content,
                        compiled_script,
                        lang_to_use, // Use the determined language here
                        frame_id,
                    );
                    // 4. Send to scheduler
                    if state
                        .sched_iface
                        .send(SchedulerMessage::UploadScript(
                            line_id, frame_id, script, timing,
                        ))
                        .is_err()
                    {
                        eprintln!("[!] Failed to send UploadScript to scheduler.");
                        ServerMessage::InternalError("Scheduler communication error.".to_string())
                    } else {
                        // Send ScriptCompiled confirmation back to client
                        ServerMessage::ScriptCompiled {
                            line_idx: line_id,
                            frame_idx: frame_id,
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[!] Script compilation failed for Line {}, Frame {} (Lang: {}): {}",
                        line_id, frame_id, lang_to_use, e
                    );
                    // Extract CompilationError if possible, otherwise send generic InternalError
                    match e {
                        crate::transcoder::TranscoderError::CompilationFailed(comp_err) => {
                            ServerMessage::CompilationErrorOccurred(comp_err)
                        }
                        _ => ServerMessage::InternalError(format!(
                            "Script compilation error (Lang: {}): {}",
                            lang_to_use, e
                        )),
                    }
                }
            }
        }
        ClientMessage::GetScript(line_idx, frame_idx) => {
            // Lock the scene image to read the script content
            let scene = state.scene_image.lock().await;
            match scene.lines.get(line_idx) {
                Some(line) => {
                    // Find the script Arc with the matching index (frame_idx) within the line's scripts vector
                    let script_opt = line
                        .scripts
                        .iter()
                        .find(|script_arc| script_arc.index == frame_idx);

                    match script_opt {
                        Some(script_arc) => {
                            // Found the script Arc, get the content from the inner Script
                            ServerMessage::ScriptContent {
                                line_idx,
                                frame_idx,
                                content: script_arc.content.clone(),
                            }
                        }
                        None => {
                            // Line valid, but no script found for this specific frame_idx
                            eprintln!(
                                "[!] No script found for Line {}, Frame {}",
                                line_idx, frame_idx
                            );
                            // Send back a placeholder script content
                            ServerMessage::ScriptContent {
                                line_idx,
                                frame_idx,
                                content: format!(
                                    "// No script found for Line {}, Frame {}",
                                    line_idx, frame_idx
                                ),
                            }
                        }
                    }
                }
                None => {
                    // Line index out of bounds
                    eprintln!("[!] Invalid line index {} requested for script.", line_idx);
                    ServerMessage::InternalError(format!("Invalid line index: {}", line_idx))
                }
            }
        }
        ClientMessage::Chat(chat_msg) => {
            // Broadcast user chat message
            let _ = state
                .update_sender
                .send(SchedulerNotification::ChatReceived(
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
                println!("[👤] Client identified as: {}", new_name);
                clients_guard.push(new_name.clone());
            } else if let Some(i) = clients_guard.iter().position(|x| *x == old_name) {
                println!(
                    "[👤] Client {} changed name to {}",
                    clients_guard[i], new_name
                );
                clients_guard[i] = new_name.clone();
            } else {
                // Should not happen if client is not new, but handle defensively
                eprintln!(
                    "[!] Error: Could not find old name '{}' to replace. Adding '{}'.",
                    old_name, new_name
                );
                clients_guard.push(new_name.clone());
            }
            *client_name = new_name; // Update local name for this connection task

            let updated_clients = clients_guard.clone();
            drop(clients_guard); // Release lock before sending notification

            // Broadcast the updated client list
            let _ = state
                .update_sender
                .send(SchedulerNotification::ClientListChanged(updated_clients));

            ServerMessage::Success
        }
        ClientMessage::SchedulerControl(sched_msg) => {
            // Forward control message directly to scheduler
            if state.sched_iface.send(sched_msg).is_ok() {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send SchedulerControl message.");
                ServerMessage::InternalError("Failed to send command to scheduler.".to_string())
            }
        }
        ClientMessage::SetTempo(tempo, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::SetTempo(tempo, timing))
                .is_err()
            {
                eprintln!("[!] Failed to send SetTempo to scheduler.");
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
        ClientMessage::SetScene(mut scene, timing) => {
            {
                // Scope for transcoder lock
                let transcoder = state.transcoder.lock().await;
                for line in scene.lines.iter_mut() {
                    for script_arc in line.scripts.iter_mut() {
                        // Compile using the language specified in the script data
                        match transcoder.compile(&script_arc.content, &script_arc.lang) {
                            Ok(compiled) => {
                                // We need exclusive access to modify the Arc's inner value
                                if let Some(script) = Arc::get_mut(script_arc) {
                                    script.compiled = compiled;
                                    // Ensure lang field is preserved (it should be already set)
                                } else {
                                    // This case might happen if the Arc is shared elsewhere unexpectedly.
                                    // Recreate the Arc preserving the language.
                                    eprintln!(
                                        "[!] Failed to get mutable access to script Arc during SetScene compilation. Line: {}, Frame: {}. Recreating Arc.",
                                        line.index, script_arc.index
                                    );
                                    // Fallback: Create a new Arc with the compiled script
                                    let new_script = Script::new(
                                        script_arc.content.clone(),
                                        compiled, // Use the successfully compiled instructions
                                        script_arc.lang.clone(), // Preserve original language
                                        script_arc.index,
                                    );
                                    *script_arc = Arc::new(new_script);
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "[!] Failed to pre-compile script (lang: {}) for Line {}, Frame {} during SetScene: {}",
                                    script_arc.lang, line.index, script_arc.index, e
                                );
                                // Optionally clear the compiled_script field if compilation fails
                                if let Some(script) = Arc::get_mut(script_arc) {
                                    script.compiled = Default::default();
                                } else {
                                    // As above, handle Arc sharing issues
                                    let mut new_script = (**script_arc).clone(); // Clone the inner Script
                                    new_script.compiled = Default::default();
                                    *script_arc = Arc::new(new_script);
                                }
                            }
                        }
                    }
                }
            } // Transcoder lock released here

            // Forward the processed scene to the scheduler
            if state
                .sched_iface
                .send(SchedulerMessage::SetScene(scene, timing))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send Setscene to scheduler.");
                ServerMessage::InternalError(
                    "Failed to apply scene update to scheduler.".to_string(),
                )
            }
        }
        ClientMessage::UpdateLineFrames(line_id, frames, timing) => {
            // Forward to scheduler
            if state
                .sched_iface
                .send(SchedulerMessage::UpdateLineFrames(line_id, frames, timing))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send UpdateLineFrames to scheduler.");
                ServerMessage::InternalError("Failed to send line update to scheduler.".to_string())
            }
        }
        ClientMessage::InsertFrame(line_id, position, duration, timing) => {
            // Forward to scheduler with the received duration
            if state
                .sched_iface
                .send(SchedulerMessage::InsertFrame(
                    line_id, position, duration, // Use the received duration
                    timing,
                ))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send InsertFrame to scheduler.");
                ServerMessage::InternalError(
                    "Failed to send insert frame update to scheduler.".to_string(),
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
                eprintln!("[!] Failed to send RemoveLine to scheduler.");
                ServerMessage::InternalError(
                    "Failed to send remove line update to scheduler.".to_string(),
                )
            }
        }
        ClientMessage::SetLineStartFrame(line_id, start_frame, timing) => {
            // Forward to scheduler
            if state
                .sched_iface
                .send(SchedulerMessage::SetLineStartFrame(
                    line_id,
                    start_frame,
                    timing,
                ))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send SetLineStartFrame to scheduler.");
                ServerMessage::InternalError(
                    "Failed to send line start frame update to scheduler.".to_string(),
                )
            }
        }
        ClientMessage::SetLineEndFrame(line_id, end_frame, timing) => {
            // Forward to scheduler
            if state
                .sched_iface
                .send(SchedulerMessage::SetLineEndFrame(
                    line_id, end_frame, timing,
                ))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send SetLineEndFrame to scheduler.");
                ServerMessage::InternalError(
                    "Failed to send line end frame update to scheduler.".to_string(),
                )
            }
        }
        ClientMessage::GetSnapshot => {
            // Get scene and clock state to build the snapshot
            let scene = state.scene_image.lock().await.clone();
            let clock = Clock::from(&state.clock_server);
            let snapshot = Snapshot {
                scene: scene,
                tempo: clock.tempo(),
                beat: clock.beat(),
                micros: clock.micros(),
                quantum: clock.quantum(),
            };
            ServerMessage::Snapshot(snapshot)
        }
        ClientMessage::UpdateGridSelection(selection) => {
            // Don't send a direct response, broadcast via notification
            let _ = state
                .update_sender
                .send(SchedulerNotification::PeerGridSelectionChanged(
                    client_name.clone(),
                    selection,
                ));
            // Return Success just to acknowledge receipt, though no client-side action needed for this specifically
            ServerMessage::Success
        }
        ClientMessage::StartedEditingFrame(line_idx, frame_idx) => {
            // Broadcast notification that this client started editing
            let _ = state
                .update_sender
                .send(SchedulerNotification::PeerStartedEditingFrame(
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
                .send(SchedulerNotification::PeerStoppedEditingFrame(
                    client_name.clone(),
                    line_idx,
                    frame_idx,
                ));
            ServerMessage::Success // Acknowledge receipt
        }
        ClientMessage::GetSceneLength => {
            // Read the length from the scene image
            let scene = state.scene_image.lock().await;
            ServerMessage::SceneLength(scene.length)
        }
        ClientMessage::SetSceneLength(length, timing) => {
            // Forward to scheduler
            if state
                .sched_iface
                .send(SchedulerMessage::SetSceneLength(length, timing))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send SetSceneLength to scheduler.");
                ServerMessage::InternalError(
                    "Failed to send scene length update to scheduler.".to_string(),
                )
            }
        }
        ClientMessage::SetLineLength(line_idx, length_opt, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::SetLineLength(
                    line_idx, length_opt, timing,
                ))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send SetLineLength to scheduler.");
                ServerMessage::InternalError(
                    "Failed to send line length update to scheduler.".to_string(),
                )
            }
        }
        ClientMessage::SetLineSpeedFactor(line_idx, speed_factor, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::SetLineSpeedFactor(
                    line_idx,
                    speed_factor,
                    timing,
                ))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send SetLineSpeedFactor to scheduler.");
                ServerMessage::InternalError(
                    "Failed to send line speed factor update to scheduler.".to_string(),
                )
            }
        }
        ClientMessage::TransportStart(timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::TransportStart(timing))
                .is_err()
            {
                eprintln!("[!] Failed to send TransportStart to scheduler.");
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
                eprintln!("[!] Failed to send TransportStop to scheduler.");
                return ServerMessage::InternalError("Scheduler communication error.".to_string());
            }
            // Revert: No longer send immediate status based on atomic
            ServerMessage::Success
        }
        ClientMessage::RequestDeviceList => {
            println!("[ info ] Client '{}' requested device list.", client_name);
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
                        .send(SchedulerNotification::DeviceListChanged(
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
                        .send(SchedulerNotification::DeviceListChanged(
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
                        .send(SchedulerNotification::DeviceListChanged(
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
                        .send(SchedulerNotification::DeviceListChanged(
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
                        .send(SchedulerNotification::DeviceListChanged(
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
        // --- Add handlers for OSC device messages ---
        ClientMessage::CreateOscDevice(name, ip, port) => {
            match state.devices.create_osc_output_device(&name, &ip, port) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    let _ = state
                        .update_sender
                        .send(SchedulerNotification::DeviceListChanged(
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
            match state.devices.remove_osc_output_device(&name) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    let _ = state
                        .update_sender
                        .send(SchedulerNotification::DeviceListChanged(
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
        // ----------------------------------------
        // Handle deprecated messages explicitly
        ClientMessage::ConnectMidiDeviceById(device_id) => {
            eprintln!(
                "[!] Received deprecated ConnectMidiDeviceById({}) from '{}'",
                device_id, client_name
            );
            ServerMessage::InternalError(
                "ConnectMidiDeviceById is deprecated. Use ConnectMidiDeviceByName.".to_string(),
            )
        }
        ClientMessage::DisconnectMidiDeviceById(device_id) => {
            eprintln!(
                "[!] Received deprecated DisconnectMidiDeviceById({}) from '{}'",
                device_id, client_name
            );
            ServerMessage::InternalError(
                "DisconnectMidiDeviceById is deprecated. Use DisconnectMidiDeviceByName."
                    .to_string(),
            )
        }
        ClientMessage::DuplicateFrameRange {
            src_line_idx,
            src_frame_start_idx,
            src_frame_end_idx,
            target_insert_idx,
            timing,
        } => {
            let scene = state.scene_image.lock().await;
            if let Some(src_line) = scene.lines.get(src_line_idx) {
                // Validate frame range
                if src_frame_start_idx <= src_frame_end_idx
                    && src_frame_end_idx < src_line.frames.len()
                {
                    let mut frames_data = Vec::new();
                    for i in src_frame_start_idx..=src_frame_end_idx {
                        let frame_length = src_line.frames[i];
                        let is_enabled = src_line.is_frame_enabled(i);
                        let script_arc_opt = src_line.scripts.get(i).cloned(); // Clone original Arc
                        // Compile script here before putting into DuplicatedFrameData
                        let compiled_script_arc = if let Some(src_arc) = script_arc_opt {
                            match state
                                .transcoder
                                .lock()
                                .await
                                .compile_active(&src_arc.content)
                            {
                                Ok(compiled_prog) => {
                                    let new_script = Script::new(
                                        src_arc.content.clone(),
                                        compiled_prog,
                                        src_arc.lang.clone(),
                                        0, // Placeholder index
                                    );
                                    Some(Arc::new(new_script))
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[!] Script compile failed in DuplicateFrameRange ({}, {}): {}",
                                        src_line_idx, i, e
                                    );
                                    // Decide how to handle: return error? Send None? For now, send None.
                                    None
                                }
                            }
                        } else {
                            None // No original script
                        };
                        let frame_name = src_line.frame_names.get(i).cloned().flatten(); // Get name

                        frames_data.push(crate::schedule::DuplicatedFrameData {
                            length: frame_length,
                            is_enabled,
                            script: compiled_script_arc, // Store the compiled Option<Arc<Script>>
                            name: frame_name,            // Store the name
                            repetitions: src_line.frame_repetitions.get(i).copied().unwrap_or(1).max(1), // Copy repetitions
                        });
                    }

                    // Send to scheduler
                    if state
                        .sched_iface
                        .send(SchedulerMessage::InternalDuplicateFrameRange {
                            target_line_idx: src_line_idx, // Assuming duplication happens on the same line for now
                            target_insert_idx,
                            frames_data,
                            timing,
                        })
                        .is_ok()
                    {
                        ServerMessage::Success
                    } else {
                        eprintln!("[!] Failed to send InternalDuplicateFrameRange to scheduler.");
                        ServerMessage::InternalError(
                            "Failed to send duplicate frame range command to scheduler."
                                .to_string(),
                        )
                    }
                } else {
                    eprintln!(
                        "[!] DuplicateFrameRange failed: Invalid source frame range ({}-{}) for line {}.",
                        src_frame_start_idx, src_frame_end_idx, src_line_idx
                    );
                    ServerMessage::InternalError(
                        "Invalid source frame range for duplication.".to_string(),
                    )
                }
            } else {
                eprintln!(
                    "[!] DuplicateFrameRange failed: Invalid source line index {}.",
                    src_line_idx
                );
                ServerMessage::InternalError(
                    "Invalid source line index for duplication.".to_string(),
                )
            }
        }
        ClientMessage::RemoveFramesMultiLine {
            lines_and_indices,
            timing,
        } => {
            if state
                .sched_iface
                .send(SchedulerMessage::InternalRemoveFramesMultiLine {
                    lines_and_indices,
                    timing,
                })
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send InternalRemoveFramesMultiLine to scheduler.");
                ServerMessage::InternalError(
                    "Failed to send remove frames command to scheduler.".to_string(),
                )
            }
        }
        ClientMessage::RequestDuplicationData {
            src_top,
            src_left,
            src_bottom,
            src_right,
            target_cursor_row,
            target_cursor_col,
            insert_before,
            timing,
        } => {
            let scene = state.scene_image.lock().await;
            let mut duplicated_data: Vec<Vec<crate::schedule::DuplicatedFrameData>> = Vec::new();
            let transcoder = state.transcoder.lock().await;
            let mut valid_data = true;
            let mut compilation_failed = false;

            // Determine the target insert index based on insert_before flag
            let target_frame_idx = if insert_before {
                target_cursor_row // Insert at the cursor row (top of selection)
            } else {
                target_cursor_row // Insert after the cursor row (bottom + 1 of selection)
            };
            let target_line_idx = target_cursor_col; // Use the target column from the request

            // Iterate through columns in the source selection
            for col_idx in src_left..=src_right {
                if let Some(src_line) = scene.lines.get(col_idx) {
                    let mut column_data = Vec::new();
                    // Iterate through rows in the source selection
                    for row_idx in src_top..=src_bottom {
                        if row_idx < src_line.frames.len() {
                            let frame_length = src_line.frames[row_idx];
                            let is_enabled = src_line.is_frame_enabled(row_idx);
                            let script_arc_opt = src_line
                                .scripts
                                .iter()
                                .find(|s| s.index == row_idx)
                                .cloned(); // Clone the Arc if it exists

                            // Compile script content if available
                            let compiled_script_arc_opt = if let Some(source_arc) = script_arc_opt {
                                match transcoder.compile_active(&source_arc.content) {
                                    Ok(compiled_prog) => {
                                        // Create a new Script instance with compiled code
                                        let new_script = Script::new(
                                            source_arc.content.clone(), // Keep original content
                                            compiled_prog,
                                            source_arc.lang.clone(),
                                            0, // Placeholder index, scheduler handles final index
                                        );
                                        Some(Arc::new(new_script))
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "[!] Script compilation failed during duplication ({},{}): {}",
                                            col_idx, row_idx, e
                                        );
                                        compilation_failed = true;
                                        None // Mark as None if compilation fails
                                    }
                                }
                            } else {
                                None // No source script
                            };

                            // Break early if compilation failed for any script
                            if compilation_failed {
                                break;
                            }

                            column_data.push(crate::schedule::DuplicatedFrameData {
                                length: frame_length,
                                is_enabled,
                                script: compiled_script_arc_opt, // Store Arc<Script> with compiled code
                                name: src_line.frame_names.get(row_idx).cloned().flatten(), // Copy name
                                repetitions: src_line.frame_repetitions.get(row_idx).copied().unwrap_or(1).max(1), // Copy repetitions
                            });
                        } else {
                            // If any part of the selection is out of bounds, it's invalid
                            eprintln!(
                                "[!] RequestDuplicationData failed: Invalid source index ({}, {})",
                                col_idx, row_idx
                            );
                            valid_data = false;
                            break; // Stop processing this column
                        }
                    }
                    if !valid_data {
                        break;
                    } // Stop processing columns if invalid data found
                    duplicated_data.push(column_data);
                } else {
                    eprintln!(
                        "[!] RequestDuplicationData failed: Invalid source line index {}",
                        col_idx
                    );
                    valid_data = false;
                    break; // Stop processing columns
                }
            }

            // Check for both valid selection and successful compilation
            if valid_data && !compilation_failed && !duplicated_data.is_empty() {
                // Send the structured data to the scheduler
                if state
                    .sched_iface
                    .send(SchedulerMessage::InternalInsertDuplicatedBlocks {
                        duplicated_data,
                        target_line_idx,
                        target_frame_idx,
                        timing,
                    })
                    .is_ok()
                {
                    ServerMessage::Success
                } else {
                    eprintln!("[!] Failed to send InternalInsertDuplicatedBlocks to scheduler.");
                    ServerMessage::InternalError(
                        "Failed to send duplication command to scheduler.".to_string(),
                    )
                }
            } else {
                // Provide more specific error
                let error_msg = if compilation_failed {
                    "Script compilation failed during duplication preparation.".to_string()
                } else {
                    "Invalid source selection for duplication.".to_string()
                };
                ServerMessage::InternalError(error_msg)
            }
        }
        ClientMessage::PasteDataBlock {
            data,
            target_row,
            target_col,
            timing,
        } => {
            let scene = state.scene_image.lock().await;
            let transcoder = state.transcoder.lock().await;
            let mut messages_to_scheduler = Vec::new();
            let mut compilation_errors: Vec<String> = Vec::new();
            let mut frames_updated = 0;

            for (col_offset, column_data) in data.iter().enumerate() {
                let current_target_line_idx = target_col + col_offset;

                // Check if target line exists
                if let Some(target_line) = scene.lines.get(current_target_line_idx) {
                    let mut updated_frames = target_line.frames.clone(); // Clone current frames for length update
                    let mut line_frames_modified = false;
                    let mut frames_to_enable = Vec::new();
                    let mut frames_to_disable = Vec::new();

                    for (row_offset, pasted_frame) in column_data.iter().enumerate() {
                        let current_target_frame_idx = target_row + row_offset;

                        // Check if target frame exists within the line
                        if current_target_frame_idx < target_line.frames.len() {
                            // 1. Update Frame Length
                            updated_frames[current_target_frame_idx] = pasted_frame.length;
                            line_frames_modified = true;

                            // 2. Update Enabled State
                            if pasted_frame.is_enabled {
                                frames_to_enable.push(current_target_frame_idx);
                            } else {
                                frames_to_disable.push(current_target_frame_idx);
                            }

                            // 3. Compile and Update Script (if provided)
                            if let Some(script_content) = &pasted_frame.script_content {
                                // TODO: PastedFrameData needs a 'lang' field. Assuming "bali" for now.
                                let lang_to_use = "bali".to_string(); // Default language assumption
                                match transcoder.compile(script_content, &lang_to_use) {
                                    Ok(compiled_script) => {
                                        let script = Script::new(
                                            script_content.clone(),
                                            compiled_script,
                                            lang_to_use, // Use the assumed language
                                            current_target_frame_idx, // Use correct target index
                                        );
                                        messages_to_scheduler.push(SchedulerMessage::UploadScript(
                                            current_target_line_idx,
                                            current_target_frame_idx,
                                            script,
                                            timing,
                                        ));
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "[!] Script compilation failed during paste ({},{}): {}",
                                            current_target_line_idx, current_target_frame_idx, e
                                        );
                                        compilation_errors.push(format!(
                                            "Error at ({},{}): {}",
                                            current_target_line_idx, current_target_frame_idx, e
                                        ));
                                        // Continue pasting other elements, but report errors
                                    }
                                }
                            }

                            // 4. Update Frame Name (if provided in paste data)
                            if let Some(pasted_name) = &pasted_frame.name {
                                messages_to_scheduler.push(SchedulerMessage::SetFrameName(
                                    current_target_line_idx,
                                    current_target_frame_idx,
                                    Some(pasted_name.clone()),
                                    timing,
                                ));
                            } else {
                                // If paste data doesn't have a name, explicitly clear it on the target
                                messages_to_scheduler.push(SchedulerMessage::SetFrameName(
                                    current_target_line_idx,
                                    current_target_frame_idx,
                                    None,
                                    timing,
                                ));
                            }

                            // 5. Update Frame Repetitions (if provided in paste data)
                            if let Some(pasted_reps) = pasted_frame.repetitions {
                                messages_to_scheduler.push(SchedulerMessage::SetFrameRepetitions(
                                    current_target_line_idx,
                                    current_target_frame_idx,
                                    pasted_reps, // Use the value from paste data
                                    timing,
                                ));
                            } // If None, the repetition count is unchanged

                            frames_updated += 1;
                        } else {
                            // Target frame index out of bounds for this line - skip
                            println!(
                                "[!] Paste skipped: Target frame ({}, {}) out of bounds.",
                                current_target_line_idx, current_target_frame_idx
                            );
                        }
                    }

                    // Queue scheduler messages for this line if modifications occurred
                    if line_frames_modified {
                        messages_to_scheduler.push(SchedulerMessage::UpdateLineFrames(
                            current_target_line_idx,
                            updated_frames,
                            timing,
                        ));
                    }
                    if !frames_to_enable.is_empty() {
                        messages_to_scheduler.push(SchedulerMessage::EnableFrames(
                            current_target_line_idx,
                            frames_to_enable,
                            timing,
                        ));
                    }
                    if !frames_to_disable.is_empty() {
                        messages_to_scheduler.push(SchedulerMessage::DisableFrames(
                            current_target_line_idx,
                            frames_to_disable,
                            timing,
                        ));
                    }
                } else {
                    // Target line index out of bounds - skip entire column
                    println!(
                        "[!] Paste skipped: Target line {} out of bounds.",
                        current_target_line_idx
                    );
                }
            }

            // Send collected messages to scheduler
            for msg in messages_to_scheduler {
                if state.sched_iface.send(msg).is_err() {
                    eprintln!("[!] Failed to send paste-related message to scheduler.");
                    // Don't stop, try sending others, but return error at the end
                    compilation_errors
                        .push("Scheduler communication error during paste.".to_string());
                }
            }

            // Report outcome
            if !compilation_errors.is_empty() {
                ServerMessage::InternalError(format!(
                    "Paste partially failed. {} frames updated. Errors: {}",
                    frames_updated,
                    compilation_errors.join("; ")
                ))
            } else if frames_updated > 0 {
                ServerMessage::Success
            } else {
                ServerMessage::InternalError(
                    "Paste failed: No target frames found or no data provided.".to_string(),
                )
            }
        }
        // --- Add handler for SetFrameRepetitions ---
        ClientMessage::SetFrameRepetitions(line_idx, frame_idx, repetitions, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::SetFrameRepetitions(
                    line_idx, frame_idx, repetitions, timing,
                ))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send SetFrameRepetitions to scheduler.");
                ServerMessage::InternalError(
                    "Failed to send frame repetition update to scheduler.".to_string(),
                )
            }
        }
        // --- Add handler for SetFrameName ---
        ClientMessage::SetFrameName(line_idx, frame_idx, name, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::SetFrameName(
                    line_idx, frame_idx, name, timing,
                ))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send SetFrameName to scheduler.");
                ServerMessage::InternalError(
                    "Failed to send frame name update to scheduler.".to_string(),
                )
            }
        }
        // --- Add handler for SetScriptLanguage ---
        ClientMessage::SetScriptLanguage(line_idx, frame_idx, lang, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::SetScriptLanguage(
                    line_idx, frame_idx, lang, timing,
                ))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send SetScriptLanguage to scheduler.");
                ServerMessage::InternalError(
                    "Failed to send script language update to scheduler.".to_string(),
                )
            }
        } // ---------------------------------
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
fn compress_message_intelligently(msg: &ServerMessage, msgpack_bytes: &[u8]) -> io::Result<(Vec<u8>, bool)> {
    use crate::server::client::CompressionStrategy;
    
    match msg.compression_strategy() {
        CompressionStrategy::Never => {
            // Never compress frequent/small messages
            Ok((msgpack_bytes.to_vec(), false))
        },
        CompressionStrategy::Always => {
            // Always compress large content, but only if beneficial
            if msgpack_bytes.len() > 64 {
                let compression_level = if msgpack_bytes.len() < 1024 { 1 } else { 3 };
                let compressed = zstd::encode_all(msgpack_bytes, compression_level).map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, format!("Compression failed: {}", e))
                })?;
                // Only use compressed if it's actually smaller
                if compressed.len() < msgpack_bytes.len() {
                    Ok((compressed, true))
                } else {
                    Ok((msgpack_bytes.to_vec(), false))
                }
            } else {
                Ok((msgpack_bytes.to_vec(), false))
            }
        },
        CompressionStrategy::Adaptive => {
            // Original size-based logic
            if msgpack_bytes.len() < 256 {
                Ok((msgpack_bytes.to_vec(), false))
            } else {
                let compression_level = if msgpack_bytes.len() < 1024 { 1 } else { 3 };
                let compressed = zstd::encode_all(msgpack_bytes, compression_level).map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, format!("Compression failed: {}", e))
                })?;
                Ok((compressed, true))
            }
        }
    }
}

impl BuboCoreServer {
    /// Creates a new `BuboCoreServer` instance with the specified address and port.
    pub fn new(ip: String, port: u16) -> Self {
        BuboCoreServer { ip, port }
    }

    /// Starts the TCP server, listens for connections, and handles graceful shutdown.
    ///
    /// This function enters the main server loop, accepting new connections and
    /// spawning `process_client` tasks. It also listens for a Ctrl+C signal
    /// to initiate a shutdown.
    ///
    /// # Arguments
    /// * `state` - The shared `ServerState` to be cloned for each client task.
    pub async fn start(&self, state: ServerState) -> io::Result<()> {
        let addr = format!("{}:{}", self.ip, self.port);
        let listener = TcpListener::bind(&addr).await?;
        println!("[+] Server listening on {}", addr);

        loop {
            select! {
                // Accept new TCP connections
                Ok((socket, client_addr)) = listener.accept() => {
                     println!("[🔌] New connection from {}", client_addr);
                     let client_state = state.clone(); // Clone state for the new task
                     // Spawn a task to handle this client independently
                     tokio::spawn(async move {
                         match process_client(socket, client_state).await {
                             Ok(client_name) => {
                                // Log graceful disconnection
                                println!("[🔌] Client '{}' disconnected.", client_name);
                             },
                             Err(e) => {
                                 // Log errors during client processing
                                 eprintln!("[!] Error handling client {}: {}", client_addr, e);
                             }
                         }
                     });
                 }
                 // Handle Ctrl+C for graceful shutdown
                 _ = signal::ctrl_c() => {
                    println!("
            [!] Ctrl+C received, shutting down server...");
                    // TODO: Implement graceful shutdown logic:
                    // - Notify clients of shutdown?
                    // - Signal scheduler/world tasks to stop?
                    // - Wait for tasks to finish?
                    break; // Exit the main loop
                 }
                 // Avoid 100% CPU usage if no events occur
                 _ = tokio::time::sleep(Duration::from_millis(10)) => {}
            }
        }
        Ok(())
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

    // --- Handshake: Expect SetName first ---
    let hello_msg: ServerMessage; // Declare hello_msg variable

    match read_message_internal(&mut reader, &client_addr_str).await {
        Ok(Some(ClientMessage::SetName(new_name))) => {
            // Validate name (e.g., non-empty, allowed characters, uniqueness)
            if new_name.is_empty() || new_name == DEFAULT_CLIENT_NAME {
                eprintln!(
                    "[!] Connection rejected: Invalid username '{}' from {}",
                    new_name, client_addr_str
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
                eprintln!(
                    "[!] Connection rejected: Username '{}' already taken by {}",
                    new_name, client_addr_str
                );
                let refuse_msg =
                    ServerMessage::ConnectionRefused(format!("Username '{}' is already taken.", new_name));
                let _ = send_msg(&mut writer, refuse_msg).await; // Attempt to notify client
                drop(clients_guard); // Release lock
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Username taken",
                ));
            }

            // Name is valid and unique, accept connection
            client_name = new_name; // Assign the validated name
            println!("[👤] Client {} identified as: {}", client_addr_str, client_name);
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
                .send(SchedulerNotification::ClientListChanged(
                    updated_peers_for_broadcast,
                ));


            // --- THEN fetch dynamic state like clock/playing status ---
            let clock = Clock::from(&state.clock_server);
            let initial_link_state = (
                clock.tempo(),
                clock.beat(),
                clock.beat() % clock.quantum(),
                state.clock_server.link.num_peers() as u32, // Cast u64 to u32
                state.clock_server.link.is_start_stop_sync_enabled(),
            );
            let initial_is_playing = state.shared_atomic_is_playing.load(Ordering::Relaxed);

             // --- Get available compilers and their syntax definitions ---
            let transcoder_guard = state.transcoder.lock().await;
            let available_compilers = transcoder_guard.available_compilers();
            let mut syntax_definitions = std::collections::HashMap::new();
            for compiler_name in &available_compilers {
                if let Some(compiler) = transcoder_guard.compilers.get(compiler_name) {
                    if let Some(Cow::Borrowed(content)) = compiler.syntax() {
                        syntax_definitions.insert(compiler_name.clone(), content.to_string());
                    }
                }
            }
            drop(transcoder_guard); // Release lock

            // --- Construct the Hello message ---
            println!(
                "[ handshake ] Sending Hello to {} ({}). Initial is_playing state: {}",
                 client_addr_str, client_name, initial_is_playing
            );
             hello_msg = ServerMessage::Hello {
                username: client_name.clone(), // Send the *accepted* name
                scene: initial_scene,
                devices: initial_devices,
                peers: initial_peers, // Send the updated list
                link_state: initial_link_state,
                is_playing: initial_is_playing,
                available_compilers,
                syntax_definitions,
            };

            // Send Hello
            if send_msg(&mut writer, hello_msg).await.is_err() {
                 eprintln!("[!] Failed to send Hello to {}", client_name);
                 // Don't remove from list yet, cleanup will handle it
                 return Err(io::Error::new(
                     io::ErrorKind::WriteZero, // Or other appropriate error
                     "Failed to send Hello message",
                 ));
             }

        }
        Ok(Some(other_msg)) => {
            // First message was not SetName
            eprintln!(
                "[!] Connection rejected: Expected SetName, received {:?} from {}",
                other_msg, client_addr_str
            );
             let refuse_msg = ServerMessage::ConnectionRefused("Invalid handshake sequence.".to_string());
             let _ = send_msg(&mut writer, refuse_msg).await;
             return Err(io::Error::new(
                 io::ErrorKind::InvalidData,
                 "Invalid handshake sequence",
             ));

        }
         Ok(None) => {
             // Connection closed during handshake before sending SetName
             println!("[🔌] Connection closed by {} during handshake.", client_addr_str);
             return Ok(client_name); // Return default name as it wasn't set
         }
         Err(e) => {
             // Read error during handshake
             eprintln!("[!] Read error during handshake with {}: {}", client_addr_str, e);
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
                             eprintln!("[!] Failed write direct response to {}", client_name);
                             break; // Assume connection broken
                         }
                    },
                    Ok(None) => {
                         // Clean disconnect (EOF)
                         println!("[🔌] Connection closed cleanly by {}.", client_name);
                         break;
                    },
                    Err(_e) => {
                         // Read error occurred and was logged by read_message_internal
                         eprintln!("[!] Read error for client {}. Closing connection.", client_name);
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
                let _is_scene_update = matches!(notification, SchedulerNotification::UpdatedScene(_)); // Simpler way to check
                let broadcast_msg_opt: Option<ServerMessage> = match notification {
                    SchedulerNotification::TransportStarted => {
                        Some(ServerMessage::TransportStarted)
                    },
                    SchedulerNotification::TransportStopped => {
                        Some(ServerMessage::TransportStopped)
                    },
                    SchedulerNotification::UpdatedScene(p) => {
                        // Remove log
                        Some(ServerMessage::SceneValue(p))
                    },
                    SchedulerNotification::Log(log_msg) => {
                         Some(ServerMessage::LogString(log_msg.to_string()))
                    }
                    SchedulerNotification::TempoChanged(_) => {
                        let clock = Clock::from(&state.clock_server);
                        Some(ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum()))
                    }
                    SchedulerNotification::ClientListChanged(clients) => {
                        Some(ServerMessage::PeersUpdated(clients))
                    }
                    SchedulerNotification::ChatReceived(sender_name, chat_msg) => {
                        if sender_name != *client_name {
                           Some(ServerMessage::Chat(format!("({}) {}", sender_name, chat_msg)))
                        } else {
                            None
                        }
                    }
                    SchedulerNotification::FramePositionChanged(positions) => {
                        Some(ServerMessage::FramePosition(positions))
                    }
                    SchedulerNotification::PeerGridSelectionChanged(sender_name, selection) => {
                        // Don't send the update back to the originator
                        if sender_name != *client_name {
                            Some(ServerMessage::PeerGridSelectionUpdate(sender_name, selection))
                        } else {
                            None
                        }
                    }
                    SchedulerNotification::PeerStartedEditingFrame(sender_name, line_idx, frame_idx) => {
                         // Don't send the update back to the originator
                         if sender_name != *client_name {
                             Some(ServerMessage::PeerStartedEditing(sender_name, line_idx, frame_idx))
                         } else {
                             None
                         }
                    }
                    SchedulerNotification::PeerStoppedEditingFrame(sender_name, line_idx, frame_idx) => {
                         // Don't send the update back to the originator
                         if sender_name != *client_name {
                             Some(ServerMessage::PeerStoppedEditing(sender_name, line_idx, frame_idx))
                         } else {
                             None
                         }
                    }
                    SchedulerNotification::SceneLengthChanged(length) => {
                        Some(ServerMessage::SceneLength(length))
                    }
                    // Add handler for DeviceListChanged
                    SchedulerNotification::DeviceListChanged(devices) => {
                        println!("[ broadcast ] Sending updated device list ({} devices) to {}", devices.len(), client_name);
                        Some(ServerMessage::DeviceList(devices))
                    }
                    // Map scene-modifying notifications to SceneValue to trigger client refresh
                    SchedulerNotification::UpdatedLine(_, _) |
                    SchedulerNotification::EnableFrames(_, _) |
                    SchedulerNotification::DisableFrames(_, _) |
                    SchedulerNotification::UploadedScript(_, _, _) |
                    SchedulerNotification::UpdatedLineFrames(_, _) |
                    SchedulerNotification::AddedLine(_) |
                    SchedulerNotification::RemovedLine(_) => {
                        // Fetch the latest scene state and send it
                        let scene = state.scene_image.lock().await.clone();
                        Some(ServerMessage::SceneValue(scene))
                    }
                    SchedulerNotification::Nothing => { None } // Explicitly ignore Nothing
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
    println!("[🔌] Cleaning up connection for client: {}", client_name);
    // Only remove the client if they successfully completed the handshake (i.e., name is not default)
    if client_name != DEFAULT_CLIENT_NAME {
        let mut clients_guard = state.clients.lock().await;
        if let Some(i) = clients_guard.iter().position(|x| *x == client_name) {
            clients_guard.remove(i);
            println!("[👤] Removed {} from client list.", client_name);
            // Broadcast the updated client list after removal
            let updated_clients = clients_guard.clone();
            drop(clients_guard); // Drop lock before sending notification
            let _ = state
                .update_sender
                .send(SchedulerNotification::ClientListChanged(updated_clients));
        } else {
            // This case might happen if the client disconnected right after handshake
            // before the main loop really started, or if there's a race condition.
            eprintln!(
                "[!] Client '{}' not found in list during cleanup, though name was set.",
                client_name
            );
        }
    } else {
        println!(
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
            deserialize_message(&final_bytes, client_id_for_logging)
        }
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
            println!("[🔌] Connection closed by {} (EOF before header).", client_id_for_logging);
            Ok(None) // Indicate clean closure
        }
        Err(e) => {
            eprintln!("[!] Error reading message header from {}: {}", client_id_for_logging, e);
            Err(e)
        }
    }
}

/// Decompresses a message buffer using Zstd
fn decompress_message(message_buf: &[u8], client_id: &str) -> io::Result<Vec<u8>> {
    zstd::decode_all(message_buf).map_err(|e| {
        eprintln!("[!] Failed to decompress Zstd data from {}: {}", client_id, e);
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Zstd decompression error: {}", e),
        )
    })
}

/// Deserializes a MessagePack buffer into a ClientMessage
fn deserialize_message(final_bytes: &[u8], client_id: &str) -> io::Result<Option<ClientMessage>> {
    match rmp_serde::from_slice::<ClientMessage>(final_bytes) {
        Ok(msg) => Ok(Some(msg)),
        Err(e) => {
            eprintln!("[!] Failed to deserialize MessagePack from {}: {}", client_id, e);
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("MessagePack deserialization error: {}", e),
            ))
        }
    }
}
