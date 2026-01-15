use crate::{
    clock::{Clock, SyncTime},
    log_println,
    scene::Scene,
};

use serde::{Deserialize, Serialize};

const INACTIVE_LINK_UPDATE_MICROS: u64 = 100_000;
const ACTIVE_LINK_UPDATE_MICROS: u64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Starting(f64),
    Playing,
}

impl PlaybackState {
    pub fn is_playing(&self) -> bool {
        matches!(self, PlaybackState::Playing)
    }
}

#[derive(Debug, Default)]
pub struct PlaybackManager {
    playback_state: PlaybackState,
    has_changed: bool,
}

impl PlaybackManager {
    pub fn update_state(&mut self, clock: &Clock, scene: &mut Scene) -> Option<SyncTime> {
        self.has_changed = false;
        let current_beat = clock.beat();
        let link_is_playing = clock.session_state.is_playing();

        match self.playback_state {
            PlaybackState::Stopped => {
                if link_is_playing {
                    let start_beat = clock.next_phase_reset_beat();
                    log_println!(
                        "Link is playing, scheduler was stopped. Waiting for beat {:.4} to start.",
                        start_beat
                    );
                    self.playback_state = PlaybackState::Starting(start_beat);
                    self.has_changed = true;
                    Some(ACTIVE_LINK_UPDATE_MICROS)
                } else {
                    Some(INACTIVE_LINK_UPDATE_MICROS)
                }
            }
            PlaybackState::Starting(target_beat) => {
                if link_is_playing {
                    if current_beat >= target_beat {
                        log_println!("Target beat {:.4} reached. Starting playback.", target_beat);

                        scene.kill_executions();
                        scene.reset();

                        self.playback_state = PlaybackState::Playing;
                        self.has_changed = true;
                        None
                    } else {
                        Some(ACTIVE_LINK_UPDATE_MICROS)
                    }
                } else {
                    log_println!(
                        "Link stopped while waiting to start. Returning to Stopped state."
                    );
                    self.playback_state = PlaybackState::Stopped;
                    self.has_changed = true;
                    scene.kill_executions();
                    Some(INACTIVE_LINK_UPDATE_MICROS)
                }
            }
            PlaybackState::Playing => {
                if link_is_playing {
                    None
                } else {
                    log_println!("Link stopped. Stopping playback and clearing executions.");
                    self.playback_state = PlaybackState::Stopped;
                    self.has_changed = true;
                    scene.kill_executions();
                    Some(INACTIVE_LINK_UPDATE_MICROS)
                }
            }
        }
    }

    pub fn state_has_changed(&self) -> bool {
        self.has_changed
    }

    pub fn state(&self) -> PlaybackState {
        self.playback_state
    }
}
