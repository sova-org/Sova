mod control_memory;
pub mod midi_constants;

use midir::os::unix::VirtualOutput;
use midir::{MidiInput, MidiOutput, MidiOutputConnection};
use serde::{Deserialize, Serialize};

use control_memory::MidiInMemory;
use midi_constants::*;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::sync::{Arc, Mutex};

use crate::clock::SyncTime;
use crate::lang::event::ConcreteEvent;
use crate::protocol::payload::ProtocolPayload;

/// Represents an error encountered during MIDI processing.
///
/// Wraps a descriptive string detailing the error.
#[derive(Debug, Default, Clone)]
pub struct MidiError(pub String);

impl<T: ToString> From<T> for MidiError {
    /// Creates a `MidiError` from any type that implements `ToString`.
    ///
    /// This provides a convenient way to convert various error types or string literals
    /// into a `MidiError`.
    fn from(value: T) -> Self {
        MidiError(value.to_string())
    }
}

/// Represents a MIDI message, including its payload type and channel.
///
/// Channels are typically 0-15.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MIDIMessage {
    /// The specific type and data of the MIDI message.
    pub payload: MIDIMessageType,
    /// The MIDI channel (0-15) the message applies to.
    /// Ignored for System Common messages.
    pub channel: u8,
}

impl Display for MIDIMessage {
    /// Formats the MIDI message for display, including channel and payload.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MIDIMessage sur canal ({}) : [{}]",
            self.channel, self.payload
        )
    }
}

