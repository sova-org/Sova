use crate::{
    clock::{Clock, ClockServer, SyncTime},
    device_map::DeviceMap,
    lang::variable::{VariableStore, VariableValue},
    protocol::message::TimedMessage,
    scene::{Scene, script::ScriptExecution},
    schedule::{
        action_timing::ActionTiming,
        execution::ExecutionManager,
        frame_index::calculate_frame_index,
        message::SchedulerMessage,
        notification::SchedulerNotification,
        playback::PlaybackManager,
        scheduler_actions::ActionProcessor,
        scheduler_state::{DeferredAction, SCHEDULED_DRIFT},
    },
};

use crossbeam_channel::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
pub use scheduler_state::DuplicatedFrameData;
use std::{
    sync::{Arc, atomic::AtomicBool},
    thread::JoinHandle,
    time::Duration,
    usize,
};
use thread_priority::ThreadBuilder;

pub mod action_timing;
pub mod execution;
pub mod frame_index;
pub mod message;
pub mod notification;
pub mod playback;
pub mod scheduler_actions;
pub mod scheduler_state;

pub struct Scheduler {
    pub scene: Scene,
    pub global_vars: VariableStore,
    pub executions: Vec<ScriptExecution>,

    world_iface: Sender<TimedMessage>,
    devices: Arc<DeviceMap>,
    clock: Clock,
    message_source: Receiver<SchedulerMessage>,
    update_notifier: Sender<SchedulerNotification>,

    next_wait: Option<SyncTime>,
    processed_scene_modification: bool,
    deferred_actions: Vec<DeferredAction>,
    playback_manager: PlaybackManager,
    shutdown_requested: bool,

    current_positions: Vec<(usize, usize)>,
    audio_engine_events: Vec<(crate::lang::event::ConcreteEvent, SyncTime)>,
}

impl Scheduler {
    pub fn create(
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        world_iface: Sender<TimedMessage>,
        shared_atomic_is_playing: Arc<AtomicBool>,
    ) -> (
        JoinHandle<()>,
        Sender<SchedulerMessage>,
        Receiver<SchedulerNotification>,
    ) {
        let (tx, rx) = crossbeam_channel::unbounded();
        let (p_tx, p_rx) = crossbeam_channel::unbounded();

        let shared_atomic_clone = shared_atomic_is_playing.clone(); // Clone for the thread

        let handle = ThreadBuilder::default()
            .name("BuboCore-scheduler")
            .spawn(move |_| {
                let mut sched = Scheduler::new(
                    clock_server.into(),
                    devices,
                    world_iface,
                    rx,
                    p_tx,
                    shared_atomic_clone, // Pass the clone to new
                );
                sched.do_your_thing();
            })
            .expect("Unable to start Scheduler");
        (handle, tx, p_rx)
    }

    pub fn new(
        clock: Clock,
        devices: Arc<DeviceMap>,
        world_iface: Sender<TimedMessage>,
        receiver: Receiver<SchedulerMessage>,
        update_notifier: Sender<SchedulerNotification>,
        shared_atomic_is_playing: Arc<AtomicBool>,
    ) -> Scheduler {
        Scheduler {
            world_iface,
            scene: Default::default(),
            global_vars: VariableStore::new(),
            executions: Vec::new(),
            devices,
            clock,
            message_source: receiver,
            update_notifier,
            next_wait: None,
            processed_scene_modification: false,
            deferred_actions: Vec::new(),
            playback_manager: PlaybackManager::new(shared_atomic_is_playing),
            shutdown_requested: false,
            current_positions: Vec::new(),
            audio_engine_events: Vec::with_capacity(256),
        }
    }

    pub fn change_scene(&mut self, mut scene: Scene) {
        let date = self.theoretical_date();
        scene.make_consistent();
        let scene_len = scene.length(); // Get length before mutable borrow
        for line in scene.lines_iter_mut() {
            let (frame, iter, _rep, _, _) =
                calculate_frame_index(&self.clock, scene_len, line, date);
            line.current_frame = frame;
            line.current_iteration = iter;
            line.first_iteration_index = iter;
            line.current_repetition = 0;
        }
        self.executions.clear();

        for line in scene.lines.iter() {
            let (frame, _, _, scheduled_date, _) =
                calculate_frame_index(&self.clock, scene_len, line, date);
            if frame < usize::MAX && line.is_frame_enabled(frame) {
                let script = Arc::clone(&line.scripts[frame]);
                self.executions.push(ScriptExecution::execute_at(
                    script,
                    line.index,
                    scheduled_date,
                ));
            }
        }

        self.scene = scene;
        // Notify clients about the completely new scene state
        let _ = self
            .update_notifier
            .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }

