use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::compiler::CompilationState;
use crate::error::SovaError;
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

    CompilationUpdated(usize, usize, u64, CompilationState),

    TempoChanged(f64),
    QuantumChanged(f64),
    Log(LogMessage),
    PlaybackStateChanged(PlaybackState),
    /// Current frame position for each playing line (line_idx, frame_idx, repetition_idx)
    FramePositionChanged(Vec<Vec<(usize, usize)>>),
    /// List of connected clients changed.
    ClientListChanged(Vec<String>),
    /// A chat message was received from a client.
    ChatReceived(String, String), // (sender_name, message)
    /// A peer started editing a specific frame.
    PeerStartedEditingFrame(String, usize, usize),
    /// A peer stopped editing a specific frame.
    PeerStoppedEditingFrame(String, usize, usize),
    /// The list of available/connected devices changed.
    DeviceListChanged(Vec<DeviceInfo>),
    /// Global variables have been updated
    GlobalVariablesChanged(HashMap<String, VariableValue>),
    /// Oscilloscope waveform data as min/max peak pairs.
    ScopeData(Vec<(f32, f32)>),
    /// An internal error occured
    Error(SovaError)
}
