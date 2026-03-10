use crossbeam_channel::{self, Receiver, Sender, TryRecvError};

use std::{collections::BinaryHeap, sync::Arc, thread::JoinHandle};
use thread_priority::{ThreadBuilder, ThreadPriority};

use crate::{
    clock::{Clock, ClockServer, SyncTime},
    log_println, log_eprintln, get_logger,
    protocol::{ProtocolPayload, TimedMessage},
};

pub const ACTIVE_WAITING_SWITCH_MICROS: SyncTime = 30;
pub const MIDI_EARLY_THRESHOLD: SyncTime = 2_000;
pub const NON_MIDI_LOOKAHEAD: SyncTime = 20_000;

pub struct World {
    queue: BinaryHeap<TimedMessage>,
    message_source: Receiver<TimedMessage>,
    clock: Clock,
    /// MIDI interface latency compensation (2ms)
    midi_early_threshold: SyncTime,
    /// Lookahead for non-MIDI messages (OSC, AudioEngine) - send early for internal scheduling
    non_midi_lookahead: SyncTime,
}

impl World {
    pub fn create(clock_server: Arc<ClockServer>) -> (JoinHandle<()>, Sender<TimedMessage>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        let handle = ThreadBuilder::default()
            .name("sova-world")
            .priority(ThreadPriority::Max)
            .spawn(move |_| {
                match audio_thread_priority::promote_current_thread_to_real_time(128, 44100) {
                    Ok(_) => log_println!("World: real-time priority set"),
                    Err(e) => log_eprintln!("World: failed to set RT priority: {:?}", e),
                }
                let mut world = World {
                    queue: Default::default(),
                    message_source: rx,
                    clock: clock_server.into(),
                    midi_early_threshold: MIDI_EARLY_THRESHOLD, // 2ms for MIDI interface compensation
                    non_midi_lookahead: NON_MIDI_LOOKAHEAD, // 20ms lookahead for OSC/AudioEngine
                };
                world.live();
            })
            .expect("Unable to start World");
        (handle, tx)
    }

    pub fn live(&mut self) {
        log_println!("Starting world");
        loop {
            match self.message_source.try_recv() {
                Err(TryRecvError::Disconnected) => break,
                Ok(timed_message) => {
                    self.handle_timed_message(timed_message);
                }
                Err(TryRecvError::Empty) => (), // Received nothing
            }
            let Some(next) = self.queue.peek() else {
                continue;
            };
            let mut time = self.clock.micros();

            // Active waiting when not enough time to wait again
            while next.time > time && next.time.saturating_sub(time) <= ACTIVE_WAITING_SWITCH_MICROS
            {
                time = self.clock.micros();
            }

            if next.time <= time {
                let msg = self.queue.pop().unwrap();
                self.execute_message(msg);
            }
        }
        log_println!("[-] Exiting world...");
    }

    fn handle_timed_message(&mut self, mut timed_message: TimedMessage) {
        // Regular message - add to queue for timed execution
        let offset = match &timed_message.message.payload {
            ProtocolPayload::LOG(_) => 0,
            ProtocolPayload::MIDI(_) => self.midi_early_threshold,
            ProtocolPayload::AudioEngine(_) => self.non_midi_lookahead,
            ProtocolPayload::OSC(osc) if osc.timetag.is_some() => {
                self.execute_message(timed_message);
                return;
            }
            _ => self.non_midi_lookahead,
        };
        timed_message.time = timed_message.time.saturating_sub(offset);
        self.queue.push(timed_message);
    }

    pub fn execute_message(&mut self, msg: TimedMessage) {
        let message = msg.message;
        match message.payload {
            ProtocolPayload::LOG(log_msg) => {
                get_logger().log_message(log_msg);
            }
            _ => {
                // Other protocols: Send with precise target timestamp
                let _ = message.send();
            }
        }
    }
}
