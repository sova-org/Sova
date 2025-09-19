use crate::{
    clock::{Clock, SyncTime}, lang::interpreter::InterpreterDirectory, log_println, scene::{script::ScriptExecution, Scene}, schedule::{
        notification::SchedulerNotification,
        scheduler_state::PlaybackState, Scheduler,
    }
};
use crossbeam_channel::Sender;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

const INACTIVE_LINK_UPDATE_MICROS : u64 = 100_000;
const ACTIVE_LINK_UPDATE_MICROS : u64 = 1000;

pub struct PlaybackManager {
    playback_state: PlaybackState,
    shared_atomic_is_playing: Arc<AtomicBool>,
    pub last_beat: f64,
}

impl PlaybackManager {
    pub fn new(shared_atomic_is_playing: Arc<AtomicBool>) -> Self {
        Self {
            playback_state: PlaybackState::Stopped,
            shared_atomic_is_playing,
            last_beat: 0.0,
        }
    }

    pub fn update_state(
        &mut self,
        clock: &Clock,
        current_beat: f64,
        interpreters: &InterpreterDirectory,
        scene: &mut Scene,
        executions: &mut Vec<ScriptExecution>,
        update_notifier: &Sender<SchedulerNotification>,
    ) -> Option<SyncTime> {
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

                        self.reset_scene_state(scene);
                        executions.clear();

                        let start_date = clock.date_at_beat(target_beat);
                        self.schedule_initial_scripts(clock, scene, interpreters, executions, start_date);

                        self.playback_state = PlaybackState::Playing;
                        self.shared_atomic_is_playing.store(true, Ordering::Relaxed);
                        None
                    } else {
                        Some(ACTIVE_LINK_UPDATE_MICROS)
                    }
                } else {
                    log_println!(
                        "[SCHEDULER] Link stopped while waiting to start. Returning to Stopped state."
                    );
                    self.playback_state = PlaybackState::Stopped;
                    self.shared_atomic_is_playing
                        .store(false, Ordering::Relaxed);
                    if !executions.is_empty() {
                        executions.clear();
                    }
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
                    self.shared_atomic_is_playing
                        .store(false, Ordering::Relaxed);
                    if !executions.is_empty() {
                        executions.clear();
                    }
                    let _ = update_notifier.send(SchedulerNotification::TransportStopped);
                    Some(INACTIVE_LINK_UPDATE_MICROS)
                }
            }
        }
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.playback_state, PlaybackState::Playing)
    }

    pub fn process_transport_start(
        &mut self,
        clock: &mut Clock,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let current_micros = clock.micros();
        let current_beat = clock.beat_at_date(current_micros);
        let quantum = clock.quantum();
        // High-precision quantum synchronization for transport start requests
        use fraction::Fraction;
        let current_fraction = Fraction::from(current_beat);
        let quantum_fraction = Fraction::from(quantum);
        let start_beat = ((current_fraction / quantum_fraction).floor() + 1) * quantum_fraction;
        let start_beat =
            f64::try_from(start_beat).unwrap_or(((current_beat / quantum).floor() + 1.0) * quantum);
        let start_micros = clock.date_at_beat(start_beat);

        log_println!(
            "[SCHEDULER] Requesting transport start via Link at beat {} ({} micros)",
            start_beat, start_micros
        );

        clock.session_state.set_is_playing(true, start_micros);
        clock.commit_app_state();
        let _ = update_notifier.send(SchedulerNotification::TransportStarted);
    }

    pub fn process_transport_stop(
        &mut self,
        clock: &mut Clock,
        executions: &mut Vec<ScriptExecution>,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let now_micros = clock.micros();
        log_println!("[SCHEDULER] Requesting transport stop via Link now");

        clock.session_state.set_is_playing(false, now_micros);
        clock.commit_app_state();

        executions.clear();
        let _ = update_notifier.send(SchedulerNotification::TransportStopped);
        self.shared_atomic_is_playing
            .store(false, Ordering::Relaxed);
    }

    fn reset_scene_state(&self, scene: &mut Scene) {
        for line in scene.lines_iter_mut() {
            line.current_frame = usize::MAX;
            line.current_iteration = 0;
            line.first_iteration_index = 0;
            line.frames_passed = 0;
            line.frames_executed = 0;
        }
    }

    fn schedule_initial_scripts(
        &self,
        clock: &Clock,
        scene: &Scene,
        interpreters: &InterpreterDirectory,
        executions: &mut Vec<ScriptExecution>,
        start_date: SyncTime,
    ) {
        for line in scene.lines.iter() {
            let (frame, iter, rep, _scheduled_date, _) =
                line.calculate_frame_index(clock, start_date);
            if frame == line.get_effective_start_frame()
                && line.is_frame_enabled(frame)
                && iter == 0
                && rep == 0
            {
                let script = Arc::clone(&line.frame(frame).script);
                Scheduler::execute_script(executions, &script, interpreters, start_date);
                log_println!(
                    "[SCHEDULER] Queued script for Line {} Frame {} at start",
                    line.index, frame
                );
            }
        }
    }
}
