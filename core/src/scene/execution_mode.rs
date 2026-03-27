use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{Scene, clock::{Clock, NEVER, SyncTime}, schedule::ActionTiming};

/// The global execution mode of the scene
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionMode {
    #[default]
    Free,
    AtQuantum,
    LongestLine,
}

impl ExecutionMode {
    /// Computes the time remaining before the next trigger of the mode,
    /// when the mode is not [ExecutionMode::Free].
    pub fn remaining(&self, scene: &Scene, date: SyncTime, clock: &Clock) -> SyncTime {
        match self {
            ExecutionMode::AtQuantum => {
                ActionTiming::AtNextPhase.remaining(date, clock)
            }
            ExecutionMode::Free => {
                NEVER
            }
            ExecutionMode::LongestLine => {
                let Some(line) = scene.longest_line() else {
                    return NEVER;
                };
                ActionTiming::AtNextModulo(line.length()).remaining(date, clock)
            }
        }
    }

    pub fn is_free(&self) -> bool {
        matches!(self, ExecutionMode::Free)
    }
}

impl Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Free => "Free",
            Self::AtQuantum => "Quantum",
            Self::LongestLine => "Longest"
        };
        write!(f, "{name}")
    }
}

impl From<String> for ExecutionMode {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Free" => Self::Free,
            "Quantum" | "AtQuantum" => Self::AtQuantum,
            "Longest" | "LongestLine" => Self::LongestLine,
            _ => Default::default()
        }
    }
}