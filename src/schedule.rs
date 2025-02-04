// Doit faire traduction (Event, MusicTime) en (ProtocolMessage, SyncTime)

use std::sync::mpsc::Sender;

use crate::{pattern::Pattern, protocol::TimedMessage};

pub struct Scheduler {
    pub pattern : Pattern,
    world_iface : Sender<TimedMessage>
}

impl Scheduler {

    pub fn new(world_iface : Sender<TimedMessage>) -> Scheduler {
        Scheduler {
            world_iface, pattern : Pattern::default()
        }
    }

}
