use crate::clock::SyncTime;
use crate::protocol::error::ProtocolError;
use crate::protocol::log;
use crate::protocol::midi::MidiIn;
use crate::protocol::osc::Argument as BuboArgument;
use crate::protocol::{midi::MidiOut, payload::ProtocolPayload};
use midir::MidiOutputConnection;
use rosc::{OscBundle, OscMessage as RoscOscMessage, OscPacket, OscTime, OscType};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};
use std::net::UdpSocket;
// SystemTime and UNIX_EPOCH no longer needed - using target_time directly
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

/// Represents the different types of devices the system can interact with.
///
/// Each variant encapsulates the specific logic for communicating with a type
/// of device, whether it's an input (MIDI, OSC) or an output (Log, MIDI, OSC).
/// Input devices are typically used for discovery and mapping,
/// while output devices handle sending messages.
#[derive(Serialize, Deserialize)]
pub enum ProtocolDevice {
    /// Internal logging device, typically writing to standard output.
    Log,
    /// Represents an OSC input source. (Future functionality)
    OSCInDevice,
    /// A physical or virtual MIDI input device, wrapping a `MidiIn` handler.
    /// Access is shared and thread-safe via `Arc<Mutex<>>`.
    MIDIInDevice(Arc<Mutex<MidiIn>>),
    /// A physical MIDI output device, wrapping a `MidiOut` handler.
    /// Access is shared and thread-safe via `Arc<Mutex<>>`.
    MIDIOutDevice(Arc<Mutex<MidiOut>>),
    /// A virtual MIDI output device created via the `midir` library.
    VirtualMIDIOutDevice {
        /// The name given to the virtual MIDI port upon creation.
        name: String,
        /// The underlying `midir` connection, managed in a thread-safe manner.
        /// This field is not serialized.
        #[serde(skip)]
        connection: Arc<Mutex<Option<MidiOutputConnection>>>,
    },
    /// An OSC output device targeting a specific network address.
    OSCOutputDevice {
        /// User-defined name to identify this device.
        name: String,
        /// The network address (IP and port) for destination OSC messages.
        address: SocketAddr,
        /// Estimated network latency (in seconds) used to calculate the timestamp
        /// of sent OSC packets (`OscBundle`).
        latency: f64,
        /// The UDP socket used for sending, managed in a thread-safe manner.
        /// This field is not serialized.
        #[serde(skip)]
        socket: Option<Arc<UdpSocket>>,
    },
    /// Internal audio engine (Sova) - no external connectivity required
    AudioEngine,
    /// Used for system control messages (shutdown, etc.)
    Control,
}

