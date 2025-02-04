use std::cmp::Ordering;

use log::LogMessage;
use midi::MIDIMessage;
use osc::OSCMessage;

use crate::clock::SyncTime;

pub mod midi;
pub mod osc;
pub mod log;

/// Unified message type to transmit any message supported by a protocol
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProtocolMessage {
    OSC(OSCMessage),
    MIDI(MIDIMessage),
    LOG(LogMessage),
}

/// ProtocolMessage salted with a time information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimedMessage {
    pub message : ProtocolMessage,
    pub time : SyncTime
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

impl From<OSCMessage> for ProtocolMessage {
    fn from(value: OSCMessage) -> Self {
        Self::OSC(value)
    }
}

impl From<MIDIMessage> for ProtocolMessage {
    fn from(value: MIDIMessage) -> Self {
        Self::MIDI(value)
    }
}
