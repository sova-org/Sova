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
pub const DEFAULT_CLIENT_NAME: &str = "Unkown musician";

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
    pub client_name: String,
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
            client_name: DEFAULT_CLIENT_NAME.to_owned(),
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

async fn on_message(msg: ClientMessage, mut state: ServerState) -> ServerMessage {
    match msg {
        ClientMessage::SetName(name) => {
            let mut guard = state.clients.lock().await;
            let Some(i) = guard.iter().position(|x| *x == state.client_name) else {
                return ServerMessage::InternalError;
            };
            guard[i] = name.clone();
            state.client_name = name;
            ServerMessage::Success
        },
        ClientMessage::SchedulerControl(sched_msg) => {
            println!("[📅] Sending scheduler message");
            if state.sched_iface.send(sched_msg).is_ok() {
                ServerMessage::Success
            } else {
                // Indicate failure if the channel is closed (e.g., scheduler crashed)
                ServerMessage::InternalError
            }
        },
        ClientMessage::SetTempo(tempo) => {
            println!("[🕒] Setting tempo to {}", tempo);
            // Create a temporary Clock handle to modify the shared ClockServer state
            let mut clock = Clock::from(state.clock_server);
            clock.set_tempo(tempo);
            ServerMessage::Success
        },
        ClientMessage::GetClock => {
            println!("[🕒] Sending clock state");
            let clock = Clock::from(state.clock_server);
            // Respond with the current clock parameters
            ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum())
        }
        // Default success for unhandled messages for now
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

async fn process_client(mut socket: TcpStream, mut state: ServerState) -> io::Result<()> {
    let mut buff = Vec::new();
    let mut ready_check = [0];
    send_msg(&mut socket, generate_hello(&state).await).await?;
    loop {
        select! {
            // biased; // Prioritize checking for internal updates before reading from socket?

            // Watch for changes in the scheduler state
            a = state.update_notifier.changed() => {
                if a.is_err() {
                    // Error likely means the sender (scheduler) was dropped, close connection.
                    return Ok(())
                }
                // Get the latest notification and generate a message
                let res = generate_update_message(&state.update_notifier.borrow());
                send_msg(&mut socket, res).await?;
            },
            // Check if there's data to read from the client socket
            _ = socket.peek(&mut ready_check) => {
                // Use a BufReader for efficient reading up to the delimiter
                let mut buf_reader = BufReader::new(&mut socket);
                // Read until the ENDING_BYTE is encountered
                let n = buf_reader.read_until(ENDING_BYTE, &mut buff).await?;
                if n == 0 {
                    // Connection closed by client (EOF)
                    return Ok(());
                }
                buff.pop(); 
                // Attempt to deserialize the received data into a ClientMessage
                if let Ok(msg) = serde_json::from_slice::<ClientMessage>(&buff) {
                    // Process the valid message
                    let res = on_message(msg, state.clone()).await;
                    send_msg(&mut socket, res).await?;
                } else {
                    send_msg(&mut socket, ServerMessage::InternalError).await?;
                }
                buff.clear(); // Clear buffer for next message
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
            // Clone the state for the new client task
            let client_state = state.clone();

            state.clients.lock().await.push(state.client_name.clone());

            tokio::spawn(async move {
                if let Err(e) = process_client(socket, client_state).await {
                    eprintln!("[!] Error processing client {}: {}", c_addr, e);
                }
                println!("[👋] Client disconnected {}", c_addr);
            });
        }
    }
}
