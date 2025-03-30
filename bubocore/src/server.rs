use std::{
    io::ErrorKind, net::SocketAddrV4, sync::{mpsc::Sender, Arc}
};

use client::ClientMessage;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
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
    /// Receiver channel to get notifications about scheduler updates (like pattern changes).
    pub update_notifier: watch::Receiver<SchedulerNotification>,
    /// List of connected clients
    pub clients: Arc<Mutex<Vec<String>>>,
    /// The current pattern image
    pub pattern_image: Arc<Mutex<Pattern>>,
}

impl ServerState {
    /// Creates a new `ServerState` instance with the provided components.
    ///
    /// # Arguments
    ///
    /// * `pattern_image` - Shared reference to the current pattern being executed
    /// * `clock_server` - Shared reference to the central clock server
    /// * `devices` - Shared reference to the device mapping
    /// * `world_iface` - Channel for sending messages to the "world" (e.g. OSC)
    /// * `sched_iface` - Channel for sending control messages to the scheduler
    /// * `update_notifier` - Channel for receiving notifications about scheduler updates
    ///
    /// # Returns
    ///
    /// A new `ServerState` instance initialized with the provided components and an empty clients list
    pub fn new(
        pattern_image : Arc<Mutex<Pattern>>,
        clock_server : Arc<ClockServer>, 
        devices : Arc<DeviceMap>, 
        world_iface : Sender<TimedMessage>,
        sched_iface : Sender<SchedulerMessage>,
        update_notifier : watch::Receiver<SchedulerNotification>,
    ) -> Self {
        Self {
            pattern_image,
            clock_server,
            devices,
            world_iface,
            sched_iface,
            update_notifier,
            clients: Default::default(),
        }
    }
}

/// Represents the BuboCore TCP server configuration.
pub struct BuboCoreServer {
    /// The IP address the server will bind to.
    pub ip: String,
    /// The port number the server will listen on.
    pub port: u16,
}

/// Enumerates the possible messages the server can send to a connected client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// A log message, potentially timestamped.
    LogMessage(TimedMessage),
    /// The current step position within patterns.
    StepPosition(Vec<usize>),
    /// Generate a greeting message for the client:
    /// also includes the current pattern, devices, and clients.
    /// The client will block until the server sends this message.
    Hello { pattern : Pattern, devices : Vec<(String, String)>, clients : Vec<String> },
    PatternValue(Pattern),
    /// The layout definition of a pattern.
    PatternLayout(Vec<Vec<(f64, bool)>>),
    /// The current state of the master clock (tempo, beat, time, quantum).
    ClockState(f64, f64, SyncTime, f64),
    /// A generic success response to a client request.
    Success(String),
    /// Indicates an internal error occurred while processing a request.
    InternalError(String),
}

/// Generates a welcome message for a newly connected client containing the current server state.
///
/// This function creates a `ServerMessage::Hello` containing:
/// - The current pattern being executed
/// - A list of available devices and their types
/// - A list of currently connected clients
///
/// The client will block until receiving this message during connection handshake.
///
/// # Arguments
/// * `state` - Reference to the server state containing pattern, devices and client information
///
/// # Returns
/// A `ServerMessage::Hello` containing the current server state
async fn generate_hello(state : &ServerState) -> ServerMessage {
    ServerMessage::Hello { 
        pattern: state.pattern_image.lock().await.clone(), 
        devices: state.devices.device_list(), 
        clients: state.clients.lock().await.clone(),
    }
}

