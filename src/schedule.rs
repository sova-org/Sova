// Doit faire traduction (Event, TimeSpan) en (ProtocolMessage, SyncTime)

use std::{collections::HashMap, rc::Rc, sync::{mpsc::{self, Receiver, Sender, TryRecvError}, Arc}, thread::JoinHandle};

use thread_priority::ThreadBuilder;

use crate::{clock::{Clock, ClockServer, SyncTime}, device_map::DeviceMap, lang::variable::VariableStore, pattern::{script::{Script, ScriptExecution}, Pattern}, protocol::TimedMessage};

pub const SCHEDULED_DRIFT : SyncTime = 30_000;

pub struct SchedulerMessage;

pub struct Scheduler {
    pub pattern : Pattern,
    pub globals : VariableStore,

    pub executions : Vec<ScriptExecution>,

    world_iface : Sender<TimedMessage>,
    devices : Arc<DeviceMap>,
    clock : Clock,

    message_source : Receiver<SchedulerMessage>
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
                let mut sched = Scheduler::new(clock_server.into(), devices, world_iface, rx);
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
            message_source : receiver
        }
    }

    pub fn do_your_thing(&mut self) {
        loop {
            self.clock.capture_app_state();
            match self.message_source.try_recv() {
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => (),
                Ok(_) => (),
            }
            let track = self.pattern.current_track();
            self.execution_loop();
        }
    }

    pub fn kill_all(&mut self) {
        self.executions.clear();
    }

    fn execution_loop(&mut self) {
        let scheduled_date = self.clock.micros() + SCHEDULED_DRIFT;
        self.executions.retain_mut(|exec| {
            if !exec.is_ready(scheduled_date) {
                return true;
            }
            if let Some((event, date)) = exec.execute_next(&mut self.globals, &self.clock) {
                let messages = self.devices.map_event(event, date, &self.clock);
                for message in messages {
                    let _ = self.world_iface.send(message);
                }
            }
            !exec.has_terminated()
        });
    }

    pub fn start_execution(&mut self, script : &Rc<Script>, scheduled_date : SyncTime) {
        let execution = ScriptExecution::execute_at(Rc::clone(script), scheduled_date);
        self.executions.push(execution);
    } 

}
