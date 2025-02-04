use std::{
        cmp::Reverse, collections::BinaryHeap, sync::mpsc::{Receiver, RecvTimeoutError}, thread::JoinHandle, time::{
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

pub struct World {
    queue : BinaryHeap<TimedMessage>,
    message_source : Receiver<TimedMessage>,
    next_timeout : Duration
}

impl World {

    pub fn start(channel : Receiver<TimedMessage>) -> Result<JoinHandle<()>, std::io::Error> {
        let thread = ThreadBuilder::default()
            .name("deep-BuboCore")
            .priority(ThreadPriority::Max)
            .spawn(move |_| {
                let mut world = World {
                    queue : Default::default(),
                    message_source : channel,
                    next_timeout : Duration::MAX
                };
                world.live();
            });
        thread
    }

    pub fn live(&mut self) {
        loop {
            match self.message_source.recv_timeout(
                self.next_timeout
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
            let time = self.get_clock_micros();
            if next.time <= time {
                let msg = self.queue.pop().unwrap();
                self.execute_message(msg);
            }
        }
    }

    pub fn add_message(&mut self, msg : TimedMessage) {
        self.queue.push(msg);
        self.refresh_next_timeout();
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
            ProtocolMessage::OSC(oscmessage) => todo!(),
            ProtocolMessage::MIDI(midimessage) => todo!(),
            ProtocolMessage::LOG(log_message) => {
                let mut clock_time_ms = self.get_clock_micros() / 1000;
                clock_time_ms %= (60 * 1000);
                let time_ms = time % (60 * 1000);
                println!("{} {} | Time : {} ; Wanted : {}", log_message.level, log_message.msg, clock_time_ms, time_ms);
            },
        }
    }

    // TODO: replace with real clock
    pub fn get_clock_micros(&self) -> SyncTime {
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backward");
        since_epoch.as_micros() as u64
    }

}
