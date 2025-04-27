//! Defines the core structures and enums for handling different communication protocols.
//!
//! This module provides unified ways to represent messages (`ProtocolPayload`, `ProtocolMessage`, `TimedMessage`),
//! target devices (`ProtocolDevice`), and errors (`ProtocolError`) across various protocols like MIDI, OSC, and internal logging.
//! It relies on submodules for protocol-specific implementations:
//! - `midi`: Handles MIDI message structures and interactions using the `midir` crate.
//! - `osc`: Handles OSC message structures, potentially using the `rosc` crate for encoding/decoding.
//! - `log`: Handles internal logging messages.

use std::net::{SocketAddr, UdpSocket};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display},
    sync::{Arc, Mutex},
};

use log::LogMessage;
use midi::{MIDIMessage, MIDIMessageType, MidiError, MidiIn, MidiOut, midi_constants::*};
use midir::MidiOutputConnection;
use osc::OSCMessage;
use rosc::{OscBundle, OscMessage as RoscOscMessage, OscPacket, OscTime, OscType};

use crate::clock::SyncTime;
use serde::{Deserialize, Serialize};

use crate::protocol::osc::Argument as BuboArgument;

pub mod log;
pub mod midi;
pub mod osc;

/// Represents the actual data payload for different protocols.
///
/// This enum unifies message types from various protocols (OSC, MIDI, Log)
/// into a single type for easier handling within the system.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum ProtocolPayload {
    OSC(OSCMessage),
    MIDI(MIDIMessage),
    LOG(LogMessage),
}

impl Display for ProtocolPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolPayload::OSC(m) => std::fmt::Display::fmt(m, f),
            ProtocolPayload::MIDI(m) => std::fmt::Display::fmt(m, f),
            ProtocolPayload::LOG(m) => std::fmt::Display::fmt(m, f),
        }
    }
}

/// Associates a protocol-specific payload with its target device.
///
/// Holds the message content (`payload`) and a reference-counted handle
/// to the destination `ProtocolDevice`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolMessage {
    /// The target device for this message.
    pub device: Arc<ProtocolDevice>,
    /// The actual message content (MIDI, OSC, Log).
    pub payload: ProtocolPayload,
}

impl ProtocolMessage {
    /// Sends the message to its target device immediately.
    ///
    /// Note: For time-sensitive protocols like OSC, the `time` parameter might be used
    /// internally by the device's `send` method to schedule the message appropriately
    /// (e.g., using OSC bundles with timestamps).
    ///
    /// # Arguments
    /// * `time` - The intended send time (`SyncTime`). Primarily relevant for scheduling OSC bundles.
    ///
    /// # Returns
    /// - `Ok(())` on successful sending (or queuing).
    /// - `Err(ProtocolError)` if sending fails (e.g., connection error, invalid format).
    pub fn send(self, time: SyncTime) -> Result<(), ProtocolError> {
        self.device.send(self.payload, time)
    }

    /// Wraps the `ProtocolMessage` in a `TimedMessage` with the specified timestamp.
    pub fn timed(self, time: SyncTime) -> TimedMessage {
        TimedMessage {
            message: self,
            time,
        }
    }
}

impl Display for ProtocolMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] -> Device: {}", self.payload, self.device)
    }
}

/// Represents the different types of devices the system can interact with.
///
/// Includes internal logging, MIDI input/output (physical and virtual),
/// and OSC output devices. Input devices are typically used for discovery and mapping,
/// while output devices handle sending messages.
#[derive(Serialize, Deserialize)]
pub enum ProtocolDevice {
    /// Internal logging device (typically prints to stdout).
    Log,
    /// Represents an OSC input source.
    OSCInDevice, // Placeholder, implementation needed
    /// A MIDI input device, wrapping a `MidiIn` handler.
    MIDIInDevice(Arc<Mutex<MidiIn>>),
    /// A physical MIDI output device, wrapping a `MidiOut` handler.
    MIDIOutDevice(Arc<Mutex<MidiOut>>),
    /// A virtual MIDI output device created via `midir`.
    VirtualMIDIOutDevice {
        name: String,
        /// The underlying `midir` connection, managed within a Mutex and Option.
        #[serde(skip)] // Skip serialization as MidiOutputConnection is not serializable
        connection: Arc<Mutex<Option<MidiOutputConnection>>>,
    },
    /// An OSC output device targeting a specific network address.
    OSCOutputDevice {
        /// User-defined name for the device.
        name: String,
        /// Target network address (IP and port).
        address: SocketAddr,
        /// Estimated network latency in seconds for timestamped OSC bundles.
        latency: f64,
        /// The underlying UDP socket used for sending, managed within an Arc and Option.
        #[serde(skip)] // Skip serialization as UdpSocket is not serializable
        socket: Option<Arc<UdpSocket>>,
    },
}

