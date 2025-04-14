//! Defines the TCP client for interacting with the BuboCore server.

use super::{ENDING_BYTE, ServerMessage};
use crate::schedule::SchedulerMessage;
use serde::{Deserialize, Serialize};
use std::net::SocketAddrV4;
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpSocket, TcpStream},
};

/// Enumerates the messages that a client can send to the BuboCore server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Send a control command to the scheduler.
    SchedulerControl(SchedulerMessage),
    /// Request to set the master tempo.
    SetTempo(f64),
    /// Request to set the client name.
    SetName(String),
    /// Toggle multiple steps
    EnableSteps(usize, Vec<usize>),
    /// Untoggle multiple steps
    DisableSteps(usize, Vec<usize>),
    /// Set the script associated to sequence/step
    SetScript(usize, usize, String),
    /// Get the script associated to sequence/step
    GetScript(usize, usize),
    /// Request the current pattern data.
    GetPattern,
    /// Replace the entire pattern on the server.
    SetPattern(crate::pattern::Scene),
    /// Request the current state of the master clock.
    GetClock,
    /// Get peer list
    GetPeers,
    /// Send a chat message to other clients.
    Chat(String),
    /// Send the updated steps vector for a sequence
    UpdateSequenceSteps(usize, Vec<f64>),
    /// Insert a step with a default value (e.g., 1.0) at the specified position.
    InsertStep(usize, usize), // sequence_idx, position
    /// Remove the step at the specified position.
    RemoveStep(usize, usize), // sequence_idx, position
    /// Set the start step (inclusive) for sequence playback loop. None resets to default (0).
    SetSequenceStartStep(usize, Option<usize>),
    /// Set the end step (inclusive) for sequence playback loop. None resets to default (last step).
    SetSequenceEndStep(usize, Option<usize>),
    /// Request a complete snapshot of the current server state (Pattern, Clock, etc.).
    GetSnapshot,
    /// Informs the server about the client's current grid selection/cursor.
    UpdateGridSelection(crate::shared_types::GridSelection),
    /// Informs the server the client started editing a specific step.
    StartedEditingStep(usize, usize), // (sequence_idx, step_idx)
    /// Informs the server the client stopped editing a specific step.
    StoppedEditingStep(usize, usize), // (sequence_idx, step_idx)
}

/// Represents a client connection to a BuboCore server.
pub struct BuboCoreClient {
    /// The IP address of the server to connect to.
    pub ip: String,
    /// The port number of the server to connect to.
    pub port: u16,
    /// The underlying TCP stream, established after connection.
    pub stream: Option<TcpStream>,
    /// Flag indicating whether the client believes it's currently connected.
    pub connected: bool,
}

impl BuboCoreClient {
    /// Creates a new `BuboCoreClient` instance with the target server address.
    /// Note: This does not establish a connection yet.
    pub fn new(ip: String, port: u16) -> Self {
        BuboCoreClient {
            ip,
            port,
            stream: None,
            connected: false,
        }
    }

    /// Attempts to establish a TCP connection to the configured server address.
    /// Stores the resulting `TcpStream` if successful and sets `connected` to true.
    pub async fn connect(&mut self) -> io::Result<()> {
        let addr = SocketAddrV4::new(self.ip.parse().expect("Invalid IP format"), self.port);
        let socket = TcpSocket::new_v4()?;
        self.stream = Some(socket.connect(addr.into()).await?);
        self.connected = true;
        Ok(())
    }

    /// Serializes a `ClientMessage` into JSON, appends the `ENDING_BYTE` delimiter,
    /// and sends it to the server over the TCP stream.
    /// Sets `connected` to false if a write error occurs.
    pub async fn send(&mut self, message: ClientMessage) -> io::Result<()> {
        let mut msg = serde_json::to_vec(&message).expect("Failed to serialize ClientMessage");
        msg.push(ENDING_BYTE);
        let socket = self.mut_socket()?;
        let res = socket.write_all(&msg).await;
        if res.is_err() {
            self.connected = false;
        }
        return res;
    }

    /// Returns a mutable reference to the underlying `TcpStream` if connected.
    /// Returns `io::ErrorKind::NotConnected` if the stream is `None`.
    pub fn mut_socket(&mut self) -> io::Result<&mut TcpStream> {
        match &mut self.stream {
            Some(x) => Ok(x),
            None => Err(io::ErrorKind::NotConnected.into()),
        }
    }

    /// Returns an immutable reference to the underlying `TcpStream` if connected.
    /// Returns `io::ErrorKind::NotConnected` if the stream is `None`.
    pub fn socket(&self) -> io::Result<&TcpStream> {
        match &self.stream {
            Some(x) => Ok(x),
            None => Err(io::ErrorKind::NotConnected.into()),
        }
    }

    /// Checks if the socket is ready for reading or has been disconnected.
    ///
    /// Uses `peek` to check for available data without consuming it.
    /// Sets `connected` to false if `peek` returns an error or 0 bytes (indicating disconnection).
    /// Returns true if the socket is connected and potentially has data, false otherwise.
    pub async fn ready(&mut self) -> bool {
        let mut buf = [0];
        let Ok(socket) = self.socket() else {
            // Already know we're not connected if we can't get the socket
            return false;
        };
        let n = socket.peek(&mut buf).await;
        if n.is_err() || n.unwrap() == 0 {
            // Peek failed or returned 0 bytes, indicating disconnection
            self.connected = false;
        }
        self.connected
    }

    /// Reads data from the server until the `ENDING_BYTE` delimiter is found,
    /// then attempts to deserialize the data (excluding the delimiter) into a `ServerMessage`.
    ///
    /// Returns `io::ErrorKind::NotConnected` if the client is not connected or the
    /// connection is closed during the read.
    /// Returns `io::ErrorKind::InvalidData` if deserialization fails.
    pub async fn read(&mut self) -> io::Result<ServerMessage> {
        if !self.connected {
            return Err(io::ErrorKind::NotConnected.into());
        }
        let mut buff = Vec::new();
        let socket = self.mut_socket()?;
        let mut buf_reader = BufReader::new(socket);
        // Read into the buffer until the delimiter byte is found
        let n = buf_reader.read_until(ENDING_BYTE, &mut buff).await?;
        if n == 0 {
            // Read 0 bytes, indicating the connection was closed
            self.connected = false;
            return Err(io::ErrorKind::NotConnected.into());
        }
        buff.pop(); // Remove the delimiter byte
        // Attempt to deserialize the received JSON data
        if let Ok(msg) = serde_json::from_slice::<ServerMessage>(&buff) {
            Ok(msg)
        } else {
            eprintln!("[!] Failed to deserialize server message: {:?}", std::str::from_utf8(&buff));
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}