/// Processes incoming client messages and updates server state accordingly.
///
/// This function handles various client requests including:
/// - Setting/changing client names
/// - Sending control messages to the scheduler
/// - Modifying clock/tempo settings
/// - Retrieving clock state
///
/// # Arguments
///
/// * `msg` - The client message to process
/// * `state` - Mutable reference to the shared server state
/// * `client_name` - Mutable reference to the client's current name
///
/// # Returns
///
/// Returns a `ServerMessage` indicating the result of processing the request:
/// - `Success` for most successful operations
/// - `InternalError` if scheduler communication fails
/// - `ClockState` containing current timing information when requested
///
/// # Examples
///
/// ```no_run
/// let msg = ClientMessage::SetName("client1".to_string());
/// let response = on_message(msg, server_state, &mut client_name).await;
/// // Response will be ServerMessage::Success if name was set
/// ```
async fn on_message(msg: ClientMessage, mut state: ServerState, client_name: &mut String) -> ServerMessage {
    match msg {
        ClientMessage::SetName(new_name) => {
            let mut clients_guard = state.clients.lock().await;
            let display_name = new_name.clone();
            if *client_name == DEFAULT_CLIENT_NAME {
                println!("[👤] Client identified as: {}", new_name);
                clients_guard.push(new_name.clone());
                *client_name = new_name;
            } else {
                if let Some(i) = clients_guard.iter().position(|x| *x == *client_name) {
                    println!("[👤] Client {} changed name to {}", clients_guard[i], new_name);
                    clients_guard[i] = new_name.clone();
                    *client_name = new_name;
                } else {
                    eprintln!("[!] Error: Could not find old local name '{}' in shared list to replace. Adding new name '{}'.", *client_name, new_name);
                    clients_guard.push(new_name.clone());
                    *client_name = new_name;
                }
            }
            ServerMessage::Success(format!("Client name set to {}", display_name).to_string())
        },
        ClientMessage::SchedulerControl(sched_msg) => {
            if state.sched_iface.send(sched_msg).is_ok() {
                ServerMessage::Success(format!("Scheduler message sent").to_string())
            } else {
                ServerMessage::InternalError(format!("Failed to send scheduler message").to_string())
            }
        },
        ClientMessage::SetTempo(tempo) => {
            let mut clock = Clock::from(state.clock_server);
            clock.set_tempo(tempo);
            ServerMessage::Success(format!("Tempo set to {}", tempo).to_string())
        },
        ClientMessage::GetClock => {
            let clock = Clock::from(state.clock_server);
            ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum())
        },
        _ => ServerMessage::Success(format!("Unknown client message").to_string()),
    }
}

/// Converts a `SchedulerNotification` into a `ServerMessage` suitable for sending to clients.
/// This function maps internal application events to messages clients can understand.
fn generate_update_message(pattern: &SchedulerNotification) -> ServerMessage {
    match pattern {
        SchedulerNotification::Log(msg) => ServerMessage::LogMessage(msg.clone()),
        // TODO: implement more responses for other notification types (see schedule.rs)
        _ => todo!(), // Placeholder for unimplemented notification types
    }
}

/// Sends a `ServerMessage` to a client over a TCP stream.
///
/// This function serializes the message to JSON, appends a delimiter byte,
/// and writes the resulting data to the socket.
///
/// # Arguments
///
/// * `socket` - The TCP stream to write to
/// * `msg` - The server message to send
///
/// # Returns
///
/// Returns `Ok(())` if the message was successfully sent, or an error if:
/// - Message serialization fails (`ErrorKind::InvalidData`)
/// - Writing to the socket fails (`io::Error`)
async fn send_msg(socket: &mut TcpStream, msg : ServerMessage) -> io::Result<()> {
    let Ok(mut res) = serde_json::to_vec(&msg) else {
        return Err(ErrorKind::InvalidData.into());
    };
    res.push(ENDING_BYTE);
    socket.write_all(&res).await?;
    Ok(())
}