impl MIDIMessage {
    /// Converts the `MIDIMessage` payload into its raw byte representation.
    ///
    /// Handles standard MIDI message types (Note On/Off, CC, etc.) and System Exclusive messages.
    /// Combines the status byte prefix with the channel where applicable.
    /// Clamps Pitch Bend values to the valid 14-bit range.
    ///
    /// # Errors
    ///
    /// Returns `Err(MidiError)` if the `SystemExclusive` data contains the `F7` (End SysEx) byte,
    /// as this is invalid within the data payload.
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
                if data.contains(&SYSTEM_EXCLUSIVE_END_MSG) {
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

    /// Generates `ProtocolPayload`s containing `MIDIMessage` payloads from a `ConcreteEvent`.
    ///
    /// Handles mapping various `ConcreteEvent::Midi*` variants to their corresponding
    /// MIDI message types (NoteOn/Off, CC, ProgramChange, etc.).
    /// Note durations are handled by scheduling a corresponding NoteOff message.
    /// MIDI channels are converted from 1-based (in `ConcreteEvent`) to 0-based (in `MIDIMessage`).
    /// System messages (Start, Stop, etc.) are sent on channel 0.
    pub fn generate_messages(
        event: ConcreteEvent,
        date: SyncTime,
    ) -> Vec<(ProtocolPayload, SyncTime)> {
        match event {
            ConcreteEvent::MidiNote(note, vel, chan, dur, _device_id) => {
                let midi_chan = (chan.saturating_sub(1) % 16) as u8; // Convert to 0-based MIDI channel
                vec![(
                        MIDIMessage {
                            payload: MIDIMessageType::NoteOff {
                                note: note as u8,
                                velocity: 0,
                            },
                            channel: midi_chan,
                        }.into(), date
                    ),
                    // NoteOn
                    (
                        MIDIMessage {
                            payload: MIDIMessageType::NoteOn {
                                note: note as u8,
                                velocity: vel as u8,
                            },
                            channel: midi_chan,
                        }.into(), date + 1
                    ),
                    // NoteOff
                    (
                        MIDIMessage {
                            payload: MIDIMessageType::NoteOff {
                                note: note as u8,
                                velocity: 0,
                            },
                            channel: midi_chan,
                        }.into(), date + dur,
                    ),
                ]
            }
            ConcreteEvent::MidiControl(control, value, chan, _device_id) => {
                let midi_chan = (chan.saturating_sub(1) % 16) as u8;
                vec![
                    (
                        MIDIMessage {
                            payload: MIDIMessageType::ControlChange {
                                control: control as u8,
                                value: value as u8,
                            },
                            channel: midi_chan,
                        }.into(), date
                    ),
                ]
            }
            ConcreteEvent::MidiProgram(program, chan, _device_id) => {
                let midi_chan = (chan.saturating_sub(1) % 16) as u8;
                vec![
                    (
                        MIDIMessage {
                            payload: MIDIMessageType::ProgramChange {
                                program: program as u8,
                            },
                            channel: midi_chan,
                        }.into(), date
                    ),
                ]
            }
            ConcreteEvent::MidiAftertouch(note, pressure, chan, _device_id) => {
                let midi_chan = (chan.saturating_sub(1) % 16) as u8;
                vec![
                    (
                        MIDIMessage {
                            payload: MIDIMessageType::Aftertouch {
                                note: note as u8,
                                value: pressure as u8,
                            },
                            channel: midi_chan,
                        }.into(), date
                    ),
                ]
            }
            ConcreteEvent::MidiChannelPressure(pressure, chan, _device_id) => {
                let midi_chan = (chan.saturating_sub(1) % 16) as u8;
                vec![
                    (
                        MIDIMessage {
                            payload: MIDIMessageType::ChannelPressure {
                                value: pressure as u8,
                            },
                            channel: midi_chan,
                        }.into(), date
                    ),
                ]
            }
            ConcreteEvent::MidiStart(_device_id) => {
                vec![
                    (
                        MIDIMessage {
                            payload: MIDIMessageType::Start {},
                            channel: 0, // System messages use channel 0
                        }.into(), date
                    ),
                ]
            }
            ConcreteEvent::MidiStop(_device_id) => {
                vec![
                    (
                        MIDIMessage {
                            payload: MIDIMessageType::Stop {},
                            channel: 0,
                        }.into(), date
                    ),
                ]
            }
            ConcreteEvent::MidiContinue(_device_id) => {
                vec![
                    (
                        MIDIMessage {
                            payload: MIDIMessageType::Continue {},
                            channel: 0,
                        }.into(), date
                    ),
                ]
            }
            ConcreteEvent::MidiClock(_device_id) => {
                vec![
                    (
                        MIDIMessage {
                            payload: MIDIMessageType::Clock {},
                            channel: 0,
                        }.into(), date
                    ),
                ]
            }
            ConcreteEvent::MidiReset(_device_id) => {
                vec![
                    (
                        MIDIMessage {
                            payload: MIDIMessageType::Reset {},
                            channel: 0,
                        }.into(), date
                    ),
                ]
            }
            ConcreteEvent::MidiSystemExclusive(data, _device_id) => {
                let data = data.iter().map(|x| *x as u8).collect();
                vec![
                    (
                        MIDIMessage {
                            payload: MIDIMessageType::SystemExclusive { data },
                            channel: 0,
                        }.into(), date
                    ),
                ]
            }
            _ => Vec::new(), // Ignore Nop or other non-MIDI events
        }
    }

}

/// Enumerates the supported types of MIDI message payloads.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MIDIMessageType {
    /// Note On message: Starts a note playing.
    NoteOn {
        /// MIDI note number (0-127).
        note: u8,
        /// Velocity (0-127), typically indicating loudness.
        velocity: u8,
    },
    /// Note Off message: Stops a note playing.
    NoteOff {
        /// MIDI note number (0-127).
        note: u8,
        /// Release velocity (0-127), sometimes used for release characteristics.
        velocity: u8,
    },
    /// Control Change (CC) message: Modifies various parameters.
    ControlChange {
        /// Control number (0-127).
        control: u8,
        /// Control value (0-127).
        value: u8,
    },
    /// Program Change message: Selects an instrument or patch.
    ProgramChange {
        /// Program number (0-127).
        program: u8,
    },
    /// Pitch Bend message: Adjusts the pitch of sounding notes on a channel.
    PitchBend {
        /// 14-bit pitch bend value (0-16383). 8192 is typically center (no bend).
        value: u16,
    },
    /// Polyphonic Aftertouch message: Pressure applied to individual keys after initial strike.
    Aftertouch {
        /// MIDI note number (0-127).
        note: u8,
        /// Pressure value (0-127).
        value: u8,
    },
    /// Channel Pressure (Channel Aftertouch) message: Overall pressure applied after initial strike for the channel.
    ChannelPressure {
        /// Pressure value (0-127).
        value: u8,
    },
    /// System Exclusive (SysEx) message: Manufacturer-specific data.
    SystemExclusive {
        /// The raw SysEx data bytes, excluding the starting `F0` and ending `F7`.
        data: Vec<u8>,
    },
    /// MIDI Clock message: Used for timing synchronization.
    Clock,
    /// MIDI Start message: Starts sequence playback from the beginning.
    Start,
    /// MIDI Continue message: Resumes sequence playback from where it stopped.
    Continue,
    /// MIDI Stop message: Stops sequence playback.
    Stop,
    /// MIDI System Reset message: Resets devices to their default state.
    Reset,
    /// Represents an undefined or raw MIDI byte, potentially for passthrough.
    Undefined(u8),
}

