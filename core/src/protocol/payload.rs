use crate::lang::event::ConcreteEvent;
use crate::protocol::osc::Argument;
use crate::protocol::{log::LogMessage, midi::MIDIMessage, osc::OSCMessage};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Represents the actual data payload for different protocols.
///
/// This enum unifies message types from various protocols (OSC, MIDI, Log)
/// into a single type for easier handling within the system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioEnginePayload(pub Vec<Argument>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProtocolPayload {
    OSC(OSCMessage),
    MIDI(MIDIMessage),
    LOG(LogMessage),
    AudioEngine(AudioEnginePayload),
}

impl Display for ProtocolPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolPayload::OSC(m) => std::fmt::Display::fmt(m, f),
            ProtocolPayload::MIDI(m) => std::fmt::Display::fmt(m, f),
            ProtocolPayload::LOG(m) => std::fmt::Display::fmt(m, f),
            ProtocolPayload::AudioEngine(m) => write!(
                f,
                "AudioEngine: {} args",
                m.0.len(),
            ),
        }
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

impl From<AudioEnginePayload> for ProtocolPayload {
    fn from(value: AudioEnginePayload) -> Self {
        Self::AudioEngine(value)
    }
}

impl From<ConcreteEvent> for Option<AudioEnginePayload> {
    fn from(event: ConcreteEvent) -> Self {
        match event {
            ConcreteEvent::Dirt { args, device_id: _ } => {
                Some(AudioEnginePayload(args))
            }
            _ => None,
        }
    }
}
