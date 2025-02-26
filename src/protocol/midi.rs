mod control_memory;
mod midi_constants;

use midir::os::unix::VirtualOutput;
use midir::{
    MidiInput,
    MidiOutput,
    MidiOutputConnection,
    MidiInputConnection,
};
use std::error::Error;
use std::sync::{Arc, Mutex};
use midi_constants::*;
use control_memory::MidiInMemory;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MIDIMessage {
    pub payload: MIDIMessageType,
    pub channel: u8,
    pub port: String,
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
    TimeCodeQuarterFrame { value: u8 },
    Clock {},
    Start {},
    Continue {},
    Stop {},
    Reset,
    Undefined(u8),
}

/// Shared behavior of all MIDI interfaces
pub trait MidiInterface {
    fn new(client_name: &str) -> Result<Self, Box<dyn Error>> 
    where
        Self: Sized;
    fn ports(&self) -> Vec<String>;
    fn connect(&mut self, port_name: &str) -> Result<(), Box<dyn Error>>;
}

/// MIDI Output: sends MIDI messages
pub struct MidiOut {
    pub name: String,
    pub midi_out: Option<MidiOutput>,
    pub connection: Option<MidiOutputConnection>,
}

impl MidiOut {

    pub fn send(&mut self, message: MIDIMessage) -> Result<(), Box<dyn Error>> {

        let connection = self.connection.as_mut()
            .ok_or(format!(
                "Midi Interface {} not connected to any MIDI port",
                self.name)
            )?;

        let result = match message.payload {
            MIDIMessageType::NoteOn { note, velocity } => {
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
            MIDIMessageType::TimeCodeQuarterFrame { value } => {
                connection.send(&[TIME_CODE_QUARTER_MSG, value])
            }
            MIDIMessageType::Undefined(byte) => connection.send(&[byte]),
        };

        result.map_err(|e| format!("Failed to send MIDI message: {}", e).into())
    }

    pub fn connect_to_default(&mut self, use_virtual: bool) -> Result<(), Box<dyn Error>> {
        let midi_out = self.midi_out.take().ok_or("MIDI output not initialized")?;

        if use_virtual {
            #[cfg(not(target_os = "windows"))]
            {
                self.connection = Some(midi_out.create_virtual(&self.name)?);
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

        self.connection = Some(midi_out.connect(&ports[0], &self.name)?);
        Ok(())
    }
}

impl MidiInterface for MidiOut {

    fn new(client_name: &str) -> Result<Self, Box<dyn Error>> {
        Ok(
            MidiOut {
                name: client_name.to_string(),
                midi_out: Some(MidiOutput::new(client_name)?),
                connection: None,
            }
        )
    }

    fn ports(&self) -> Vec<String> {
        self.midi_out.as_ref()
            .map(|m| {
                m.ports().iter()
                    .filter_map(|p| m.port_name(p).ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn connect(&mut self, port_name: &str)
    -> Result<(), Box<dyn Error>> {
        let midi_out = self.midi_out.take().ok_or("MIDI output not initialized")?;
        let out_port = midi_out.ports().into_iter()
            .find(|p| midi_out.port_name(p)
                .unwrap_or_default() == port_name
            )
            .ok_or(format!("No MIDI output port named '{}' found", port_name))?;

        self.connection = Some(midi_out.connect(&out_port, &self.name)?);
        Ok(())
    }


}

pub struct MidiIn {
    pub name: String,
    pub midi_in: Option<MidiInput>,
    pub connection: Option<MidiInputConnection<()>>,
    pub memory: Arc<Mutex<MidiInMemory>>,
}

impl MidiInterface for MidiIn {

    fn new(client_name: &str) -> Result<Self, Box<dyn Error>> {
        Ok(MidiIn {
            name: client_name.to_string(),
            midi_in: Some(MidiInput::new(client_name)?),
            connection: None,
            memory: Arc::new(Mutex::new(MidiInMemory::new())),
        })
    }

    fn ports(&self) -> Vec<String> {
        self.midi_in.as_ref()
            .map(|m| {
                m.ports().iter()
                    .filter_map(|p| m.port_name(p).ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn connect(&mut self, port_name: &str) -> Result<(), Box<dyn Error>> {
        let midi_in = self.midi_in.take().ok_or("MIDI input not initialized")?;
        let in_port = midi_in.ports().into_iter()
            .find(|p| midi_in.port_name(p).unwrap_or_default() == port_name)
            .ok_or(format!("No MIDI input port named '{}' found", port_name))?;

        let memory = Arc::clone(&self.memory);
        self.connection = Some(midi_in.connect(
            &in_port,
            &self.name,
            move |_stamp, message, _| {
                // Spotting a control change message
                // CC_MSG + 0..15
                let is_cc_message = 
                    CONTROL_CHANGE_MSG < message[0]
                    && message[0] < CONTROL_CHANGE_MSG + 16
                ;

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
        )?);
        Ok(())
    }
}