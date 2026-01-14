use serde::{Deserialize, Serialize};

use crate::{clock::{Clock, NEVER, SyncTime}};

/// Specifies when a scheduler action should be applied.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ActionTiming {
    /// Apply the action immediately upon processing.
    Immediate,
    /// Apply the action when the clock beat reaches or exceeds this value.
    AtBeat(u64), // Using u64 for beats to simplify comparison/storage
    #[default]
    AtNextBeat,
    AtNextPhase,
    /// Apply the action when reaching the next multiple of this value.
    AtNextModulo(u64),
    Never
}

impl ActionTiming {

    pub fn remaining(&self, date: SyncTime, clock: &Clock) -> SyncTime {
        let beat = clock.beat_at_date(date);
        match self {
            ActionTiming::Immediate => 0,
            ActionTiming::AtNextModulo(m) => {
                let m = *m as f64;
                let rem = m - (beat % m);
                clock.beats_to_micros(rem) 
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
            ActionTiming::AtNextPhase => {
                clock.next_phase_reset_date().saturating_sub(date)
            }
            ActionTiming::Never => NEVER
        }
    }

    pub fn should_apply(&self, clock: &Clock, previous_beat: f64, current_beat: f64) -> bool {
        match self {
            ActionTiming::Immediate => false,
            ActionTiming::AtBeat(target) => current_beat >= *target as f64,
            ActionTiming::AtNextBeat => {
                previous_beat.floor() != current_beat.floor()
            }
            ActionTiming::AtNextPhase => {
                let quantum = clock.quantum();
                (previous_beat.div_euclid(quantum)) != (current_beat.div_euclid(quantum))
            }
            ActionTiming::AtNextModulo(m) => {
                let m = *m as f64;
                (previous_beat.div_euclid(m)) != (current_beat.div_euclid(m))
            }
            ActionTiming::Never => false
        }
    }

}
