// Doit faire traduction (Event, TimeSpan) en (ProtocolMessage, SyncTime)

use std::sync::mpsc::Sender;

use crate::{clock::SyncTime, lang::variable::VariableStore, pattern::{script::ScriptExecution, Pattern}, protocol::TimedMessage};

pub struct Scheduler {
    pub pattern : Pattern,
    pub globals : VariableStore,

    world_iface : Sender<TimedMessage>,
}

impl Scheduler {

    pub fn new(world_iface : Sender<TimedMessage>) -> Scheduler {
        Scheduler {
            world_iface,
            pattern : Default::default(),
            globals : Default::default()
        }
    }

}
