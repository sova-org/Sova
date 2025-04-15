use crate::scene::script::Script;
use client::ClientMessage;
use serde::{Deserialize, Serialize};
use std::{
    io::ErrorKind,
    sync::{Arc, mpsc::Sender},
};
use tokio::time::Duration;
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    select, signal,
    sync::{Mutex, watch},
};
use thread_priority::ThreadBuilder;

use crate::{
    clock::{Clock, ClockServer, SyncTime},
    device_map::DeviceMap,
    protocol::TimedMessage,
    scene::Scene,
    schedule::{SchedulerMessage, SchedulerNotification},
    shared_types::GridSelection,
    transcoder::Transcoder,
    lang::variable::VariableStore,
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
    pub fn new(
        scene_image: Arc<Mutex<Scene>>,
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        world_iface: Sender<TimedMessage>,
        sched_iface: Sender<SchedulerMessage>,
        update_sender: watch::Sender<SchedulerNotification>,
        update_receiver: watch::Receiver<SchedulerNotification>,
        transcoder: Arc<Mutex<Transcoder>>,
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

/// Messages sent from the server *to* a client.
///
/// These are typically responses to client requests or broadcasted updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// A log message originating from the server or scheduler.
    LogMessage(TimedMessage),
    /// A chat message broadcast from another client or the server itself.
    Chat(String),
    /// The current frame positions within each line of the scene.
    /// (Currently unused in favor of sending the whole `SceneValue`).
    FramePosition(Vec<usize>),
    /// Initial greeting message sent upon successful connection.
    /// Includes necessary state for the client to initialize (scene, devices, peers).
    Hello {
        /// The current scene state.
        scene: Scene,
        /// List of available output devices (name, type).
        devices: Vec<(String, String)>,
        /// List of names of other currently connected clients.
        clients: Vec<String>,
    },
    /// Broadcast containing the complete current state of the scene.
    SceneValue(Scene),
    /// The layout/structure definition of the scene.
    /// (Currently unused in favor of sending the whole `SceneValue`).
    SceneLayout(Vec<Vec<(f64, bool)>>),
    /// Broadcast containing the current state of the master clock.
    ClockState(
        /// Tempo in beats per minute (BPM).
        f64,
        /// Current beat time within the Ableton Link session.
        f64,
        /// Current microsecond time within the Ableton Link session.
        SyncTime,
        /// The musical quantum (e.g., 4.0 for 4/4 time).
        f64,
    ),
    /// Generic success response, indicating a requested action was accepted.
    Success,
    /// Indicates an internal server error occurred while processing a request.
    InternalError(String),
    /// Broadcast containing the updated list of connected client names.
    PeersUpdated(Vec<String>),
    /// Confirmation that a specific frame has been enabled.
    /// (Currently unused in favor of sending the whole `SceneValue`).
    FrameEnabbled(usize, usize),
    /// Confirmation that a specific frame has been disabled.
    /// (Currently unused in favor of sending the whole `SceneValue`).
    FrameDisabled(usize, usize),
    /// Sends the requested script content to the client.
    ScriptContent {
        /// The index of the line the script belongs to.
        line_idx: usize,
        /// The index of the frame within the line.
        frame_idx: usize,
        /// The script content as a string.
        content: String,
    },
    /// A complete snapshot of the current server state.
    Snapshot(Snapshot),
    /// Broadcasts an update to a specific peer's grid selection.
    PeerGridSelectionUpdate(String, GridSelection),
    /// Broadcasts that a peer started editing a specific frame.
    PeerStartedEditing(String, usize, usize), // (username, line_idx, frame_idx)
    /// Broadcasts that a peer stopped editing a specific frame
    PeerStoppedEditing(String, usize, usize), // (username, line_idxx, frame_idx)
    /// The current length of the scene.
    SceneLength(usize),
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

/// Generates the `ServerMessage::Hello` message for a newly connected client.
///
/// Acquires necessary locks on shared state (scene, clients) to provide
/// the client with its initial view of the server state.
///
/// # Arguments
/// * `state` - A reference to the shared `ServerState`.
///
/// # Returns
/// A `ServerMessage::Hello` containing the current scene, device list, and client list.
async fn generate_hello(state: &ServerState) -> ServerMessage {
    ServerMessage::Hello {
        scene: state.scene_image.lock().await.clone(),
        devices: state.devices.device_list(),
        clients: state.clients.lock().await.clone(),
    }
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
    // Log the incoming request to the server console and broadcast as a server log message.
    let log_string = format!("[➡️ ] Client '{}' sent: {:?}", client_name, msg);
    println!("{}", log_string);

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
            // Compile and forward to scheduler
            match state
                .transcoder
                .lock()
                .await
                .compile_active(&script_content)
            {
                Ok(compiled_script) => {
                    let script = Script::new(
                        script_content,
                        compiled_script,
                        "bali".to_string(),
                        frame_id,
                    );
                    if state
                        .sched_iface
                        .send(SchedulerMessage::UploadScript(line_id, frame_id, script, timing))
                        .is_err()
                    {
                        eprintln!("[!] Failed to send UploadScript to scheduler.");
                        ServerMessage::InternalError("Scheduler communication error.".to_string())
                    } else {
                        ServerMessage::Success
                    }
                }
                Err(e) => {
                    eprintln!("[!] {}", e);
                    ServerMessage::InternalError(format!("Script compilation failed: {}", e))
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
            if state.sched_iface.send(SchedulerMessage::SetTempo(tempo, timing)).is_err() {
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
            { // Scope for transcoder lock
                let transcoder = state.transcoder.lock().await;
                for line in scene.lines.iter_mut() {
                    for script_arc in line.scripts.iter_mut() {
                        match transcoder.compile_active(&script_arc.content) {
                            Ok(compiled) => {
                                // We need exclusive access to modify the Arc's inner value
                                if let Some(script) = Arc::get_mut(script_arc) {
                                    script.compiled = compiled;
                                } else {
                                    // This case might happen if the Arc is shared elsewhere unexpectedly.
                                    // We might need to clone/recreate the Arc if modification fails.
                                    // For now, let's log a warning.
                                     eprintln!("[!] Failed to get mutable access to script Arc during SetScene compilation. Line: {}, Frame: {}", line.index, script_arc.index);
                                     // Fallback: Create a new Arc with the compiled script
                                    let new_script = Script::new(
                                        script_arc.content.clone(),
                                        compiled, // Use the successfully compiled instructions
                                        (**script_arc).lang.clone(), // Correct field and access
                                        script_arc.index
                                    );
                                    *script_arc = Arc::new(new_script);
                                }
                            }
                            Err(e) => {
                                eprintln!("[!] Failed to pre-compile script for Line {}, Frame {} during SetScene: {}", line.index, script_arc.index, e);
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
        ClientMessage::InsertFrame(line_id, position, timing) => {
            // Forward to scheduler with a default value (e.g., 1.0)
            let default_frame_value = 1.0;
            if state
                .sched_iface
                .send(SchedulerMessage::InsertFrame(
                    line_id,
                    position,
                    default_frame_value,
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
                .send(SchedulerMessage::SetLineStartFrame(line_id, start_frame, timing))
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
                .send(SchedulerMessage::SetLineEndFrame(line_id, end_frame, timing))
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
                ServerMessage::InternalError("Failed to send scene length update to scheduler.".to_string())
            }
        }
        ClientMessage::SetLineLength(line_idx, length_opt, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::SetLineLength(line_idx, length_opt, timing))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send SetLineLength to scheduler.");
                ServerMessage::InternalError("Failed to send line length update to scheduler.".to_string())
            }
        }
        ClientMessage::SetLineSpeedFactor(line_idx, speed_factor, timing) => {
            if state
                .sched_iface
                .send(SchedulerMessage::SetLineSpeedFactor(line_idx, speed_factor, timing))
                .is_ok()
            {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send SetLineSpeedFactor to scheduler.");
                ServerMessage::InternalError("Failed to send line speed factor update to scheduler.".to_string())
            }
        }
    }
}

/// Serializes and sends a `ServerMessage` to the client's output stream.
///
/// Appends the `ENDING_BYTE` delimiter after the JSON message.
///
/// # Arguments
/// * `writer` - An async writer (e.g., `BufWriter<&mut WriteHalf<TcpStream>>`).
/// * `msg` - The `ServerMessage` to send.
async fn send_msg<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: ServerMessage) -> io::Result<()> {
    // Consider handling serialization errors more gracefully than expect.
    let msg_to_send = serde_json::to_vec(&msg).expect("Failed to serialize ServerMessage");
    writer.write_all(&msg_to_send).await?;
    writer.write_u8(ENDING_BYTE).await?;
    writer.flush().await?; // Ensure message is sent immediately
    Ok(())
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
    let mut read_buf = Vec::with_capacity(1024);
    let (reader, writer) = socket.into_split(); // Split into read/write halves
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);
    let mut client_name = DEFAULT_CLIENT_NAME.to_string(); // Start with default name

    // --- Initial Handshake ---
    let hello_msg = generate_hello(&state).await;
    if send_msg(&mut writer, hello_msg).await.is_err() {
        eprintln!("[!] Failed to send Hello to {}", client_addr);
        return Ok(client_name); // Disconnect immediately if Hello fails
    }

    // --- Main Loop: Read client messages and listen for broadcasts ---
    let mut update_receiver = state.update_receiver.clone(); // Clone receiver for this task

    loop {
        select! {
            // Prioritize reading client messages
            biased;

            // Read data from the client socket
            res = reader.read_until(ENDING_BYTE, &mut read_buf) => {
                match res {
                    Ok(0) => {
                        // Connection closed cleanly by client
                        println!("[🔌] Connection closed by {}", client_name); // Logged later during cleanup
                        break;
                    },
                    Ok(_) => {
                        // Process received message(s)
                        read_buf.pop();
                        if !read_buf.is_empty() {
                            match serde_json::from_slice::<ClientMessage>(&read_buf) {
                                Ok(msg) => {
                                    let response = on_message(msg, &state, &mut client_name).await;
                                    if send_msg(&mut writer, response).await.is_err() {
                                        eprintln!("[!] Failed write direct response to {}", client_name);
                                        break; // Assume connection broken
                                    }
                                },
                                Err(e) => {
                                     // Log deserialization errors
                                     eprintln!("[!] Failed to deserialize message from {}: {:?}. Raw: {:?}", client_name, e, String::from_utf8_lossy(&read_buf));
                                     // Optionally send an error message back
                                     let err_resp = ServerMessage::InternalError(format!("Invalid message format: {}", e));
                                     if send_msg(&mut writer, err_resp).await.is_err() {
                                          eprintln!("[!] Failed write error response to {}", client_name);
                                          break; // Assume connection broken
                                      }
                                 }
                            }
                        }
                        read_buf.clear(); // Important: Clear buffer for next message
                    },
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        // This should ideally not happen often with BufReader and read_until.
                        // If it does, log it but continue. Might indicate slow client or network issues.
                        tokio::time::sleep(Duration::from_millis(5)).await; // Small sleep to prevent busy-looping
                        continue;
                    }
                    Err(e) => {
                        // Other read error (e.g., connection reset)
                        eprintln!("[!] Error reading from client {}: {}", client_name, e);
                        break;
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
                    SchedulerNotification::UpdatedScene(p) => {
                        // Remove log
                        Some(ServerMessage::SceneValue(p))
                    },
                    SchedulerNotification::Log(log_msg) => {
                         Some(ServerMessage::LogMessage(log_msg))
                    },
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
                    SchedulerNotification::Nothing |
                    SchedulerNotification::UpdatedLine(_, _) |
                    SchedulerNotification::EnableFrames(_, _) |
                    SchedulerNotification::DisableFrames(_, _) |
                    SchedulerNotification::UploadedScript(_, _, _) |
                    SchedulerNotification::UpdatedLineFrames(_, _) |
                    SchedulerNotification::AddedLine(_) |
                    SchedulerNotification::RemovedLine(_) => { None }
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
    } else if *client_name != *DEFAULT_CLIENT_NAME {
        eprintln!(
            "[!] Client '{}' not found in list during cleanup.",
            client_name
        );
    }

    Ok(client_name) // Return the final name for logging by the caller
}

