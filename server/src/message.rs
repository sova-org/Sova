use std::collections::HashMap;

use crate::audio::AudioEngineState;
use serde::{Deserialize, Serialize};
use sova_core::{
    clock::SyncTime, compiler::CompilationState, error::SovaError, protocol::{DeviceInfo, log::LogMessage}, scene::{ExecutionMode, Frame, Line, Scene, script::Script}, schedule::{SchedulerMessage, playback::PlaybackState}, vm::{language::LanguageDefinition, variable::VariableValue}
};

use crate::server::Snapshot;

fn default_true() -> bool {
    true
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
        languages: Vec<LanguageDefinition>,
        audio_engine_state: AudioEngineState,
        #[serde(default = "default_true")]
        link_enabled: bool,
    },
    PeersUpdated(Vec<String>),
    PeerStartedEditing(String, usize, usize),
    PeerStoppedEditing(String, usize, usize),
    PeerCursorMoved(String, usize, usize, Option<(usize, usize)>),
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
    SceneMode(ExecutionMode),
    LineValues(Vec<(usize, Line)>),
    LineConfigurations(Vec<(usize, Line)>),
    AddLine(usize, Line),
    RemoveLine(usize),
    FrameValues(Vec<(usize, usize, Frame)>),
    AddFrame(usize, usize, Frame),
    RemoveFrame(usize, usize),
    FramePosition(Vec<Vec<(usize, usize)>>),
    GlobalVariablesUpdate(HashMap<String, VariableValue>),
    CompilationUpdate(usize, usize, u64, CompilationState),
    DevicesRestored {
        missing_devices: Vec<String>,
    },
    AudioEngineState(AudioEngineState),
    ScopeData(Vec<f32>),
    PeakData(Vec<f32>),
    Error(SovaError),
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
    ScenePrelude(Vec<Script>),
    HydraCode(String, String),
    ScriptEdit {
        sender: String,
        li: usize,
        fi: usize,
        ops: Vec<crate::TextOp>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use sova_core::{
        compiler::{CompilationError, CompilationState},
        error::SovaError,
        protocol::{
            log::{LogMessage, Severity},
            DeviceInfo,
        },
        scene::{ExecutionMode, Frame, Line, Scene},
        schedule::{ActionTiming, SchedulerMessage, playback::PlaybackState},
        vm::{language::LanguageDefinition, variable::VariableValue},
    };

    fn roundtrip(msg: &ServerMessage) {
        let bytes = rmp_serde::to_vec_named(msg)
            .unwrap_or_else(|e| panic!("serialize failed for {:?}: {e}", std::mem::discriminant(msg)));
        rmp_serde::from_slice::<ServerMessage>(&bytes)
            .unwrap_or_else(|e| {
                panic!(
                    "deserialize failed for {:?} (len={}, first 32 bytes: {:02x?}): {e}",
                    std::mem::discriminant(msg),
                    bytes.len(),
                    &bytes[..bytes.len().min(32)]
                )
            });
    }

    #[test]
    fn server_message_roundtrip_all_variants() {
        let scene = Scene::default();
        let line = Line::default();
        let frame = Frame::default();
        let device = DeviceInfo {
            slot_id: Some(1),
            name: "Test MIDI".into(),
            kind: sova_core::protocol::DeviceKind::Midi,
            direction: sova_core::protocol::DeviceDirection::Output,
            is_connected: true,
            address: None,
            latency: 0.0,
        };
        let audio = AudioEngineState::default();
        let snapshot = Snapshot {
            scene: scene.clone(),
            tempo: 120.0,
            beat: 0.0,
            micros: 0,
            quantum: 4.0,
            devices: vec![device.clone()],
        };

        let variants: Vec<ServerMessage> = vec![
            ServerMessage::Hello {
                username: "test".into(),
                scene: scene.clone(),
                devices: vec![device.clone()],
                peers: vec!["peer1".into()],
                link_state: (120.0, 0.0, 4.0, 4, true),
                is_playing: true,
                languages: vec![LanguageDefinition::default()],
                audio_engine_state: audio.clone(),
                link_enabled: true,
            },
            ServerMessage::PeersUpdated(vec!["a".into(), "b".into()]),
            ServerMessage::PeerStartedEditing("user".into(), 0, 1),
            ServerMessage::PeerStoppedEditing("user".into(), 0, 1),
            ServerMessage::PeerCursorMoved("user".into(), 2, 3, None),
            ServerMessage::PeerCursorMoved("user".into(), 2, 3, Some((10, 5))),
            ServerMessage::PlaybackStateChanged(PlaybackState::Stopped),
            ServerMessage::PlaybackStateChanged(PlaybackState::Starting(1.0)),
            ServerMessage::PlaybackStateChanged(PlaybackState::Playing),
            ServerMessage::Log(LogMessage {
                level: Severity::Info,
                event: None,
                msg: "hello".into(),
            }),
            ServerMessage::Chat("user".into(), "hi".into()),
            ServerMessage::Success,
            ServerMessage::InternalError("oops".into()),
            ServerMessage::ConnectionRefused("nope".into()),
            ServerMessage::Snapshot(snapshot),
            ServerMessage::DeviceList(vec![device.clone()]),
            ServerMessage::ClockState(120.0, 1.5, 1000, 4.0),
            ServerMessage::SceneValue(scene.clone()),
            ServerMessage::SceneMode(ExecutionMode::Free),
            ServerMessage::SceneMode(ExecutionMode::AtQuantum),
            ServerMessage::SceneMode(ExecutionMode::LongestLine),
            ServerMessage::LineValues(vec![(0, line.clone())]),
            ServerMessage::LineConfigurations(vec![(1, line.clone())]),
            ServerMessage::AddLine(0, line.clone()),
            ServerMessage::RemoveLine(2),
            ServerMessage::FrameValues(vec![(0, 0, frame.clone())]),
            ServerMessage::AddFrame(0, 1, frame.clone()),
            ServerMessage::RemoveFrame(1, 2),
            ServerMessage::FramePosition(vec![vec![(0, 1)], vec![(2, 3)]]),
            ServerMessage::GlobalVariablesUpdate(HashMap::from([
                ("x".into(), VariableValue::Integer(42)),
                ("y".into(), VariableValue::Float(3.14)),
                ("z".into(), VariableValue::Bool(true)),
                ("s".into(), VariableValue::Str("hi".into())),
                ("d".into(), VariableValue::Dur(sova_core::clock::TimeSpan::Beats(1.0))),
                ("m".into(), VariableValue::Map(HashMap::from([
                    ("nested_int".into(), VariableValue::Integer(7)),
                    ("nested_str".into(), VariableValue::Str("deep".into())),
                ]))),
                ("v".into(), VariableValue::Vec(vec![
                    VariableValue::Integer(1),
                    VariableValue::Float(2.5),
                    VariableValue::Bool(false),
                ])),
            ])),
            // CompilationState variants — the most likely to cause issues
            ServerMessage::CompilationUpdate(0, 0, 1, CompilationState::NotCompiled),
            ServerMessage::CompilationUpdate(0, 0, 2, CompilationState::Compiling),
            ServerMessage::CompilationUpdate(0, 0, 3, CompilationState::Compiled(Default::default())),
            ServerMessage::CompilationUpdate(0, 0, 4, CompilationState::Parsed(None)),
            ServerMessage::CompilationUpdate(0, 0, 5, CompilationState::Error(CompilationError {
                lang: "bob".into(),
                info: "parse error".into(),
                from: 0,
                to: 10,
            })),
            ServerMessage::DevicesRestored {
                missing_devices: vec!["MIDI1".into()],
            },
            ServerMessage::AudioEngineState(audio),
            ServerMessage::ScopeData(vec![0.1, -0.5, 0.9]),
            ServerMessage::PeakData(vec![0.8, 0.6]),
            ServerMessage::Error(SovaError {
                line: 0,
                frame: 1,
                position: None,
                text: "runtime error".into(),
            }),
            ServerMessage::FeedbackEnabled {
                scene: scene.clone(),
                tempo: 120.0,
                quantum: 4.0,
                is_playing: true,
            },
            ServerMessage::Feedback(SchedulerMessage::SetTempo(140.0, ActionTiming::Immediate)),
            ServerMessage::Feedback(SchedulerMessage::TransportStart(ActionTiming::AtNextBeat)),
            ServerMessage::Feedback(SchedulerMessage::SetScene(scene.clone(), ActionTiming::Immediate)),
            ServerMessage::CoreRestarted,
            ServerMessage::LinkState {
                enabled: true,
                start_stop_sync: false,
                num_peers: 3,
            },
            ServerMessage::HydraCode("alice".into(), "osc().out()".into()),
            ServerMessage::ScenePrelude(vec![Script::default()]),
        ];

        for msg in &variants {
            roundtrip(msg);
        }
    }
}
