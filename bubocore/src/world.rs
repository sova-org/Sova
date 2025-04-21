use std::{
        collections::BinaryHeap, sync::{mpsc::{self, Receiver, RecvTimeoutError, Sender}, Arc}, thread::JoinHandle, time::Duration,
};
use thread_priority::{
    ThreadBuilder,
    ThreadPriority
};

use crate::{clock::{Clock, ClockServer, SyncTime}, protocol::{ProtocolPayload, TimedMessage}, protocol::log::LogMessage};
use crate::lang::event::ConcreteEvent;

const WORLD_TIME_MARGIN : u64 = 300;

pub struct World {
    queue : BinaryHeap<TimedMessage>,
    message_source : Receiver<TimedMessage>,
    next_timeout : Duration,
    clock : Clock
}

impl World {

    pub fn create(clock_server : Arc<ClockServer>) -> (JoinHandle<()>, Sender<TimedMessage>) {
        let (tx,rx) = mpsc::channel();
        let handle = ThreadBuilder::default()
            .name("deep-BuboCore-world")
            .priority(ThreadPriority::Max)
            .spawn(move |_| {
                let mut world = World {
                    queue : Default::default(),
                    message_source : rx,
                    next_timeout : Duration::MAX,
                    clock : clock_server.into()
                };
                world.live();
            }).expect("Unable to start World");
        (handle, tx)
    }

    pub fn live(&mut self) {
        let start_date = self.get_clock_micros();
        println!("[+] Starting world at {start_date}");
        loop {
            let remaining =
                self.next_timeout.saturating_sub(Duration::from_micros(WORLD_TIME_MARGIN));
            match self.message_source.recv_timeout(remaining) {
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
            // TODO : attention, que se passe-t'il si un message arrive pendant ce temps ?
            while next.time > time && next.time + WORLD_TIME_MARGIN <= time {
                time = self.get_clock_micros();
            }

            if next.time <= time {
                let msg = self.queue.pop().unwrap();
                self.execute_message(msg);
            }
            self.refresh_next_timeout();
        }
        println!("[-] Exiting world...");
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
        let remaining = next_msg.time.saturating_sub(now);
        self.next_timeout = Duration::from_micros(remaining);
    }

    pub fn execute_message(&self, msg : TimedMessage) {
        let TimedMessage { message, time } = msg;
        if let ProtocolPayload::LOG(log_message) = message.payload {
            
            let log_output = match log_message.event {
                 Some(event) => {
                    match event {
                        ConcreteEvent::MidiNote(note, vel, chan, dur_micros, dev_id) => {
                            let dur_ms = dur_micros as f64 / 1000.0;
                            let dur_beats = self.clock.micros_to_beats(dur_micros);
                            format!(
                                "MidiNote(Note: {}, Vel: {}, Chan: {}, Dur: {:.1}ms / {:.2} beats, Dev: {})",
                                note, vel, chan, dur_ms, dur_beats, dev_id
                            )
                        }
                        _ => format!("{:?}", event),
                    }
                }
                None => log_message.msg,
            };

            let mut clock_time = self.get_clock_micros();
            let drift = clock_time.abs_diff(time);
            clock_time %= 60 * 1000 * 1000;
            let time = time % (60 * 1000 * 1000);
            
            println!("{} {} | Time : {clock_time} ; Wanted : {time} ; Drift : {drift}",
                log_message.level,
                log_output,
            );
        } else {
            let _ = message.send();
        }
    }

    fn get_clock_micros(&self) -> SyncTime {
        self.clock.micros()
    }

}
