use midi::MIDIMessage;
use osc::OSDMessage;

use crate::clock::SyncTime;

pub mod midi;
pub mod osc;

/// Unified message type to transmit any message supported by a protocol
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProtocolMessage {
    OSD(OSDMessage),
    MIDI(MIDIMessage)
}

/// ProtocolMessage salted with a time information
pub struct TimedMessage {
    message : ProtocolMessage,
    time : SyncTime
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

impl From<OSDMessage> for ProtocolMessage {
    fn from(value: OSDMessage) -> Self {
        Self::OSD(value)
    }
}

impl From<MIDIMessage> for ProtocolMessage {
    fn from(value: MIDIMessage) -> Self {
        Self::MIDI(value)
    }
}
