use std::error::Error;
use std::io::{stdin, stdout, Write};

use midir::{
    Ignore,
    MidiInput,
    MidiOutput,
    MidiOutputPort,
    MidiOutputConnection
};
use midir::os::unix::VirtualOutput;

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
    todo!();
}

