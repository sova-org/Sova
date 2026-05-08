use crossbeam_channel::{self, Receiver, RecvTimeoutError, Sender};

use std::{collections::BinaryHeap, sync::Arc, thread::JoinHandle, time::Duration};
use thread_priority::{ThreadBuilder, ThreadPriority};

use crate::{
    clock::{Clock, ClockServer, SyncTime},
    log_println,
    protocol::{ProtocolPayload, TimedMessage},
};
use crate::{get_logger, log_eprintln};

pub const ACTIVE_WAITING_SWITCH_MICROS: SyncTime = 10;
pub const MIDI_EARLY_THRESHOLD: SyncTime = 2_000;
pub const NON_MIDI_LOOKAHEAD: SyncTime = 20_000;

const RT_YIELD_FLOOR: Duration = Duration::from_micros(50);
const RT_BUDGET_REQUEST_FRAMES: u32 = 200_000;
const RT_BUDGET_REQUEST_HZ: u32 = 1_000_000;

/// Creates an audio-priority thread that receives [TimedMessage] and send them to the corresponding device at their precise date.
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

fn recv_remaining(next_timeout: Duration) -> Duration {
    next_timeout
        .saturating_sub(Duration::from_micros(ACTIVE_WAITING_SWITCH_MICROS))
        .max(RT_YIELD_FLOOR)
}

impl World {
    /// Initiate and start an audio-thread synchronized to the [ClockServer].
    /// # Returns
    /// - The handle to the thread,
    /// - A channel [Sender] to send [TimedMessage].
    pub fn create(clock_server: Arc<ClockServer>) -> (JoinHandle<()>, Sender<TimedMessage>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        let handle = ThreadBuilder::default()
            .name("sova-world")
            .priority(ThreadPriority::Max)
            .spawn(move |_| {
                match audio_thread_priority::promote_current_thread_to_real_time(
                    RT_BUDGET_REQUEST_FRAMES,
                    RT_BUDGET_REQUEST_HZ,
                ) {
                    Ok(_) => log_println!("World: real-time priority set"),
                    Err(e) => {
                        log_eprintln!("World: failed to set RT priority: {:?}", e);
                        #[cfg(target_os = "linux")]
                        eprintln!(
                            "[sova] WARNING: Real-time audio priority unavailable. \
                             Set rtprio in /etc/security/limits.conf or run with CAP_SYS_NICE. \
                             Audio glitches are likely on this system."
                        );
                    }
                }
                let mut world = World {
                    queue: BinaryHeap::with_capacity(4096),
                    message_source: rx,
                    next_timeout: Duration::MAX,
                    clock: clock_server.into(),
                    midi_early_threshold: MIDI_EARLY_THRESHOLD,
                    non_midi_lookahead: NON_MIDI_LOOKAHEAD,
                };
                if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    world.live();
                })) {
                    log_eprintln!("World thread panicked: {:?}", e);
                }
            })
            .expect("Unable to start World");
        (handle, tx)
    }

    /// Main loop of the [World], performing until the channel is closed:
    /// - Wait for a [TimedMessage] until a timeout corresponding to the next event, minus an active waiting threshold
    /// - If the time until the next [TimedMessage] date is smaller than the active waiting threshold, active wait
    /// - Execute the message
    /// - Refresh the next timeout
    pub fn live(&mut self) {
        log_println!("Starting world");
        loop {
            let remaining = recv_remaining(self.next_timeout);
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
            while next.time > time && next.time.saturating_sub(time) <= ACTIVE_WAITING_SWITCH_MICROS
            {
                time = self.clock.micros();
            }

            if next.time <= time
                && let Some(msg) = self.queue.pop()
            {
                self.execute_message(msg);
            }
            self.refresh_next_timeout();
        }
        log_println!("[-] Exiting world...");
    }

    /// Add the [TimedMessage] to the priority queue,
    /// eventually subtracting micros to the scheduled date to anticipate latency
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

    /// Look for the next message to execute (i.e., the nearest date)
    /// and set the next timeout accordingly
    fn refresh_next_timeout(&mut self) {
        let Some(next_msg) = self.queue.peek() else {
            self.next_timeout = Duration::MAX;
            return;
        };

        let now = self.clock.micros();
        let remaining = next_msg.time.saturating_sub(now);
        self.next_timeout = Duration::from_micros(remaining);
    }

    /// Execute the given [TimedMessage] at the instant this function is called
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