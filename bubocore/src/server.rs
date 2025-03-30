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
    pub clients: Arc<Mutex<Vec<String>>>,
    pub pattern_image: Arc<Mutex<Pattern>>,
}

impl ServerState {

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
    Success,
    /// Indicates an internal error occurred while processing a request.
    InternalError,
}

async fn generate_hello(state : &ServerState) -> ServerMessage {
    ServerMessage::Hello { 
        pattern: state.pattern_image.lock().await.clone(), 
        devices: state.devices.device_list(), 
        clients: state.clients.lock().await.clone(),
    }
}

async fn on_message(msg: ClientMessage, mut state: ServerState, client_name: &mut String) -> ServerMessage {
    match msg {
        ClientMessage::SetName(new_name) => {
            let mut clients_guard = state.clients.lock().await;
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
            ServerMessage::Success
        },
        ClientMessage::SchedulerControl(sched_msg) => {
            println!("[📅] Sending scheduler message");
            if state.sched_iface.send(sched_msg).is_ok() {
                ServerMessage::Success
            } else {
                ServerMessage::InternalError
            }
        },
        ClientMessage::SetTempo(tempo) => {
            println!("[🕒] Setting tempo to {}", tempo);
            let mut clock = Clock::from(state.clock_server);
            clock.set_tempo(tempo);
            ServerMessage::Success
        },
        ClientMessage::GetClock => {
            println!("[🕒] Sending clock state");
            let clock = Clock::from(state.clock_server);
            ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum())
        },
        _ => ServerMessage::Success,
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

async fn send_msg(socket: &mut TcpStream, msg : ServerMessage) -> io::Result<()> {
    let Ok(mut res) = serde_json::to_vec(&msg) else {
        return Err(ErrorKind::InvalidData.into());
    };
    res.push(ENDING_BYTE);
    socket.write_all(&res).await?;
    Ok(())
}

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
                    eprintln!("[!] Failed to deserialize client message: {:?}", String::from_utf8_lossy(&buff));
                    send_msg(&mut socket, ServerMessage::InternalError).await?;
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
