// Doit faire traduction (Event, TimeSpan) en (ProtocolMessage, SyncTime)

use std::{collections::HashMap, rc::Rc, sync::{mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError}, Arc}, thread::JoinHandle, time::Duration, usize};

use thread_priority::ThreadBuilder;

use crate::{clock::{Clock, ClockServer, SyncTime}, device_map::DeviceMap, lang::variable::VariableStore, pattern::{script::{Script, ScriptExecution}, Pattern}, protocol::TimedMessage};

pub const SCHEDULED_DRIFT : SyncTime = 30_000;

pub enum SchedulerMessage {
    UploadPattern(Pattern)
}

pub struct Scheduler {
    pub pattern : Pattern,
    pub globals : VariableStore,

    pub executions : Vec<ScriptExecution>,

    world_iface : Sender<TimedMessage>,
    devices : Arc<DeviceMap>,
    clock : Clock,

    message_source : Receiver<SchedulerMessage>,

    current_step : usize,
    next_wait : Option<SyncTime>
}

impl Scheduler {

    pub fn create(
        clock_server : Arc<ClockServer>,
        devices : Arc<DeviceMap>,
        world_iface : Sender<TimedMessage>
    ) -> (JoinHandle<()>, Sender<SchedulerMessage>) {
        let (tx,rx) = mpsc::channel();
        let handle = ThreadBuilder::default()
            .name("deep-BuboCore-scheduler")
            .spawn(move |_| {
                let mut sched = Scheduler::new(
                    clock_server.into(),
                    devices,
                    world_iface,
                    rx
                );
                sched.do_your_thing();
            }).expect("Unable to start World");
        (handle, tx)
    }

    pub fn new(
        clock : Clock,
        devices : Arc<DeviceMap>,
        world_iface : Sender<TimedMessage>,
        receiver : Receiver<SchedulerMessage>
    ) -> Scheduler {
        Scheduler {
            world_iface,
            pattern : Default::default(),
            globals : HashMap::new(),
            executions : Vec::new(),
            devices,
            clock,
            message_source : receiver,
            current_step : usize::MAX,
            next_wait : None
        }
    }

    fn step_index(&self, date : SyncTime) -> (usize, SyncTime, SyncTime) {
        let Some(track) = self.pattern.current_track() else {
            return (usize::MAX, SyncTime::MAX, SyncTime::MAX);
        };
        let track_len : f64 = track.steps.iter().sum();
        let beat = self.clock.beat_at_date(date);
        let mut acc_beat = beat % (track_len / track.speed_factor);
        let track_begin = beat - acc_beat;
        let mut start_beat = 0.0f64;
        for i in 0..track.steps.len() {
            let step_len = track.steps[i] / track.speed_factor;
            if acc_beat <= step_len {
                let start_date = self.clock.date_at_beat(track_begin + start_beat);
                let remaining = self.clock.beats_to_micros(step_len - acc_beat);
                return (i, start_date, remaining);
            }
            acc_beat -= step_len;
            start_beat += track.steps[i];
        }
        return (
            usize::MAX,
            SyncTime::MAX,
            SyncTime::MAX
        );
    }

    pub fn change_pattern(&mut self, pattern : Pattern) {
        self.pattern = pattern;
        let date = self.theoretical_date();
        let (step, _, _) = self.step_index(date);
        self.current_step = step;// usize::MAX;
    }

    pub fn process_message(&mut self, msg : SchedulerMessage) {
        match msg {
            SchedulerMessage::UploadPattern(pattern) => self.change_pattern(pattern),
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

            let (step, scheduled_date, next_step_delay) = self.step_index(date);

            if step < usize::MAX && step != self.current_step {
                let track = self.pattern.current_track().unwrap();
                let script = Arc::clone(&track.scripts[step]);
                self.start_execution(script, scheduled_date);
                self.current_step = step;
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
        let mut next_timeout = SyncTime::MAX;
        self.executions.retain_mut(|exec| {
            if !exec.is_ready(scheduled_date) {
                next_timeout = std::cmp::min(next_timeout, exec.remaining_before(scheduled_date));
                return true;
            }
            next_timeout = 0;
            if let Some((event, date)) = exec.execute_next(&mut self.globals, &self.clock) {
                let messages = self.devices.map_event(event, date, &self.clock);
                for message in messages {
                    let _ = self.world_iface.send(message);
                }
            }
            !exec.has_terminated()
        });
        next_timeout
    }

    pub fn start_execution(&mut self, script : Arc<Script>, scheduled_date : SyncTime) {
        let execution = ScriptExecution::execute_at(script, scheduled_date);
        self.executions.push(execution);
    }

}
