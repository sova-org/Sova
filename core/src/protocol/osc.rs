use rosc::{OscBundle, OscMessage, OscPacket, OscTime, OscType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::net::{SocketAddr, UdpSocket};

use crate::clock::{Clock, SyncTime};
use crate::lang::event::ConcreteEvent;
use crate::protocol::error::ProtocolError;
use crate::protocol::payload::ProtocolPayload;

/// Represents the different types of arguments an OSC (Open Sound Control) message can contain.
///
/// This enum covers common OSC argument types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Argument {
    /// An OSC 32-bit integer (`i`).
    Int(i32),
    /// An OSC 32-bit float (`f`).
    Float(f32),
    /// An OSC string (`s`).
    String(String),
    /// An OSC blob (binary data) (`b`).
    Blob(Vec<u8>),
    /// An OSC 64-bit timetag (`t`), usually representing NTP time.
    Timetag(u64),
    // Other types like Double(f64), Char(char), RGBA(u32), Midi(Vec<u8>), etc.,
    // can be added here if needed in the future.
}

// Manual implementation of Eq because f32 doesn't derive Eq.
// PartialEq is already derived and handles f32 comparison appropriately (within tolerance).
// This Eq implementation relies on the PartialEq logic.
impl Eq for Argument {}

// Manual implementation of Hash because f32 doesn't derive Hash.


/// Represents a single OSC message, consisting of an address pattern and a list of arguments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OSCMessage {
    /// The OSC address pattern (e.g., "/synth/play").
    pub addr: String,
    /// The list of arguments associated with the message.
    pub args: Vec<Argument>,
    /// An optional Timetag
    pub timetag: Option<SyncTime>
}

impl Display for OSCMessage {
    /// Formats the `OSCMessage` for display, showing the address and arguments.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OSCMessage {{ addr: \"{}\", args: {:?} }}",
            self.addr, self.args
        )
    }
}

impl OSCMessage {
    /// Creates a new `OSCMessage` with the given address and arguments.
    ///
    /// # Arguments
    /// * `addr` - The OSC address pattern string.
    /// * `args` - A vector containing the `Argument` values for the message.
    pub fn new(addr: String, args: Vec<Argument>) -> Self {
        OSCMessage { addr, args, timetag: None }
    }

    /// Creates an OSC message specifically formatted for SuperDirt's `/dirt/play` address.
    ///
    /// SuperDirt expects arguments in a flattened key-value format.
    /// This function takes a `HashMap` where keys are SuperDirt parameter names
    /// (e.g., "s", "n", "amp") and values are the corresponding `Argument` types.
    /// The arguments in the resulting `OSCMessage` will be ordered as `[key1, val1, key2, val2, ...]`, though the specific order
    /// might vary due to HashMap iteration order (which is usually acceptable for SuperDirt).
    ///
    /// # Arguments
    /// * `data` - A `HashMap` mapping SuperDirt parameter names (String) to their values (`Argument`).
    /// * `cps` - The cps (cycles per second) parameter.
    /// * `cycle` - The cycle parameter.
    /// * `delta` - The delta parameter.
    /// * `orbit` - The orbit parameter.
    ///
    /// # Returns
    /// An `OSCMessage` with `addr` set to "/dirt/play" and `args` containing the flattened key-value pairs.
    pub fn dirt(
        data: HashMap<String, Argument>,
        cps: f32,
        cycle: f32,
        delta: f32,
        orbit: i32,
    ) -> Self {
        let mut args = Vec::with_capacity(data.len() * 2 + 8); // +8 for the 4 temporal key-value pairs
        // Optional: Sort keys for deterministic argument order, though usually not required by SuperDirt.

        // Unordered iteration is fine for SuperDirt:
        for (key, value) in data {
            args.push(Argument::String(key));
            args.push(value);
        }

        // Add temporal information required by SuperDirt
        args.push(Argument::String("cps".to_string()));
        args.push(Argument::Float(cps));
        args.push(Argument::String("cycle".to_string()));
        args.push(Argument::Float(cycle));
        args.push(Argument::String("delta".to_string()));
        args.push(Argument::Float(delta));
        args.push(Argument::String("orbit".to_string()));
        args.push(Argument::Int(orbit));

        OSCMessage {
            addr: "/dirt/play".to_string(),
            args,
            timetag: None
        }
    }

