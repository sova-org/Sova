use std::{cmp::Ordering, fmt::Display, sync::Arc};

use log::LogMessage;
use osc::OSCMessage;
use midi::{MIDIMessage, MidiError, MidiIn, MidiInterface, MidiOut};

use crate::clock::SyncTime;
use serde::{Deserialize, Serialize};

pub mod midi;
pub mod osc;
pub mod log;

/// Unified message type to transmit any message supported by a protocol
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolPayload {
    OSC(OSCMessage),
    MIDI(MIDIMessage),
    LOG(LogMessage),
}

impl Display for ProtocolPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolPayload::OSC(m) => m.fmt(f),
            ProtocolPayload::MIDI(m) => m.fmt(f),
            ProtocolPayload::LOG(m) => m.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub device : Arc<ProtocolDevice>,
    pub payload : ProtocolPayload
}

impl ProtocolMessage {

    pub fn send(self) -> Result<(), ProtocolError> {
        self.device.send(self.payload)
    }

}

impl Display for ProtocolMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] -> Device : {}", self.payload, self.device)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProtocolDevice {
    Log,
    OSCInDevice,
    OSCOutDevice,
    MIDIInDevice(MidiIn),
    MIDIOutDevice(MidiOut)
}

impl Display for ProtocolDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolDevice::Log => write!(f, "Log"),
            ProtocolDevice::OSCInDevice => write!(f, "OSCInDevice"), // TODO: Change when OSC is implemented
            ProtocolDevice::OSCOutDevice => write!(f, "OSCOutDevice"),
            ProtocolDevice::MIDIInDevice(midi_in) => midi_in.fmt(f),
            ProtocolDevice::MIDIOutDevice(midi_out) => midi_out.fmt(f),
        }
    }
}

impl PartialEq for ProtocolDevice {
    fn eq(&self, other: &Self) -> bool {
        *self.address() == *other.address()
    }
}
impl Eq for ProtocolDevice {}

pub struct ProtocolError(pub String);
impl From<MidiError> for ProtocolError {
    fn from(value: MidiError) -> Self {
        ProtocolError(value.0)
    }
}

impl ProtocolDevice {

    pub fn connect(&mut self) -> Result<(), ProtocolError> {
        match self {
            ProtocolDevice::OSCInDevice => todo!(),
            ProtocolDevice::OSCOutDevice => todo!(),
            ProtocolDevice::MIDIInDevice(midi_in) => midi_in.connect().map_err(ProtocolError::from),
            ProtocolDevice::MIDIOutDevice(midi_out) => midi_out.connect().map_err(ProtocolError::from),
            _ => Ok(())
        }
    }

    pub fn send(&self, message : ProtocolPayload) -> Result<(), ProtocolError> {
        match self {
            ProtocolDevice::OSCOutDevice => todo!(),
            ProtocolDevice::MIDIOutDevice(midi_out) => {
                let ProtocolPayload::MIDI(midi_msg) = message else {
                    return Err(ProtocolError("Invalid message format for MIDI device !".to_owned()));
                };
                midi_out.send(midi_msg).map_err(ProtocolError::from)
            },
            _ => Ok(())
        }
    }

    pub fn flush(&self) {
        match self {
            ProtocolDevice::MIDIOutDevice(midi_out) => midi_out.flush(),
            _ => ()
        }
    }

    pub fn address(&self) -> &str {
        match self {
            ProtocolDevice::Log => "log",
            ProtocolDevice::OSCInDevice => todo!(),
            ProtocolDevice::OSCOutDevice => todo!(),
            ProtocolDevice::MIDIInDevice(midi_in) => &midi_in.name,
            ProtocolDevice::MIDIOutDevice(midi_out) => &midi_out.name,
        }
    }

}

impl From<MidiOut> for ProtocolDevice {
    fn from(value: MidiOut) -> Self {
        Self::MIDIOutDevice(value)
    }
}
impl From<MidiIn> for ProtocolDevice {
    fn from(value: MidiIn) -> Self {
        Self::MIDIInDevice(value)
    }
}

/// ProtocolMessage salted with a time information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedMessage {
    pub message : ProtocolMessage,
    pub time : SyncTime
}

impl Display for TimedMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ Date : {}", self.message, self.time)
    }
}

impl ProtocolMessage {
    /// Add a timing information to a ProtocolMessage in order to make a TimedMessage
    pub fn timed(self, time : SyncTime) -> TimedMessage {
        TimedMessage {
            message : self,
            time
        }
    }
}

impl TimedMessage {
    pub fn untimed(self) -> (ProtocolMessage, SyncTime) {
        (self.message, self.time)
    }
}

/// A TimedMessage is ordered greater if its timestamp is lesser (reversed ordering on time)
impl Ord for TimedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.cmp(&self.time)
    }
}

impl PartialOrd for TimedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<OSCMessage> for ProtocolPayload {
    fn from(value: OSCMessage) -> Self {
        Self::OSC(value)
    }
}

impl From<MIDIMessage> for ProtocolPayload {
    fn from(value: MIDIMessage) -> Self {
        Self::MIDI(value)
    }
}

impl From<LogMessage> for ProtocolPayload {
    fn from(value: LogMessage) -> Self {
        Self::LOG(value)
    }
}