// Custom Debug implementation to avoid printing the full (potentially large)
// internal state of handlers like MidiIn/MidiOut/UdpSocket/MidiOutputConnection.
impl Debug for ProtocolDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolDevice::Log => write!(f, "Log"),
            ProtocolDevice::OSCInDevice => write!(f, "OSCInDevice"),
            ProtocolDevice::MIDIInDevice(arg0_mutex) => {
                // Display only the name from the MidiIn handler inside the Mutex
                let name_res = arg0_mutex.lock().map(|guard| guard.name.clone());
                f.debug_tuple("MIDIInDevice")
                    .field(&name_res.unwrap_or_else(|_| "<Mutex Poisoned>".to_string()))
                    .finish()
            }
            ProtocolDevice::MIDIOutDevice(arg0_mutex) => {
                // Display only the name from the MidiOut handler inside the Mutex
                let name_res = arg0_mutex.lock().map(|guard| guard.name.clone());
                f.debug_tuple("MIDIOutDevice")
                    .field(&name_res.unwrap_or_else(|_| "<Mutex Poisoned>".to_string()))
                    .finish()
            }
            ProtocolDevice::VirtualMIDIOutDevice {
                name,
                connection: connection_arc_mutex,
            } => {
                // Show connection status (connected/disconnected) rather than the object itself
                let connection_status = connection_arc_mutex
                    .lock()
                    .map(|guard| {
                        if guard.is_some() {
                            "<Connected>"
                        } else {
                            "<Disconnected>"
                        }
                    })
                    .unwrap_or("<Mutex Poisoned>");

                f.debug_struct("VirtualMIDIOutDevice")
                    .field("name", name)
                    .field("connection", &connection_status)
                    .finish()
            }
            ProtocolDevice::OSCOutputDevice {
                name,
                address,
                latency,
                socket,
            } => {
                // Show socket status (bound/unbound) rather than the object itself
                let socket_status = if socket.is_some() {
                    "<Bound>"
                } else {
                    "<Unbound>"
                };
                f.debug_struct("OSCOutputDevice")
                    .field("name", name)
                    .field("address", address)
                    .field("latency", latency)
                    .field("socket", &socket_status)
                    .finish()
            }
        }
    }
}

impl Display for ProtocolDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolDevice::Log => write!(f, "Log"),
            ProtocolDevice::OSCInDevice => write!(f, "OSCInDevice"),
            ProtocolDevice::MIDIInDevice(midi_in_arc_mutex) => {
                // Display the name from the MidiIn handler
                midi_in_arc_mutex
                    .lock()
                    .map_err(|_| fmt::Error) // Handle Mutex poison error
                    .and_then(|guard| std::fmt::Display::fmt(&*guard, f))
            }
            ProtocolDevice::MIDIOutDevice(midi_out_arc_mutex) => {
                // Display the name from the MidiOut handler
                midi_out_arc_mutex
                    .lock()
                    .map_err(|_| fmt::Error) // Handle Mutex poison error
                    .and_then(|guard| std::fmt::Display::fmt(&*guard, f))
            }
            // For virtual/OSC, display the assigned name directly
            ProtocolDevice::VirtualMIDIOutDevice { name, .. } => {
                write!(f, "VirtualMIDIOutDevice({})", name)
            }
            ProtocolDevice::OSCOutputDevice { name, .. } => write!(f, "OSCOutputDevice({})", name),
        }
    }
}

