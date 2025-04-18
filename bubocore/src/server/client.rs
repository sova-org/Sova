//! Defines the TCP client for interacting with the BuboCore server.

use super::{ENDING_BYTE, ServerMessage};
use crate::schedule::SchedulerMessage;
use serde::{Deserialize, Serialize};
use std::net::SocketAddrV4;
use crate::scene::Scene;
use crate::schedule::ActionTiming;
use crate::shared_types::GridSelection;
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
    SetTempo(f64, ActionTiming),
    /// Request to set the client name.
    SetName(String),
    /// Toggle multiple frames
    EnableFrames(usize, Vec<usize>, ActionTiming),
    /// Untoggle multiple frames
    DisableFrames(usize, Vec<usize>, ActionTiming),
    /// Set the script associated to line/frame
    SetScript(usize, usize, String, ActionTiming),
    /// Get the script associated to line/frame
    GetScript(usize, usize),
    /// Request the current scene data.
    GetScene,
    /// Replace the entire scene on the server.
    SetScene(Scene, ActionTiming),
    /// Request the current state of the master clock.
    GetClock,
    /// Get peer list
    GetPeers,
    /// Send a chat message to other clients.
    Chat(String),
    /// Send the updated frames vector for a line
    UpdateLineFrames(usize, Vec<f64>, ActionTiming),
    /// Insert a frame with a default value (e.g., 1.0) at the specified position.
    InsertFrame(usize, usize, ActionTiming), // line_idx, position
    /// Remove the frame at the specified position.
    RemoveFrame(usize, usize, ActionTiming), // line_idx, position
    /// Set the start frame (inclusive) for line playback loop. None resets to default (0).
    SetLineStartFrame(usize, Option<usize>, ActionTiming),
    /// Set the end frame (inclusive) for line playback loop. None resets to default (last frame).
    SetLineEndFrame(usize, Option<usize>, ActionTiming),
    /// Request a complete snapshot of the current server state (Scene, Clock, etc.).
    GetSnapshot,
    /// Informs the server about the client's current grid selection/cursor.
    UpdateGridSelection(GridSelection),
    /// Informs the server the client started editing a specific frame.
    StartedEditingFrame(usize, usize), // (line_idx, frame_idx)
    /// Informs the server the client stopped editing a specific frame.
    StoppedEditingFrame(usize, usize), // (line_idx, frame_idx)
    /// Request the current scene length.
    GetSceneLength,
    /// Set the scene length.
    SetSceneLength(usize, ActionTiming),
    /// Set a custom loop length for a specific line.
    SetLineLength(usize, Option<f64>, ActionTiming),
    /// Set the playback speed factor for a specific line.
    SetLineSpeedFactor(usize, f64, ActionTiming),
    /// Request the transport to start playback.
    TransportStart(ActionTiming),
    /// Request the transport to stop playback.
    TransportStop(ActionTiming),
    /// Request the full list of devices from the server.
    RequestDeviceList,
    /// Use ConnectMidiDeviceByName. Request connection to a specific MIDI device by its internal ID.
    ConnectMidiDeviceById(usize), // Internal Device ID
    /// Use DisconnectMidiDeviceByName. Request disconnection from a specific MIDI device by its internal ID.
    DisconnectMidiDeviceById(usize), // Internal Device ID
    /// Request connection to a specific MIDI device by its name.
    ConnectMidiDeviceByName(String), // Device Name
    /// Request disconnection from a specific MIDI device by its name.
    DisconnectMidiDeviceByName(String), // Device Name
    /// Request creation of a new virtual MIDI output device.
    CreateVirtualMidiOutput(String), // Requested device name (server assigns ID)
    /// Request to assign a device name to a specific slot ID (1-N).
    AssignDeviceToSlot(usize, String), // Slot ID, Device Name
    /// Request to unassign whatever device is in a specific slot ID (1-N).
    UnassignDeviceFromSlot(usize), // Slot ID
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
