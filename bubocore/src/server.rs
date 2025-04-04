use std::{
    io::ErrorKind, sync::{mpsc::Sender, Arc}
};

use client::ClientMessage;
use tokio::time::Duration;
use crate::pattern::script::Script;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    select, signal,
    sync::{watch, Mutex},
};

use crate::{
    clock::{Clock, ClockServer, SyncTime}, device_map::DeviceMap, pattern::Pattern, protocol::TimedMessage, schedule::{SchedulerMessage, SchedulerNotification}, transcoder::Transcoder
};

pub mod client;

/// Byte delimiter used to separate JSON messages in the TCP stream.
pub const ENDING_BYTE: u8 = 0x07;
/// Default name assigned to clients before they identify themselves.
pub const DEFAULT_CLIENT_NAME: &str = "Unknown musician";

/// Shared server state accessible by all connection handlers.
///
/// Contains references to core components like the clock, device map,
/// scheduler interfaces, and shared data like the client list and pattern image.
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
    /// (e.g., pattern updates, client list changes) to all connected clients.
    pub update_sender: watch::Sender<SchedulerNotification>,
    /// Watch channel receiver used by each client task to receive broadcasts
    /// sent via the `update_sender`.
    pub update_receiver: watch::Receiver<SchedulerNotification>,
    /// List of names of currently connected clients.
    /// Protected by a Mutex for safe concurrent access.
    pub clients: Arc<Mutex<Vec<String>>>,
    /// A snapshot of the current pattern state, shared across threads.
    /// Updated by a dedicated maintenance thread listening to scheduler notifications.
    pub pattern_image: Arc<Mutex<Pattern>>,
    /// Handles script compilation (e.g., Baliscript).
    pub transcoder: Arc<Mutex<Transcoder>>,
}

