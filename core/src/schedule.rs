use crate::{
    clock::{Clock, ClockServer, SyncTime, NEVER},
    device_map::DeviceMap,
    lang::{
        evaluation_context::PartialContext, variable::VariableStore, LanguageCenter
    },
    log_println,
    protocol::TimedMessage,
    scene::Scene,
    schedule::{
        playback::PlaybackManager,
        scheduler_actions::ActionProcessor,
    }
};

use crossbeam_channel::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::{
    sync::Arc, thread::JoinHandle, time::Duration, usize
};
use thread_priority::ThreadBuilder;

pub mod playback;

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
    languages: Arc<LanguageCenter>,
    clock: Clock,
    feedback: Sender<SchedulerMessage>,
    message_source: Receiver<SchedulerMessage>,
    update_notifier: Sender<SovaNotification>,

    next_wait: Option<SyncTime>,
    deferred_actions: Vec<SchedulerMessage>,
    playback_manager: PlaybackManager,
    shutdown_requested: bool,

    scene_structure: Vec<Vec<f64>>,
}

impl Scheduler {
    pub fn create(
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        languages: Arc<LanguageCenter>,
        world_iface: Sender<TimedMessage>,
    ) -> (
        JoinHandle<()>,
        Sender<SchedulerMessage>,
        Receiver<SovaNotification>,
    ) {
        let (tx, rx) = crossbeam_channel::unbounded();
        let (p_tx, p_rx) = crossbeam_channel::unbounded();

        let clock = Clock::from(clock_server).with_drift(SCHEDULED_DRIFT);
        let feedback = tx.clone();

        let handle = ThreadBuilder::default()
            .name("Sova-scheduler")
            .spawn(move |_| {
                let mut sched = Scheduler::new(
                    clock,
                    devices,
                    languages,
                    world_iface,
                    feedback,
                    rx,
                    p_tx,
                );
                sched.do_your_thing();
            })
            .expect("Unable to start Scheduler");
        (handle, tx, p_rx)
    }

    pub fn new(
        clock: Clock,
        devices: Arc<DeviceMap>,
        languages: Arc<LanguageCenter>,
        world_iface: Sender<TimedMessage>,
        feedback: Sender<SchedulerMessage>,
        receiver: Receiver<SchedulerMessage>,
        update_notifier: Sender<SovaNotification>,
    ) -> Scheduler {
        Scheduler {
            world_iface,
            scene: Default::default(),
            global_vars: VariableStore::new(),
            devices,
            languages,
            clock,
            feedback,
            message_source: receiver,
            update_notifier,
            next_wait: None,
            deferred_actions: Vec::new(),
            playback_manager: PlaybackManager::default(),
            shutdown_requested: false,
            scene_structure: Vec::new()
        }
    }

    pub fn change_scene(&mut self, mut scene: Scene) {
        scene.make_consistent();
        scene.reset();
        self.scene = scene;

        self.scene_structure = self.scene.structure();
        self.languages.transcoder.process_scene(&self.scene, self.feedback.clone());

        // Notify clients about the completely new scene state
        let _ = self
            .update_notifier
            .send(SovaNotification::UpdatedScene(self.scene.clone()));
    }

    fn apply_action(&mut self, action: SchedulerMessage) {
        match action {
            SchedulerMessage::TransportStart(_) => {
                self.process_transport_start();
            }
            SchedulerMessage::TransportStop(_) => {
                self.process_transport_stop();
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
            SchedulerMessage::DeviceMessage(id, msg, _) => {
                let device = self.devices.get_out_device_at_slot(id);
                if let Some(device) = device {
                    let _ = self.world_iface.send(
                        msg.with_device(device).timed(self.clock.micros())
                    );
                }
            }
            SchedulerMessage::Shutdown => {
                log_println!("[-] Scheduler received shutdown signal");
                self.shutdown_requested = true;
            }
            _ => {
                ActionProcessor::process_scene_modifications(
                    action,
                    &mut self.scene,
                    &self.update_notifier,
                    &self.languages.transcoder,
                    &self.feedback
                );
                self.scene_structure = self.scene.structure();
            }
        }
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
                &self.scene
            )
        }).collect();
        for action in to_apply {
            log_println!("Applying deferred action: {:?}", action); // Debug log
            self.apply_action(action);
        }
    }

    pub fn process_executions(&mut self, date: SyncTime) -> SyncTime {
        let mut partial = PartialContext::default();
        partial.global_vars = Some(&mut self.global_vars);
        partial.clock = Some(&self.clock);
        partial.device_map = Some(&self.devices);
        partial.structure = Some(&self.scene_structure);
        let (events, wait) = self.scene.update_executions(date, partial);
        for event in events {
            for msg in self.devices.map_event(event, date, &self.clock) {
                let _ = self.world_iface.send(msg);
            }
        }
        wait.unwrap_or(NEVER)
    }

    pub fn do_your_thing(&mut self) {
        let start_date = self.clock.micros();
        log_println!("[+] Starting scheduler at {start_date}");
        loop {
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
            let mut positions_changed = false;

            for line in self.scene.lines.iter_mut() {
                positions_changed |= line.step(&self.clock, date, &self.languages.interpreters);
                next_frame_delay = std::cmp::min(next_frame_delay, line.remaining_before_next_update(date));
            }

            if positions_changed {
                let frame_updates: Vec<(usize, usize)> = self.scene.positions().collect();
                let _ = self
                    .update_notifier
                    .send(SovaNotification::FramePositionChanged(frame_updates));
            }

            // Clone global vars to detect changes
            let one_letters_before : VariableStore = self.global_vars.one_letter_vars().collect();

            let next_exec_delay = self.process_executions(date);

            // Check if global variables changed and send notification
            let one_letter_vars : VariableStore = self.global_vars.one_letter_vars().collect();
            if one_letter_vars != one_letters_before {
                let _ = self.update_notifier
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

    pub fn process_transport_start(&mut self) {
        let current = self.clock.micros();
        let quantum = self.clock.quantum();
        let quantum = self.clock.beats_to_micros(quantum);
        // High-precision quantum synchronization for transport start requests
        let start_date = current + quantum - (current % quantum); 

        let start_beat = self.clock.micros_to_beats(start_date);
        log_println!(
            "[SCHEDULER] Requesting transport start via Link at beat {} ({} micros)",
            start_beat, start_date
        );

        self.clock.session_state.set_is_playing(true, start_date);
        self.clock.commit_app_state();
        let _ = self.update_notifier.send(SovaNotification::TransportStarted);
    }

    pub fn process_transport_stop(&mut self) {
        let now_micros = self.clock.micros();
        log_println!("[SCHEDULER] Requesting transport stop via Link now");

        self.clock.session_state.set_is_playing(false, now_micros);
        self.clock.commit_app_state();

        self.scene.kill_executions();
        let _ = self.update_notifier.send(SovaNotification::TransportStopped);
    }

}
