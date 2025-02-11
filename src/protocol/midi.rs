use std::error::Error;
use midir::{
    MidiOutput,
    MidiOutputPort,
    MidiOutputConnection
};
use midir::os::unix::VirtualOutput;

const NOTE_ON_MSG: u8 = 0x90;
const NOTE_OFF_MSG: u8 = 0x80;
const CONTROL_CHANGE_MSG: u8 = 0xB0;
const PROGRAM_CHANGE_MSG: u8 = 0xC0;
const AFTERTOUCH_MSG: u8 = 0xA0;
const CHANNEL_PRESSURE_MSG: u8 = 0xD0;
const PITCH_BEND_MSG: u8 = 0xE0;   
const CLOCK_MSG: u8 = 0xF8;
const CONTINUE_MSG: u8 = 0xFB;
const RESET_MSG: u8 = 0xFF;
const START_MSG: u8 = 0xFA;
const STOP_MSG: u8 = 0xFC;
const TIME_CODE_QUARTER_MSG : u8 = 0xF1;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MIDIMessageType {
    NoteOn { note: u8,  velocity: u8, },
    NoteOff { note: u8,  velocity: u8,  },
    ControlChange { control: u8,  value: u8, },
    ProgramChange { program: u8, },
    PitchBend { value: u16, },
    Aftertouch { note: u8,  value: u8, },
    ChannelPressure { value: u8, },
    SystemExclusive { data: Vec<u8> },
    TimeCodeQuarterFrame { value: u8, },
    Clock { },
    Start { },
    Continue { },
    Stop { },
    Reset,
    Undefined(u8)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MIDIMessage {
  pub payload : MIDIMessageType,
  pub channel : u8,
  pub port : String
}

pub fn init_default_midi_connection(use_virtual: bool)
    -> Result<MidiOutputConnection, Box<dyn Error>> {
    let midi_out = MidiOutput::new("BuboCore")?;

    // Handle virtual ports
    if use_virtual {
        #[cfg(not(target_os = "windows"))]
        {
            println!("Creating virtual MIDI port named: BuboCore.");
            return midi_out.create_virtual("BuboCore")
                .map_err(|e| format!("Failed to create virtual MIDI port: {}", e).into());
        }

        #[cfg(target_os = "windows")]
        {
            println!("Virtual MIDI ports are not supported on Windows. Falling back to physical ports.");
        }
    }

    let out_ports = midi_out.ports();

    if out_ports.is_empty() {
        eprintln!("No available MIDI ports and virtual ports are not supported on Windows. Exiting.");
        return Err("No available MIDI ports".into());
    }

    let port_name = |p: &MidiOutputPort| {
        midi_out.port_name(p)
            .unwrap_or_else(|_| "Unknown Port".to_string())
    };

    let out_port = if out_ports.len() == 1 {
        println!("Using single available output port: {}", port_name(&out_ports[0]));
        &out_ports[0]
    } else {
        println!("\nAvailable output ports:");
        for (i, p) in out_ports.iter().enumerate() {
            println!("{}: {}", i, port_name(p));
        }
        println!("Selecting first available port: {}", port_name(&out_ports[0]));
        &out_ports[0]
    };

    midi_out.connect(out_port, "BuboCore")
        .map_err(|e| format!("Failed to connect to MIDI port: {}", e).into())
}

pub fn send(connection: &mut MidiOutputConnection, message: MIDIMessage) {
    let result = match message.payload {
        MIDIMessageType::NoteOn { note, velocity } => {
            connection.send(&[NOTE_ON_MSG + message.channel, note, velocity])
        },
        MIDIMessageType::NoteOff { note, velocity } => {
            connection.send(&[NOTE_OFF_MSG + message.channel, note, velocity])
        },
        MIDIMessageType::ControlChange { control, value } => {
            connection.send(&[CONTROL_CHANGE_MSG + message.channel, control, value])
        },
        MIDIMessageType::ProgramChange { program } => {
            connection.send(&[PROGRAM_CHANGE_MSG + message.channel, program])
        },
        MIDIMessageType::Aftertouch { note, value } => {
            connection.send(&[AFTERTOUCH_MSG + message.channel, note, value])
        },
        MIDIMessageType::ChannelPressure { value } => {
            connection.send(&[CHANNEL_PRESSURE_MSG + message.channel, value])
        },
        MIDIMessageType ::PitchBend { value } => {
            connection.send(&[
                PITCH_BEND_MSG + message.channel,
                (value & 0x7F) as u8, (value >> 7) as u8
            ])
        },
        MIDIMessageType::Clock {  } => {
            connection.send(&[CLOCK_MSG])
        },
        MIDIMessageType::Continue {  } => {
            connection.send(&[CONTINUE_MSG])
        },
        MIDIMessageType::Reset  => {
            connection.send(&[RESET_MSG])
        },
        MIDIMessageType::Start {  } => {
            connection.send(&[START_MSG])
        },
        MIDIMessageType::Stop {  }  => {
            connection.send(&[STOP_MSG])
        },
        MIDIMessageType::SystemExclusive { data } => {
            let mut message = vec![0xF0];
            message.extend(data);
            message.push(0xF7);
            connection.send(&message)
        },
        MIDIMessageType::TimeCodeQuarterFrame { value } => {
            connection.send(&[TIME_CODE_QUARTER_MSG, value])
        },
        MIDIMessageType::Undefined(byte) => {
            connection.send(&[byte])
        }
    };

    if let Err(e) = result {
        eprintln!("Failed to send MIDI message: {}", e);
    }
}