use std::{
    io::ErrorKind, sync::{mpsc::Sender, Arc}
};

use client::ClientMessage;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    select, signal,
    sync::{watch, Mutex},
};

use crate::{
    clock::{Clock, ClockServer, SyncTime}, device_map::DeviceMap, pattern::Pattern, protocol::TimedMessage, schedule::{SchedulerMessage, SchedulerNotification}
};

pub mod client;

/// The byte value used to mark the end of a JSON message sent over TCP.
pub const ENDING_BYTE: u8 = 0x07;
pub const DEFAULT_CLIENT_NAME: &str = "Unknown musician";

/// Holds the shared state accessible by each client connection handler.
/// This includes interfaces to the core components of the application like
/// the clock, the world (for OSC messages), and the scheduler.
#[derive(Clone)]
pub struct ServerState {
    /// Shared access to the central clock server.
    pub clock_server: Arc<ClockServer>,
    /// Sender channel to communicate with the "world" (e.g., sending OSC).
    pub devices: Arc<DeviceMap>,
    /// Sender channel to communicate with the scheduler.
    pub world_iface: Sender<TimedMessage>,
    /// Sender channel to communicate with the scheduler.
    pub sched_iface: Sender<SchedulerMessage>,
    /// Sender for broadcasting server-wide updates (like tempo/client changes).
    pub update_sender: watch::Sender<SchedulerNotification>,
    /// Receiver channel to get notifications about scheduler updates (like pattern changes).
    pub update_receiver: watch::Receiver<SchedulerNotification>,
    /// List of connected clients
    pub clients: Arc<Mutex<Vec<String>>>,
    /// The current pattern image
    pub pattern_image: Arc<Mutex<Pattern>>,
}

impl ServerState {
    /// Creates a new ServerState instance.
    pub fn new(
        pattern_image: Arc<Mutex<Pattern>>,
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        world_iface: Sender<TimedMessage>,
        sched_iface: Sender<SchedulerMessage>,
        update_sender: watch::Sender<SchedulerNotification>,
        update_receiver: watch::Receiver<SchedulerNotification>,
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
        }
    }
}

/// Represents the main BuboCore server application.
pub struct BuboCoreServer {
    /// The IP address the server listens on.
    pub ip: String,
    /// The port number the server listens on.
    pub port: u16,
}

/// Enumerates the messages that the server can send back to a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// A log message, potentially timestamped.
    LogMessage(TimedMessage),
    /// A chat message.
    Chat(String),
    /// The current step position within patterns.
    StepPosition(Vec<usize>),
    /// Generate a greeting message for the client:
    /// also includes the current pattern, devices, and clients.
    /// The client will block until the server sends this message.
    Hello { pattern : Pattern, devices : Vec<(String, String)>, clients : Vec<String> },
    /// The current value (state) of the entire pattern.
    PatternValue(Pattern),
    /// The layout definition of a pattern.
    PatternLayout(Vec<Vec<(f64, bool)>>),
    /// The current state of the master clock (tempo, beat, time, quantum).
    ClockState(f64, f64, SyncTime, f64),
    /// A generic success response to a client request.
    Success,
    /// Indicates an internal error occurred while processing a request.
    InternalError(String),
    /// Broadcasts the updated list of connected peer names.
    PeersUpdated(Vec<String>),
}

/// Generates a welcome message for a newly connected client containing the current server state.
/// # Arguments
/// * `state` - Reference to the server state containing pattern, devices and client information
/// # Returns
/// A `ServerMessage::Hello` containing the current server state
async fn generate_hello(state : &ServerState) -> ServerMessage {
    ServerMessage::Hello {
        pattern: state.pattern_image.lock().await.clone(),
        devices: state.devices.device_list(),
        clients: state.clients.lock().await.clone(),
    }
}

