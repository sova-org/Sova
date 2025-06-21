use serde::{Deserialize, Serialize};

/// Specifies when a scheduler action should be applied.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionTiming {
    /// Apply the action immediately upon processing.
    Immediate,
    /// Apply the action at the start of the next scene loop (quantized to scene length).
    EndOfScene,
    /// Apply the action when the clock beat reaches or exceeds this value.
    AtBeat(u64), // Using u64 for beats to simplify comparison/storage
}

impl Default for ActionTiming {
    fn default() -> Self {
        ActionTiming::Immediate
    }
}