// Equality is based on the device's unique address/identifier string.
impl PartialEq for ProtocolDevice {
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}

impl Eq for ProtocolDevice {}

/// Represents errors that can occur during protocol operations.
#[derive(Debug)]
pub struct ProtocolError(pub String);

impl Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Protocol Error: {}", self.0)
    }
}
impl std::error::Error for ProtocolError {}

impl From<MidiError> for ProtocolError {
    fn from(value: MidiError) -> Self {
        ProtocolError(value.0)
    }
}

impl From<std::io::Error> for ProtocolError {
    fn from(e: std::io::Error) -> Self {
        ProtocolError(format!("IO Error: {}", e))
    }
}

impl From<rosc::OscError> for ProtocolError {
    fn from(e: rosc::OscError) -> Self {
        ProtocolError(format!("OSC Error: {}", e))
    }
}

impl ProtocolDevice {
    /// Attempts to establish the necessary connection for the device.
    ///
    /// - For `OSCOutputDevice`, binds a local UDP socket for sending.
    /// - For MIDI devices (physical/virtual), connection is typically handled
    ///   externally (e.g., in `DeviceMap`), so this method might be a no-op or log a message.
    /// - No action needed for `Log` or `OSCInDevice` (input connection TBD).
    ///
    /// Returns `Ok(())` if the connection exists or is established, `Err(ProtocolError)` otherwise.
    pub fn connect(&mut self) -> Result<(), ProtocolError> {
        match self {
            ProtocolDevice::OSCInDevice => {
                // Placeholder: Implement OSC input connection logic if needed
                eprintln!("[!] ProtocolDevice::connect() called for OSCInDevice (Not Implemented)");
                Ok(())
            }
            ProtocolDevice::MIDIInDevice(midi_in_arc_mutex) => {
                // Connection typically managed externally by DeviceMap::connect_midi_by_name
                // Log acquisition is safe here as it's just reading the name
                let name = midi_in_arc_mutex
                    .lock()
                    .map(|g| g.name.clone())
                    .unwrap_or_default();
                println!(
                    "[~] ProtocolDevice::connect() called for MIDIInDevice '{}'. Connection handled elsewhere.",
                    name
                );
                Ok(())
            }
            ProtocolDevice::MIDIOutDevice(midi_out_arc_mutex) => {
                // Connection typically managed externally by DeviceMap::connect_midi_by_name
                let name = midi_out_arc_mutex
                    .lock()
                    .map(|g| g.name.clone())
                    .unwrap_or_default();
                println!(
                    "[~] ProtocolDevice::connect() called for MIDIOutDevice '{}'. Connection handled elsewhere.",
                    name
                );
                Ok(())
            }
            ProtocolDevice::VirtualMIDIOutDevice {
                name,
                connection: connection_arc_mutex,
            } => {
                // Connection established during creation in DeviceMap::create_virtual_midi_port
                println!(
                    "[~] ProtocolDevice::connect() called for VirtualMIDIOutDevice '{}'",
                    name
                );
                let conn_opt_guard = connection_arc_mutex.lock().map_err(|_| {
                    ProtocolError(format!(
                        "Mutex poisoned for VirtualMIDIOutDevice '{}'",
                        name
                    ))
                })?;

                if conn_opt_guard.is_some() {
                    println!("    Already connected.");
                    Ok(())
                } else {
                    // This state implies the connection might have been dropped or creation failed partially
                    eprintln!(
                        "    Warning: VirtualMIDIOutDevice '{}' connection state is None.",
                        name
                    );
                    Err(ProtocolError(format!(
                        "VirtualMIDIOutDevice '{}' is not connected.",
                        name
                    )))
                }
            }
            ProtocolDevice::OSCOutputDevice {
                name,
                address,
                latency: _,
                socket,
            } => {
                println!(
                    "[~] ProtocolDevice::connect() called for OSCOutputDevice '{}' @ {}",
                    name, address
                );
                if socket.is_some() {
                    println!("    Already connected.");
                    Ok(())
                } else {
                    // Bind to any available local port for sending
                    let local_addr: SocketAddr = "0.0.0.0:0"
                        .parse()
                        .expect("Failed to parse local UDP bind address");
                    match UdpSocket::bind(local_addr) {
                        Ok(udp_socket) => {
                            println!(
                                "    Created UDP socket bound to {}",
                                udp_socket.local_addr()?
                            );
                            *socket = Some(Arc::new(udp_socket));
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!(
                                "[!] Failed to bind UDP socket for OSCOutputDevice '{}': {}",
                                name, e
                            );
                            Err(ProtocolError::from(e))
                        }
                    }
                }
            }
            ProtocolDevice::Log => Ok(()), // Log device doesn't need connection
        }
    }

