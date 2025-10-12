use std::collections::HashMap;

use crate::{compiler::CompilationState, lang::variable::VariableValue, protocol::DeviceInfo, scene::{Frame, Line}, server::Snapshot};
use serde::{Deserialize, Serialize};

use crate::{
    clock::SyncTime,
    compiler::CompilationError,
    scene::Scene,
};

/// Represents messages sent FROM the server TO a client.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    /// Initial greeting message sent upon successful connection.
    /// Includes necessary state for the client to initialize.
    Hello {
        /// The client's assigned username.
        username: String,
        /// The current scene state.
        scene: Scene,
        /// List of available/connected devices.
        devices: Vec<DeviceInfo>,
        /// List of names of other currently connected clients.
        peers: Vec<String>,
        /// Current Link state (tempo, beat, phase, peers, is_enabled).
        link_state: (f64, f64, f64, u32, bool),
        /// Current transport playing state.
        is_playing: bool,
        /// List of available languages names.
        available_languages: Vec<String>,
        /// Map of compiler name to its .sublime-syntax content.
        syntax_definitions: std::collections::HashMap<String, String>,
    },
    /// Broadcast containing the updated list of connected client names.
    PeersUpdated(Vec<String>),
    /// Broadcasts that a peer started editing a specific frame.
    PeerStartedEditing(String, usize, usize),
    /// Broadcasts that a peer stopped editing a specific frame.
    PeerStoppedEditing(String, usize, usize),
    /// Confirms a script was successfully compiled and uploaded.
    ScriptCompiled { line_idx: usize, frame_idx: usize },
    /// Sends compilation error details back to the client.
    CompilationErrorOccurred(CompilationError),
    /// Indicates the transport playback has started.
    TransportStarted,
    /// Indicates the transport playback has stopped.
    TransportStopped,
    /// A log message originating from the server or scheduler.
    LogString(String),
    /// A chat message broadcast from another client or the server itself.
    Chat(String, String),
    /// Generic success response, indicating a requested action was accepted.
    Success,
    /// Indicates an internal server error occurred while processing a request.
    InternalError(String),
    /// Indicate connection refused (e.g., username taken).
    ConnectionRefused(String),
    /// A complete snapshot of the current server state (used for save/load?).
    Snapshot(Snapshot),
    /// Sends the full list of available/connected devices (can be requested).
    DeviceList(Vec<DeviceInfo>),
    /// tempo, beat, micros, quantum
    ClockState(f64, f64, SyncTime, f64),
    /// Broadcast containing the complete current state of the scene.
    SceneValue(Scene),
    /// Broadcast the value of specific lines
    LineValues(Vec<(usize, Line)>),
    /// Broadcast the configurations (without frames) of specific lines
    LineConfigurations(Vec<(usize, Line)>),
    /// Broadcast a line insertion
    AddLine(usize, Line),
    /// Broadcast a line removal
    RemoveLine(usize),
    /// Broadcast the values of specific frames
    FrameValues(Vec<(usize, usize, Frame)>),
    /// The current frame positions within each line (line_idx, frame_idx, repetition_idx)
    FramePosition(Vec<(usize, usize)>),
    /// Broadcast a frame insertion
    AddFrame(usize, usize, Frame),
    /// Broadcast a frame removal
    RemoveFrame(usize, usize),
    /// Update of global variables (single-letter variables A-Z)
    GlobalVariablesUpdate(HashMap<String, VariableValue>),
    /// Compilation status update for a frame
    CompilationUpdate(usize, usize, u64, CompilationState)
}

impl ServerMessage {
    /// Get the compression strategy for this message type based on semantics
    pub fn compression_strategy(&self) -> crate::server::client::CompressionStrategy {
        use crate::server::client::CompressionStrategy;
        match self {
            // Real-time/frequent messages that should never be compressed
            | ServerMessage::PeerStartedEditing(_, _, _)
            | ServerMessage::PeerStoppedEditing(_, _, _)
            | ServerMessage::ClockState(_, _, _, _)
            | ServerMessage::FramePosition(_)
            | ServerMessage::TransportStarted
            | ServerMessage::TransportStopped
            | ServerMessage::GlobalVariablesUpdate(_) => CompressionStrategy::Never,

            // Large content messages that should always be compressed if beneficial
            ServerMessage::Hello { .. }
            | ServerMessage::SceneValue(_)
            | ServerMessage::LineValues(_)
            | ServerMessage::Snapshot(_)
            | ServerMessage::DeviceList(_) => CompressionStrategy::Always,

            // Everything else uses adaptive compression
            _ => CompressionStrategy::Adaptive,
        }
    }
}