impl Display for MIDIMessageType {
    /// Formats the MIDI message type and its data for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MIDIMessageType::NoteOn { note, velocity } => {
                write!(f, "NoteOn : note = {note} ; velocity = {velocity}")
            }
            MIDIMessageType::NoteOff { note, velocity } => {
                write!(f, "NoteOff : note = {note} ; velocity = {velocity}")
            }
            MIDIMessageType::ControlChange { control, value } => {
                write!(f, "ControlChange : control = {control} ; value = {value}")
            }
            MIDIMessageType::ProgramChange { program } => {
                write!(f, "ProgramChange : program = {program}")
            }
            MIDIMessageType::PitchBend { value } => write!(
                f,
                "PitchBend : pitch = {} ; bend = {}",
                value % 0x100,
                value >> 8
            ),
            MIDIMessageType::Aftertouch { note, value } => {
                write!(f, "AfterTouch : note = {note} ; value = {value}")
            }
            MIDIMessageType::ChannelPressure { value } => {
                write!(f, "ChannelPressure : value = {value}")
            }
            MIDIMessageType::SystemExclusive { data } => {
                write!(f, "SystemExclusive : data = {:?}", data)
            }
            MIDIMessageType::Clock => write!(f, "Clock"),
            MIDIMessageType::Start => write!(f, "Start"),
            MIDIMessageType::Continue => write!(f, "Continue"),
            MIDIMessageType::Stop => write!(f, "Stop"),
            MIDIMessageType::Reset => write!(f, "Reset"),
            MIDIMessageType::Undefined(x) => write!(f, "Undefined : {x}"),
        }
    }
}

/// A common interface trait for MIDI Input and Output devices.
///
/// Defines basic functionalities like creation, listing available ports,
/// and checking connection status.
pub trait MidiInterface {
    /// Creates a new instance of the MIDI interface (Input or Output).
    ///
    /// # Arguments
    /// * `client_name` - A name for the MIDI client application.
    ///
    /// # Errors
    /// Returns `Err(MidiError)` if the underlying `midir` instance cannot be created.
    fn new(client_name: String) -> Result<Self, MidiError>
    where
        Self: Sized;

    /// Returns a list of available MIDI port names for this interface type (Input or Output).
    fn ports(&self) -> Vec<String>;

    /// Checks if the interface is currently connected to a MIDI port.
    fn is_connected(&self) -> bool;
}

/// Represents a MIDI Output interface for sending messages.
///
/// Wraps a `midir::MidiOutputConnection` within an `Arc<Mutex<Option<...>>>`
/// to allow shared, thread-safe access and connection management.
/// Also tracks active notes to prevent sending duplicate Note On messages.
#[derive(Serialize, Deserialize)]
pub struct MidiOut {
    /// The name assigned to this MIDI output client/connection.
    pub name: String,
    /// The underlying `midir` output connection, managed thread-safely.
    /// This field is not serialized.
    #[serde(skip)]
    pub connection: Arc<Mutex<Option<MidiOutputConnection>>>,
    /// Tracks currently active notes per channel to avoid sending duplicate Note Ons.
    /// Maps channel (u8) to a set of active notes (u8).
    /// This field is not serialized and has a default initializer.
    #[serde(skip, default = "default_active_notes")]
    pub active_notes: Mutex<HashMap<u8, HashSet<u8>>>,
}

