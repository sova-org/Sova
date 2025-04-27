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

/// Représente une erreur dans le traitement MIDI
#[derive(Debug, Default, Clone)]
pub struct MidiError(pub String);

impl<T: ToString> From<T> for MidiError {
    fn from(value: T) -> Self {
        MidiError(value.to_string())
    }
}

/// Message MIDI avec un type de charge utile et un canal
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MIDIMessage {
    pub payload: MIDIMessageType,
    pub channel: u8,
}

impl Display for MIDIMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MIDIMessage sur canal ({}) : [{}]",
            self.channel, self.payload
        )
    }
}

/// Types de messages MIDI supportés
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MIDIMessageType {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8, velocity: u8 },
    ControlChange { control: u8, value: u8 },
    ProgramChange { program: u8 },
    PitchBend { value: u16 },
    Aftertouch { note: u8, value: u8 },
    ChannelPressure { value: u8 },
    SystemExclusive { data: Vec<u8> },
    Clock,
    Start,
    Continue,
    Stop,
    Reset,
    Undefined(u8),
}

impl Display for MIDIMessageType {
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

/// Interface commune pour tous les périphériques MIDI
pub trait MidiInterface {
    /// Crée une nouvelle instance de l'interface
    fn new(client_name: String) -> Result<Self, MidiError>
    where
        Self: Sized;

    /// Renvoie la liste des ports disponibles
    fn ports(&self) -> Vec<String>;

    /// Vérifie si l'interface est connectée
    fn is_connected(&self) -> bool;
}

/// Sortie MIDI pour envoyer des messages
#[derive(Serialize, Deserialize)]
pub struct MidiOut {
    pub name: String,
    #[serde(skip)]
    pub connection: Arc<Mutex<Option<MidiOutputConnection>>>,
    #[serde(skip, default = "default_active_notes")]
    pub active_notes: Mutex<HashMap<u8, HashSet<u8>>>,
}

impl Display for MidiOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiOut({})", self.name)
    }
}

impl Debug for MidiOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiOut({})", self.name)
    }
}

impl MidiOut {
    /// Envoie un message MIDI
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

    /// Connects the MidiOut to a specific port by its name.
    pub fn connect_to_port_by_name(&mut self, port_name: &str) -> Result<(), MidiError> {
        let midi_out = self.get_midi_out()?;
        let target_port = midi_out
            .ports()
            .into_iter()
            .find(|p| {
                midi_out
                    .port_name(p)
                    .map_or(false, |name| name == port_name)
            })
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

    /// Creates a virtual MIDI output port (on supported platforms).
    /// Note: This only handles the output connection. The server logic
    /// should handle creating and connecting a corresponding MidiIn instance.
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

    /// Kept temporarily for compatibility check, should be removed later.
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

    /// Crée une instance de MidiOutput
    fn get_midi_out(&self) -> Result<MidiOutput, MidiError> {
        MidiOutput::new(&self.name).map_err(|e| e.into())
    }

    /// Vide la file d'attente (no-op pour midir)
    pub fn flush(&self) {}
}

impl Drop for MidiOut {
    fn drop(&mut self) {
        if let Ok(mut c) = self.connection.lock() {
            c.take();
        }
    }
}

impl MidiInterface for MidiOut {
    fn new(name: String) -> Result<Self, MidiError> {
        Ok(MidiOut {
            name,
            connection: Arc::new(Mutex::new(None)),
            active_notes: Mutex::new(HashMap::new()),
        })
    }

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

    fn is_connected(&self) -> bool {
        self.connection.lock().unwrap().is_some()
    }
}

/// Entrée MIDI pour recevoir des messages
#[derive(Serialize, Deserialize)]
pub struct MidiIn {
    pub name: String,
    #[serde(skip)]
    pub connection: Arc<Mutex<Option<midir::MidiInputConnection<()>>>>,
    #[serde(skip)]
    pub memory: Arc<Mutex<MidiInMemory>>,
}

impl Debug for MidiIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiIn({})", self.name)
    }
}

impl Display for MidiIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiIn({})", self.name)
    }
}

impl MidiIn {
    /// Crée une instance de MidiInput
    fn get_midi_in(&self) -> Result<MidiInput, MidiError> {
        MidiInput::new(&self.name).map_err(|e| e.into())
    }

    /// Connects the MidiIn to a specific port by its name.
    pub fn connect_to_port_by_name(&mut self, port_name: &str) -> Result<(), MidiError> {
        let midi_in = self.get_midi_in()?;
        let target_port = midi_in
            .ports()
            .into_iter()
            .find(|p| midi_in.port_name(p).map_or(false, |name| name == port_name))
            .ok_or_else(|| MidiError(format!("Input port '{}' not found", port_name)))?;

        let memory_clone = Arc::clone(&self.memory);
        let connection_name = format!("BuboCoreIn-{}", self.name); // Keep consistent connection naming
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

    /// Creates a virtual MIDI input port (on supported platforms).
    pub fn create_virtual_port(&mut self) -> Result<(), MidiError> {
        let midi_in = self.get_midi_in()?;
        let memory_clone = Arc::clone(&self.memory);
        // Use a distinct connection name for the virtual input
        let connection_name = format!("BuboCoreIn-Virtual-{}", self.name);
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
    fn new(name: String) -> Result<Self, MidiError> {
        Ok(MidiIn {
            name,
            connection: Arc::new(Mutex::new(None)),
            memory: Arc::new(Mutex::new(MidiInMemory::new())),
        })
    }

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

    fn is_connected(&self) -> bool {
        self.connection.lock().unwrap().is_some()
    }
}

/// Crée un Mutex contenant une HashMap vide pour les notes actives
fn default_active_notes() -> Mutex<HashMap<u8, HashSet<u8>>> {
    Mutex::new(HashMap::new())
}
