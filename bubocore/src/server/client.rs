//! Defines the TCP client for interacting with the BuboCore server.

use super::ServerMessage;
use crate::scene::Scene;
use crate::schedule::ActionTiming;
use crate::schedule::SchedulerMessage;
use crate::shared_types::GridSelection;
use serde::{Deserialize, Serialize};
use std::net::SocketAddrV4;
use tokio::io::AsyncReadExt;
use tokio::{
    io::{self, AsyncWriteExt},
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
    InsertFrame(usize, usize, f64, ActionTiming), // line_idx, position, duration
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
    /// Request assignment of a device (by name) to a specific slot ID (1-N).
    AssignDeviceToSlot(usize, String), // Slot ID, Device Name
    /// Request unassignment of whatever device is in a specific slot ID (1-N).
    UnassignDeviceFromSlot(usize), // Slot ID

    // --- New OSC Messages ---
    /// Request creation of a new OSC output device.
    CreateOscDevice(String, String, u16), // name, ip_address, port
    /// Request removal of an OSC output device by its name.
    RemoveOscDevice(String), // name
    /// Request to duplicate a range of frames on a line and insert them.
    DuplicateFrameRange {
        src_line_idx: usize,
        src_frame_start_idx: usize,
        src_frame_end_idx: usize, // Inclusive
        target_insert_idx: usize,
        timing: ActionTiming,
    },
    /// Remove frames across potentially multiple lines. Inner Vec contains indices for the given line_idx.
    RemoveFramesMultiLine {
        lines_and_indices: Vec<(usize, Vec<usize>)>,
        timing: ActionTiming,
    },
    /// Request server to fetch data for duplication based on selection bounds.
    RequestDuplicationData {
        src_top: usize,
        src_left: usize,
        src_bottom: usize,
        src_right: usize,
        target_cursor_row: usize,
        target_cursor_col: usize,
        insert_before: bool, // true = 'a' (before cursor), false = 'd' (after cursor)
        timing: ActionTiming,
    },
    /// Paste a block of data (previously copied by the client) onto the grid.
    PasteDataBlock {
        /// The clipboard data (outer vec = cols, inner vec = rows).
        data: Vec<Vec<crate::shared_types::PastedFrameData>>,
        /// Target row index for the top-left corner of the paste.
        target_row: usize,
        /// Target column index for the top-left corner of the paste.
        target_col: usize,
        /// Timing for the paste action.
        timing: ActionTiming,
    },
    /// Set the name for a specific frame.
    SetFrameName(usize, usize, Option<String>, ActionTiming), // line_idx, frame_idx, name, timing
    /// Set the language identifier for a specific frame's script.
    SetScriptLanguage(usize, usize, String, ActionTiming), // line_idx, frame_idx, lang, timing
    /// Set the number of repetitions for a specific frame.
    SetFrameRepetitions(usize, usize, usize, ActionTiming), // line_idx, frame_idx, repetitions, timing
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

    /// Serializes a `ClientMessage` into MessagePack, compresses with Zstd,
    /// prepends a 4-byte length prefix, and sends it to the server.
    /// Sets `connected` to false if a write error occurs.
    pub async fn send(&mut self, message: ClientMessage) -> io::Result<()> {
        let socket = self.mut_socket()?; // Get socket ref early

        // 1. Serialize to MessagePack
        let msgpack_bytes = rmp_serde::to_vec_named(&message).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize ClientMessage to MessagePack: {}", e),
            )
        })?;

        // 2. Compress using Zstd (level 3)
        let compressed_bytes = zstd::encode_all(msgpack_bytes.as_slice(), 3).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to compress message with Zstd: {}", e),
            )
        })?;

        // 3. Get length and prepare prefix
        let len = compressed_bytes.len() as u32;
        let len_bytes = len.to_be_bytes();

        // 4. Write length prefix
        let write_len_res = socket.write_all(&len_bytes).await;
        if write_len_res.is_err() {
            self.connected = false;
            return write_len_res;
        }

        // 5. Write compressed data
        let write_data_res = socket.write_all(&compressed_bytes).await;
        if write_data_res.is_err() {
            self.connected = false;
        }
        write_data_res
    }

    /// Returns a mutable reference to the underlying `TcpStream` if connected.
    /// Returns `io::ErrorKind::NotConnected` if the stream is `None`.
    pub fn mut_socket(&mut self) -> io::Result<&mut TcpStream> {
        match &mut self.stream {
            Some(x) => Ok(x),
            None => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client not connected",
            )),
        }
    }

    /// Returns an immutable reference to the underlying `TcpStream` if connected.
    /// Returns `io::ErrorKind::NotConnected` if the stream is `None`.
    pub fn socket(&self) -> io::Result<&TcpStream> {
        match &self.stream {
            Some(x) => Ok(x),
            None => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client not connected",
            )),
        }
    }

    /// Checks if the socket is ready for reading or has been disconnected using peek.
    /// Note: This only checks if *any* data is available, not if a full message is ready.
    /// The actual read operation might still block or fail.
    /// Sets `connected` to false if peek returns an error or 0 bytes.
    pub async fn ready(&mut self) -> bool {
        let mut buf = [0];
        let Ok(socket) = self.socket() else {
            return false;
        };
        match socket.peek(&mut buf).await {
            Ok(0) => {
                // Connection closed cleanly by peer
                self.connected = false;
                false
            }
            Ok(_) => true, // Some data is likely available
            Err(_) => {
                // Error during peek likely means connection is broken
                self.connected = false;
                false
            }
        }
    }

    /// Reads a length-prefixed, Zstd-compressed, MessagePack-encoded `ServerMessage`.
    ///
    /// Reads the 4-byte length, then the compressed body, decompresses, and deserializes.
    /// Sets `connected` to false if reads fail or indicate disconnection.
    pub async fn read(&mut self) -> io::Result<ServerMessage> {
        if !self.connected {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client not connected",
            ));
        }
        let socket = self.mut_socket()?;

        // 1. Read 4-byte length prefix
        let mut len_buf = [0u8; 4];
        match socket.read_exact(&mut len_buf).await {
            Ok(_) => { /* Length read successfully */ }
            Err(e) => {
                self.connected = false; // Assume disconnected on read error
                return Err(e);
            }
        }
        let len = u32::from_be_bytes(len_buf);

        if len == 0 {
            // Handle zero-length message - might be an error or keepalive?
            // For now, treat as unexpected data.
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Received zero-length message",
            ));
        }

        // 2. Read the compressed message body
        let mut compressed_buf = vec![0u8; len as usize];
        match socket.read_exact(&mut compressed_buf).await {
            Ok(_) => { /* Body read successfully */ }
            Err(e) => {
                self.connected = false; // Assume disconnected on read error
                return Err(e);
            }
        }

        // 3. Decompress using Zstd
        let decompressed_bytes = zstd::decode_all(compressed_buf.as_slice()).map_err(|e| {
            eprintln!("[!] Failed to decompress Zstd data from server: {}", e);
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Zstd decompression failed: {}", e),
            )
        })?;

        // 4. Deserialize MessagePack
        rmp_serde::from_slice::<ServerMessage>(&decompressed_bytes).map_err(|e| {
            eprintln!("[!] Failed to deserialize MessagePack from server: {}", e);
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("MessagePack deserialization failed: {}", e),
            )
        })
    }
}
