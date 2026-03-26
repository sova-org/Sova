use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::compiler::CompilationState;
use crate::error::SovaError;
use crate::scene::script::Script;
use crate::vm::interpreter::Annotation;
use crate::vm::variable::VariableValue;
use crate::scene::{ExecutionMode, Frame, Line, Scene};
use crate::protocol::DeviceInfo;
use crate::LogMessage;
use crate::schedule::playback::PlaybackState;

/// Enum representing notifications broadcast by the Scheduler.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum SovaNotification {
    #[default]
    Tick,
    /// New scene value
    UpdatedScene(Scene),
    /// New global execution mode
    UpdatedSceneMode(ExecutionMode),
    /// New prelude
    UpdatedScenePrelude(Vec<Script>),
    /// New lines values
    UpdatedLines(Vec<(usize, Line)>),
    /// New lines configurations (without frames)
    UpdatedLineConfigurations(Vec<(usize, Line)>),
    /// Added a line
    AddedLine(usize, Line),
    /// Removed a line
    RemovedLine(usize),
    /// New frames values
    UpdatedFrames(Vec<(usize, usize, Frame)>),
    /// Added a frame
    AddedFrame(usize, usize, Frame),
    /// Removed a frame
    RemovedFrame(usize, usize),
    /// Updates the compilation state of a frame
    CompilationUpdated(usize, usize, u64, CompilationState),
    /// Updates the tempo
    TempoChanged(f64),
    /// Updates the quantum
    QuantumChanged(f64),
    /// Relays a log message
    Log(LogMessage),
    /// Updates the playback state
    PlaybackStateChanged(PlaybackState),
    /// Current frame position for each playing line (line_idx, frame_idx, repetition_idx)
    FramePositionChanged(Vec<Vec<(usize, usize)>>),
    /// The list of available/connected devices changed.
    DeviceListChanged(Vec<DeviceInfo>),
    /// Global variables have been updated
    GlobalVariablesChanged(HashMap<String, VariableValue>),
    /// Updates scene annotations
    Annotations(Vec<Vec<Vec<Annotation>>>),
    /// An internal error occured
    Error(SovaError)
}
