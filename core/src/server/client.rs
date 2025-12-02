//! Defines the TCP client for interacting with the Sova server.

use super::ServerMessage;
use crate::log_eprintln;
use crate::scene::{Frame, Line, Scene};
use crate::schedule::ActionTiming;
use crate::schedule::SchedulerMessage;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
};

/// Message compression strategy based on content type and frequency
#[derive(Debug, Clone, Copy)]
pub enum CompressionStrategy {
    /// Never compress (frequent, small messages)
    Never,
    /// Always compress (large content)
    Always,
    /// Compress based on size threshold
    Adaptive,
}

/// Enumerates the messages that a client can send to the Sova server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Send a control command to the scheduler.
    SchedulerControl(SchedulerMessage),
    /// Request to set the master tempo.
    SetTempo(f64, ActionTiming),
    /// Request to set the client name.
    SetName(String),

    /// Request the current scene data.
    GetScene,
    /// Replace the entire scene on the server.
    SetScene(Scene, ActionTiming),

    /// Request a specific line
    GetLine(usize),
    /// Replace specified lines
    SetLines(Vec<(usize, Line)>, ActionTiming),
    /// Configure properties (omitting frames) for a specific line
    ConfigureLines(Vec<(usize, Line)>, ActionTiming),
    /// Insert a line at given index
    AddLine(usize, Line, ActionTiming),
    /// Remove the line at given index
    RemoveLine(usize, ActionTiming),

    /// Request a specific frame
    GetFrame(usize, usize),
    /// Replace specified frames
    SetFrames(Vec<(usize, usize, Frame)>, ActionTiming),
    /// Insert a frame a specified index
    AddFrame(usize, usize, Frame, ActionTiming),
    /// Remove a frame at specified index
    RemoveFrame(usize, usize, ActionTiming),

    /// Request the current state of the master clock.
    GetClock,
    /// Get peer list
    GetPeers,
    /// Send a chat message to other clients.
    Chat(String),
    /// Request a complete snapshot of the current server state (Scene, Clock, etc.).
    GetSnapshot,
    /// Informs the server the client started editing a specific frame.
    StartedEditingFrame(usize, usize), // (line_idx, frame_idx)
    /// Informs the server the client stopped editing a specific frame.
    StoppedEditingFrame(usize, usize), // (line_idx, frame_idx)
    /// Request the transport to start playback.
    TransportStart(ActionTiming),
    /// Request the transport to stop playback.
    TransportStop(ActionTiming),
    /// Request the full list of devices from the server.
    RequestDeviceList,
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
}

impl ClientMessage {
    /// Get the compression strategy for this message type based on semantics
    pub fn compression_strategy(&self) -> CompressionStrategy {
        match self {
            // Real-time/frequent messages that should never be compressed
            ClientMessage::StartedEditingFrame(_, _)
            | ClientMessage::StoppedEditingFrame(_, _)
            | ClientMessage::GetClock
            | ClientMessage::GetPeers
            | ClientMessage::GetScene
            | ClientMessage::GetSnapshot
            | ClientMessage::RequestDeviceList => CompressionStrategy::Never,

            // Large content messages that should always be compressed if beneficial
            ClientMessage::SetScene(_, _) | ClientMessage::SetLines(_, _) => {
                CompressionStrategy::Always
            }

            // Everything else uses adaptive compression
            _ => CompressionStrategy::Adaptive,
        }
    }

    /// Deserializes a MessagePack buffer into a ClientMessage
    pub fn deserialize(final_bytes: &[u8]) -> io::Result<Option<Self>> {
        match rmp_serde::from_slice::<ClientMessage>(final_bytes) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("MessagePack deserialization error: {}", e),
            )),
        }
    }
}

/// Simple buffer pool to reduce allocations
struct BufferPool {
    small_buffers: Vec<Vec<u8>>, // < 1KB buffers
    large_buffers: Vec<Vec<u8>>, // >= 1KB buffers
}

impl BufferPool {
    fn new() -> Self {
        BufferPool {
            small_buffers: Vec::new(),
            large_buffers: Vec::new(),
        }
    }

    fn get_buffer(&mut self, size: usize) -> Vec<u8> {
        if size < 1024 {
            self.small_buffers
                .pop()
                .map(|mut buf| {
                    buf.clear();
                    buf.reserve(size);
                    buf
                })
                .unwrap_or_else(|| Vec::with_capacity(size.max(512)))
        } else {
            self.large_buffers
                .pop()
                .map(|mut buf| {
                    buf.clear();
                    buf.reserve(size);
                    buf
                })
                .unwrap_or_else(|| Vec::with_capacity(size.max(2048)))
        }
    }

    #[allow(dead_code)]
    fn return_buffer(&mut self, mut buffer: Vec<u8>) {
        if buffer.capacity() < 1024 && self.small_buffers.len() < 8 {
            buffer.clear();
            self.small_buffers.push(buffer);
        } else if buffer.capacity() >= 1024 && self.large_buffers.len() < 4 {
            buffer.clear();
            self.large_buffers.push(buffer);
        }
        // Otherwise let it drop to avoid memory bloat
    }
}

/// Represents a client connection to a Sova server.
pub struct SovaClient {
    /// The IP address of the server to connect to.
    pub ip: String,
    /// The port number of the server to connect to.
    pub port: u16,
    /// The underlying TCP stream, established after connection.
    pub stream: Option<TcpStream>,
    /// Flag indicating whether the client believes it's currently connected.
    pub connected: bool,
    /// Buffer pool to reduce allocations
    buffer_pool: BufferPool,
}

