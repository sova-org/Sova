use serde::{Deserialize, Serialize};

use crate::schedule::ActionTiming;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ExecutionMode {
    pub starting: ActionTiming,
    pub looping: bool,
    pub trailing: bool,
}