use crate::compiler::CompilationState;
use crate::protocol::ProtocolPayload;
use crate::scene::Frame;
use crate::scene::{Scene, Line};
use crate::schedule::action_timing::ActionTiming;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulerMessage {
    /// Set the entire scene.
    SetScene(Scene, ActionTiming),
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
    SetScript(usize, usize, String, String, ActionTiming),
    
    /// Set the master tempo.
    SetTempo(f64, ActionTiming),
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
            | SchedulerMessage::SetLines(_, t)
            | SchedulerMessage::ConfigureLines(_, t)
            | SchedulerMessage::AddLine(_, _, t)
            | SchedulerMessage::RemoveLine(_, t)
            | SchedulerMessage::SetFrames(_, t)
            | SchedulerMessage::AddFrame(_, _, _, t)
            | SchedulerMessage::RemoveFrame(_, _, t)
            | SchedulerMessage::SetTempo(_, t)
            | SchedulerMessage::TransportStart(t) 
            | SchedulerMessage::TransportStop(t)
            | SchedulerMessage::DeviceMessage(_, _, t) 
            | SchedulerMessage::GoToFrame(_, _, t) 
            | SchedulerMessage::SetScript(_, _, _, _, t)
                => *t,
            SchedulerMessage::CompilationUpdate(_, _, _, _)
            | SchedulerMessage::Shutdown => ActionTiming::Immediate,
        }
    }

    pub fn should_apply(&self, current_beat: f64, last_beat: f64, scene: &Scene) -> bool {
        match self.timing() {
            ActionTiming::Immediate => false,
            ActionTiming::AtBeat(target) => current_beat >= target as f64,
            ActionTiming::EndOfLine(i) => {
                let Some(line) = scene.line(i) else {
                    return false;
                };
                let len = line.length();
                if len <= 0.0 {
                    false
                } else {
                    (last_beat % len) > (current_beat % len)
                }
            }
        }
    }

}