    /// Sends a message payload using this device.
    ///
    /// Handles the protocol-specific sending logic:
    /// - `MIDIOutDevice`/`VirtualMIDIOutDevice`: Sends MIDI bytes via `midir`.
    /// - `OSCOutputDevice`: Encodes the `OSCMessage` into an OSC bundle with a timestamp
    ///   (calculated as `now + latency`) using `rosc` and sends it via the UDP socket.
    /// - `Log`: Prints the `LogMessage` content to standard output.
    /// - Other types: May return an error or do nothing.
    ///
    /// # Arguments
    /// * `message` - The `ProtocolPayload` to send.
    /// * `time` - The intended send time (`SyncTime`). Used for OSC bundle timestamps.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(ProtocolError)` on failure (e.g., invalid message format for device, network error, unconnected device).
    pub fn send(&self, message: ProtocolPayload, _time: SyncTime) -> Result<(), ProtocolError> {
        // `time` currently only used for OSC latency calculation relative to now
        match self {
            ProtocolDevice::MIDIOutDevice(midi_out_arc_mutex) => {
                let ProtocolPayload::MIDI(midi_msg) = message else {
                    return Err(ProtocolError(
                        "Invalid message format for MIDI device!".to_owned(),
                    ));
                };

                let midi_out_guard = midi_out_arc_mutex
                    .lock()
                    .map_err(|e| ProtocolError(format!("MIDIOut Mutex poisoned: {}", e)))?;
                midi_out_guard.send(midi_msg).map_err(ProtocolError::from)
            }
            ProtocolDevice::VirtualMIDIOutDevice {
                name,
                connection: connection_arc_mutex,
            } => {
                let ProtocolPayload::MIDI(midi_msg) = message else {
                    return Err(ProtocolError(
                        "Invalid message format for Virtual MIDI device!".to_owned(),
                    ));
                };

                // Need mutable access to the Option<MidiOutputConnection> inside the Mutex
                let mut conn_opt_guard = connection_arc_mutex.lock().map_err(|e| {
                    ProtocolError(format!(
                        "Virtual Connection Mutex poisoned for '{}': {}",
                        name, e
                    ))
                })?;

                if let Some(conn) = conn_opt_guard.as_mut() {
                    let bytes = midi_msg.to_bytes()?;
                    conn.send(&bytes).map_err(|e| {
                        ProtocolError(format!("Failed to send to Virtual MIDI '{}': {}", name, e))
                    })
                } else {
                    Err(ProtocolError(format!(
                        "Virtual MIDI device '{}' not connected.",
                        name
                    )))
                }
            }
            ProtocolDevice::OSCOutputDevice {
                name,
                address,
                latency,
                socket,
            } => {
                let ProtocolPayload::OSC(crate_osc_msg) = message else {
                    return Err(ProtocolError(format!(
                        "Invalid message format for OSC device '{}'!",
                        name
                    )));
                };

                if let Some(sock) = socket {
                    // Convert our internal OSC Arguments to rosc::OscType arguments
                    let rosc_args: Result<Vec<OscType>, ProtocolError> = crate_osc_msg
                        .args
                        .into_iter()
                        .map(|arg| {
                            match arg {
                                BuboArgument::Int(i) => Ok(OscType::Int(i)),
                                BuboArgument::Float(f) => Ok(OscType::Float(f)),
                                BuboArgument::String(s) => Ok(OscType::String(s)),
                                BuboArgument::Blob(b) => Ok(OscType::Blob(b)),
                                BuboArgument::Timetag(t) => Ok(OscType::Time(OscTime {
                                    seconds: (t >> 32) as u32,
                                    fractional: (t & 0xFFFFFFFF) as u32,
                                })),
                                // BuboArgument::Double(d) => Ok(OscType::Double(d)), // If needed
                                // BuboArgument::Char(c) => Ok(OscType::Char(c)),     // If needed
                                // ... etc.
                            }
                        })
                        .collect();
                    let rosc_args = rosc_args?; // Propagate potential conversion errors

                    let rosc_msg = RoscOscMessage {
                        addr: crate_osc_msg.addr,
                        args: rosc_args,
                    };

                    // Calculate OSC Timestamp (NTP format) based on Current Time + Latency
                    // This ensures messages are scheduled relative to when send() is called.
                    let now = SystemTime::now();
                    let since_epoch = now
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| ProtocolError(format!("System time error: {}", e)))?;

                    let latency_micros = (*latency * 1_000_000.0) as u64;
                    let target_time_micros = since_epoch.as_micros() as u64 + latency_micros;

                    // Convert microseconds since UNIX epoch to NTP seconds and fractional parts
                    const NTP_UNIX_OFFSET_SECS: u64 = 2_208_988_800; // Offset between 1900 (NTP) and 1970 (Unix)
                    let target_time_secs = target_time_micros / 1_000_000;
                    let target_micros_remainder = target_time_micros % 1_000_000;
                    let ntp_secs = target_time_secs + NTP_UNIX_OFFSET_SECS;
                    // Calculate fractional part: (microseconds / 1_000_000.0) * 2^32
                    let ntp_frac = ((target_micros_remainder as f64 / 1_000_000.0)
                        * (1u64 << 32) as f64) as u32;

                    let osc_time = OscTime {
                        seconds: ntp_secs as u32,
                        fractional: ntp_frac,
                    };

                    // Create an OSC bundle containing the single message with the calculated timetag
                    let bundle = OscBundle {
                        timetag: osc_time,
                        content: vec![OscPacket::Message(rosc_msg)],
                    };

                    // Encode the bundle
                    match rosc::encoder::encode(&OscPacket::Bundle(bundle)) {
                        Ok(buf) => {
                            // Send the encoded buffer to the target address
                            sock.send_to(&buf, *address)
                                .map_err(|e| ProtocolError::from(e))?; // Convert IO error
                            Ok(())
                        }
                        Err(e) => Err(ProtocolError::from(e)), // Convert OSC encoding error
                    }
                } else {
                    Err(ProtocolError(format!(
                        "OSC device '{}' socket not connected.",
                        name
                    )))
                }
            }
            ProtocolDevice::Log => {
                let ProtocolPayload::LOG(log_msg) = message else {
                    return Err(ProtocolError(
                        "Invalid message format for Log device!".to_owned(),
                    ));
                };
                // Simple stdout logging implementation
                println!("[LOG][{}] {}", log_msg.level, log_msg.msg);
                if let Some(event) = log_msg.event {
                    // Use debug formatting for the associated event if present
                    println!("    Associated Event: {:?}", event);
                }
                Ok(())
            }
            ProtocolDevice::MIDIInDevice(_) | ProtocolDevice::OSCInDevice => {
                // Cannot send to input devices
                Err(ProtocolError(format!(
                    "Cannot send message to input device: {}",
                    self.address()
                )))
            }
        }
    }

    /// Flushes any pending outgoing messages for the device, if applicable.
    ///
    /// - For MIDI devices managed by `MidiOut`, calls the underlying flush mechanism.
    /// - For UDP-based OSC, this is typically a no-op as sends are immediate.
    /// - No action needed for Log or input devices.
    pub fn flush(&self) {
        match self {
            ProtocolDevice::MIDIOutDevice(midi_out_arc_mutex) => {
                if let Ok(midi_out_guard) = midi_out_arc_mutex.lock() {
                    // Call the flush method on the MidiOut handler
                    midi_out_guard.flush();
                } else {
                    eprintln!(
                        "[!] Failed to lock MIDIOut Mutex for flush on device: {}",
                        self.address()
                    );
                }
            }
            ProtocolDevice::VirtualMIDIOutDevice {
                name,
                connection: _,
            } => {
                // midir's MidiOutputConnection doesn't expose a flush, it sends immediately.
                println!(
                    "[~] Flush called on VirtualMIDIOutDevice '{}' (no-op for midir connection)",
                    name
                );
            }
            ProtocolDevice::OSCOutputDevice { name, address, .. } => {
                // UDP sends are typically fire-and-forget, no explicit flush needed at socket level.
                println!(
                    "[~] Flush called on OSCOutputDevice '{}' @ {} (no-op for UDP)",
                    name, address
                );
            }
            ProtocolDevice::Log | ProtocolDevice::MIDIInDevice(_) | ProtocolDevice::OSCInDevice => {
                // No flushing mechanism for Log or input devices
                ()
            }
        }
    }

    /// Returns a unique string identifier or address for the device.
    ///
    /// Used for comparisons (`PartialEq`), display, and potentially for map keys.
    /// - `Log`: Returns "log".
    /// - MIDI devices (In/Out/Virtual): Returns the device name.
    /// - `OSCOutputDevice`: Returns the assigned name.
    /// - `OSCInDevice`: TBD (might need configuration).
    pub fn address(&self) -> String {
        match self {
            ProtocolDevice::Log => log::LOG_NAME.to_string(), // Use constant if available
            ProtocolDevice::OSCInDevice => "OSC_IN_ADDRESS_TBD".to_string(), // Placeholder
            ProtocolDevice::MIDIInDevice(midi_in_arc_mutex) => {
                midi_in_arc_mutex.lock().map_or_else(
                    |_| "<MIDIIn Mutex Poisoned>".to_string(),
                    |guard| guard.name.clone(), // Use the name stored in MidiIn
                )
            }
            ProtocolDevice::MIDIOutDevice(midi_out_arc_mutex) => {
                midi_out_arc_mutex.lock().map_or_else(
                    |_| "<MIDIOut Mutex Poisoned>".to_string(),
                    |guard| guard.name.clone(), // Use the name stored in MidiOut
                )
            }
            // Return the assigned name for Virtual and OSC output devices
            ProtocolDevice::VirtualMIDIOutDevice { name, .. } => name.clone(),
            ProtocolDevice::OSCOutputDevice { name, .. } => name.clone(),
        }
    }
}

