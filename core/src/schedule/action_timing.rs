use serde::{Deserialize, Serialize};

/// Specifies when a scheduler action should be applied.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ActionTiming {
    /// Apply the action immediately upon processing.
    #[default]
    Immediate,
    /// Apply the action at the start of the next line loop.
    EndOfLine(usize),
    /// Apply the action when the clock beat reaches or exceeds this value.
    AtBeat(u64), // Using u64 for beats to simplify comparison/storage
}