/// Processes an incoming `ClientMessage`, updates server state, and triggers notifications.
/// Returns a `ServerMessage` to be sent back *only* to the requesting client.
async fn on_message(
    msg: ClientMessage,
    state: &ServerState,
    client_name: &mut String, // The name of the client sending the message
) -> ServerMessage {
    match msg {
        ClientMessage::Chat(chat_msg) => {
            // Trigger notification for broadcast
            let _ = state.update_sender.send(SchedulerNotification::ChatReceived(client_name.clone(), chat_msg));
            ServerMessage::Success
        },
        ClientMessage::SetName(new_name) => {
            let mut clients_guard = state.clients.lock().await;
            let old_name = client_name.clone();
            let is_new_client = *client_name == DEFAULT_CLIENT_NAME;

            if is_new_client {
                println!("[👤] Client identified as: {}", new_name);
                clients_guard.push(new_name.clone());
            } else {
                if let Some(i) = clients_guard.iter().position(|x| *x == old_name) {
                    println!("[👤] Client {} changed name to {}", clients_guard[i], new_name);
                    clients_guard[i] = new_name.clone();
                } else {
                    eprintln!("[!] Error: Could not find old local name '{}' in shared list to replace. Adding new name '{}'.", old_name, new_name);
                    clients_guard.push(new_name.clone());
                }
            }
            *client_name = new_name; // Update the local name for this connection

            // Get the updated list *after* modification
            let updated_clients = clients_guard.clone();
            // Drop the lock before sending notification
            drop(clients_guard);

            // Notify all clients about the change
            let _ = state.update_sender.send(SchedulerNotification::ClientListChanged(updated_clients));

            ServerMessage::Success
        },
        ClientMessage::SchedulerControl(sched_msg) => {
            if state.sched_iface.send(sched_msg).is_ok() {
                ServerMessage::Success
            } else {
                ServerMessage::InternalError("Failed to send scheduler message".to_string())
            }
        },
        ClientMessage::SetTempo(tempo) => {
            let mut clock = Clock::from(&state.clock_server);
            clock.set_tempo(tempo);
            // Notify all clients about the tempo change
            let _ = state.update_sender.send(SchedulerNotification::TempoChanged(tempo));
            ServerMessage::Success
        },
        ClientMessage::GetClock => {
            let clock = Clock::from(&state.clock_server);
            ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum())
        },
         ClientMessage::GetPattern => {
            ServerMessage::PatternValue(state.pattern_image.lock().await.clone())
        },
         ClientMessage::GetPeers => {
             ServerMessage::PeersUpdated(state.clients.lock().await.clone())
         },
        // Note: Removed _ => catch-all. Explicitly handle messages or return an error/unknown message.
        // If new ClientMessages are added, they must be handled here.
        // _ => ServerMessage::InternalError(format!("Unhandled client message type")),
    }
}

/// Helper function to serialize and send a ServerMessage to a client writer.
async fn send_msg<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: ServerMessage) -> io::Result<()> {
    let msg_to_send = serde_json::to_vec(&msg).expect("Failed to serialize ServerMessage");
    writer.write_all(&msg_to_send).await?;
    writer.write_u8(ENDING_BYTE).await?;
    writer.flush().await?;
    Ok(())
}


impl BuboCoreServer {
    /// Creates a new `BuboCoreServer` instance.
    pub fn new(ip: String, port: u16) -> Self {
        BuboCoreServer { ip, port }
    }