impl From<MidiOut> for ProtocolDevice {
    fn from(value: MidiOut) -> Self {
        Self::MIDIOutDevice(Arc::new(Mutex::new(value)))
    }
}

impl From<MidiIn> for ProtocolDevice {
    fn from(value: MidiIn) -> Self {
        Self::MIDIInDevice(Arc::new(Mutex::new(value)))
    }
}

/// Associates a `ProtocolMessage` with a specific time (`SyncTime`).
///
/// Used for scheduling messages in time-ordered queues (like a priority queue).
/// Implements `Ord` based *inversely* on time, so earlier times have higher priority.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimedMessage {
    /// The underlying message (payload and device).
    pub message: ProtocolMessage,
    /// The timestamp associated with the message.
    pub time: SyncTime,
}

impl Display for TimedMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Delegate formatting to ProtocolMessage and add the time
        write!(f, "{} @ Time: {}", self.message, self.time)
    }
}

impl TimedMessage {
    /// Consumes the `TimedMessage` and returns its components.
    pub fn untimed(self) -> (ProtocolMessage, SyncTime) {
        (self.message, self.time)
    }
}

impl Eq for TimedMessage {}

/// Ordering for `TimedMessage` is based on the `time` field, but reversed.
///
/// This means messages with *earlier* timestamps are considered "greater",
/// making them higher priority in a standard `BinaryHeap` (which acts as a min-heap
/// based on this reversed ordering, effectively becoming a max-heap on priority/earliness).
impl Ord for TimedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse the comparison: earlier time means greater ordering priority
        other.time.cmp(&self.time)
    }
}