    fn apply_action(&mut self, action: SchedulerMessage) {
        match action {
            SchedulerMessage::TransportStart(_) => {
                self.playback_manager
                    .process_transport_start(&mut self.clock, &self.update_notifier);
            }
            SchedulerMessage::TransportStop(_) => {
                self.playback_manager.process_transport_stop(
                    &mut self.clock,
                    &mut self.executions,
                    &self.update_notifier,
                );
            }
            SchedulerMessage::SetTempo(tempo, _) => {
                self.clock.set_tempo(tempo);
                let _ = self
                    .update_notifier
                    .send(SchedulerNotification::TempoChanged(tempo));
            }
            SchedulerMessage::UploadScene(scene) => {
                self.change_scene(scene);
            }
            SchedulerMessage::SetScene(scene, timing) => {
                self.change_scene(scene.clone());
                if timing == ActionTiming::Immediate {
                    let _ = self
                        .update_notifier
                        .send(SchedulerNotification::UpdatedScene(scene.clone()));
                }
            }
            SchedulerMessage::Shutdown => {
                println!("[-] Scheduler received shutdown signal");
                self.shutdown_requested = true;
                return;
            }
            _ => {
                if ActionProcessor::process_scene_modifications(
                    action,
                    &mut self.scene,
                    &self.update_notifier,
                ) {
                    self.processed_scene_modification = true;
                }
            }
        }
        self.processed_scene_modification = true;
    }

    pub fn process_message(&mut self, msg: SchedulerMessage) {
        let timing = match &msg {
            SchedulerMessage::EnableFrames(_, _, t)
            | SchedulerMessage::DisableFrames(_, _, t)
            | SchedulerMessage::UploadScript(_, _, _, t)
            | SchedulerMessage::UpdateLineFrames(_, _, t)
            | SchedulerMessage::InsertFrame(_, _, _, t)
            | SchedulerMessage::RemoveFrame(_, _, t)
            | SchedulerMessage::RemoveLine(_, t)
            | SchedulerMessage::SetLine(_, _, t)
            | SchedulerMessage::SetLineStartFrame(_, _, t)
            | SchedulerMessage::SetLineEndFrame(_, _, t)
            | SchedulerMessage::SetSceneLength(_, t)
            | SchedulerMessage::SetTempo(_, t)
            | SchedulerMessage::SetLineLength(_, _, t)
            | SchedulerMessage::SetLineSpeedFactor(_, _, t) => *t,
            SchedulerMessage::SetScene(_, t) => *t,
            SchedulerMessage::UploadScene(_)
            | SchedulerMessage::AddLine
            | SchedulerMessage::Shutdown => ActionTiming::Immediate,
            SchedulerMessage::TransportStart(t) | SchedulerMessage::TransportStop(t) => *t,
            SchedulerMessage::InternalDuplicateFrame { timing, .. } => *timing,
            SchedulerMessage::InternalDuplicateFrameRange { timing, .. } => *timing,
            SchedulerMessage::InternalRemoveFramesMultiLine { timing, .. } => *timing,
            SchedulerMessage::InternalInsertDuplicatedBlocks { timing, .. } => *timing,
            SchedulerMessage::SetFrameName(_, _, _, t) => *t,
            SchedulerMessage::SetScriptLanguage(_, _, _, t) => *t,
            SchedulerMessage::SetFrameRepetitions(_, _, _, t) => *t,
        };

        if timing == ActionTiming::Immediate {
            self.apply_action(msg);
        } else {
            let current_beat = self.clock.beat().floor() as u64;
            let scene_len_beats = self.scene.length() as u64;
            let _target_beat = if timing == ActionTiming::EndOfScene && scene_len_beats > 0 {
                Some(((current_beat / scene_len_beats) + 1) * scene_len_beats)
            } else {
                None
            }; // AtBeat timing doesn't need pre-calculation here

            self.deferred_actions.push(DeferredAction::new(msg, timing));
            println!(
                "Deferred action: {:?}, target: {:?}",
                self.deferred_actions.last().unwrap().action,
                self.deferred_actions.last().unwrap().timing
            ); // Debug log
        }
    }

