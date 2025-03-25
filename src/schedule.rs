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

use thread_priority::ThreadBuilder;

use crate::{
    clock::{Clock, ClockServer, SyncTime},
    device_map::DeviceMap,
    lang::variable::VariableStore,
    pattern::{
        script::ScriptExecution,
        Pattern, Sequence,
    },
    protocol::TimedMessage,
};

pub const SCHEDULED_DRIFT: SyncTime = 30_000;

pub enum SchedulerMessage {
    UploadPattern(Pattern),
    ToggleStep(usize, usize)
}

pub struct Scheduler {
    pub pattern: Pattern,
    pub global_vars: VariableStore,

    pub executions: Vec<ScriptExecution>,

    world_iface: Sender<TimedMessage>,
    devices: Arc<DeviceMap>,
    clock: Clock,

    message_source: Receiver<SchedulerMessage>,

    next_wait: Option<SyncTime>,
}

impl Scheduler {
    pub fn create(
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        world_iface: Sender<TimedMessage>,
    ) -> (JoinHandle<()>, Sender<SchedulerMessage>) {
        let (tx, rx) = mpsc::channel();
        let handle = ThreadBuilder::default()
            .name("deep-BuboCore-scheduler")
            .spawn(move |_| {
                let mut sched = Scheduler::new(clock_server.into(), devices, world_iface, rx);
                sched.do_your_thing();
            })
            .expect("Unable to start World");
        (handle, tx)
    }

    pub fn new(
        clock: Clock,
        devices: Arc<DeviceMap>,
        world_iface: Sender<TimedMessage>,
        receiver: Receiver<SchedulerMessage>,
    ) -> Scheduler {
        Scheduler {
            world_iface,
            pattern: Default::default(),
            global_vars: HashMap::new(),
            executions: Vec::new(),
            devices,
            clock,
            message_source: receiver,
            next_wait: None,
        }
    }

    fn step_index(clock : &Clock, sequence : &Sequence, date: SyncTime) -> (usize, SyncTime, SyncTime) {
        let beats_len : f64 = sequence.beats_len();
        let beat = clock.beat_at_date(date);
        let mut acc_beat = beat % (beats_len / sequence.speed_factor);
        let sequence_begin = beat - acc_beat;
        let mut start_beat = 0.0f64;
        for i in 0..sequence.n_steps() {
            let step_len = sequence.step_len(i) / sequence.speed_factor;
            if acc_beat <= step_len {
                let start_date = clock.date_at_beat(sequence_begin + start_beat);
                let remaining = clock.beats_to_micros(step_len - acc_beat);
                return (i, start_date, remaining);
            }
            acc_beat -= step_len;
            start_beat += sequence.step_len(i);
        }
        return (usize::MAX, SyncTime::MAX, SyncTime::MAX);
    }

    pub fn change_pattern(&mut self, mut pattern: Pattern) {
        let date = self.theoretical_date();
        for sequence in pattern.sequences.iter_mut() {
            let (step, _, _) = Self::step_index(&self.clock, sequence, date);
            sequence.current_step = step;
        }
        self.pattern = pattern;
    }

    pub fn process_message(&mut self, msg: SchedulerMessage) {
        match msg {
            SchedulerMessage::UploadPattern(pattern) => self.change_pattern(pattern),
            SchedulerMessage::ToggleStep(sequence, step) => self.pattern.sequences[sequence].toggle_step(step),
        }
    }

    pub fn do_your_thing(&mut self) {
        let start_date = self.clock.micros();
        println!("[+] Starting scheduler at {start_date}");
        loop {
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
            for track in self.pattern.sequences.iter_mut() {
                let (step, scheduled_date, track_step_delay) = Self::step_index(&self.clock, track, date);
                next_step_delay = std::cmp::min(next_step_delay, track_step_delay);
                if step < usize::MAX && step != track.current_step {
                    let script = Arc::clone(&track.scripts[step]);
                    self.executions.push(ScriptExecution::execute_at(script, track.index, scheduled_date));
                    track.current_step = step;
                }
            }

            let next_exec_delay = self.execution_loop();

            let next_delay = std::cmp::min(next_exec_delay, next_step_delay);
            if next_delay > 0 {
                self.next_wait = Some(next_delay);
            } else {
                self.next_wait = None;
            }
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
        let scheduled_date = self.theoretical_date();
        // TODO: Read MIDI input controller values
        let mut next_timeout = SyncTime::MAX;

        self.executions.retain_mut(|exec| {
            let sequence = self.pattern.mut_sequence(exec.sequence_index);

            if !exec.is_ready(scheduled_date) {
                next_timeout = std::cmp::min(next_timeout, exec.remaining_before(scheduled_date));
                return true;
            }
            next_timeout = 0;
            if let Some((event, date)) = exec.execute_next(&self.clock, &mut self.global_vars, sequence) {
                let messages = self.devices.map_event(event, date, &self.clock);
                for message in messages {
                    let _ = self.world_iface.send(message);
                }
            }
            !exec.has_terminated()
        });
        next_timeout
    }

}
