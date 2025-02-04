use std::{
        collections::BinaryHeap, sync::mpsc::{self, Receiver, RecvTimeoutError, Sender}, thread::JoinHandle, time::{
            Duration,
            SystemTime,
            UNIX_EPOCH
        }
};
use thread_priority::{
    ThreadBuilder,
    ThreadPriority
};

use crate::{clock::SyncTime, protocol::{ProtocolMessage, TimedMessage}};

const WORLD_TIME_MARGIN : u64 = 10;

pub struct World {
    queue : BinaryHeap<TimedMessage>,
    message_source : Receiver<TimedMessage>,
    next_timeout : Duration
}

impl World {

    pub fn create() -> (JoinHandle<()>, Sender<TimedMessage>) {
        let (tx,rx) = mpsc::channel();
        let handle = ThreadBuilder::default()
            .name("deep-BuboCore")
            .priority(ThreadPriority::Max)
            .spawn(move |_| {
                let mut world = World {
                    queue : Default::default(),
                    message_source : rx,
                    next_timeout : Duration::MAX
                };
                world.live();
            }).expect("Unable to start World");
        (handle, tx)
    }

    pub fn live(&mut self) {
        loop {
            match self.message_source.recv_timeout(
                self.next_timeout - Duration::from_micros(WORLD_TIME_MARGIN) // Subtracting minimal duration
            ) {
                Err(RecvTimeoutError::Disconnected) => break,
                Ok(timed_message) => {
                    self.add_message(timed_message);
                },
                Err(RecvTimeoutError::Timeout) => () // Received nothing
            }
            let Some(next) = self.queue.peek() else {
                continue;
            };
            let mut time = self.get_clock_micros();

            // Active waiting when not enough time to wait again
            while next.time > time && next.time + WORLD_TIME_MARGIN <= time {
                time = self.get_clock_micros();
            }

            if next.time <= time {
                let msg = self.queue.pop().unwrap();
                self.execute_message(msg);
            }
            self.refresh_next_timeout();
        }
    }

    pub fn add_message(&mut self, msg : TimedMessage) {
        self.queue.push(msg);
    }

    fn refresh_next_timeout(&mut self) {
        let Some(next_msg) = self.queue.peek() else {
            self.next_timeout = Duration::MAX;
            return;
        };

        // New time duration
        let now = self.get_clock_micros();
        let remaining = next_msg.time - now;
        self.next_timeout = Duration::from_micros(remaining);
    }

    pub fn execute_message(&self, msg : TimedMessage) {
        let (msg, time) = msg.untimed();
        match msg {
            ProtocolMessage::OSC(_oscmessage) => todo!(),
            ProtocolMessage::MIDI(_midimessage) => todo!(),
            ProtocolMessage::LOG(log_message) => {
                let mut clock_time = self.get_clock_micros();
                clock_time %= 60 * 1000 * 1000;
                let time = time % (60 * 1000 * 1000);
                println!("{} {} | Time : {} ; Wanted : {}", log_message.level, log_message.msg, clock_time, time);
            },
        }
    }

    // TODO: replace with real clock
    fn get_clock_micros(&self) -> SyncTime {
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backward");
        since_epoch.as_micros() as u64
    }

}
