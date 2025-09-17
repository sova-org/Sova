use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::scene::script::Script;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    /// The duration of the frame in beats.
    pub duration: f64,
    /// Specifies how many times each frame should repeat consecutively before moving to the next.
    /// A value of `1` means the frame plays once. Defaults to `1`.
    #[serde(default="default_repetitions")]
    pub repetitions: usize,
    /// Tracks whether the frame in is currently active for playback.
    #[serde(default="default_enabledness")]
    pub enabled: bool,
    /// Scripts associated with the frame. Executed when the frame becomes active.
    /// Stored in `Arc` for potentially shared ownership or cheaper cloning.
    pub script: Arc<Script>,
    /// Optional user-defined names for each frame. Useful for identification in UIs or debugging.
    #[serde(default)]
    pub name: Option<String>
}

impl Frame {

    pub fn effective_duration(&self) -> f64 {
        self.duration * (self.repetitions as f64)
    }

}

fn default_repetitions() -> usize {
    1
}

fn default_enabledness() -> bool {
    true
}

impl From<f64> for Frame {
    fn from(value: f64) -> Self {
        Frame {
            duration: value,
            repetitions: default_repetitions(),
            enabled: default_enabledness(),
            script: Default::default(),
            name: None
        }
    }
}

impl From<Script> for Frame {
    fn from(value: Script) -> Self {
        Frame {
            duration: 1.0,
            repetitions: default_repetitions(),
            enabled: default_enabledness(),
            script: Arc::new(value),
            name: None
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Frame {
            duration: 1.0,
            repetitions: default_repetitions(),
            enabled: default_enabledness(),
            script: Default::default(),
            name: None
        }
    }
}
