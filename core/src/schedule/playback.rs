use crate::{
    clock::{Clock, SyncTime}, log_println, scene::{script::ScriptExecution, Scene}, schedule::{
        notification::SovaNotification
    }
};
use crossbeam_channel::Sender;

const INACTIVE_LINK_UPDATE_MICROS : u64 = 100_000;
const ACTIVE_LINK_UPDATE_MICROS : u64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Starting(f64),
    Playing,
}

#[derive(Debug, Default)]
pub struct PlaybackManager {
    playback_state: PlaybackState,
    pub last_beat: f64,
}

impl PlaybackManager {

    pub fn update_state(
        &mut self,
        clock: &Clock,
        scene: &mut Scene,
        update_notifier: &Sender<SovaNotification>,
    ) -> Option<SyncTime> {
        let current_beat = clock.beat();
        let link_is_playing = clock.session_state.is_playing();
        self.last_beat = current_beat;

        match self.playback_state {
            PlaybackState::Stopped => {
                if link_is_playing {
                    let quantum = clock.quantum();
                    // High-precision quantum synchronization using rational arithmetic
                    // Eliminates floating-point precision loss in transport start timing
                    use fraction::Fraction;
                    let current_fraction = Fraction::from(current_beat);
                    let quantum_fraction = Fraction::from(quantum);
                    let target_beat =
                        ((current_fraction / quantum_fraction).floor() + 1) * quantum_fraction;
                    let target_beat = f64::try_from(target_beat)
                        .unwrap_or(((current_beat / quantum).floor() + 1.0) * quantum);
                    log_println!(
                        "[SCHEDULER] Link is playing, scheduler was stopped. Waiting for beat {:.4} to start.",
                        target_beat
                    );
                    self.playback_state = PlaybackState::Starting(target_beat);
                    Some(ACTIVE_LINK_UPDATE_MICROS)
                } else {
                    Some(INACTIVE_LINK_UPDATE_MICROS)
                }
            }
            PlaybackState::Starting(target_beat) => {
                if link_is_playing {
                    if current_beat >= target_beat {
                        log_println!(
                            "[SCHEDULER] Target beat {:.4} reached. Starting playback.",
                            target_beat
                        );

                        scene.kill_executions();
                        scene.reset();

                        self.playback_state = PlaybackState::Playing;
                        None
                    } else {
                        Some(ACTIVE_LINK_UPDATE_MICROS)
                    }
                } else {
                    log_println!(
                        "[SCHEDULER] Link stopped while waiting to start. Returning to Stopped state."
                    );
                    self.playback_state = PlaybackState::Stopped;
                    scene.kill_executions();
                    Some(INACTIVE_LINK_UPDATE_MICROS)
                }
            }
            PlaybackState::Playing => {
                if link_is_playing {
                    None
                } else {
                    log_println!(
                        "[SCHEDULER] Link stopped. Stopping playback and clearing executions."
                    );
                    self.playback_state = PlaybackState::Stopped;
                    scene.kill_executions();
                    let _ = update_notifier.send(SovaNotification::TransportStopped);
                    Some(INACTIVE_LINK_UPDATE_MICROS)
                }
            }
        }
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.playback_state, PlaybackState::Playing)
    }

}