    pub fn do_your_thing(&mut self) {
        let start_date = self.clock.micros();
        println!("[+] Starting scheduler at {start_date}");
        loop {
            // Check for shutdown request
            if self.shutdown_requested {
                break;
            }

            self.processed_scene_modification = false;
            self.clock.capture_app_state();

            // Receive incoming messages
            if let Some(timeout) = self.next_wait {
                let duration = Duration::from_micros(timeout);
                match self.message_source.recv_timeout(duration) {
                    Err(RecvTimeoutError::Disconnected) => break,
                    Err(RecvTimeoutError::Timeout) => (),
                    Ok(msg) => self.process_message(msg),
                }
            } else {
                match self.message_source.try_recv() {
                    Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => (),
                    Ok(msg) => self.process_message(msg),
                }
            }

            let current_micros = self.clock.micros();
            let current_beat = self.clock.beat_at_date(current_micros);

            // Process deferred actions
            let scene_len_beats = self.scene.length() as f64;
            let mut _applied_deferred;
            let mut indices_to_apply = Vec::new();

            for (index, deferred) in self.deferred_actions.iter().enumerate() {
                if deferred.should_apply(
                    current_beat,
                    self.playback_manager.last_beat,
                    scene_len_beats,
                ) {
                    indices_to_apply.push(index);
                }
            }

            // Step 2: Apply identified actions (if any)
            if !indices_to_apply.is_empty() {
                let actions_to_run: Vec<SchedulerMessage> = indices_to_apply
                    .iter()
                    .map(|&index| self.deferred_actions[index].action.clone())
                    .collect();

                for action in actions_to_run {
                    println!("Applying deferred action: {:?}", action); // Debug log
                    self.apply_action(action);
                }
                _applied_deferred = true;

                // Step 3: Remove applied actions (using retain with index check)
                let mut current_index = 0;
                self.deferred_actions.retain(|_| {
                    let keep = !indices_to_apply.contains(&current_index);
                    current_index += 1;
                    keep
                });
            }

            self.playback_manager.last_beat = current_beat;

            if let Some(wait_time) = self.playback_manager.update_state(
                &self.clock,
                current_beat,
                &mut self.scene,
                &mut self.executions,
                &self.update_notifier,
            ) {
                self.next_wait = Some(wait_time);
            } else if self.playback_manager.is_playing() {
                let date = self.theoretical_date();
                let scene_len = self.scene.length();
                let mut next_frame_delay = SyncTime::MAX;
                self.current_positions.clear();
                self.current_positions.reserve(self.scene.n_lines());
                let mut positions_changed = false;

                for line in self.scene.lines_iter_mut() {
                    let (frame, iter, rep, scheduled_date, track_frame_delay) =
                        calculate_frame_index(&self.clock, scene_len, line, date);
                    next_frame_delay = std::cmp::min(next_frame_delay, track_frame_delay);

                    self.current_positions.push((frame, rep));

                    let has_changed = (frame != line.current_frame)
                        || (iter != line.current_iteration)
                        || (rep != line.current_repetition);

                    if has_changed {
                        if frame != line.current_frame || iter != line.current_iteration {
                            line.frames_passed += 1;
                        }
                        positions_changed = true;
                    }

                    if frame < usize::MAX && has_changed && line.is_frame_enabled(frame) {
                        let script = Arc::clone(&line.scripts[frame]);
                        self.executions.push(ScriptExecution::execute_at(
                            script,
                            line.index,
                            scheduled_date,
                        ));
                        if frame != line.current_frame || iter != line.current_iteration {
                            line.frames_executed += 1;
                        }
                    }
                    line.current_frame = frame;
                    line.current_iteration = iter;
                    line.current_repetition = rep;
                }

                if positions_changed && !self.processed_scene_modification {
                    let frame_updates: Vec<(usize, usize, usize)> = self
                        .current_positions
                        .iter()
                        .enumerate()
                        .map(|(i, &(f, r))| (i, f, r))
                        .collect();
                    let _ = self
                        .update_notifier
                        .send(SchedulerNotification::FramePositionChanged(frame_updates));
                }

                // Clone global vars to detect changes
                let global_vars_before = self.global_vars.clone();

                let next_exec_delay = ExecutionManager::process_executions(
                    &self.clock,
                    &mut self.scene,
                    &mut self.executions,
                    &mut self.global_vars,
                    self.devices.clone(),
                    &self.world_iface,
                    SCHEDULED_DRIFT,
                    &mut self.audio_engine_events,
                );

                // Check if global variables changed and send notification
                if self.global_vars != global_vars_before {
                    // Filter to only send single-letter global variables
                    let single_letter_vars: std::collections::HashMap<String, VariableValue> = self
                        .global_vars
                        .iter()
                        .filter(|(k, _)| k.len() == 1)
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();

                    let _ =
                        self.update_notifier
                            .send(SchedulerNotification::GlobalVariablesChanged(
                                single_letter_vars,
                            ));
                }

                let next_delay = std::cmp::min(next_exec_delay, next_frame_delay);
                if next_delay > 0 {
                    self.next_wait = Some(next_delay);
                } else {
                    self.next_wait = None;
                }
            }
        }
        println!("[-] Exiting scheduler...");
        for (_, (_, device)) in self.devices.output_connections.lock().unwrap().iter() {
            device.flush();
        }
    }

    #[inline]
    pub fn theoretical_date(&self) -> SyncTime {
        self.clock.micros() + SCHEDULED_DRIFT
    }

    #[inline]
    pub fn kill_all(&mut self) {
        self.executions.clear();
    }
}