impl ServerState {
    /// Creates a new `ServerState`.
    ///
    /// # Arguments
    ///
    /// * `pattern_image` - The initial shared pattern image.
    /// * `clock_server` - The shared clock server instance.
    /// * `devices` - The shared device map.
    /// * `world_iface` - Sender channel to the `World` task.
    /// * `sched_iface` - Sender channel to the `Scheduler` task.
    /// * `update_sender` - Sender part of the broadcast channel.
    /// * `update_receiver` - Receiver template for the broadcast channel.
    /// * `transcoder` - The shared script transcoder.
    pub fn new(
        pattern_image: Arc<Mutex<Pattern>>,
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
            pattern_image,
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
    /// The current step positions within each sequence of the pattern.
    /// (Currently unused in favor of sending the whole `PatternValue`).
    StepPosition(Vec<usize>),
    /// Initial greeting message sent upon successful connection.
    /// Includes necessary state for the client to initialize (pattern, devices, peers).
    Hello {
        /// The current pattern state.
        pattern: Pattern,
        /// List of available output devices (name, type).
        devices: Vec<(String, String)>,
        /// List of names of other currently connected clients.
        clients: Vec<String>,
    },
    /// Broadcast containing the complete current state of the pattern.
    PatternValue(Pattern),
    /// The layout/structure definition of the pattern.
    /// (Currently unused in favor of sending the whole `PatternValue`).
    PatternLayout(Vec<Vec<(f64, bool)>>),
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
    /// Confirmation that a specific step has been enabled.
    /// (Currently unused in favor of sending the whole `PatternValue`).
    StepEnabled(usize, usize),
    /// Confirmation that a specific step has been disabled.
    /// (Currently unused in favor of sending the whole `PatternValue`).
    StepDisabled(usize, usize),
    /// Sends the requested script content to the client.
    ScriptContent {
        /// The index of the sequence the script belongs to.
        sequence_idx: usize,
        /// The index of the step within the sequence.
        step_idx: usize,
        /// The script content as a string.
        content: String
    },
    /// A complete snapshot of the current server state.
    Snapshot(Snapshot),
}

/// Represents a complete snapshot of the server's current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// The current pattern state, including sequences and scripts.
    pub pattern: Pattern,
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
/// Acquires necessary locks on shared state (pattern, clients) to provide
/// the client with its initial view of the server state.
///
/// # Arguments
/// * `state` - A reference to the shared `ServerState`.
///
/// # Returns
/// A `ServerMessage::Hello` containing the current pattern, device list, and client list.
async fn generate_hello(state: &ServerState) -> ServerMessage {
    ServerMessage::Hello {
        pattern: state.pattern_image.lock().await.clone(),
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
/// Broadcast updates resulting from the message (e.g., `PatternValue`, `PeersUpdated`)
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
        ClientMessage::EnableSteps(sequence_id, steps) => {
            if state.sched_iface.send(SchedulerMessage::EnableSteps(sequence_id, steps)).is_err() {
                eprintln!("[!] Failed to send EnableSteps to scheduler.");
            }
            ServerMessage::Success
        },
        ClientMessage::DisableSteps(sequence_id, steps) => {
            // Forward to scheduler with the vector of steps
            if state.sched_iface.send(SchedulerMessage::DisableSteps(sequence_id, steps)).is_err() {
                 eprintln!("[!] Failed to send DisableSteps to scheduler.");
            }
            ServerMessage::Success
        },
        ClientMessage::SetScript(sequence_id, step_id, script_content) => {
            // Compile and forward to scheduler
            match state.transcoder.lock().await.compile_active(&script_content) {
                Ok(compiled_script) => {
                    let script = Script::new(script_content, compiled_script, "bali".to_string(), step_id);
                    if state.sched_iface.send(SchedulerMessage::UploadScript(sequence_id, step_id, script)).is_err() {
                        eprintln!("[!] Failed to send UploadScript to scheduler.");
                         ServerMessage::InternalError("Scheduler communication error.".to_string())
                    } else {
                        ServerMessage::Success
                    }
                },
                Err(e) => {
                     eprintln!("[!] Script compilation failed: {}", e);
                     ServerMessage::InternalError(format!("Script compilation failed: {}", e))
                }
            }
        },
        ClientMessage::GetScript(sequence_idx, step_idx) => {
            // Lock the pattern image to read the script content
            let pattern = state.pattern_image.lock().await;
            match pattern.sequences.get(sequence_idx) {
                Some(sequence) => {
                    // Find the script Arc with the matching index (step_idx) within the sequence's scripts vector
                    let script_opt = sequence.scripts.iter().find(|script_arc| script_arc.index == step_idx);

                    match script_opt {
                        Some(script_arc) => {
                             // Found the script Arc, get the content from the inner Script
                            ServerMessage::ScriptContent {
                                sequence_idx,
                                step_idx,
                                content: script_arc.content.clone(), // Access content via the Arc
                            }
                        }
                        None => {
                             // Sequence valid, but no script found for this specific step_idx
                             eprintln!("[!] No script found for Seq {}, Step {}", sequence_idx, step_idx);
                             // Send back a placeholder script content
                            ServerMessage::ScriptContent {
                                sequence_idx,
                                step_idx,
                                content: format!("// No script found for Seq {}, Step {}", sequence_idx, step_idx),
                            }
                        }
                    }
                }
                None => {
                     // Sequence index out of bounds
                     eprintln!("[!] Invalid sequence index {} requested for script.", sequence_idx);
                     ServerMessage::InternalError(format!("Invalid sequence index: {}", sequence_idx))
                }
            }
        },
        ClientMessage::Chat(chat_msg) => {
             // Broadcast user chat message
             let _ = state.update_sender.send(SchedulerNotification::ChatReceived(
                 client_name.clone(),
                 chat_msg
             ));
             ServerMessage::Success
        },
        ClientMessage::SetName(new_name) => {
             // Update client name in shared list and broadcast the change
             let mut clients_guard = state.clients.lock().await;
             let old_name = client_name.clone();
             let is_new_client = *client_name == DEFAULT_CLIENT_NAME;

             if is_new_client {
                 println!("[👤] Client identified as: {}", new_name);
                 clients_guard.push(new_name.clone());
             } else if let Some(i) = clients_guard.iter().position(|x| *x == old_name) {
                 println!("[👤] Client {} changed name to {}", clients_guard[i], new_name);
                 clients_guard[i] = new_name.clone();
             } else {
                 // Should not happen if client is not new, but handle defensively
                 eprintln!("[!] Error: Could not find old name '{}' to replace. Adding '{}'.", old_name, new_name);
                 clients_guard.push(new_name.clone());
             }
             *client_name = new_name; // Update local name for this connection task

             let updated_clients = clients_guard.clone();
             drop(clients_guard); // Release lock before sending notification

             // Broadcast the updated client list
             let _ = state.update_sender.send(SchedulerNotification::ClientListChanged(updated_clients));

             ServerMessage::Success
        },
        ClientMessage::SchedulerControl(sched_msg) => {
             // Forward control message directly to scheduler
             if state.sched_iface.send(sched_msg).is_ok() {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send SchedulerControl message.");
                ServerMessage::InternalError("Failed to send command to scheduler.".to_string())
            }
        },
        ClientMessage::SetTempo(tempo) => {
            // Update clock and broadcast tempo change
            // Note: ClockServer methods handle internal locking
            let mut clock = Clock::from(&state.clock_server); // Creates a lightweight handle
            clock.set_tempo(tempo);
            // The clock itself might trigger Link updates; broadcast the change via scheduler notification
            let _ = state.update_sender.send(SchedulerNotification::TempoChanged(tempo));
            ServerMessage::Success
        },
        ClientMessage::GetClock => {
             // Return current clock state directly
             let clock = Clock::from(&state.clock_server);
            ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum())
        },
         ClientMessage::GetPattern => {
              // Return current pattern snapshot directly
              ServerMessage::PatternValue(state.pattern_image.lock().await.clone())
        },
         ClientMessage::GetPeers => {
              // Return current client list directly
              ServerMessage::PeersUpdated(state.clients.lock().await.clone())
         },
        ClientMessage::SetPattern(pattern) => {
            // Forward the entire pattern to the scheduler
            if state.sched_iface.send(SchedulerMessage::SetPattern(pattern)).is_ok() {
                ServerMessage::Success
            } else {
                eprintln!("[!] Failed to send SetPattern to scheduler.");
                ServerMessage::InternalError("Failed to apply pattern update to scheduler.".to_string())
            }
        },
        ClientMessage::UpdateSequenceSteps(sequence_id, steps) => {
             // Forward to scheduler
              if state.sched_iface.send(SchedulerMessage::UpdateSequenceSteps(sequence_id, steps)).is_ok() {
                 ServerMessage::Success
             } else {
                 eprintln!("[!] Failed to send UpdateSequenceSteps to scheduler.");
                 ServerMessage::InternalError("Failed to send sequence update to scheduler.".to_string())
             }
         },
        ClientMessage::SetSequenceStartStep(sequence_id, start_step) => {
             // Forward to scheduler
              if state.sched_iface.send(SchedulerMessage::SetSequenceStartStep(sequence_id, start_step)).is_ok() {
                 ServerMessage::Success
             } else {
                 eprintln!("[!] Failed to send SetSequenceStartStep to scheduler.");
                 ServerMessage::InternalError("Failed to send sequence start step update to scheduler.".to_string())
             }
        },
        ClientMessage::SetSequenceEndStep(sequence_id, end_step) => {
             // Forward to scheduler
              if state.sched_iface.send(SchedulerMessage::SetSequenceEndStep(sequence_id, end_step)).is_ok() {
                 ServerMessage::Success
             } else {
                 eprintln!("[!] Failed to send SetSequenceEndStep to scheduler.");
                 ServerMessage::InternalError("Failed to send sequence end step update to scheduler.".to_string())
             }
        },
        ClientMessage::GetSnapshot => {
            // Get pattern and clock state to build the snapshot
            let pattern = state.pattern_image.lock().await.clone();
            let clock = Clock::from(&state.clock_server);
            let snapshot = Snapshot {
                pattern,
                tempo: clock.tempo(),
                beat: clock.beat(),
                micros: clock.micros(),
                quantum: clock.quantum(),
            };
            ServerMessage::Snapshot(snapshot)
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
                // Map the notification to an optional ServerMessage to broadcast
                let broadcast_msg_opt: Option<ServerMessage> = match notification {
                    SchedulerNotification::UpdatedPattern(p) => {
                        Some(ServerMessage::PatternValue(p))
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
                    SchedulerNotification::StepPositionChanged(positions) => {
                        Some(ServerMessage::StepPosition(positions)) // This case should now be hit
                    }
                    SchedulerNotification::Nothing |
                    SchedulerNotification::UpdatedSequence(_, _) |
                    SchedulerNotification::EnableSteps(_, _) |      
                    SchedulerNotification::DisableSteps(_, _) |     
                    SchedulerNotification::UploadedScript(_, _, _) |
                    SchedulerNotification::UpdatedSequenceSteps(_, _) |
                    SchedulerNotification::AddedSequence(_) |      
                    SchedulerNotification::RemovedSequence(_) => { None }
                };

                // Send the broadcast message if one was generated
                if let Some(broadcast_msg) = broadcast_msg_opt {
                    let send_res = send_msg(&mut writer, broadcast_msg).await;
                     if send_res.is_err() {
                         eprintln!("[!] Failed broadcast update to {}", client_name);
                         break; // Assume connection broken
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
        let _ = state.update_sender.send(SchedulerNotification::ClientListChanged(updated_clients));
    } else if *client_name != *DEFAULT_CLIENT_NAME {
         eprintln!("[!] Client '{}' not found in list during cleanup.", client_name);
    }

    Ok(client_name) // Return the final name for logging by the caller
}