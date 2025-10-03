use crate::{
    clock::{Clock, ClockServer, SyncTime},
    device_map::DeviceMap,
    lang::{
        event::ConcreteEvent, interpreter::InterpreterDirectory, Transcoder, variable::VariableStore
    },
    log_println,
    protocol::message::TimedMessage,
    scene::Scene,
    schedule::{
        execution::ExecutionManager,
        playback::PlaybackManager,
        scheduler_actions::ActionProcessor,
    },
};

use crossbeam_channel::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::{
    sync::{atomic::AtomicBool, Arc}, thread::JoinHandle, time::Duration, usize
};
use thread_priority::ThreadBuilder;

pub mod playback;


mod execution;
mod scheduler_actions;
mod action_timing;
mod message;
mod notification;

pub use action_timing::ActionTiming;
pub use message::SchedulerMessage;
pub use notification::SovaNotification;

pub const SCHEDULED_DRIFT: SyncTime = 1_000;

pub struct Scheduler {
    pub scene: Scene,
    pub global_vars: VariableStore,

    world_iface: Sender<TimedMessage>,
    devices: Arc<DeviceMap>,
    interpreters: Arc<InterpreterDirectory>,
    transcoder: Arc<Transcoder>,
    clock: Clock,
    message_source: Receiver<SchedulerMessage>,
    update_notifier: Sender<SovaNotification>,

    next_wait: Option<SyncTime>,
    processed_scene_modification: bool,
    deferred_actions: Vec<SchedulerMessage>,
    playback_manager: PlaybackManager,
    shutdown_requested: bool,

    current_positions: Vec<(usize, usize)>,
    audio_engine_events: Vec<(ConcreteEvent, SyncTime)>,
}

