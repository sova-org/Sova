use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

use crate::clock::SyncTime;
use crate::vm::event::ConcreteEvent;
use crate::protocol::error::ProtocolError;
use crate::protocol::midi::midi_constants::*;
use crate::protocol::payload::ProtocolPayload;
use crate::vm::variable::VariableValue;

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
    pub fn to_bytes(&self) -> Result<Vec<u8>, ProtocolError> {
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
                    return Err(ProtocolError("SysEx data cannot contain F7 byte".to_string()));
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
        epsilon: SyncTime
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
                        }.into(), date + epsilon
                    ),
                    // NoteOff
                    (
                        MIDIMessage {
                            payload: MIDIMessageType::NoteOff {
                                note: note as u8,
                                velocity: 0,
                            },
                            channel: midi_chan,
                        }.into(), date + dur - epsilon,
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
            ConcreteEvent::Generic(args, duration, channel, _device_id) => {
                let midi_chan = channel.parse::<u64>().unwrap_or(1).saturating_sub(1) % 16;
                match args {
                    VariableValue::Integer(i) => {
                        Self::generate_messages(
                            ConcreteEvent::MidiNote(i as u64, 100, midi_chan, duration, _device_id), 
                            date, epsilon
                        )
                    }
                    VariableValue::Map(mut map) => {
                        let note = match map.remove("note").unwrap_or_default() {
                            VariableValue::Integer(i) => i as u64,
                            _ => 0
                        };
                        let velocity = match map.remove("velocity").unwrap_or_default() {
                            VariableValue::Integer(i) => i as u64,
                            _ => 100
                        };
                        Self::generate_messages(
                            ConcreteEvent::MidiNote(note, velocity, midi_chan, duration, _device_id),
                            date, epsilon
                        )
                    },
                    _ => Vec::new()
                }
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
