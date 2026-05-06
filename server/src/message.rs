use crate::audio::AudioEngineState;
use serde::{Deserialize, Serialize};
use sova_core::{
    clock::SyncTime,
    protocol::DeviceInfo,
    scene::Scene,
    schedule::{SchedulerMessage, SovaNotification},
    vm::language::LanguageDefinition,
};

use crate::FrameTextId;
use crate::server::Snapshot;

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Hello {
        username: String,
        peer_id: u64,
        scene: Scene,
        devices: Vec<DeviceInfo>,
        peers: Vec<String>,
        link_state: (f64, f64, f64, u32, bool),
        is_playing: bool,
        languages: Vec<LanguageDefinition>,
        audio_engine_state: AudioEngineState,
        #[serde(default = "default_true")]
        link_enabled: bool,
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
    ClockState(f64, f64, SyncTime, f64),
    DevicesRestored {
        missing_devices: Vec<String>,
    },
    AudioEngineState(AudioEngineState),
    ScopeData(Vec<f32>),
    PeakData(Vec<f32>),

    FeedbackEnabled {
        scene: Scene,
        tempo: f64,
        quantum: f64,
        is_playing: bool,
    },
    Feedback(SchedulerMessage),
    CoreRestarted,
    LinkState {
        enabled: bool,
        start_stop_sync: bool,
        num_peers: u32,
    },
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