impl Display for MidiOut {
    /// Formats the `MidiOut` instance for display, showing its name.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiOut({})", self.name)
    }
}

impl Debug for MidiOut {
    /// Formats the `MidiOut` instance for debugging, showing its name.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiOut({})", self.name)
    }
}

impl MidiOut {
    /// Sends a `MIDIMessage` through the connected output port.
    ///
    /// Converts the `MIDIMessage` to raw bytes and sends it via the `midir` connection.
    /// Includes logic to prevent sending duplicate Note On messages for the same note/channel.
    /// Automatically handles Note Off messages only if the corresponding Note On was tracked.
    ///
    /// # Errors
    /// Returns `Err(MidiError)` if:
    /// - The connection Mutex is poisoned.
    /// - The `MidiOut` is not connected to a port.
    /// - The underlying `midir` connection fails to send the message.
    /// - The `MIDIMessage` contains invalid SysEx data (see `to_bytes`).
    pub fn send(&self, message: MIDIMessage) -> Result<(), MidiError> {
        let mut connection_opt_guard = self
            .connection
            .lock()
            .map_err(|_| MidiError("MidiOut connection Mutex poisoned".to_string()))?;

        let Some(connection) = connection_opt_guard.as_mut() else {
            return Err(
                format!("Interface MIDI {} non connectée à un port MIDI", self.name).into(),
            );
        };

        let mut active_notes_guard = self.active_notes.lock().unwrap();
        let bytes = match message.payload {
            MIDIMessageType::NoteOn { note, velocity } => {
                let channel_notes = active_notes_guard.entry(message.channel).or_default();
                if channel_notes.contains(&note) {
                    return Ok(());
                    //connection.send(&vec![NOTE_OFF_MSG + message.channel, note, velocity]);
                } else {
                    channel_notes.insert(note);
                }
                vec![NOTE_ON_MSG + message.channel, note, velocity]
            }
            MIDIMessageType::NoteOff { note, velocity } => {
                let channel_notes = active_notes_guard.entry(message.channel).or_default();
                if channel_notes.contains(&note) {
                    channel_notes.remove(&note);
                    vec![NOTE_OFF_MSG + message.channel, note, velocity]
                } else {
                    return Ok(());
                }
            }
            MIDIMessageType::ControlChange { control, value } => {
                vec![CONTROL_CHANGE_MSG + message.channel, control, value]
            }
            MIDIMessageType::ProgramChange { program } => {
                vec![PROGRAM_CHANGE_MSG + message.channel, program]
            }
            MIDIMessageType::Aftertouch { note, value } => {
                vec![AFTERTOUCH_MSG + message.channel, note, value]
            }
            MIDIMessageType::ChannelPressure { value } => {
                vec![CHANNEL_PRESSURE_MSG + message.channel, value]
            }
            MIDIMessageType::PitchBend { value } => vec![
                PITCH_BEND_MSG + message.channel,
                (value & 0x7F) as u8,
                (value >> 7) as u8,
            ],
            MIDIMessageType::Clock => vec![CLOCK_MSG],
            MIDIMessageType::Continue => vec![CONTINUE_MSG],
            MIDIMessageType::Reset => vec![RESET_MSG],
            MIDIMessageType::Start => vec![START_MSG],
            MIDIMessageType::Stop => vec![STOP_MSG],
            MIDIMessageType::SystemExclusive { ref data } => {
                let mut m = vec![0xF0];
                m.extend(data);
                m.push(0xF7);
                m
            }
            MIDIMessageType::Undefined(byte) => vec![byte],
        };

        connection
            .send(&bytes)
            .map_err(|e| format!("Échec d'envoi du message MIDI : {}", e).into())
    }