impl SovaClient {
    /// Creates a new `SovaClient` instance with the target server address.
    /// Note: This does not establish a connection yet.
    pub fn new(ip: String, port: u16) -> Self {
        SovaClient {
            ip,
            port,
            stream: None,
            connected: false,
            buffer_pool: BufferPool::new(),
        }
    }

    /// Attempts to establish a TCP connection to the configured server address.
    /// Stores the resulting `TcpStream` if successful and sets `connected` to true.
    pub async fn connect(&mut self) -> io::Result<()> {
        let addr = format!("{}:{}", self.ip, self.port);
        self.stream = Some(TcpStream::connect(&addr).await?);
        self.connected = true;
        Ok(())
    }

    /// Serializes a `ClientMessage` into MessagePack, uses intelligent compression,
    /// prepends an optimized header, and sends it to the server.
    /// Sets `connected` to false if a write error occurs.
    pub async fn send(&mut self, message: ClientMessage) -> io::Result<()> {
        // Serialize to MessagePack
        let msgpack_bytes = rmp_serde::to_vec_named(&message).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize ClientMessage to MessagePack: {}", e),
            )
        })?;

        // Use intelligent compression based on message type
        let (final_bytes, is_compressed) = self.compress_intelligently(&message, &msgpack_bytes)?;

        // Use old 4-byte header format for compatibility: [length_with_compression_flag: u32]
        let mut length = final_bytes.len() as u32;
        if is_compressed {
            length |= 0x80000000; // Set high bit to indicate compression
        }

        // Get socket after all data preparation
        let socket = self.mut_socket()?;

        // Write old-style header (4 bytes total)
        if let Err(e) = socket.write_all(&length.to_be_bytes()).await {
            self.connected = false;
            return Err(e);
        }

        // Write payload
        if let Err(e) = socket.write_all(&final_bytes).await {
            self.connected = false;
            return Err(e);
        }

        Ok(())
    }

    /// Intelligently compress message based on type and content, using buffer pool
    fn compress_intelligently(
        &mut self,
        message: &ClientMessage,
        msgpack_bytes: &[u8],
    ) -> io::Result<(Vec<u8>, bool)> {
        match message.compression_strategy() {
            CompressionStrategy::Never => {
                // Never compress frequent/small messages - reuse buffer
                let mut buffer = self.buffer_pool.get_buffer(msgpack_bytes.len());
                buffer.extend_from_slice(msgpack_bytes);
                Ok((buffer, false))
            }
            CompressionStrategy::Always => {
                // Always compress large content, but only if beneficial
                if msgpack_bytes.len() > 64 {
                    let compression_level = if msgpack_bytes.len() < 1024 { 1 } else { 3 };
                    let compressed = zstd::encode_all(msgpack_bytes, compression_level)
                        .map_err(|e| io::Error::other(format!("Compression failed: {}", e)))?;
                    // Only use compressed if it's actually smaller
                    if compressed.len() < msgpack_bytes.len() {
                        Ok((compressed, true))
                    } else {
                        let mut buffer = self.buffer_pool.get_buffer(msgpack_bytes.len());
                        buffer.extend_from_slice(msgpack_bytes);
                        Ok((buffer, false))
                    }
                } else {
                    let mut buffer = self.buffer_pool.get_buffer(msgpack_bytes.len());
                    buffer.extend_from_slice(msgpack_bytes);
                    Ok((buffer, false))
                }
            }
            CompressionStrategy::Adaptive => {
                // Original size-based logic - use buffer pool
                if msgpack_bytes.len() < 256 {
                    let mut buffer = self.buffer_pool.get_buffer(msgpack_bytes.len());
                    buffer.extend_from_slice(msgpack_bytes);
                    Ok((buffer, false))
                } else {
                    let compression_level = if msgpack_bytes.len() < 1024 { 1 } else { 3 };
                    let compressed = zstd::encode_all(msgpack_bytes, compression_level)
                        .map_err(|e| io::Error::other(format!("Compression failed: {}", e)))?;
                    Ok((compressed, true))
                }
            }
        }
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

    pub async fn disconnect(&mut self) -> io::Result<()> {
        self.connected = false;
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
        }
        Ok(())
    }
    /// Reads an optimized header and message payload from the server.
    /// Handles both old (4-byte) and new (8-byte) header formats for compatibility.
    /// Sets `connected` to false if reads fail or indicate disconnection.
    pub async fn read(&mut self) -> io::Result<ServerMessage> {
        if !self.connected {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client not connected",
            ));
        }
        let socket = self.mut_socket()?;

        // Read old 4-byte header format: [length_with_compression_flag: u32]
        let mut len_buf = [0u8; 4];
        if let Err(e) = socket.read_exact(&mut len_buf).await {
            self.connected = false;
            return Err(e);
        }

        let len_with_flag = u32::from_be_bytes(len_buf);
        let is_compressed = (len_with_flag & 0x80000000) != 0;
        let length = len_with_flag & 0x7FFFFFFF;

        if length == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Received zero-length message",
            ));
        }

        // Read the message payload
        let mut message_buf = vec![0u8; length as usize];
        if let Err(e) = socket.read_exact(&mut message_buf).await {
            self.connected = false;
            return Err(e);
        }

        // Decompress if needed
        let final_bytes = if is_compressed {
            zstd::decode_all(message_buf.as_slice()).map_err(|e| {
                log_eprintln!("[!] Failed to decompress Zstd data from server: {}", e);
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Zstd decompression failed: {}", e),
                )
            })?
        } else {
            message_buf
        };

        // Deserialize MessagePack
        rmp_serde::from_slice::<ServerMessage>(&final_bytes).map_err(|e| {
            log_eprintln!("[!] Failed to deserialize MessagePack from server: {}", e);
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("MessagePack deserialization failed: {}", e),
            )
        })
    }
}
