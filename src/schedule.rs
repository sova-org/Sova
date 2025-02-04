// Doit faire traduction (Event, MusicTime) en (ProtocolMessage, SyncTime)

use std::sync::mpsc::Sender;

use crate::protocol::TimedMessage;

pub struct Scheduler {
    world_iface : Sender<TimedMessage>
}

impl Scheduler {

    pub fn new(world_iface : Sender<TimedMessage>) -> Scheduler {
        Scheduler {
            world_iface
        }
    }

}