    /// Connects this `MidiOut` instance to a specific physical output port identified by its name.
    ///
    /// # Arguments
    /// * `port_name` - The exact name of the target MIDI output port.
    ///
    /// # Errors
    /// Returns `Err(MidiError)` if:
    /// - The `MidiOutput` instance cannot be created.
    /// - A port with the specified `port_name` is not found.
    /// - The connection Mutex is poisoned.
    /// - The underlying `midir` connection attempt fails.
    pub fn connect_to_port_by_name(&mut self, port_name: &str) -> Result<(), MidiError> {
        let midi_out = self.get_midi_out()?;
        let target_port = midi_out
            .ports()
            .into_iter()
            .find(|p| midi_out.port_name(p).is_ok_and(|name| name == port_name))
            .ok_or_else(|| MidiError(format!("Output port '{}' not found", port_name)))?;

        match midi_out.connect(&target_port, &self.name) {
            Ok(connection) => {
                *self.connection.lock().unwrap() = Some(connection);
                Ok(())
            }
            Err(e) => Err(format!(
                "Failed to connect '{}' to '{}': {}",
                self.name, port_name, e
            )
            .into()),
        }
    }

    /// Creates a virtual MIDI output port with the name specified in `self.name`.
    ///
    /// Other MIDI applications can connect to this virtual port to receive messages sent from this `MidiOut` instance.
    /// This functionality is typically available on macOS and Linux.
    ///
    /// # Errors
    /// Returns `Err(MidiError)` if:
    /// - The `MidiOutput` instance cannot be created.
    /// - Virtual ports are not supported on the current platform (e.g., Windows).
    /// - The connection Mutex is poisoned.
    /// - The underlying `midir` virtual port creation fails.
    pub fn create_virtual_port(&mut self) -> Result<(), MidiError> {
        let midi_out = self.get_midi_out()?;

        #[cfg(not(target_os = "windows"))]
        {
            match midi_out.create_virtual(&self.name) {
                Ok(connection) => {
                    *self.connection.lock().unwrap() = Some(connection);
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        }
        #[cfg(target_os = "windows")]
        {
            Err(MidiError(
                "Virtual MIDI ports are not supported on Windows.".to_string(),
            ))
        }
    }

    /// Connects to a default port (virtual if `use_virtual` is true).
    ///
    /// **Deprecated:** Prefer using `connect_to_port_by_name` for physical ports
    /// or `create_virtual_port` for virtual ports.
    #[deprecated(note = "Prefer connect_to_port_by_name or create_virtual_port")]
    pub fn connect_to_default(&mut self, use_virtual: bool) -> Result<(), MidiError> {
        if use_virtual {
            self.create_virtual_port()
        } else {
            Err(MidiError(
                "Connecting to default physical port is deprecated. Use connect_to_port_by_name."
                    .to_string(),
            ))
            // Original logic connecting to ports[0] removed.
            // let midi_out = self.get_midi_out()?;
            // let ports = midi_out.ports();
            // if ports.is_empty() {
            //     return Err("Aucun port MIDI disponible".into());
            // }
            // midi_out.connect(&ports[0], &self.name).map_err(|e| e.into())
            // ... assignment logic ...
        }
    }

    /// Creates a temporary `midir::MidiOutput` instance.
    /// Used internally to query ports or establish connections.
    fn get_midi_out(&self) -> Result<MidiOutput, MidiError> {
        MidiOutput::new(&self.name).map_err(|e| e.into())
    }

    /// Flushes any pending outgoing MIDI messages.
    ///
    /// Note: For `midir`, sends are typically immediate, so this is often a no-op.
    pub fn flush(&self) {}
}

impl Drop for MidiOut {
    /// Ensures the MIDI connection is closed when the `MidiOut` instance is dropped.
    fn drop(&mut self) {
        if let Ok(mut c) = self.connection.lock() {
            c.take();
        }
    }
}

impl MidiInterface for MidiOut {
    /// Creates a new, unconnected `MidiOut` instance.
    fn new(name: String) -> Result<Self, MidiError> {
        Ok(MidiOut {
            name,
            connection: Arc::new(Mutex::new(None)),
            active_notes: Mutex::new(HashMap::new()),
        })
    }

    /// Returns a list of available MIDI output port names.
    fn ports(&self) -> Vec<String> {
        self.get_midi_out()
            .map(|m| {
                m.ports()
                    .iter()
                    .map(|p| m.port_name(p).unwrap_or_default())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Checks if this `MidiOut` instance is currently connected to a port.
    fn is_connected(&self) -> bool {
        self.connection.lock().unwrap().is_some()
    }
}

/// Represents a MIDI Input interface for receiving messages.
///
/// Wraps a `midir::MidiInputConnection` within an `Arc<Mutex<Option<...>>>`
/// for connection management and includes an `Arc<Mutex<MidiInMemory>>` to store
/// the state of received Control Change messages.
#[derive(Serialize, Deserialize)]
pub struct MidiIn {
    /// The name assigned to this MIDI input client/connection.
    pub name: String,
    /// The underlying `midir` input connection, managed thread-safely.
    /// This field is not serialized.
    #[serde(skip)]
    pub connection: Arc<Mutex<Option<midir::MidiInputConnection<()>>>>,
    /// Shared, thread-safe storage for the last received value of each Control Change message
    /// per channel.
    /// This field is not serialized.
    #[serde(skip)]
    pub memory: Arc<Mutex<MidiInMemory>>,
}

impl Debug for MidiIn {
    /// Formats the `MidiIn` instance for debugging, showing its name.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiIn({})", self.name)
    }
}

impl Display for MidiIn {
    /// Formats the `MidiIn` instance for display, showing its name.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiIn({})", self.name)
    }
}

impl MidiIn {
    /// Creates a temporary `midir::MidiInput` instance.
    /// Used internally to query ports or establish connections.
    fn get_midi_in(&self) -> Result<MidiInput, MidiError> {
        MidiInput::new(&self.name).map_err(|e| e.into())
    }

    /// Connects this `MidiIn` instance to a specific physical input port identified by its name.
    ///
    /// Sets up a callback that receives incoming MIDI messages. Currently, the callback:
    /// - Prints all raw incoming messages for debugging.
    /// - Parses Control Change messages and updates the shared `MidiInMemory` state.
    /// - Prints parsed Control Change messages for debugging.
    ///
    /// # Arguments
    /// * `port_name` - The exact name of the target MIDI input port.
    ///
    /// # Errors
    /// Returns `Err(MidiError)` if:
    /// - The `MidiInput` instance cannot be created.
    /// - A port with the specified `port_name` is not found.
    /// - The connection Mutex is poisoned.
    /// - The underlying `midir` connection attempt fails.
    pub fn connect_to_port_by_name(&mut self, port_name: &str) -> Result<(), MidiError> {
        let midi_in = self.get_midi_in()?;
        let target_port = midi_in
            .ports()
            .into_iter()
            .find(|p| midi_in.port_name(p).is_ok_and(|name| name == port_name))
            .ok_or_else(|| MidiError(format!("Input port '{}' not found", port_name)))?;

        let memory_clone = Arc::clone(&self.memory);
        let connection_name = format!("SovaIn-{}", self.name); // Keep consistent connection naming
        let connection_name_clone = connection_name.clone(); // Clone for the closure

        let connection = midi_in
            .connect(
                &target_port,
                &connection_name,
                move |_timestamp, message, _| {
                    // --- Debug: Print ALL incoming messages ---
                    println!(
                        "[MIDI IN RAW] Port: {}, Data: {:?}",
                        connection_name_clone, message
                    );

                    // Original CC processing logic:
                    if message.len() == 3 && (message[0] & 0xF0) == CONTROL_CHANGE_MSG {
                        let channel = (message[0] & 0x0F) as i8;
                        let control = message[1] as i8;
                        let value = message[2] as i8;
                        let mut memory_guard = memory_clone.lock().unwrap();
                        (*memory_guard).set(channel, control, value);
                        // Print the received CC message details
                        println!(
                            "[MIDI IN] CC Received - Port: {}, Channel: {}, Control: {}, Value: {}",
                            connection_name_clone, channel, control, value
                        );
                    }
                    // TODO: Add processing for other message types if needed later
                },
                (),
            )
            .map_err(|e| {
                MidiError(format!(
                    "Failed to connect input \'{}\' to \'{}\': {}",
                    self.name, port_name, e
                ))
            })?;

        *self.connection.lock().unwrap() = Some(connection);
        Ok(())
    }

    /// Creates a virtual MIDI input port with the name specified in `self.name`.
    ///
    /// Other MIDI applications can send messages to this virtual port, which will be received
    /// by the callback set up in this `MidiIn` instance.
    /// The callback behavior is the same as for `connect_to_port_by_name`.
    /// This functionality is typically available on macOS and Linux.
    ///
    /// # Errors
    /// Returns `Err(MidiError)` if:
    /// - The `MidiInput` instance cannot be created.
    /// - Virtual input ports are not supported on the current platform (e.g., Windows).
    /// - The connection Mutex is poisoned.
    /// - The underlying `midir` virtual port creation fails.
    pub fn create_virtual_port(&mut self) -> Result<(), MidiError> {
        let midi_in = self.get_midi_in()?;
        let memory_clone = Arc::clone(&self.memory);
        // Use a distinct connection name for the virtual input
        let connection_name = format!("SovaIn-Virtual-{}", self.name);
        let connection_name_clone = connection_name.clone(); // Clone for the closure

        #[cfg(not(target_os = "windows"))] // VirtualInput is usually not on Windows
        {
            use midir::os::unix::VirtualInput; // Import the trait
            match midi_in.create_virtual(
                &self.name, // The name other apps will see for this input port
                move |_timestamp, message, _| {
                    // --- Debug: Print ALL incoming messages ---
                    println!("[MIDI IN VIRTUAL RAW] Port: {}, Data: {:?}", connection_name_clone, message);
                    // Original CC processing logic (or add more later)
                    if message.len() == 3 && (message[0] & 0xF0) == CONTROL_CHANGE_MSG {
                        let channel = (message[0] & 0x0F) as i8;
                        let control = message[1] as i8;
                        let value = message[2] as i8;
                        let mut memory_guard = memory_clone.lock().unwrap();
                        (*memory_guard).set(channel, control, value);
                        println!("[MIDI IN VIRTUAL] CC Received - Port: {}, Channel: {}, Control: {}, Value: {}", connection_name_clone, channel, control, value);
                    }
                },
                (), // No user data needed for this simple callback
            ) {
                Ok(connection) => {
                    *self.connection.lock().unwrap() = Some(connection);
                    Ok(())
                }
                Err(e) => Err(MidiError(format!("Failed to create virtual input '{}': {}", self.name, e))),
            }
        }
        #[cfg(target_os = "windows")]
        {
            Err(MidiError(
                "Virtual MIDI input ports are not supported on Windows.".to_string(),
            ))
        }
    }
}

impl MidiInterface for MidiIn {
    /// Creates a new, unconnected `MidiIn` instance with its own `MidiInMemory` storage.
    fn new(name: String) -> Result<Self, MidiError> {
        Ok(MidiIn {
            name,
            connection: Arc::new(Mutex::new(None)),
            memory: Arc::new(Mutex::new(MidiInMemory::new())),
        })
    }

    /// Returns a list of available MIDI input port names.
    fn ports(&self) -> Vec<String> {
        self.get_midi_in()
            .map(|m| {
                m.ports()
                    .iter()
                    .map(|p| m.port_name(p).unwrap_or_default())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Checks if this `MidiIn` instance is currently connected to a port.
    fn is_connected(&self) -> bool {
        self.connection.lock().unwrap().is_some()
    }
}

/// Creates a default `Mutex<HashMap<u8, HashSet<u8>>>` for `MidiOut.active_notes`.
/// Used by `serde` when deserializing `MidiOut` if the `active_notes` field is missing.
fn default_active_notes() -> Mutex<HashMap<u8, HashSet<u8>>> {
    Mutex::new(HashMap::new())
}
