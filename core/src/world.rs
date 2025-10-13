use crossbeam_channel::{self, Receiver, RecvTimeoutError, Sender};

use std::{
    collections::BinaryHeap,
    sync::Arc,
    thread::JoinHandle,
    time::Duration,
};
use thread_priority::{ThreadBuilder, ThreadPriority};

use crate::get_logger;
use crate::{
    clock::{Clock, ClockServer, SyncTime},
    protocol::{
        TimedMessage,
        ProtocolPayload,
    },
    log_println,
};

pub const ACTIVE_WAITING_SWITCH_MICROS : SyncTime = 50;
pub const TIMEBASE_CAIBRATION_INTERVAL : SyncTime = 1_000_000;
pub const MIDI_EARLY_THRESHOLD : SyncTime = 2_000;
pub const NON_MIDI_LOOKAHEAD : SyncTime = 20_000;

pub struct World {
    queue: BinaryHeap<TimedMessage>,
    message_source: Receiver<TimedMessage>,
    next_timeout: Duration,
    clock: Clock,
    /// MIDI interface latency compensation (2ms)
    midi_early_threshold: SyncTime,
    /// Lookahead for non-MIDI messages (OSC, AudioEngine) - send early for internal scheduling
    non_midi_lookahead: SyncTime,
}

impl World {
    pub fn create(
        clock_server: Arc<ClockServer>,
    ) -> (JoinHandle<()>, Sender<TimedMessage>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        let handle = ThreadBuilder::default()
            .name("sova-world")
            .priority(ThreadPriority::Max)
            .spawn(move |_| {
                let mut world = World {
                    queue: Default::default(),
                    message_source: rx,
                    next_timeout: Duration::MAX,
                    clock: clock_server.into(),
                    midi_early_threshold: MIDI_EARLY_THRESHOLD,                     // 2ms for MIDI interface compensation
                    non_midi_lookahead: NON_MIDI_LOOKAHEAD,                         // 20ms lookahead for OSC/AudioEngine
                };
                world.live();
            })
            .expect("Unable to start World");
        (handle, tx)
    }

    pub fn live(&mut self) {
        let start_date = self.clock.micros();
        log_println!("[+] Starting world at {start_date}");
        loop {
            let remaining = self.next_timeout.saturating_sub(
                Duration::from_micros(ACTIVE_WAITING_SWITCH_MICROS)
            ); // Reduced for better precision
            match self.message_source.recv_timeout(remaining) {
                Err(RecvTimeoutError::Disconnected) => break,
                Ok(timed_message) => {
                    self.handle_timed_message(timed_message);
                }
                Err(RecvTimeoutError::Timeout) => (), // Received nothing
            }
            let Some(next) = self.queue.peek() else {
                continue;
            };
            let mut time = self.clock.micros();

            // Active waiting when not enough time to wait again
            while next.time > time && next.time.saturating_sub(time) <= ACTIVE_WAITING_SWITCH_MICROS {
                time = self.clock.micros();
            }

            if next.time <= time {
                let msg = self.queue.pop().unwrap();
                self.execute_message(msg);
            }
            self.refresh_next_timeout();
        }
        log_println!("[-] Exiting world...");
    }

    fn handle_timed_message(&mut self, mut timed_message: TimedMessage) {
        // Regular message - add to queue for timed execution
        let offset = match &timed_message.message.payload {
            ProtocolPayload::LOG(_) => 0,
            ProtocolPayload::MIDI(_) => self.midi_early_threshold,
            ProtocolPayload::OSC(_)
            | ProtocolPayload::AudioEngine(_) => self.non_midi_lookahead,
        };
        timed_message.time = timed_message.time.saturating_sub(offset);
        self.queue.push(timed_message);
    }

    fn refresh_next_timeout(&mut self) {
        let Some(next_msg) = self.queue.peek() else {
            self.next_timeout = Duration::MAX;
            return;
        };

        let now = self.clock.micros();
        let remaining = next_msg.time.saturating_sub(now);
        self.next_timeout = Duration::from_micros(remaining);
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

    // /// Calibrate the Link↔SystemTime offset with maximum precision
    // fn calibrate_timebase(&mut self) {
    //     // Multi-sample calibration to minimize race condition uncertainty
    //     let mut best_offset = 0i64;
    //     let mut min_latency = u64::MAX;

    //     // Take 8 samples and use the one with minimum measurement latency
    //     for _ in 0..8 {
    //         let before_system = SystemTime::now()
    //             .duration_since(UNIX_EPOCH)
    //             .unwrap()
    //             .as_micros() as u64;
    //         let link_time = self.clock.micros();
    //         let after_system = SystemTime::now()
    //             .duration_since(UNIX_EPOCH)
    //             .unwrap()
    //             .as_micros() as u64;

    //         let measurement_latency = after_system - before_system;
    //         if measurement_latency < min_latency {
    //             min_latency = measurement_latency;
    //             // Interpolate to midpoint to minimize race condition
    //             let interpolated_system_time = before_system + (measurement_latency / 2);
    //             best_offset = interpolated_system_time as i64 - link_time as i64;
    //         }
    //     }

    //     self.timebase_calibration.link_to_system_offset = best_offset;
    //     self.timebase_calibration.last_calibration = self.clock.micros();
    // }

    
}

// High-precision Link ↔ SystemTime conversion calibration
// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
// struct TimebaseCalibration {
//     /// SystemTime - LinkTime offset at calibration point
//     link_to_system_offset: i64,
//     /// When we last calibrated (Link time in microseconds)
//     last_calibration: u64,
// }

// impl TimebaseCalibration {
//     fn new() -> Self {
//         Self::default()
//     }
// }