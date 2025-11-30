use serde::{Deserialize, Serialize};

use crate::{clock::{Clock, SyncTime, NEVER}, Scene};

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
    AtNextBeat,
}

impl ActionTiming {

    pub fn remaining(&self, date: SyncTime, clock: &Clock, scene: &Scene) -> SyncTime {
        let beat = clock.beat_at_date(date);
        match self {
            ActionTiming::Immediate => 0,
            ActionTiming::EndOfLine(i) => {
                let Some(line) = scene.line(*i) else {
                    return 0;
                };
                if line.length() <= 0.0 {
                    NEVER
                } else {
                    line.before_end(clock, date)
                }
            }
            ActionTiming::AtBeat(b) => {
                let target = *b as f64;
                if target <= beat {
                    0
                } else {
                    clock.beats_to_micros(target - beat)
                }
            }
            ActionTiming::AtNextBeat => {
                let rem = 1.0 - (beat % 1.0);
                clock.beats_to_micros(rem) 
            }
        }
    }

    pub fn should_apply(&self, previous_beat: f64, current_beat: f64, scene: &Scene) -> bool {
        match self {
            ActionTiming::Immediate => false,
            ActionTiming::AtBeat(target) => current_beat >= *target as f64,
            ActionTiming::EndOfLine(i) => {
                scene.line(*i).map(|l| l.end_flag).unwrap_or_default()
            }
            ActionTiming::AtNextBeat => {
                previous_beat.floor() != current_beat.floor()
            }
        }
    }

}
