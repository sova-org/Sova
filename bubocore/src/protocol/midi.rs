mod control_memory;
mod midi_constants;

use midir::os::unix::VirtualOutput;
use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use serde::{Deserialize, Serialize};

use control_memory::MidiInMemory;
use midi_constants::*;
use std::fmt::{Debug, Display};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct MidiError(pub String);

impl<T: ToString> From<T> for MidiError {
    fn from(value: T) -> Self {
        MidiError(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MIDIMessage {
    pub payload: MIDIMessageType,
    pub channel: u8,
}

impl Display for MIDIMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MIDIMessage on channel ({}) : [{}]", self.payload, self.channel)
    }
}

/// MIDI Message Types: some are missing
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
            MIDIMessageType::NoteOn { note, velocity } => write!(f, "NoteOn : note = {note} ; velocity = {velocity}"),
            MIDIMessageType::NoteOff { note, velocity } => write!(f, "NoteOff : note = {note} ; velocity = {velocity}"),
            MIDIMessageType::ControlChange { control, value } => write!(f, "ControlChange : control = {control} ; value = {value}"),
            MIDIMessageType::ProgramChange { program } => write!(f, "ProgramChange : program = {program}"),
            MIDIMessageType::PitchBend { value } => write!(f, "PitchBend : pitch = {} ; bend = {}", value % 0x100, value >> 8),
            MIDIMessageType::Aftertouch { note, value } => write!(f, "AfterTouch : note = {note} ; value = {value}"),
            MIDIMessageType::ChannelPressure { value } => write!(f, "ChannelPressure : value = {value}"),
            MIDIMessageType::SystemExclusive { data } => write!(f, "SystemExclusive : data = {:?}", data),
            MIDIMessageType::Clock => write!(f, "Clock"),
            MIDIMessageType::Start => write!(f, "Start"),
            MIDIMessageType::Continue => write!(f, "Continue"),
            MIDIMessageType::Stop => write!(f, "Stop"),
            MIDIMessageType::Reset => write!(f, "Reset"),
            MIDIMessageType::Undefined(x) => write!(f, "Undefined : {x}"),
        }
    }
}

/// Shared behavior of all MIDI interfaces
pub trait MidiInterface {
    fn new(client_name: String) -> Result<Self, MidiError>
    where
        Self: Sized;
    fn ports(&self) -> Vec<String>;
    fn connect(&mut self) -> Result<(), MidiError>;
    fn is_connected(&self) -> bool;
}

/// MIDI Output: sends MIDI messages
#[derive(Serialize, Deserialize)]
pub struct MidiOut {
    pub name: String,
    #[serde(skip)]
    pub connection: Option<Mutex<MidiOutputConnection>>,
    #[serde(skip, default = "default_active_notes")]
    active_notes: Mutex<HashMap<u8, HashSet<u8>>>,
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
    pub fn send(&self, message: MIDIMessage) -> Result<(), MidiError> {
        let Some(ref connection_mutex) = self.connection else {
            return Err(format!(
                "Midi Interface {} not connected to any MIDI port",
                self.name
            )
            .into());
        };
        let mut connection = connection_mutex.lock().unwrap();
        let mut active_notes_guard = self.active_notes.lock().unwrap();

        let result = match message.payload {
            MIDIMessageType::NoteOn { note, velocity } => {
                let channel_notes = active_notes_guard.entry(message.channel).or_default();
                if channel_notes.contains(&note) {
                    let _ = connection.send(&[NOTE_OFF_MSG + message.channel, note, 0]);
                }
                let send_result = connection.send(&[NOTE_ON_MSG + message.channel, note, velocity]);
                if send_result.is_ok() {
                    channel_notes.insert(note);
                }
                send_result
            }
            MIDIMessageType::NoteOff { note, velocity } => {
                let channel_notes = active_notes_guard.entry(message.channel).or_default();
                if channel_notes.contains(&note) {
                    let send_result = connection.send(&[NOTE_OFF_MSG + message.channel, note, velocity]);
                    channel_notes.remove(&note);
                    send_result
                } else {
                    Ok(())
                }
            }
            MIDIMessageType::ControlChange { control, value } => {
                connection.send(&[CONTROL_CHANGE_MSG + message.channel, control, value])
            }
            MIDIMessageType::ProgramChange { program } => {
                connection.send(&[PROGRAM_CHANGE_MSG + message.channel, program])
            }
            MIDIMessageType::Aftertouch { note, value } => {
                connection.send(&[AFTERTOUCH_MSG + message.channel, note, value])
            }
            MIDIMessageType::ChannelPressure { value } => {
                connection.send(&[CHANNEL_PRESSURE_MSG + message.channel, value])
            }
            MIDIMessageType::PitchBend { value } => connection.send(&[
                PITCH_BEND_MSG + message.channel,
                (value & 0x7F) as u8,
                (value >> 7) as u8,
            ]),
            MIDIMessageType::Clock {} => connection.send(&[CLOCK_MSG]),
            MIDIMessageType::Continue {} => connection.send(&[CONTINUE_MSG]),
            MIDIMessageType::Reset => connection.send(&[RESET_MSG]),
            MIDIMessageType::Start {} => connection.send(&[START_MSG]),
            MIDIMessageType::Stop {} => connection.send(&[STOP_MSG]),
            MIDIMessageType::SystemExclusive { data } => {
                let mut message = vec![0xF0];
                message.extend(data);
                message.push(0xF7);
                connection.send(&message)
            }
            MIDIMessageType::Undefined(byte) => connection.send(&[byte]),
        };

        result.map_err(|e| format!("Failed to send MIDI message: {}", e).into())
    }