/// Partial ordering follows the total ordering defined by `Ord`.
impl PartialOrd for TimedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// --- From implementations for ProtocolPayload ---

impl From<OSCMessage> for ProtocolPayload {
    fn from(value: OSCMessage) -> Self {
        Self::OSC(value)
    }
}

impl From<MIDIMessage> for ProtocolPayload {
    fn from(value: MIDIMessage) -> Self {
        Self::MIDI(value)
    }
}

impl From<LogMessage> for ProtocolPayload {
    fn from(value: LogMessage) -> Self {
        Self::LOG(value)
    }
}

// --- MIDI Byte Conversion ---

impl MIDIMessage {
    /// Converts the `MIDIMessage` payload into its raw byte representation.
    ///
    /// Handles standard MIDI message types and System Exclusive messages.
    /// Returns `Err(MidiError)` if the message type is unsupported or invalid.
    pub fn to_bytes(&self) -> Result<Vec<u8>, MidiError> {
        // Combine status byte prefix with channel (0-15)
        let channel_nybble = self.channel & 0x0F; // Ensure channel is within 0-15
        match self.payload {
            MIDIMessageType::NoteOn { note, velocity } => {
                Ok(vec![NOTE_ON_MSG | channel_nybble, note, velocity])
            }

            MIDIMessageType::NoteOff { note, velocity } => {
                Ok(vec![NOTE_OFF_MSG | channel_nybble, note, velocity])
            }

            MIDIMessageType::ControlChange { control, value } => {
                Ok(vec![CONTROL_CHANGE_MSG | channel_nybble, control, value])
            }

            MIDIMessageType::ProgramChange { program } => {
                Ok(vec![PROGRAM_CHANGE_MSG | channel_nybble, program])
            }

            MIDIMessageType::Aftertouch { note, value } =>
            // Polyphonic Aftertouch
            {
                Ok(vec![AFTERTOUCH_MSG | channel_nybble, note, value])
            }

            MIDIMessageType::ChannelPressure { value } =>
            // Channel Aftertouch
            {
                Ok(vec![CHANNEL_PRESSURE_MSG | channel_nybble, value])
            }

            MIDIMessageType::PitchBend { value } => {
                // Ensure value is within 14-bit range (0-16383)
                let clamped_value = value.clamp(0, 0x3FFF);
                Ok(vec![
                    PITCH_BEND_MSG | channel_nybble,
                    (clamped_value & 0x7F) as u8, // LSB (7 bits)
                    (clamped_value >> 7) as u8,   // MSB (7 bits)
                ])
            }

            // System Common Messages (no channel)
            MIDIMessageType::Clock => Ok(vec![CLOCK_MSG]),
            MIDIMessageType::Continue => Ok(vec![CONTINUE_MSG]),
            MIDIMessageType::Reset => Ok(vec![RESET_MSG]),
            MIDIMessageType::Start => Ok(vec![START_MSG]),
            MIDIMessageType::Stop => Ok(vec![STOP_MSG]),

            // System Exclusive
            MIDIMessageType::SystemExclusive { ref data } => {
                // Ensure data doesn't contain the End SysEx byte prematurely
                if data.iter().any(|&b| b == SYSTEM_EXCLUSIVE_END_MSG) {
                    return Err(MidiError("SysEx data cannot contain F7 byte".to_string()));
                }
                let mut message = Vec::with_capacity(data.len() + 2);
                message.push(SYSTEM_EXCLUSIVE_MSG);
                message.extend(data);
                message.push(SYSTEM_EXCLUSIVE_END_MSG);
                Ok(message)
            }
            // Undefined/Raw byte (pass through)
            MIDIMessageType::Undefined(byte) => Ok(vec![byte]),
        }
    }
}