impl Scheduler {
    pub fn create(
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        interpreters: Arc<InterpreterDirectory>,
        transcoder: Arc<Transcoder>,
        world_iface: Sender<TimedMessage>,
        shared_atomic_is_playing: Arc<AtomicBool>,
    ) -> (
        JoinHandle<()>,
        Sender<SchedulerMessage>,
        Receiver<SovaNotification>,
    ) {
        let (tx, rx) = crossbeam_channel::unbounded();
        let (p_tx, p_rx) = crossbeam_channel::unbounded();

        let shared_atomic_clone = shared_atomic_is_playing.clone(); // Clone for the thread

        let clock = Clock::from(clock_server).with_drift(SCHEDULED_DRIFT);

        let handle = ThreadBuilder::default()
            .name("Sova-scheduler")
            .spawn(move |_| {
                let mut sched = Scheduler::new(
                    clock,
                    devices,
                    interpreters,
                    transcoder,
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
        interpreters: Arc<InterpreterDirectory>,
        transcoder: Arc<Transcoder>,
        world_iface: Sender<TimedMessage>,
        receiver: Receiver<SchedulerMessage>,
        update_notifier: Sender<SovaNotification>,
        shared_atomic_is_playing: Arc<AtomicBool>,
    ) -> Scheduler {
        Scheduler {
            world_iface,
            scene: Default::default(),
            global_vars: VariableStore::new(),
            devices,
            interpreters,
            transcoder,
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
        let date = self.clock.micros();
        scene.make_consistent();

        self.transcoder.compile_scene(&mut scene);

        for line in scene.lines.iter_mut() {
            let (frame, iter, _rep, _, _) = line.calculate_frame_index(&self.clock, date);
            line.current_frame = frame;
            line.current_iteration = iter;
            line.current_repetition = 0;
        }
        
        for line in scene.lines.iter_mut() {
            let (frame_id, _, _, scheduled_date, _) = line.calculate_frame_index(&self.clock, date);
            if frame_id == usize::MAX {
                continue;
            }
            line.trigger(scheduled_date, &self.interpreters);
        }

        self.scene = scene;
        // Notify clients about the completely new scene state
        let _ = self
            .update_notifier
            .send(SovaNotification::UpdatedScene(self.scene.clone()));
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
                    .send(SovaNotification::TempoChanged(tempo));
            }
            SchedulerMessage::SetScene(scene, _) => {
                self.change_scene(scene.clone());
                let _ = self
                    .update_notifier
                    .send(SovaNotification::UpdatedScene(scene.clone()));
            }
            SchedulerMessage::DeviceMessage(msg, _) => {
                let _ = self.world_iface.send(msg);
            }
            SchedulerMessage::Shutdown => {
                log_println!("[-] Scheduler received shutdown signal");
                self.shutdown_requested = true;
                return;
            }
            _ => {
                ActionProcessor::process_scene_modifications(
                    action,
                    &mut self.scene,
                    &self.update_notifier,
                    &self.transcoder,
                );
            }
        }
        self.processed_scene_modification = true;
    }

    pub fn process_message(&mut self, msg: SchedulerMessage) {
        let timing = msg.timing();

        if timing == ActionTiming::Immediate {
            self.apply_action(msg);
        } else {
            log_println!(
                "Deferred action: {:?}, target: {:?}",
                msg,
                msg.timing()
            ); // Debug log
            self.deferred_actions.push(msg);
        }
    }

    fn wait_for_message(&mut self) -> bool {
        if let Some(timeout) = self.next_wait {
            let duration = Duration::from_micros(timeout);
            match self.message_source.recv_timeout(duration) {
                Err(RecvTimeoutError::Disconnected) => false,
                Err(RecvTimeoutError::Timeout) => true,
                Ok(msg) => {
                    self.process_message(msg);
                    true
                }
            }
        } else {
            match self.message_source.try_recv() {
                Err(TryRecvError::Disconnected) => false,
                Err(TryRecvError::Empty) => true,
                Ok(msg) => {
                    self.process_message(msg);
                    true
                }
            }
        }
    }

    pub fn process_deferred(&mut self, beat: f64) {
        let to_apply : Vec<SchedulerMessage> = self.deferred_actions.extract_if(.., |action| {
            action.should_apply(
                beat, 
                self.playback_manager.last_beat, 
                &self.scene.lines
            )
        }).collect();
        for action in to_apply {
            log_println!("Applying deferred action: {:?}", action); // Debug log
            self.apply_action(action);
        }
    }

    pub fn do_your_thing(&mut self) {
        let start_date = self.clock.micros();
        log_println!("[+] Starting scheduler at {start_date}");
        loop {
            self.processed_scene_modification = false;
            self.clock.capture_app_state();

            // Check for shutdown request and
            // Receive incoming messages
            if self.shutdown_requested || !self.wait_for_message() {
                break;
            }

            let current_beat = self.clock.beat(); // self.clock.beat_at_date(current_micros);

            // Process deferred actions
            self.process_deferred(current_beat);

            self.playback_manager.last_beat = current_beat;

            if let Some(wait_time) = self.playback_manager.update_state(
                &self.clock,
                &self.interpreters,
                &mut self.scene,
                &self.update_notifier,
            ) {
                self.next_wait = Some(wait_time);
            }

            if !self.playback_manager.is_playing() {
                continue;
            }

            let date = self.clock.micros();
            let mut next_frame_delay = SyncTime::MAX;
            self.current_positions.clear();
            self.current_positions.reserve(self.scene.n_lines());
            let mut positions_changed = false;

            for line in self.scene.lines.iter_mut() {
                let (frame, iter, rep, scheduled_date, track_frame_delay) =
                    line.calculate_frame_index(&self.clock, date);
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

                if frame < usize::MAX && has_changed {
                    line.current_frame = frame;
                    line.trigger(scheduled_date, &self.interpreters);
                    if frame != line.current_frame || iter != line.current_iteration {
                        line.frames_executed += 1;
                    }
                }
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
                    .send(SovaNotification::FramePositionChanged(frame_updates));
            }

            // Clone global vars to detect changes
            let one_letters_before : VariableStore = self.global_vars.one_letter_vars().collect();

            let next_exec_delay = ExecutionManager::process_executions(
                &self.clock,
                &mut self.scene,
                &mut self.global_vars,
                self.devices.clone(),
                &self.world_iface,
                &mut self.audio_engine_events,
                date,
            );

            // Check if global variables changed and send notification
            let one_letter_vars : VariableStore = self.global_vars.one_letter_vars().collect();
            if one_letter_vars != one_letters_before {
                let _ =
                    self.update_notifier
                        .send(SovaNotification::GlobalVariablesChanged(
                            one_letter_vars.into()
                        ));
            }

            let next_delay = std::cmp::min(next_exec_delay, next_frame_delay);
            if next_delay > 0 {
                self.next_wait = Some(next_delay);
            } else {
                self.next_wait = None;
            }
        }
        log_println!("[-] Exiting scheduler...");
        for (_, (_, device)) in self.devices.output_connections.lock().unwrap().iter() {
            device.flush();
        }
    }

    #[inline]
    pub fn kill_all(&mut self) {
        self.scene.kill_executions();
    }

}