    pub fn generate_messages(event: ConcreteEvent, date: SyncTime, clock: &Clock) 
        -> Vec<(ProtocolPayload, SyncTime)>
    {
        match event {
            // Handle Generic OSC Event (pass-through)
            ConcreteEvent::Osc {
                mut message,
                device_id: _,
            } => {
                if message.timetag.is_none() {
                    message.timetag = Some(date);
                }
                vec![(message.into(), date)]
            }
            // Handle Dirt Event (map to /dirt/play with context)
            ConcreteEvent::Dirt { args, device_id: _ } => {
                // Calculate SuperDirt context using the clock
                let tempo_bpm = clock.tempo();
                let cps_val = tempo_bpm / 60.0;
                let cycle_val = clock.beat_at_date(date); // Beat at the event's specific time
                let delta_micros = clock.beats_to_micros(1.0); // Use 1 beat for delta
                let delta_val = delta_micros as f64 / 1_000_000.0;
                let orbit_val = 0i32; // Default orbit

                let capacity = 4 * 2 + 2 + args.len();
                let mut full_args: Vec<Argument> = Vec::with_capacity(capacity);

                // Add context parameters
                full_args.push(Argument::String("cps".to_string()));
                full_args.push(Argument::Float(cps_val as f32));
                full_args.push(Argument::String("cycle".to_string()));
                full_args.push(Argument::Float(cycle_val as f32));
                full_args.push(Argument::String("delta".to_string()));
                full_args.push(Argument::Float(delta_val as f32));
                full_args.push(Argument::String("orbit".to_string()));
                full_args.push(Argument::Int(orbit_val));

                // Add other parameters
                full_args.extend(args);

                vec![(OSCMessage {
                    addr: "/dirt/play".to_string(),
                    args: full_args,
                    timetag: Some(date)
                }.into(), date)]
            }
            // Legacy MIDI-to-OSC mappings (consider removal/refinement)
            ConcreteEvent::MidiNote(note, vel, chan, _dur, _device_id) => {
                vec![(OSCMessage {
                    addr: "/midi/noteon".to_string(),
                    args: vec![
                        Argument::Int(note as i32),
                        Argument::Int(vel as i32),
                        Argument::Int(chan as i32),
                    ],
                    timetag: Some(date)
                }.into(), date)]
            }
            ConcreteEvent::MidiControl(control, value, chan, _device_id) => {
                vec![(OSCMessage {
                    addr: "/midi/cc".to_string(),
                    args: vec![
                        Argument::Int(control as i32),
                        Argument::Int(value as i32),
                        Argument::Int(chan as i32),
                    ],
                    timetag: Some(date)
                }.into(), date)]
            }
            ConcreteEvent::MidiProgram(program, chan, _device_id) => {
                vec![(OSCMessage {
                    addr: "/midi/program".to_string(),
                    args: vec![
                        Argument::Int(program as i32),
                        Argument::Int(chan as i32),
                    ],
                    timetag: Some(date)
                }.into(), date)]
            }
            _ => Vec::new(), // Ignore other events for OSC for now
        }
    }

}

pub struct OSCOut {
    /// User-defined name to identify this device.
    pub name: String,
    /// The network address (IP and port) for destination OSC messages.
    pub address: SocketAddr,
    /// Estimated network latency (in seconds) used to calculate the timestamp
    /// of sent OSC packets (`OscBundle`).
    pub latency: f64,
    /// The UDP socket used for sending, managed in a thread-safe manner.
    pub socket: Option<UdpSocket>,
}

impl OSCOut {

    pub fn connect(&mut self) -> Result<(), ProtocolError> {
        crate::log_println!(
            "[~] connect() called for OSCOutDevice '{}' @ {}",
            self.name, self.address
        );
        if self.socket.is_some() {
            crate::log_println!("    Already connected.");
            Ok(())
        } else {
            // Bind to any available local port for sending
            let local_addr: SocketAddr = "0.0.0.0:0"
                .parse()
                .expect("Failed to parse local UDP bind address");
            match UdpSocket::bind(local_addr) {
                Ok(udp_socket) => {
                    crate::log_println!(
                        "    Created UDP socket bound to {}",
                        udp_socket.local_addr()?
                    );
                    self.socket = Some(udp_socket);
                    Ok(())
                }
                Err(e) => {
                    crate::log_eprintln!(
                        "[!] Failed to bind UDP socket for OSCOutDevice '{}': {}",
                        self.name, e
                    );
                    Err(ProtocolError::from(e))
                }
            }
        }
    }

    pub fn send(&self, message: OSCMessage) -> Result<(), ProtocolError> {
        if let Some(sock) = &self.socket {
            // Convert our internal OSC Arguments to rosc::OscType arguments
            let rosc_args: Result<Vec<OscType>, rosc::OscError> = message
                .args
                .into_iter()
                .map(|arg| {
                    match arg {
                        Argument::Int(i) => Ok(OscType::Int(i)),
                        Argument::Float(f) => Ok(OscType::Float(f)),
                        Argument::String(s) => Ok(OscType::String(s)),
                        Argument::Blob(b) => Ok(OscType::Blob(b)),
                        Argument::Timetag(t) => Ok(OscType::Time(OscTime {
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

            let rosc_msg = OscMessage {
                addr: message.addr,
                args: rosc_args,
            };
            let rosc_msg = OscPacket::Message(rosc_msg);

            let packet = if let Some(timetag) = message.timetag {
                // CRITICAL FIX: Calculate OSC Timestamp from target_time, not current time
                // This enables precise OSC bundle timestamping for sample-accurate timing
                let latency_micros = (self.latency * 1_000_000.0) as u64;
                let target_time_micros = timetag + latency_micros;

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
                OscPacket::Bundle(OscBundle {
                    timetag: osc_time,
                    content: vec![rosc_msg],
                })
            } else {
                rosc_msg
            };

            match rosc::encoder::encode(&packet) {
                Ok(buf) => {
                    // Send the encoded buffer to the target address
                    sock.send_to(&buf, self.address).map_err(ProtocolError::from)?; // Convert IO error
                    Ok(())
                }
                Err(e) => Err(ProtocolError::from(e)), // Convert OSC encoding error
            }
        } else {
            Err(ProtocolError(format!(
                "OSC device '{}' socket not connected.",
                self.name
            )))
        }
    }

}

impl fmt::Debug for OSCOut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show socket status (bound/unbound) rather than the object itself
        let socket_status = if self.socket.is_some() {
            "<Bound>"
        } else {
            "<Unbound>"
        };
        f.debug_struct("OSCOutDevice")
            .field("name", &self.name)
            .field("address", &self.address)
            .field("latency", &self.latency)
            .field("socket", &socket_status)
            .finish()
    }
}