    pub fn connect_to_default(&mut self, use_virtual: bool) -> Result<(), MidiError> {
        let midi_out = self.get_midi_out()?;

        if use_virtual {
            #[cfg(not(target_os = "windows"))]
            {
                self.connection = Some(Mutex::new(midi_out.create_virtual(&self.name)?));
                return Ok(());
            }

            #[cfg(target_os = "windows")]
            {
                eprintln!("Virtual MIDI ports are not supported on Windows. Falling back to physical ports.");
            }
        }

        let ports = midi_out.ports();
        if ports.is_empty() {
            return Err("No available MIDI ports".into());
        }

        self.connection = Some(Mutex::new(midi_out.connect(&ports[0], &self.name)?));
        Ok(())
    }

    fn get_midi_out(&self) -> Result<MidiOutput, MidiError> {
        MidiOutput::new(&self.name)
            .map_err(|_| MidiError(format!("Cannot create MIDI connection named {}", self.name)))
    }

    pub fn flush(&self) {
        if !self.is_connected() {
            return;
        }
        let Some(ref connection_mutex) = self.connection else {
            return;
        };
        let mut connection = connection_mutex.lock().unwrap();
        let mut active_notes_guard = self.active_notes.lock().unwrap();

        println!("[*] Flushing MIDI notes for {}", self.name);
        for (channel, notes) in active_notes_guard.iter() {
            for note in notes.iter() {
                println!("  - Sending Note Off: Channel {}, Note {}", channel, note);
                let _ = connection.send(&[NOTE_OFF_MSG + channel, *note, 0]);
            }
        }
        active_notes_guard.clear();
        println!("[*] MIDI flush complete for {}.", self.name);
    }
}

impl Drop for MidiOut {
    fn drop(&mut self) {
        self.flush();
    }
}

impl MidiInterface for MidiOut {
    fn new(client_name: String) -> Result<Self, MidiError> {
        Ok(MidiOut {
            name: client_name,
            connection: None,
            active_notes: Mutex::new(HashMap::new()),
        })
    }

    fn ports(&self) -> Vec<String> {
        let Ok(midi_out) = self.get_midi_out() else {
            return Vec::new();
        };
        midi_out
            .ports()
            .iter()
            .filter_map(|p| midi_out.port_name(p).ok())
            .collect()
    }

    fn connect(&mut self) -> Result<(), MidiError> {
        if self.is_connected() {
            return Ok(()); // Already connected
        }

        let midi_out = self.get_midi_out()?;
        let target_name = self.name.clone(); // Clone name for searching

        let target_port = midi_out.ports()
            .into_iter()
            .find(|p| midi_out.port_name(p).map(|name| name == target_name).unwrap_or(false));

        match target_port {
            Some(port) => {
                println!("[+] Connecting MidiOut '{}' to port...", target_name);
                match midi_out.connect(&port, &target_name) {
                    Ok(connection) => {
                        self.connection = Some(Mutex::new(connection));
                        println!("[+] MidiOut '{}' connected successfully.", self.name);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("[!] Failed to connect MidiOut '{}': {}", self.name, e);
                        Err(e.into())
                    }
                }
            }
            None => {
                eprintln!("[!] MIDI output port named '{}' not found.", self.name);
                Err(MidiError(format!("Port '{}' not found", self.name)))
            }
        }
    }

    fn is_connected(&self) -> bool {
        self.connection.is_some()
    }
}

#[derive(Serialize, Deserialize)]
pub struct MidiIn {
    pub name: String,
    #[serde(skip)]
    pub connection: Option<Mutex<MidiInputConnection<()>>>,
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
    fn get_midi_in(&self) -> Result<MidiInput, MidiError> {
        MidiInput::new(&self.name)
            .map_err(|_| MidiError(format!("Cannot create MIDI connection named {}", self.name)))
    }
}

impl MidiInterface for MidiIn {
    fn new(client_name: String) -> Result<Self, MidiError> {
        Ok(MidiIn {
            name: client_name,
            connection: None,
            memory: Arc::new(Mutex::new(MidiInMemory::new())),
        })
    }

    fn ports(&self) -> Vec<String> {
        let Ok(midi_in) = self.get_midi_in() else {
            return Vec::new();
        };
        midi_in
            .ports()
            .iter()
            .filter_map(|p| midi_in.port_name(p).ok())
            .collect()
    }

    fn connect(&mut self) -> Result<(), MidiError> {
        let midi_in = self.get_midi_in()?;
        let in_port = midi_in
            .ports()
            .into_iter()
            .find(|p| midi_in.port_name(p).unwrap_or_default() == self.name)
            .ok_or(format!("No MIDI input port named '{}' found", self.name))?;

        let memory = Arc::clone(&self.memory);
        let connection = midi_in.connect(
            &in_port,
            &self.name,
            move |_stamp, message, _| {
                // Spotting a control change message
                // CC_MSG + 0..15
                let is_cc_message =
                    CONTROL_CHANGE_MSG < message[0] && message[0] < CONTROL_CHANGE_MSG + 16;

                // Store the last received value in memory
                if is_cc_message {
                    let mut mem = memory.lock().unwrap();
                    mem.set(
                        (message[0] - CONTROL_CHANGE_MSG) as i8,
                        message[1] as i8,
                        message[2] as i8,
                    )
                }
                // For debug purposes only
                println!("{:?}", message);
            },
            (),
        )?;
        self.connection = Some(Mutex::new(connection));
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connection.is_some()
    }
}

fn default_active_notes() -> Mutex<HashMap<u8, HashSet<u8>>> {
    Mutex::new(HashMap::new())
}
