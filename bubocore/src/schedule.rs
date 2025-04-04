// Doit faire traduction (Event, TimeSpan) en (ProtocolMessage, SyncTime)

use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError},
        Arc,
    },
    thread::JoinHandle,
    time::Duration, usize,
};

use serde::{Deserialize, Serialize};
use thread_priority::ThreadBuilder;

use crate::{
    clock::{Clock, ClockServer, SyncTime},
    device_map::DeviceMap,
    lang::variable::VariableStore,
    pattern::{
        script::{Script, ScriptExecution},
        Pattern, Sequence,
    },
    protocol::TimedMessage,
    server::Snapshot,
    shared_types::GridSelection,
};

pub const SCHEDULED_DRIFT: SyncTime = 30_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulerMessage {
    UploadPattern(Pattern),
    EnableSteps(usize, Vec<usize>),
    DisableSteps(usize, Vec<usize>),
    UploadScript(usize, usize, Script),
    UpdateSequenceSteps(usize, Vec<f64>),
    AddSequence,
    RemoveSequence(usize),
    SetSequence(usize, Sequence),
    SetSequenceStartStep(usize, Option<usize>),
    SetSequenceEndStep(usize, Option<usize>),
    SetPattern(Pattern),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum SchedulerNotification {
    #[default]
    Nothing,
    UpdatedPattern(Pattern),
    UpdatedSequence(usize, Sequence),
    EnableSteps(usize, Vec<usize>),
    DisableSteps(usize, Vec<usize>),
    UploadedScript(usize, usize, Script),
    UpdatedSequenceSteps(usize, Vec<f64>),
    AddedSequence(Sequence),
    RemovedSequence(usize),
    Log(TimedMessage),
    TempoChanged(f64),
    ClientListChanged(Vec<String>),
    ChatReceived(String, String),
    StepPositionChanged(Vec<usize>),
    /// Indicates a peer's grid selection has changed.
    PeerGridSelectionChanged(String, GridSelection), // (username, selection)
    /// Indicates a peer started editing a step.
    PeerStartedEditingStep(String, usize, usize), // (username, sequence_idx, step_idx)
    /// Indicates a peer stopped editing a step.
    PeerStoppedEditingStep(String, usize, usize), // (username, sequence_idx, step_idx)
}

pub struct Scheduler {
    pub pattern: Pattern,
    pub global_vars: VariableStore,

    pub executions: Vec<ScriptExecution>,

    world_iface: Sender<TimedMessage>,
    devices: Arc<DeviceMap>,
    clock: Clock,

    message_source: Receiver<SchedulerMessage>,

    update_notifier: Sender<SchedulerNotification>,

    next_wait: Option<SyncTime>,
    processed_pattern_modification: bool,
}

impl Scheduler {
    pub fn create(
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        world_iface: Sender<TimedMessage>,
    ) -> (JoinHandle<()>, Sender<SchedulerMessage>, Receiver<SchedulerNotification>) {
        let (tx, rx) = mpsc::channel();
        let (p_tx, p_rx) = mpsc::channel();

        let handle = ThreadBuilder::default()
            .name("BuboCore-scheduler")
            .spawn(move |_| {
                let mut sched = Scheduler::new(clock_server.into(), devices, world_iface, rx, p_tx);
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
    ) -> Scheduler {
        Scheduler {
            world_iface,
            pattern: Default::default(),
            global_vars: HashMap::new(),
            executions: Vec::new(),
            devices,
            clock,
            message_source: receiver,
            update_notifier,
            next_wait: None,
            processed_pattern_modification: false,
        }
    }

    fn step_index(clock : &Clock, sequence : &Sequence, date: SyncTime) -> (usize, usize, SyncTime, SyncTime) {
        // Use the effective range defined by start_step and end_step
        let effective_start_step = sequence.get_effective_start_step();
        let effective_num_steps = sequence.get_effective_num_steps();

        if effective_num_steps == 0 {
            return (usize::MAX, usize::MAX, SyncTime::MAX, SyncTime::MAX); // No steps to play
        }

        let effective_beats_len : f64 = sequence.effective_beats_len();

        if effective_beats_len <= 0.0 {
             return (usize::MAX, usize::MAX, SyncTime::MAX, SyncTime::MAX); // Avoid division by zero or negative length
        }

        let beat = clock.beat_at_date(date);
        if beat < 0.0 {
            return (usize::MAX, usize::MAX, SyncTime::MAX, SyncTime::MAX);
        }

        // Calculate beat within the effective loop length
        let beat_in_loop = beat % (effective_beats_len / sequence.speed_factor);
        let loop_iteration = beat.div_euclid(effective_beats_len / sequence.speed_factor) as usize;

        // Calculate the beat offset corresponding to the start of the effective range
        // This assumes steps before start_step exist and have lengths.
        let start_offset_beats: f64 = sequence.steps[0..effective_start_step].iter().sum();
        let sequence_start_beat_in_loop = beat - beat_in_loop; // Beat corresponding to the start of the current loop iteration

        let mut current_beat_in_effective_range = beat_in_loop;
        let mut current_absolute_step_index = effective_start_step; // Start searching from the effective start

        // Iterate through the steps *within the effective range*
        for step_idx_in_range in 0..effective_num_steps {
            let absolute_step_index = effective_start_step + step_idx_in_range;
            let step_len_beats = sequence.step_len(absolute_step_index) / sequence.speed_factor; // Use absolute index to get length

            if current_beat_in_effective_range <= step_len_beats {
                // Found the current step within the effective range
                // Calculate the absolute start beat of this step within the current loop iteration
                let step_start_beat_absolute = sequence_start_beat_in_loop
                                              + (sequence.steps[effective_start_step..absolute_step_index].iter().sum::<f64>() / sequence.speed_factor);

                let start_date = clock.date_at_beat(step_start_beat_absolute);
                let remaining_micros = clock.beats_to_micros(step_len_beats - current_beat_in_effective_range);

                return (absolute_step_index, loop_iteration, start_date, remaining_micros);
            }

            // Move to the next step in the effective range
            current_beat_in_effective_range -= step_len_beats;
            current_absolute_step_index += 1; // This is just for tracking, loop uses step_idx_in_range
        }

        // Should theoretically not be reached if effective_beats_len > 0
        eprintln!("[!] Scheduler::step_index fell through loop unexpectedly. Beat: {}, Loop Beat: {}, Effective Length: {}", beat, beat_in_loop, effective_beats_len);
        return (usize::MAX, usize::MAX, SyncTime::MAX, SyncTime::MAX);
    }

    pub fn change_pattern(&mut self, mut pattern: Pattern) {
        let date = self.theoretical_date();
        pattern.make_consistent();
        for sequence in pattern.sequences_iter_mut() {
            let (step, iter, _, _) = Self::step_index(&self.clock, sequence, date);
            sequence.current_step = step;
            sequence.current_iteration = iter;
            sequence.first_iteration_index = iter;
        }
        self.pattern = pattern;
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedPattern(self.pattern.clone()));
    } 

    pub fn process_message(&mut self, msg: SchedulerMessage) {
        // Flag is reset at start of do_your_thing loop
        match msg {
            SchedulerMessage::UploadPattern(pattern) => {
                self.change_pattern(pattern);
                self.processed_pattern_modification = true; // Keep setting flag here
            }
            SchedulerMessage::EnableSteps(sequence, steps) => {
                self.enable_steps(sequence, &steps);
                self.processed_pattern_modification = true;
            }
            SchedulerMessage::DisableSteps(sequence, steps) => {
                self.disable_steps(sequence, &steps);
                self.processed_pattern_modification = true;
            }
            SchedulerMessage::UploadScript(sequence, step, script) => {
                self.upload_script(sequence, step, script);
                self.processed_pattern_modification = true;
            }
            SchedulerMessage::UpdateSequenceSteps(sequence, vec) => {
                self.pattern.mut_sequence(sequence).set_steps(vec);
                let _ = self.update_notifier.send(SchedulerNotification::UpdatedPattern(self.pattern.clone()));
                self.processed_pattern_modification = true;
            }
            SchedulerMessage::AddSequence => {
                let new_sequence = Sequence::new(vec![1.0]);
                self.add_sequence(new_sequence);
                self.processed_pattern_modification = true;
            },
            SchedulerMessage::RemoveSequence(index) => {
                self.remove_sequence(index);
                self.processed_pattern_modification = true;
            }
            SchedulerMessage::SetSequence(index, sequence) => {
                self.set_sequence(index, sequence);
                self.processed_pattern_modification = true;
            }
            SchedulerMessage::SetSequenceStartStep(sequence_index, start_step) => {
                 if let Some(sequence) = self.pattern.sequences.get_mut(sequence_index) {
                     sequence.start_step = start_step;
                     sequence.make_consistent();
                     let _ = self.update_notifier.send(SchedulerNotification::UpdatedPattern(self.pattern.clone()));
                     self.processed_pattern_modification = true;
                 } else {
                     eprintln!("[!] Scheduler: SetSequenceStartStep received for invalid sequence index {}", sequence_index);
                 }
            }
            SchedulerMessage::SetSequenceEndStep(sequence_index, end_step) => {
                 if let Some(sequence) = self.pattern.sequences.get_mut(sequence_index) {
                     sequence.end_step = end_step;
                     sequence.make_consistent();
                     let _ = self.update_notifier.send(SchedulerNotification::UpdatedPattern(self.pattern.clone()));
                     self.processed_pattern_modification = true;
                 } else {
                     eprintln!("[!] Scheduler: SetSequenceEndStep received for invalid sequence index {}", sequence_index);
                 }
            }
            SchedulerMessage::SetPattern(pattern) => {
                self.change_pattern(pattern);
                self.processed_pattern_modification = true;
            }
        };
    }

    pub fn set_sequence(&mut self, index: usize, sequence: Sequence) {
        self.pattern.set_sequence(index, sequence);
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedPattern(self.pattern.clone()));
    }

    pub fn upload_script(&mut self, sequence: usize, step: usize, script: Script) {
        self.pattern.mut_sequence(sequence).set_script(step, script);
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedPattern(self.pattern.clone()));
    }

    pub fn remove_sequence(&mut self, index: usize) {
        self.pattern.remove_sequence(index);
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedPattern(self.pattern.clone()));
    }

    pub fn add_sequence(&mut self, sequence: Sequence) {
        self.pattern.add_sequence(sequence);
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedPattern(self.pattern.clone()));
    }

    pub fn disable_step(&mut self, sequence: usize, step: usize) {
        self.pattern.mut_sequence(sequence).disable_step(step);
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedPattern(self.pattern.clone()));
    }
    
    pub fn enable_step(&mut self, sequence: usize, step: usize) {
        self.pattern.mut_sequence(sequence).enable_step(step);
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedPattern(self.pattern.clone()));
    }

    pub fn disable_steps(&mut self, sequence_idx: usize, steps: &[usize]) {
        if let Some(sequence) = self.pattern.sequences.get_mut(sequence_idx) {
            sequence.disable_steps(steps);
            let _ = self.update_notifier.send(SchedulerNotification::UpdatedPattern(self.pattern.clone()));
        } else {
            eprintln!("[!] Scheduler: DisableSteps received for invalid sequence index {}", sequence_idx);
        }
    }

    pub fn enable_steps(&mut self, sequence_idx: usize, steps: &[usize]) {
        if let Some(sequence) = self.pattern.sequences.get_mut(sequence_idx) {
            sequence.enable_steps(steps);
            let _ = self.update_notifier.send(SchedulerNotification::UpdatedPattern(self.pattern.clone()));
        } else {
            eprintln!("[!] Scheduler: EnableSteps received for invalid sequence index {}", sequence_idx);
        }
    }

    pub fn do_your_thing(&mut self) {
        let start_date = self.clock.micros();
        println!("[+] Starting scheduler at {start_date}");
        loop {
            self.processed_pattern_modification = false;
            self.clock.capture_app_state();

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

            let date = self.theoretical_date();

            let mut next_step_delay = SyncTime::MAX;
            let mut current_positions = Vec::with_capacity(self.pattern.n_sequences());
            let mut positions_changed = false;

            for sequence in self.pattern.sequences_iter_mut() {
                let (step, iter, scheduled_date, track_step_delay) = Self::step_index(&self.clock, sequence, date);
                next_step_delay = std::cmp::min(next_step_delay, track_step_delay);

                current_positions.push(step);

                let has_changed_step = (step != sequence.current_step) || (iter != sequence.current_iteration);

                if has_changed_step {
                    sequence.steps_passed += 1;
                    positions_changed = true;
                }

                if step < usize::MAX && has_changed_step && sequence.is_step_enabled(step) {
                    let script = Arc::clone(&sequence.scripts[step]);
                    self.executions.push(ScriptExecution::execute_at(script, sequence.index, scheduled_date));
                    sequence.current_step = step;
                    sequence.steps_executed += 1;
                }
                sequence.current_iteration = iter;
            }

            if positions_changed && !self.processed_pattern_modification { 
                let _ = self.update_notifier.send(SchedulerNotification::StepPositionChanged(current_positions));
            }

            let next_exec_delay = self.execution_loop();

            let next_delay = std::cmp::min(next_exec_delay, next_step_delay);
            if next_delay > 0 {
                self.next_wait = Some(next_delay);
            } else {
                self.next_wait = None;
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

    fn execution_loop(&mut self) -> SyncTime {
        if self.pattern.n_sequences() == 0 {
            return SyncTime::MAX;
        }

        let scheduled_date = self.theoretical_date();
        // TODO: Read MIDI input controller values
        let mut next_timeout = SyncTime::MAX;

        self.executions.retain_mut(|exec| {
            if !exec.is_ready(scheduled_date) {
                next_timeout = std::cmp::min(next_timeout, exec.remaining_before(scheduled_date));
                return true;
            }
            next_timeout = 0;
            if let Some((event, date)) = exec.execute_next(&self.clock, &mut self.global_vars, self.pattern.mut_sequences()) {
                let messages = self.devices.map_event(event, date);
                for message in messages {
                    //let _ = self.update_notifier.send(SchedulerNotification::Log(message.clone()));
                    let _ = self.world_iface.send(message);
                }
            }
            !exec.has_terminated()
        });
        next_timeout
    }

}