impl ProtocolDevice {
    /// Attempts to establish or verify the necessary connection for the device.
    ///
    /// Behavior depends on the device type:
    /// - `OSCOutputDevice`: Attempts to bind a local UDP socket if one doesn't already exist.
    /// - `VirtualMIDIOutDevice`: Checks if the internal `midir` connection is active.
    /// - MIDI devices (`MIDIInDevice`, `MIDIOutDevice`): Connection is typically
    ///   managed externally (e.g., by `DeviceMap`). This method might do nothing
    ///   or display an informational message.
    /// - `Log`, `OSCInDevice`: No connection action is currently required.
    ///
    /// # Errors
    ///
    /// Returns `Err(ProtocolError)` if the connection cannot be established
    /// (e.g., UDP socket bind failure, virtual MIDI connection not found)
    /// or if the Mutex protecting the internal state is poisoned.
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
            ProtocolDevice::AudioEngine => Ok(()), // AudioEngine doesn't need external connection
            ProtocolDevice::Control => Ok(()), // Control doesn't need external connection
        }
    }

    /// Sends a message (`ProtocolPayload`) via this device.
    ///
    /// Handles protocol-specific sending logic:
    /// - `MIDIOutDevice`/`VirtualMIDIOutDevice`: Sends MIDI bytes via `midir`.
    /// - `OSCOutputDevice`: Encodes the `OSCMessage` into an OSC `OscBundle`
    ///   with a timestamp (`now + latency`) via `rosc` and sends it over the UDP socket.
    /// - `Log`: Prints the `LogMessage` content to standard output.
    /// - Input devices (`MIDIInDevice`, `OSCInDevice`): Returns an error as sending
    ///   to an input is not possible.
    ///
    /// # Arguments
    /// * `message` - The `ProtocolPayload` to send. The inner type must match
    ///   the `ProtocolDevice` type (e.g., `ProtocolPayload::MIDI` for `MIDIOutDevice`).
    /// * `target_time` - The intended execution time (`SyncTime`). Used for precise
    ///   OSC bundle timestamping to enable sample-accurate timing.
    ///
    /// # Errors
    ///
    /// Returns `Err(ProtocolError)` if:
    /// - The `message` format is incompatible with the device type.
    /// - A network error occurs (e.g., UDP send failure).
    /// - The device is not connected (socket not bound, MIDI connection absent).
    /// - An OSC encoding error occurs.
    /// - The Mutex protecting the internal state is poisoned.
    /// - The system time cannot be read.
    pub fn send(
        &self,
        message: ProtocolPayload,
        target_time: SyncTime,
    ) -> Result<(), ProtocolError> {
        // target_time used for precise OSC timestamping and protocol timing
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

                    // CRITICAL FIX: Calculate OSC Timestamp from target_time, not current time
                    // This enables precise OSC bundle timestamping for sample-accurate timing
                    let latency_micros = (*latency * 1_000_000.0) as u64;
                    let target_time_micros = target_time + latency_micros;

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
                            sock.send_to(&buf, *address).map_err(ProtocolError::from)?; // Convert IO error
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
            ProtocolDevice::AudioEngine => {
                let ProtocolPayload::AudioEngine(_) = message else {
                    return Err(ProtocolError(
                        "Invalid message format for AudioEngine device!".to_owned(),
                    ));
                };
                // AudioEngine messages are handled by World, so this should never be called
                // But if it is, just return Ok() since the routing is already handled
                Ok(())
            }
            ProtocolDevice::Control => {
                let ProtocolPayload::Control(_) = message else {
                    return Err(ProtocolError(
                        "Invalid message format for Control device!".to_owned(),
                    ));
                };
                // Control messages are handled by World, so this should never be called
                // But if it is, just return Ok() since the routing is already handled
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

    /// Flushes the outgoing message buffer for the device, if applicable.
    ///
    /// Behavior depends on the device type:
    /// - `MIDIOutDevice`: Calls the `flush` method of the underlying `MidiOut` handler.
    /// - Others (`VirtualMIDIOutDevice`, `OSCOutputDevice`, `Log`, `MIDIInDevice`, `OSCInDevice`):
    ///   This operation is typically a no-op, as sending is immediate (UDP, virtual midir)
    ///   or not applicable (Log, inputs).
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
            ProtocolDevice::Log
            | ProtocolDevice::MIDIInDevice(_)
            | ProtocolDevice::OSCInDevice
            | ProtocolDevice::AudioEngine
            | ProtocolDevice::Control => {
                // No flushing mechanism for Log, AudioEngine, Control, or input devices
            }
        }
    }

    /// Returns a unique textual identifier or address for the device.
    ///
    /// This identifier is used for comparisons (`PartialEq`), display (`Display`, `Debug`),
    /// and potentially as a key in data structures.
    /// - `Log`: Returns the string "log".
    /// - MIDI devices (Input/Output/Virtual): Returns the device name as reported
    ///   by the system or given during creation (for virtual devices).
    /// - `OSCOutputDevice`: Returns the name assigned during creation.
    /// - `OSCInDevice`: Returns a placeholder string ("OSC_IN_ADDRESS_TBD").
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
            ProtocolDevice::AudioEngine => "AudioEngine".to_string(),
            ProtocolDevice::Control => "Control".to_string(),
        }
    }
}

impl From<MidiOut> for ProtocolDevice {
    /// Creates a `ProtocolDevice::MIDIOutDevice` from a `MidiOut` handler.
    fn from(value: MidiOut) -> Self {
        Self::MIDIOutDevice(Arc::new(Mutex::new(value)))
    }
}

impl From<MidiIn> for ProtocolDevice {
    /// Creates a `ProtocolDevice::MIDIInDevice` from a `MidiIn` handler.
    fn from(value: MidiIn) -> Self {
        Self::MIDIInDevice(Arc::new(Mutex::new(value)))
    }
}

// Custom Debug implementation to avoid printing the full internal state
// of handlers (MidiIn/Out, UdpSocket, MidiOutputConnection) which can be large.
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
            ProtocolDevice::AudioEngine => write!(f, "AudioEngine"),
            ProtocolDevice::Control => write!(f, "Control"),
        }
    }
}

impl Display for ProtocolDevice {
    /// Formats the device for display, typically using its name or type.
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
            ProtocolDevice::AudioEngine => write!(f, "AudioEngine"),
            ProtocolDevice::Control => write!(f, "Control"),
        }
    }
}

impl PartialEq for ProtocolDevice {
    /// Compares two `ProtocolDevice` instances based on their `address()`.
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}

impl Eq for ProtocolDevice {}
