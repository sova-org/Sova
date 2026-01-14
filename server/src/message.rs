use std::collections::HashMap;

use sova_core::{
    compiler::CompilationState,
    vm::variable::VariableValue,
    protocol::{log::LogMessage, DeviceInfo},
    scene::{Frame, Line, Scene},
    schedule::playback::PlaybackState,
    clock::SyncTime,
};
use serde::{Deserialize, Serialize};

use crate::server::Snapshot;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Hello {
        username: String,
        scene: Scene,
        devices: Vec<DeviceInfo>,
        peers: Vec<String>,
        link_state: (f64, f64, f64, u32, bool),
        is_playing: bool,
        available_languages: Vec<String>,
        audio_engine_status: (bool, Option<String>),
    },
    PeersUpdated(Vec<String>),
    PeerStartedEditing(String, usize, usize),
    PeerStoppedEditing(String, usize, usize),
    PlaybackStateChanged(PlaybackState),
    Log(LogMessage),
    Chat(String, String),
    Success,
    InternalError(String),
    ConnectionRefused(String),
    Snapshot(Snapshot),
    DeviceList(Vec<DeviceInfo>),
    ClockState(f64, f64, SyncTime, f64),
    SceneValue(Scene),
    LineValues(Vec<(usize, Line)>),
    LineConfigurations(Vec<(usize, Line)>),
    AddLine(usize, Line),
    RemoveLine(usize),
    FrameValues(Vec<(usize, usize, Frame)>),
    AddFrame(usize, usize, Frame),
    RemoveFrame(usize, usize),
    FramePosition(Vec<(usize, usize)>),
    GlobalVariablesUpdate(HashMap<String, VariableValue>),
    CompilationUpdate(usize, usize, u64, CompilationState),
    DevicesRestored { missing_devices: Vec<String> },
    AudioEngineStatus { running: bool, device: Option<String> },
}

impl ServerMessage {
    pub fn compression_strategy(&self) -> crate::client::CompressionStrategy {
        use crate::client::CompressionStrategy;
        match self {
            ServerMessage::PeerStartedEditing(_, _, _)
            | ServerMessage::PeerStoppedEditing(_, _, _)
            | ServerMessage::ClockState(_, _, _, _)
            | ServerMessage::FramePosition(_)
            | ServerMessage::PlaybackStateChanged(_)
            | ServerMessage::GlobalVariablesUpdate(_)
            | ServerMessage::AudioEngineStatus { .. } => CompressionStrategy::Never,

            ServerMessage::Hello { .. }
            | ServerMessage::SceneValue(_)
            | ServerMessage::LineValues(_)
            | ServerMessage::Snapshot(_)
            | ServerMessage::DeviceList(_) => CompressionStrategy::Always,

            _ => CompressionStrategy::Adaptive,
        }
    }
}
