use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{Scene, clock::{Clock, NEVER, SyncTime}, schedule::ActionTiming};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionMode {
    Free,
    #[default]
    AtQuantum,
    LongestLine,
}

impl ExecutionMode {
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
            Self::AtQuantum => "AtQuantum",
            Self::LongestLine => "LongestLine"
        };
        write!(f, "{name}")
    }
}

impl From<String> for ExecutionMode {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Free" => Self::Free,
            "AtQuantum" => Self::AtQuantum,
            "LongestLine" => Self::LongestLine,
            _ => Default::default()
        }
    }
}