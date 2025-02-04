use std::{cell::RefCell, sync::mpsc::Receiver};

use priority_queue::PriorityQueue;
use thread_priority::Thread;

use crate::{clock::SyncTime, protocol::{ProtocolMessage, TimedMessage}};

pub struct World {
    queue : RefCell<PriorityQueue<ProtocolMessage, Reverse<SyncTime>>>,
    message_source : Receiver<TimedMessage>,
}

impl World {

    pub fn new(channel : Receiver<TimedMessage>) -> Self {

    }

}
