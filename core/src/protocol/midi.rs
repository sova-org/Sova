use midir::os::unix::VirtualOutput;
use midir::{MidiInput, MidiOutput, MidiOutputConnection};

use control_memory::MidiInMemory;
use midi_constants::*;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::sync::{Arc, Mutex};

mod control_memory;
mod message;
pub use message::*;

use crate::protocol::error::ProtocolError;

pub mod midi_constants;

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
    fn new(client_name: String) -> Result<Self, ProtocolError>
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
pub struct MidiOut {
    /// The name assigned to this MIDI output client/connection.
    pub name: String,
    /// The underlying `midir` output connection, managed thread-safely.
    /// This field is not serialized.
    pub connection: Mutex<Option<MidiOutputConnection>>,
    /// Tracks currently active notes per channel to avoid sending duplicate Note Ons.
    /// Maps channel (u8) to a set of active notes (u8).
    /// This field is not serialized and has a default initializer.
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
        f.debug_tuple("MidiOut")
            .field(&self.name)
            .finish()
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
    pub fn send(&self, message: MIDIMessage) -> Result<(), ProtocolError> {
        let mut connection_opt_guard = self
            .connection
            .lock()
            .map_err(|_| ProtocolError("MidiOut connection Mutex poisoned".to_string()))?;

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
    pub fn connect_to_port_by_name(&mut self, port_name: &str) -> Result<(), ProtocolError> {
        let midi_out = self.get_midi_out()?;
        let target_port = midi_out
            .ports()
            .into_iter()
            .find(|p| midi_out.port_name(p).is_ok_and(|name| name == port_name))
            .ok_or_else(|| ProtocolError(format!("Output port '{}' not found", port_name)))?;

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

    pub fn connect(&mut self) -> Result<(), ProtocolError> {
        crate::log_println!(
            "[~] connect() called for MidiOut '{}'",
            self.name
        );
        let name = self.name.clone();
        self.connect_to_port_by_name(&name)
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
    pub fn create_virtual_port(&mut self) -> Result<(), ProtocolError> {
        let midi_out = self.get_midi_out()?;

        #[cfg(not(target_os = "windows"))]
        {
            match midi_out.create_virtual(&self.name) {
                Ok(connection) => {
                    *self.connection.lock().unwrap() = Some(connection);
                    Ok(())
                }
                Err(_) => Err(format!("MIDI Erorr: Unable to create virtual port").into()),
            }
        }
        #[cfg(target_os = "windows")]
        {
            Err(MidiError(
                "Virtual MIDI ports are not supported on Windows.".to_string(),
            ))
        }
    }

    /// Creates a temporary `midir::MidiOutput` instance.
    /// Used internally to query ports or establish connections.
    fn get_midi_out(&self) -> Result<MidiOutput, ProtocolError> {
        MidiOutput::new(&self.name).map_err(|_| format!("MIDI Error: Unable to init output interface").into())
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
    fn new(name: String) -> Result<Self, ProtocolError> {
        Ok(MidiOut {
            name,
            connection: Mutex::new(None),
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
pub struct MidiIn {
    /// The name assigned to this MIDI input client/connection.
    pub name: String,
    /// The underlying `midir` input connection, managed thread-safely.
    /// This field is not serialized.
    pub connection: Mutex<Option<midir::MidiInputConnection<()>>>,
    /// Shared, thread-safe storage for the last received value of each Control Change message
    /// per channel.
    /// This field is not serialized.
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
        f.debug_tuple("MidiIn")
            .field(&self.name)
            .finish()
    }
}

impl MidiIn {
    /// Creates a temporary `midir::MidiInput` instance.
    /// Used internally to query ports or establish connections.
    fn get_midi_in(&self) -> Result<MidiInput, ProtocolError> {
        MidiInput::new(&self.name).map_err(|_| format!("MIDI Error: Unable to create input interface").into())
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
    pub fn connect_to_port_by_name(&mut self, port_name: &str) -> Result<(), ProtocolError> {
        let midi_in = self.get_midi_in()?;
        let target_port = midi_in
            .ports()
            .into_iter()
            .find(|p| midi_in.port_name(p).is_ok_and(|name| name == port_name))
            .ok_or_else(|| ProtocolError(format!("Input port '{}' not found", port_name)))?;

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
                ProtocolError(format!(
                    "Failed to connect input \'{}\' to \'{}\': {}",
                    self.name, port_name, e
                ))
            })?;

        *self.connection.lock().unwrap() = Some(connection);
        Ok(())
    }

    pub fn connect(&mut self) -> Result<(), ProtocolError> {
        crate::log_println!(
            "[~] connect() called for MidiIn '{}'",
            self.name
        );
        let name = self.name.clone();
        self.connect_to_port_by_name(&name)
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
    pub fn create_virtual_port(&mut self) -> Result<(), ProtocolError> {
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
                Err(e) => Err(ProtocolError(format!("Failed to create virtual input '{}': {}", self.name, e))),
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
    fn new(name: String) -> Result<Self, ProtocolError> {
        Ok(MidiIn {
            name,
            connection: Mutex::new(None),
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