use crate::{
    clock::{Clock, ClockServer, NEVER, SyncTime},
    device_map::DeviceMap,
    vm::{LanguageCenter, PartialContext, variable::VariableStore},
    log_println,
    protocol::TimedMessage,
    scene::Scene,
    schedule::{playback::PlaybackManager, scheduler_actions::ActionProcessor},
    world::ACTIVE_WAITING_SWITCH_MICROS,
};

use crossbeam_channel::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::{cmp::min, sync::Arc, thread::JoinHandle, time::Duration, usize};
use thread_priority::{ThreadBuilder, ThreadPriority};

pub mod playback;

mod action_timing;
mod message;
mod notification;
mod scheduler_actions;

pub use action_timing::ActionTiming;
pub use message::SchedulerMessage;
pub use notification::SovaNotification;

pub const SCHEDULED_DRIFT: SyncTime = 30_000;
pub const SCHEDULER_ACTIVE_WAITING_SWITCH: SyncTime = 100;

pub struct Scheduler {
    pub scene: Scene,

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
            .priority(ThreadPriority::Max)
            .spawn(move |_| {
                // match audio_thread_priority::promote_current_thread_to_real_time(512, 44100) {
                //     Ok(_) => log_eprintln!("[+] Scheduler: real-time priority set"),
                //     Err(e) => log_eprintln!("[!] Scheduler: failed to set RT priority: {:?}", e),
                // }
                let mut sched =
                    Scheduler::new(clock, devices, languages, world_iface, feedback, rx, p_tx);
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
            scene_structure: Vec::new(),
        }
    }

    pub fn change_scene(&mut self, mut scene: Scene) {
        scene.make_consistent();
        scene.reset();
        self.scene = scene;

        self.scene_structure = self.scene.structure();
        self.languages
            .process_scene(&self.scene, self.feedback.clone());

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
            SchedulerMessage::SetQuantum(quantum, _) => {
                self.clock.set_quantum(quantum);
                let _ = self
                    .update_notifier
                    .send(SovaNotification::QuantumChanged(quantum));
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
                    let _ = self
                        .world_iface
                        .send(msg.with_device(device).timed(self.clock.micros()));
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
                    &self.languages,
                    &self.feedback,
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
            self.deferred_actions.push(msg);
        }
    }

    fn wait_for_message(&mut self) -> bool {
        if let Some(timeout) = self.next_wait {
            let wait = timeout.saturating_sub(ACTIVE_WAITING_SWITCH_MICROS);
            let duration = Duration::from_micros(wait);
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

    pub fn process_deferred(&mut self, previous_date: SyncTime, date: SyncTime) -> SyncTime {
        let previous_beat = self.clock.beat_at_date(previous_date);
        let beat = self.clock.beat_at_date(date);
        let quantum = self.clock.quantum();
        let to_apply: Vec<SchedulerMessage> = self
            .deferred_actions
            .extract_if(.., |action| {
                action
                    .timing()
                    .should_apply(quantum, previous_beat, beat, &self.scene)
            })
            .collect();
        for action in to_apply {
            self.apply_action(action);
        }
        self.deferred_actions
            .iter()
            .map(|a| a.timing().remaining(date, &self.clock, &self.scene))
            .min()
            .unwrap_or(NEVER)
    }

    pub fn process_executions(&mut self, date: SyncTime) -> SyncTime {
        let mut partial = PartialContext::default();
        partial.logic_date = date;
        partial.clock = Some(&self.clock);
        partial.device_map = Some(&self.devices);
        partial.structure = Some(&self.scene_structure);
        let (events, wait) = self.scene.update_executions(partial);
        for event in events {
            for msg in self.devices.map_event(event, date, &self.clock) {
                let _ = self.world_iface.send(msg);
            }
        }
        wait
    }

    pub fn active_wait(&self, date: &mut SyncTime, target: SyncTime) {
        if target.saturating_sub(*date) > ACTIVE_WAITING_SWITCH_MICROS {
            return;
        }
        while *date < target {
            *date = self.clock.micros();
        }
    }

    pub fn do_your_thing(&mut self) {
        let start_date = self.clock.micros();
        let mut previous_date = start_date;
        log_println!("[+] Starting scheduler at {start_date}");
        loop {
            self.clock.capture_app_state();

            // Check for shutdown request and
            // Receive incoming messages
            if self.shutdown_requested || !self.wait_for_message() {
                break;
            }

            let mut date = self.clock.micros();

            if let Some(wait) = self.next_wait {
                self.active_wait(&mut date, previous_date.saturating_add(wait));
            }

            // Process deferred actions
            self.next_wait = Some(self.process_deferred(previous_date, date));

            previous_date = date;

            if let Some(wait_time) = self
                .playback_manager
                .update_state(&self.clock, &mut self.scene)
            {
                self.next_wait = Some(min(wait_time, self.next_wait.unwrap_or(NEVER)));
            }
            if self.playback_manager.state_has_changed() {
                let _ = self
                    .update_notifier
                    .send(SovaNotification::PlaybackStateChanged(
                        self.playback_manager.state(),
                    ));
            }

            if !self.playback_manager.state().is_playing() {
                continue;
            }

            let mut next_frame_delay = NEVER;
            let mut positions_changed = false;

            for line in self.scene.lines.iter_mut() {
                positions_changed |= line.step(&self.clock, date, &self.languages.interpreters);
                next_frame_delay = std::cmp::min(
                    next_frame_delay,
                    line.before_next_trigger(&self.clock, date),
                );
            }

            if positions_changed {
                let frame_updates: Vec<(usize, usize)> = self.scene.positions().collect();
                let _ = self
                    .update_notifier
                    .send(SovaNotification::FramePositionChanged(frame_updates));
            }

            // Clone global vars to detect changes
            let one_letters_before: VariableStore = self.scene.vars.one_letter_vars().collect();

            let next_exec_delay = self.process_executions(date);

            // Check if global variables changed and send notification
            let one_letter_vars: VariableStore = self.scene.vars.one_letter_vars().collect();
            if one_letter_vars != one_letters_before {
                let _ = self
                    .update_notifier
                    .send(SovaNotification::GlobalVariablesChanged(
                        one_letter_vars.into(),
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
        for (_, device) in self.devices.output_connections.lock().unwrap().iter() {
            device.flush();
        }
    }

    pub fn process_transport_start(&mut self) {
        let start_date = self.clock.next_phase_reset_date();

        let start_beat = self.clock.beat_at_date(start_date);
        log_println!(
            "[SCHEDULER] Requesting transport start via Link at beat {} ({} micros)",
            start_beat,
            start_date
        );

        self.clock.session_state.set_is_playing(true, start_date as i64);
        self.clock.commit_app_state();
    }

    pub fn process_transport_stop(&mut self) {
        let now_micros = self.clock.micros();
        log_println!("[SCHEDULER] Requesting transport stop via Link now");

        self.clock.session_state.set_is_playing(false, now_micros as i64);
        self.clock.commit_app_state();

        self.scene.kill_executions();
    }
}
