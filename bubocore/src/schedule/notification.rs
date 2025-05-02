use crate::scene::{line::Line, Scene};
use crate::scene::script::Script;
use crate::protocol::message::TimedMessage;
use crate::shared_types::{DeviceInfo, GridSelection};

/// Enum representing notifications broadcast by the Scheduler.
#[derive(Debug, Clone, Default)]
pub enum SchedulerNotification {
    #[default]
    Nothing,
    UpdatedScene(Scene),
    UpdatedLine(usize, Line),
    TempoChanged(f64),
    Log(TimedMessage),
    TransportStarted,
    TransportStopped,
    /// Current frame position for each playing line (line_idx, frame_idx, repetition_idx)
    FramePositionChanged(Vec<(usize, usize, usize)>),
    /// List of connected clients changed.
    ClientListChanged(Vec<String>),
    /// A chat message was received from a client.
    ChatReceived(String, String), // (sender_name, message)
    /// Enable specific frames in a line
    EnableFrames(usize, Vec<usize>),
    /// Disable specific frames in a line
    DisableFrames(usize, Vec<usize>),
    /// Uploaded script to a line/frame
    UploadedScript(usize, usize, Script),
    /// Set line frames
    UpdatedLineFrames(usize, Vec<f64>),
    /// Added a line
    AddedLine(Line),
    /// Removed a line
    RemovedLine(usize),
    /// A peer updated their grid selection.
    PeerGridSelectionChanged(String, GridSelection),
    /// A peer started editing a specific frame.
    PeerStartedEditingFrame(String, usize, usize),
    /// A peer stopped editing a specific frame.
    PeerStoppedEditingFrame(String, usize, usize),
    /// The total length of the scene (in lines) changed.
    SceneLengthChanged(usize),
    /// The list of available/connected devices changed.
    DeviceListChanged(Vec<DeviceInfo>),
}