    /// Starts the server and listens for incoming connections.
    pub async fn start(&self, state: ServerState) -> io::Result<()> {
        let addr = format!("{}:{}", self.ip, self.port);
        let listener = TcpListener::bind(&addr).await?;
        println!("[+] Server listening on {}", addr);

        loop {
            select! {
                 // Accept new connections
                Ok((socket, client_addr)) = listener.accept() => {
                     println!("[🔌] New connection from {}", client_addr);
                     let client_state = state.clone();
                     tokio::spawn(async move {
                         // Process the client connection
                         match process_client(socket, client_state).await {
                             Ok(client_name) => {
                                println!("[🔌] Client {} disconnected gracefully.", client_name);
                             },
                             Err(e) => {
                                 eprintln!("[!] Error handling client {}: {}", client_addr, e);
                             }
                         }
                     });
                 }
                 // Graceful shutdown on Ctrl+C
                 _ = signal::ctrl_c() => {
                    println!("
[!] Ctrl+C received, shutting down server...");
                    // TODO: Add any necessary cleanup before exiting (e.g., notify clients)
                    break;
                 }
            }
        }
        Ok(())
    }
}

/// Handles an individual client connection: processes incoming messages and broadcasts server updates.
async fn process_client(socket: TcpStream, state: ServerState) -> io::Result<String> {
    let client_addr = socket.peer_addr()?; // Get address early for logging
    let mut read_buf = Vec::with_capacity(1024);
    let (mut reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(&mut reader);
    let mut writer = BufWriter::new(&mut writer); // Use BufWriter
    let mut client_name = DEFAULT_CLIENT_NAME.to_string(); // Initial name

    // Send initial Hello message
    let hello_msg = generate_hello(&state).await;
    if send_msg(&mut writer, hello_msg).await.is_err() {
        eprintln!("[!] Failed to send Hello to {}", client_addr);
        // Don't add to client list if hello fails
        return Ok(client_name); // Return initial name
    }

    // Clone receiver for this client task
    let mut update_receiver = state.update_receiver.clone();

    // Main loop to handle client messages and server updates
    loop {
        select! {
            // Bias select to check for local reads first
            biased;

            // Read client message
            res = reader.read_until(ENDING_BYTE, &mut read_buf) => {
                 match res {
                     Ok(0) => {
                         // Connection closed by client
                         break; // Exit loop gracefully
                     },
                     Ok(_) => {
                         read_buf.pop(); // remove delimiter
                         if !read_buf.is_empty() {
                             match serde_json::from_slice::<ClientMessage>(&read_buf) {
                                 Ok(msg) => {
                                     // Process message and get direct response for *this* client
                                     let response = on_message(msg, &state, &mut client_name).await;
                                     // Send the direct response back
                                     if send_msg(&mut writer, response).await.is_err() {
                                         eprintln!("[!] Failed write response to {}", client_name);
                                         break; // Assume connection broken
                                     }
                                 },
                                 Err(e) => {
                                      eprintln!("[!] Failed to deserialize message from {}: {:?}. Raw: {:?}", client_name, e, String::from_utf8_lossy(&read_buf));
                                      // Optionally send an error message back to client here
                                      let err_resp = ServerMessage::InternalError(format!("Invalid message format: {}", e));
                                      if send_msg(&mut writer, err_resp).await.is_err() {
                                           eprintln!("[!] Failed write error response to {}", client_name);
                                           break;
                                       }
                                  }
                             }
                         }
                     },
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                         // This shouldn't happen with read_until unless buffer is full?
                         eprintln!("[!] Spurious WouldBlock reading from client {}", client_name);
                         continue;
                     }
                     Err(e) => {
                         // Other read error
                         eprintln!("[!] Error reading from client {}: {}", client_name, e);
                         break; // Assume connection broken
                     }
                 }
                read_buf.clear();
            }

            // Watch for server updates to broadcast
            update_result = update_receiver.changed() => {
                if update_result.is_err() {
                    // Channel closed, server likely shutting down
                    println!("[!] Update receiver channel closed for client {}", client_name);
                    break; // Exit loop
                }
                // Clone notification because borrow() borrows for the duration of the guard
                let notification = update_receiver.borrow().clone();

                // Determine message to broadcast based on notification type
                let broadcast_msg_opt = match notification {
                    SchedulerNotification::UpdatedPattern(p) => Some(ServerMessage::PatternValue(p)),
                    SchedulerNotification::Log(log_msg) => Some(ServerMessage::LogMessage(log_msg)),
                    SchedulerNotification::TempoChanged(_) => {
                        let clock = Clock::from(&state.clock_server);
                        Some(ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum()))
                    }
                    SchedulerNotification::ClientListChanged(clients) => {
                        Some(ServerMessage::PeersUpdated(clients))
                    }
                    SchedulerNotification::ChatReceived(sender_name, chat_msg) => {
                        // Only broadcast chat if it's not from this client itself
                        if sender_name != *client_name {
                           Some(ServerMessage::Chat(format!("({}) {}", sender_name, chat_msg)))
                        } else {
                            None // Don't echo chat back to sender
                        }
                    }
                    _ => None, // Don't broadcast unhandled notification types
                };

                // Send the broadcast message if applicable
                if let Some(broadcast_msg) = broadcast_msg_opt {
                    if send_msg(&mut writer, broadcast_msg).await.is_err() {
                        eprintln!("[!] Failed broadcast update to {}", client_name);
                         break; // Assume connection broken
                    }
                }

                 // Decide if we should *also* send ClockState periodically or after certain events
                 // For now, TempoChanged and ClientListChanged handle sending clock/peer state.
                 // Could add ClockState broadcast after PatternValue too if desired.

            }
        }
    }

    // Cleanup: Remove client from the shared list when connection loop breaks
    println!("[🔌] Cleaning up connection for client: {}", client_name);
    let mut clients_guard = state.clients.lock().await;
    if let Some(i) = clients_guard.iter().position(|x| *x == client_name) {
        clients_guard.remove(i);
         println!("[👤] Removed {} from client list.", client_name);
        // Broadcast the updated client list after removal
        let updated_clients = clients_guard.clone();
        drop(clients_guard); // Drop lock before sending notification
        let _ = state.update_sender.send(SchedulerNotification::ClientListChanged(updated_clients));
    } else {
         // This might happen if SetName was never called or failed.
         println!("[!] Client {} not found in list during cleanup.", client_name);
    }

    Ok(client_name) // Return the final name of the disconnected client
}
