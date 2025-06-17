use serde::{Deserialize, Serialize};
use crate::protocol::{
    osc::OSCMessage,
    midi::MIDIMessage,
    log::LogMessage,
};
use std::fmt::Display;
use crate::lang::event::AudioEngineValue;
use std::collections::HashMap;

/// Represents the actual data payload for different protocols.
///
/// This enum unifies message types from various protocols (OSC, MIDI, Log)
/// into a single type for easier handling within the system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioEnginePayload {
    pub source_name: String,
    pub parameters: HashMap<String, AudioEngineValue>,
    pub voice_id: Option<u32>,
    pub track_id: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlMessage {
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProtocolPayload {
    OSC(OSCMessage),
    MIDI(MIDIMessage),
    LOG(LogMessage),
    AudioEngine(AudioEnginePayload),
    Control(ControlMessage),
}

impl Display for ProtocolPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolPayload::OSC(m) => std::fmt::Display::fmt(m, f),
            ProtocolPayload::MIDI(m) => std::fmt::Display::fmt(m, f),
            ProtocolPayload::LOG(m) => std::fmt::Display::fmt(m, f),
            ProtocolPayload::AudioEngine(m) => write!(f, "AudioEngine: {} (track {})", m.source_name, m.track_id),
            ProtocolPayload::Control(m) => write!(f, "Control: {:?}", m),
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

impl From<crate::lang::event::ConcreteEvent> for Option<AudioEnginePayload> {
    fn from(event: crate::lang::event::ConcreteEvent) -> Self {
        match event {
            crate::lang::event::ConcreteEvent::AudioEngine { source_name, parameters, voice_id, track_id } => {
                Some(AudioEnginePayload {
                    source_name,
                    parameters,
                    voice_id,
                    track_id,
                })
            }
            _ => None,
        }
    }
}