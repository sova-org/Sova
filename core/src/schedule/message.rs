use crate::compiler::CompilationState;
use crate::protocol::ProtocolPayload;
use crate::scene::{ExecutionMode, Frame};
use crate::scene::script::Script;
use crate::scene::{Scene, Line};
use crate::schedule::action_timing::ActionTiming;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulerMessage {
    /// Set the entire scene.
    SetScene(Scene, ActionTiming),
    SetGlobalMode(Option<ExecutionMode>, ActionTiming),
    /// Set a line at a specific index.
    SetLines(Vec<(usize, Line)>, ActionTiming),
    ConfigureLines(Vec<(usize, Line)>, ActionTiming),
    AddLine(usize, Line, ActionTiming),
    RemoveLine(usize, ActionTiming),

    /// Set the current frame in specified line
    GoToFrame(usize, usize, ActionTiming),
    
    /// Set a frame at a specific index
    SetFrames(Vec<(usize, usize, Frame)>, ActionTiming),
    /// Insert a frame with a given value at a specific position in a line.
    AddFrame(usize, usize, Frame, ActionTiming),
    /// Remove the frame at a specific position in a line.
    RemoveFrame(usize, usize, ActionTiming),

    /// Set the script content and lang for specified frame
    SetScript(usize, usize, Script, ActionTiming),
    
    /// Set the master tempo.
    SetTempo(f64, ActionTiming),
    /// Set the clock quantum.
    SetQuantum(f64, ActionTiming),
    /// Request the transport to start playback at the specified timing.
    TransportStart(ActionTiming),
    /// Request the transport to stop playback at the specified timing.
    TransportStop(ActionTiming),

    /// Sends a direct message to a device
    DeviceMessage(usize, ProtocolPayload, ActionTiming),

    /// Updates the compilation status of a frame
    CompilationUpdate(usize, usize, u64, CompilationState),

    /// Request the scheduler to shutdown cleanly.
    Shutdown,
}

impl SchedulerMessage {

    pub fn timing(&self) -> ActionTiming {
        match self {
            SchedulerMessage::SetScene(_, t)
            | SchedulerMessage::SetGlobalMode(_, t)
            | SchedulerMessage::SetLines(_, t)
            | SchedulerMessage::ConfigureLines(_, t)
            | SchedulerMessage::AddLine(_, _, t)
            | SchedulerMessage::RemoveLine(_, t)
            | SchedulerMessage::SetFrames(_, t)
            | SchedulerMessage::AddFrame(_, _, _, t)
            | SchedulerMessage::RemoveFrame(_, _, t)
            | SchedulerMessage::SetTempo(_, t)
            | SchedulerMessage::SetQuantum(_, t)
            | SchedulerMessage::TransportStart(t) 
            | SchedulerMessage::TransportStop(t)
            | SchedulerMessage::DeviceMessage(_, _, t) 
            | SchedulerMessage::GoToFrame(_, _, t) 
            | SchedulerMessage::SetScript(_, _, _, t)
                => *t,
            SchedulerMessage::CompilationUpdate(_, _, _, _)
            | SchedulerMessage::Shutdown => ActionTiming::Immediate,
        }
    }

}