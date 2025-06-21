use super::DuplicatedFrameData;
use crate::scene::script::Script;
use crate::scene::{Scene, line::Line};
use crate::schedule::action_timing::ActionTiming;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulerMessage {
    /// Upload a new scene to the scheduler.
    UploadScene(Scene),
    /// Enable multiple frames in a line.
    EnableFrames(usize, Vec<usize>, ActionTiming),
    /// Disable multiple frames in a line.
    DisableFrames(usize, Vec<usize>, ActionTiming),
    /// Upload a script to a specific line/frame.
    UploadScript(usize, usize, Script, ActionTiming),
    /// Update the frames vector for a line.
    UpdateLineFrames(usize, Vec<f64>, ActionTiming),
    /// Insert a frame with a given value at a specific position in a line.
    InsertFrame(usize, usize, f64, ActionTiming),
    /// Remove the frame at a specific position in a line.
    RemoveFrame(usize, usize, ActionTiming),
    /// Add a new line to the scene.
    AddLine,
    /// Remove a line at a specific index.
    RemoveLine(usize, ActionTiming),
    /// Set a line at a specific index.
    SetLine(usize, Line, ActionTiming),
    /// Set the start frame for a line.
    SetLineStartFrame(usize, Option<usize>, ActionTiming),
    /// Set the end frame for a line.
    SetLineEndFrame(usize, Option<usize>, ActionTiming),
    /// Set the entire scene.
    SetScene(Scene, ActionTiming),
    /// Set the scene length.
    SetSceneLength(usize, ActionTiming),
    /// Set the master tempo.
    SetTempo(f64, ActionTiming),
    /// Set a custom loop length for a specific line.
    SetLineLength(usize, Option<f64>, ActionTiming),
    /// Set the playback speed factor for a specific line.
    SetLineSpeedFactor(usize, f64, ActionTiming),
    /// Request the transport to start playback at the specified timing.
    TransportStart(ActionTiming),
    /// Request the transport to stop playback at the specified timing.
    TransportStop(ActionTiming),
    /// Set the name for a specific frame.
    SetFrameName(usize, usize, Option<String>, ActionTiming), // line_idx, frame_idx, name, timing
    /// Update the language identifier for a specific frame's script.
    SetScriptLanguage(usize, usize, String, ActionTiming), // line_idx, frame_idx, lang, timing
    /// Set the number of repetitions for a specific frame.
    SetFrameRepetitions(usize, usize, usize, ActionTiming), // line_idx, frame_idx, repetitions, timing
    /// Internal: Duplicate a frame (used by server handler)
    InternalDuplicateFrame {
        target_line_idx: usize,
        target_insert_idx: usize,
        frame_length: f64,
        is_enabled: bool,
        script: Option<Arc<Script>>,
        timing: ActionTiming,
    },
    /// Internal: Duplicate a range of frames (used by server handler)
    InternalDuplicateFrameRange {
        target_line_idx: usize,
        target_insert_idx: usize,
        frames_data: Vec<DuplicatedFrameData>,
        timing: ActionTiming,
    },
    /// Internal: Remove frames across potentially multiple lines.
    InternalRemoveFramesMultiLine {
        lines_and_indices: Vec<(usize, Vec<usize>)>,
        timing: ActionTiming,
    },
    /// Internal: Insert blocks of duplicated frame data.
    InternalInsertDuplicatedBlocks {
        // Vec<Vec<...>>: Outer Vec = columns, Inner Vec = rows within that column
        duplicated_data: Vec<Vec<DuplicatedFrameData>>,
        target_line_idx: usize,  // Top-left line index for insertion
        target_frame_idx: usize, // Top-left frame index for insertion
        timing: ActionTiming,
    },
    /// Request the scheduler to shutdown cleanly.
    Shutdown,
}
