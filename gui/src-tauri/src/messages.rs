use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompressionStrategy {
    Never,
    Always,
    Adaptive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridSelection {
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl GridSelection {
    pub fn single(row: usize, col: usize) -> Self {
        GridSelection {
            start: (row, col),
            end: (row, col),
        }
    }

    pub fn is_single(&self) -> bool {
        self.start == self.end
    }

    pub fn bounds(&self) -> ((usize, usize), (usize, usize)) {
        let top = self.start.0.min(self.end.0);
        let bottom = self.start.0.max(self.end.0);
        let left = self.start.1.min(self.end.1);
        let right = self.start.1.max(self.end.1);
        ((top, left), (bottom, right))
    }

    pub fn cursor_pos(&self) -> (usize, usize) {
        self.end
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ActionTiming {
    #[default]
    Immediate,
    EndOfScene,
    AtBeat(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastedFrameData {
    pub length: f64,
    pub is_enabled: bool,
    pub script_content: Option<String>,
    pub name: Option<String>,
    pub repetitions: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub length: usize,
    pub lines: Vec<Line>,
}

impl Default for Scene {
    fn default() -> Self {
        Scene {
            length: 16,
            lines: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub frames: Vec<f64>,
    pub enabled_frames: Vec<bool>,
    pub scripts: Vec<Script>,
    pub frame_names: Vec<Option<String>>,
    pub frame_repetitions: Vec<usize>,
    pub speed_factor: f64,
    pub index: usize,
    pub start_frame: Option<usize>,
    pub end_frame: Option<usize>,
    pub custom_length: Option<f64>,
}

impl Default for Line {
    fn default() -> Self {
        Line {
            frames: vec![1.0],
            enabled_frames: vec![true],
            scripts: vec![Script::default()],
            frame_names: vec![None],
            frame_repetitions: vec![1],
            speed_factor: 1.0,
            index: 0,
            start_frame: None,
            end_frame: None,
            custom_length: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub content: String,
    pub lang: String,
    pub index: usize,
}

impl Default for Script {
    fn default() -> Self {
        Script {
            content: String::new(),
            lang: "bali".to_string(),
            index: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: usize,
    pub name: String,
    pub kind: DeviceKind,
    pub is_connected: bool,
    pub address: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeviceKind {
    Midi,
    Osc,
    Log,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationError {
    pub lang: String,
    pub info: String,
    pub from: Option<usize>,
    pub to: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub scene: Scene,
    pub tempo: f64,
    pub beat: f64,
    pub micros: u64,
    pub quantum: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulerMessage {
    Play,
    Stop,
    Pause,
    Reset,
    Seek(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    SchedulerControl(SchedulerMessage),
    SetTempo(f64, ActionTiming),
    SetName(String),
    EnableFrames(usize, Vec<usize>, ActionTiming),
    DisableFrames(usize, Vec<usize>, ActionTiming),
    SetScript(usize, usize, String, ActionTiming),
    GetScript(usize, usize),
    GetScene,
    SetScene(Scene, ActionTiming),
    GetClock,
    GetPeers,
    Chat(String),
    UpdateLineFrames(usize, Vec<f64>, ActionTiming),
    InsertFrame(usize, usize, f64, ActionTiming),
    RemoveFrame(usize, usize, ActionTiming),
    SetLineStartFrame(usize, Option<usize>, ActionTiming),
    SetLineEndFrame(usize, Option<usize>, ActionTiming),
    GetSnapshot,
    UpdateGridSelection(GridSelection),
    StartedEditingFrame(usize, usize),
    StoppedEditingFrame(usize, usize),
    GetSceneLength,
    SetSceneLength(usize, ActionTiming),
    SetLineLength(usize, Option<f64>, ActionTiming),
    SetLineSpeedFactor(usize, f64, ActionTiming),
    TransportStart(ActionTiming),
    TransportStop(ActionTiming),
    RequestDeviceList,
    ConnectMidiDeviceById(usize),
    DisconnectMidiDeviceById(usize),
    ConnectMidiDeviceByName(String),
    DisconnectMidiDeviceByName(String),
    CreateVirtualMidiOutput(String),
    AssignDeviceToSlot(usize, String),
    UnassignDeviceFromSlot(usize),
    CreateOscDevice(String, String, u16),
    RemoveOscDevice(String),
    DuplicateFrameRange {
        src_line_idx: usize,
        src_frame_start_idx: usize,
        src_frame_end_idx: usize,
        target_insert_idx: usize,
        timing: ActionTiming,
    },
    RemoveFramesMultiLine {
        lines_and_indices: Vec<(usize, Vec<usize>)>,
        timing: ActionTiming,
    },
    RequestDuplicationData {
        src_top: usize,
        src_left: usize,
        src_bottom: usize,
        src_right: usize,
        target_cursor_row: usize,
        target_cursor_col: usize,
        insert_before: bool,
        timing: ActionTiming,
    },
    PasteDataBlock {
        data: Vec<Vec<PastedFrameData>>,
        target_row: usize,
        target_col: usize,
        timing: ActionTiming,
    },
    SetFrameName(usize, usize, Option<String>, ActionTiming),
    SetScriptLanguage(usize, usize, String, ActionTiming),
    SetFrameRepetitions(usize, usize, usize, ActionTiming),
}

impl ClientMessage {
    pub fn compression_strategy(&self) -> CompressionStrategy {
        match self {
            ClientMessage::UpdateGridSelection(_)
            | ClientMessage::StartedEditingFrame(_, _)
            | ClientMessage::StoppedEditingFrame(_, _)
            | ClientMessage::GetClock
            | ClientMessage::GetPeers
            | ClientMessage::GetScript(_, _)
            | ClientMessage::GetScene
            | ClientMessage::GetSnapshot
            | ClientMessage::GetSceneLength
            | ClientMessage::RequestDeviceList => CompressionStrategy::Never,

            ClientMessage::SetScript(_, _, _, _) | ClientMessage::SetScene(_, _) => {
                CompressionStrategy::Always
            }

            _ => CompressionStrategy::Adaptive,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Hello {
        username: String,
        scene: Scene,
        devices: Vec<DeviceInfo>,
        peers: Vec<String>,
        link_state: (f64, f64, f64, u32, bool),
        is_playing: bool,
        available_compilers: Vec<String>,
        syntax_definitions: HashMap<String, String>,
    },
    ConnectionRefused(String),
    Success,
    InternalError(String),
    SceneValue(Scene),
    ScriptContent {
        line_idx: usize,
        frame_idx: usize,
        content: String,
    },
    ScriptCompiled {
        line_idx: usize,
        frame_idx: usize,
    },
    CompilationErrorOccurred(CompilationError),
    SceneLength(usize),
    TransportStarted,
    TransportStopped,
    ClockState(f64, f64, u64, f64),
    FramePosition(Vec<(usize, usize, usize)>),
    DeviceList(Vec<DeviceInfo>),
    PeersUpdated(Vec<String>),
    PeerGridSelectionUpdate(String, GridSelection),
    PeerStartedEditing(String, usize, usize),
    PeerStoppedEditing(String, usize, usize),
    Chat(String),
    LogString(String),
    Snapshot(Snapshot),
    GlobalVariablesUpdate(HashMap<String, VariableValue>),
}