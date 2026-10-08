use serde::{Deserialize, Serialize};
use sova_core::{protocol::DeviceInfo, schedule::{ActionTiming, SchedulerMessage}};
use tokio::io;

use crate::AudioRestartConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Login {
        name: String,
        password: Option<String>,
    },
    GetScene,
    GetLine(usize),
    GetFrame(usize, usize),
    GetClock,
    GetPeers,
    Chat(String),
    GetSnapshot,
    StartedEditingFrame(usize, usize),
    StoppedEditingFrame(usize, usize),
    RequestDeviceList,
    ConnectMidiDeviceByName(String),
    DisconnectMidiDeviceByName(String),
    CreateVirtualMidiOutput(String),
    AssignDeviceToSlot(usize, String),
    UnassignDeviceFromSlot(usize),
    CreateOscDevice(String, String, u16),
    CreateOscInputDevice(String, u16),
    RemoveOscDevice(String),
    SetDeviceLatency(String, f64),
    RestoreDevices(Vec<DeviceInfo>),
    GetAudioEngineState,
    RestartAudioEngine(AudioRestartConfig),
    PreviewSample {
        folder: String,
        index: usize,
        begin: f64,
    },
    EnableFeedback,
    RestartCore,
    ResetScene(ActionTiming),
    SetMasterVolume(f32),
    Hush,
    Panic,
    ScriptEdit {
        frame_text_id: crate::FrameTextId,
        update: Vec<u8>,
    },
    Presence {
        update: Vec<u8>,
    },
    SetLinkEnabled(bool),
    SetStartStopSync(bool),
    #[serde(untagged)]
    SchedulerControl(SchedulerMessage),
}

impl ClientMessage {
    pub fn deserialize(bytes: &[u8]) -> io::Result<Option<Self>> {
        match rmp_serde::from_slice::<ClientMessage>(bytes) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("MessagePack deserialization error: {}", e),
            )),
        }
    }
}

impl From<SchedulerMessage> for ClientMessage {
    fn from(value: SchedulerMessage) -> Self {
        ClientMessage::SchedulerControl(value)
    }
}