// Doit faire traduction (Event, TimeSpan) en (ProtocolMessage, SyncTime)

use std::{rc::Rc, sync::{mpsc::Sender, Arc}};

use crate::{clock::{Clock, SyncTime}, device_map::DeviceMap, lang::variable::VariableStore, pattern::{script::{Script, ScriptExecution}, Pattern}, protocol::{ProtocolMessage, TimedMessage}};

pub const SCHEDULED_DRIFT : SyncTime = 30_000;

pub struct Scheduler {
    pub pattern : Pattern,
    pub globals : VariableStore,

    pub executions : Vec<ScriptExecution>,

    world_iface : Sender<TimedMessage>,
    devices : Arc<DeviceMap>,
    clock : Arc<Clock>
}

impl Scheduler {

    pub fn new(clock : Arc<Clock>, devices : Arc<DeviceMap>, world_iface : Sender<TimedMessage>) -> Scheduler {
        Scheduler {
            world_iface,
            pattern : Default::default(),
            globals : Default::default(),
            executions : Default::default(),
            devices,
            clock
        }
    }

    pub fn main_loop(&mut self) {
        self.execution_loop();
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
                let protocol = self.devices.map_event(event);
                let timed = protocol.timed(date);
                let _ = self.world_iface.send(timed);
            }
            !exec.has_terminated()
        });
    }

    pub fn start_execution(&mut self, script : &Rc<Script>, scheduled_date : SyncTime) {
        let execution = ScriptExecution::execute_at(Rc::clone(script), scheduled_date);
        self.executions.push(execution);
    } 

}
