use crate::{
    clock::SyncTime,
    scene::script::Script,
    schedule::{action_timing::ActionTiming, message::SchedulerMessage},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicatedFrameData {
    pub length: f64,
    pub is_enabled: bool,
    pub script: Option<Arc<Script>>,
    pub name: Option<String>,
    pub repetitions: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Starting(f64),
    Playing,
}

#[derive(Debug, Clone)]
pub struct DeferredAction {
    pub action: SchedulerMessage,
    pub timing: ActionTiming,
}

impl DeferredAction {
    pub fn new(action: SchedulerMessage, timing: ActionTiming) -> Self {
        Self { action, timing }
    }

    pub fn should_apply(&self, current_beat: f64, last_beat: f64, scene_len_beats: f64) -> bool {
        match self.timing {
            ActionTiming::Immediate => false,
            ActionTiming::AtBeat(target) => current_beat >= target as f64,
            ActionTiming::EndOfScenei(i) => {
                if scene_len_beats <= 0.0 {
                    false
                } else {
                    (last_beat % scene_len_beats) > (current_beat % scene_len_beats)
                }
            }
        }
    }
}

pub const SCHEDULED_DRIFT: SyncTime = 1_000;
