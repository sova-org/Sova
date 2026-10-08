use crate::audio::AudioEngineState;
use serde::{Deserialize, Serialize};
use sova_core::{
    clock::{ClockSnapshot, LinkState}, protocol::DeviceInfo, scene::Scene, schedule::{SchedulerMessage, SovaNotification}, vm::language::LanguageDefinition,
};

use crate::FrameTextId;
use crate::server::Snapshot;

impl From<ClockSnapshot> for ServerMessage {
    fn from(value: ClockSnapshot) -> Self {
        ServerMessage::ClockState(value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Hello {
        username: String,
        peer_id: u64,
        scene: Scene,
        devices: Vec<DeviceInfo>,
        peers: Vec<String>,
        clock_state: ClockSnapshot,
        is_playing: bool,
        languages: Vec<LanguageDefinition>,
        audio_engine_state: AudioEngineState,
        #[serde(default)]
        link_state: LinkState,
        frame_text_layout: Vec<((usize, usize), FrameTextId)>,
        frame_doc_snapshots: Vec<(FrameTextId, Vec<u8>)>,
        presence: Vec<u8>,
    },
    PeersUpdated(Vec<String>),
    PeerStartedEditing(String, usize, usize),
    PeerStoppedEditing(String, usize, usize),
    Chat(String, String),
    Success,
    InternalError(String),
    ConnectionRefused(String),
    Snapshot(Snapshot),
    ClockState(ClockSnapshot),
    DevicesRestored {
        missing_devices: Vec<String>,
    },
    AudioEngineState(AudioEngineState),
    ScopeData(Vec<f32>),
    PeakData(Vec<f32>),

    FeedbackEnabled,
    Feedback(SchedulerMessage),
    CoreRestarted,
    LinkState(LinkState),
    ScriptEdit {
        sender: String,
        frame_text_id: FrameTextId,
        update: Vec<u8>,
    },
    Presence {
        update: Vec<u8>,
    },
    FrameTextLayout {
        mapping: Vec<((usize, usize), FrameTextId)>,
        new_doc_snapshots: Vec<(FrameTextId, Vec<u8>)>,
    },
    #[serde(untagged)]
    Notification(SovaNotification),
}

