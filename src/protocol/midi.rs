mod control_memory;
mod midi_constants;

use midir::os::unix::VirtualOutput;
use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

use control_memory::MidiInMemory;
use midi_constants::*;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone)]
pub struct MidiError(pub String);

impl<T: ToString> From<T> for MidiError {
    fn from(value: T) -> Self {
        MidiError(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MIDIMessage {
    pub payload: MIDIMessageType,
    pub channel: u8,
}

/// MIDI Message Types: some are missing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
pub struct MidiOut {
    pub name: String,
    pub connection: Option<Mutex<MidiOutputConnection>>,
}

impl Debug for MidiOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiOut({})", self.name)
    }
}

impl MidiOut {
    pub fn send(&self, message: MIDIMessage) -> Result<(), MidiError> {
        let Some(ref connection) = self.connection else {
            return Err(format!(
                "Midi Interface {} not connected to any MIDI port",
                self.name
            )
            .into());
        };
        let mut connection = connection.lock().unwrap();
        let result = match message.payload {
            MIDIMessageType::NoteOn { note, velocity } => {
                let _ = connection.send(&[NOTE_OFF_MSG + message.channel, note, velocity]);
                connection.send(&[NOTE_ON_MSG + message.channel, note, velocity])
            }
            MIDIMessageType::NoteOff { note, velocity } => {
                connection.send(&[NOTE_OFF_MSG + message.channel, note, velocity])
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
}

impl MidiInterface for MidiOut {
    fn new(client_name: String) -> Result<Self, MidiError> {
        Ok(MidiOut {
            name: client_name,
            connection: None,
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
        let midi_out = self.get_midi_out()?;
        let out_port = midi_out
            .ports()
            .into_iter()
            .find(|p| midi_out.port_name(p).unwrap_or_default() == self.name)
            .ok_or(format!("No MIDI output port named '{}' found", &self.name))?;

        self.connection = Some(Mutex::new(midi_out.connect(&out_port, &self.name)?));
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connection.is_some()
    }
}

pub struct MidiIn {
    pub name: String,
    pub connection: Option<Mutex<MidiInputConnection<()>>>,
    pub memory: Arc<Mutex<MidiInMemory>>,
}

impl Debug for MidiIn {
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