/// Handles an individual client connection by processing incoming messages and sending responses.
///
/// This function runs in a loop handling two types of events:
/// - Updates from the server state that need to be broadcast to the client
/// - Incoming messages from the client that need to be processed
///
/// The function maintains a buffer for reading messages and tracks the client's name.
/// It sends an initial hello message when the client connects.
///
/// # Arguments
///
/// * `socket` - The TCP stream for this client connection
/// * `state` - The shared server state containing scheduler and other components
///
/// # Returns
///
/// Returns the final client name when the connection is closed, wrapped in `io::Result`.
/// This allows the server to clean up any client-specific state.
///
/// # Protocol
///
/// Messages are delimited by `ENDING_BYTE`. Each message is expected to be valid JSON
/// that can be deserialized into a `ClientMessage`. Responses are sent as serialized
/// `ServerMessage` values.
async fn process_client(mut socket: TcpStream, mut state: ServerState) -> io::Result<String> {
    let mut buff = Vec::new();
    let mut ready_check = [0];
    let mut client_name: String = DEFAULT_CLIENT_NAME.to_string();

    send_msg(&mut socket, generate_hello(&state).await).await?;
    loop {
        select! {
            a = state.update_notifier.changed() => {
                if a.is_err() {
                    return Ok(client_name);
                }
                let res = generate_update_message(&state.update_notifier.borrow());
                send_msg(&mut socket, res).await?;
            },
            // Check if there is data available to read
            _ = socket.peek(&mut ready_check) => {
                let mut buf_reader = BufReader::new(&mut socket);
                let n = buf_reader.read_until(ENDING_BYTE, &mut buff).await?;
                if n == 0 {
                    return Ok(client_name);
                }
                buff.pop();
                if let Ok(msg) = serde_json::from_slice::<ClientMessage>(&buff) {
                    let res = on_message(msg, state.clone(), &mut client_name).await;
                    send_msg(&mut socket, res).await?;
                } else {
                    send_msg(&mut socket, ServerMessage::InternalError(format!("Failed to deserialize client message").to_string())).await?;
                }
                buff.clear();
            }
        };
    }
}

impl BuboCoreServer {

    /// Starts the TCP server and listens for incoming connections.
    ///
    /// Binds to the configured IP address and port. Enters a loop that accepts
    /// new client connections and spawns an asynchronous task (`process_client`)
    /// to handle each client independently. Also listens for a Ctrl+C signal
    /// for graceful shutdown.
    pub fn new(ip : String, port : u16) -> Self {
        Self { ip, port }
    }

    pub async fn start(&self, state: ServerState) -> io::Result<()> {
        println!("[↕] Starting server on {}:{}", self.ip, self.port);
        let addr = SocketAddrV4::new(
            self.ip.parse().expect("Invalid IP address format"), // Panics on invalid IP
            self.port,
        );
        let listener = TcpListener::bind(addr).await?;
        println!("[👂] Listening on {}", addr);

        loop {
            // Wait for either a new connection or a shutdown signal
            let (socket, c_addr) = tokio::select! {
                // Graceful shutdown on Ctrl+C
                _ = signal::ctrl_c() => {
                    println!("
[🛑] Shutdown signal received, stopping server.");
                    return Ok(());
                },
                // Accept a new connection
                res = listener.accept() => match res {
                    Ok((socket, addr)) => (socket, addr),
                    Err(e) => {
                        eprintln!("[!] Failed to accept connection: {}", e);
                        continue; // Continue listening even if one connection fails
                    }
                }
            };
            println!("[🎺] New client connected {}", c_addr);
            let client_state = state.clone();
            // Clone the shared client list Arc *before* spawning
            let clients_arc = state.clients.clone();

            tokio::spawn(async move {
                // client_state is moved into process_client
                let final_name_result = process_client(socket, client_state).await;

                let final_name = match final_name_result {
                    Ok(name) => name,
                    Err(e) => {
                        eprintln!("[!] Error processing client {}: {}", c_addr, e);
                        format!("(Unknown - Error on {})", c_addr)
                    }
                };

                println!("[👋] Client disconnected {} ({})", c_addr, final_name);

                if final_name != DEFAULT_CLIENT_NAME && !final_name.starts_with("(Unknown - Error") {
                    let mut guard = clients_arc.lock().await;
                    if let Some(index) = guard.iter().position(|name| *name == final_name) {
                        guard.remove(index);
                        println!("[👤] Removed client: {}", final_name);
                    } else {
                        eprintln!("[!] Could not find client name '{}' in list to remove upon disconnect.", final_name);
                    }
                } else {
                    println!("[ℹ️] No client name removal needed for {} (Name: '{}')", c_addr, final_name);
                }
            });
        }
    }
}
