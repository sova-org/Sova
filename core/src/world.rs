use crossbeam_channel::{self, Receiver, RecvTimeoutError, Sender};
use tokio::sync::watch;
use std::{
    collections::BinaryHeap,
    sync::Arc,
    thread::JoinHandle,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use thread_priority::{ThreadBuilder, ThreadPriority};

use crate::lang::event::ConcreteEvent;
use crate::{
    clock::{Clock, ClockServer, SyncTime},
    protocol::{
        message::TimedMessage,
        payload::{AudioEnginePayload, ProtocolPayload},
        log::LogMessage,
    },
    schedule::SovaNotification,
    log_println,
};
use bubo_engine::{
    registry::ModuleRegistry,
    server::ScheduledEngineMessage,
    types::{EngineMessage, ScheduledMessage, EngineLogMessage},
};
use std::collections::HashMap;

// WORLD_TIME_MARGIN constant moved to TimingConfig.world_precision_margin_micros

pub const ACTIVE_WAITING_SWITCH_MICROS : SyncTime = 50;
pub const TIMEBASE_CAIBRATION_INTERVAL : SyncTime = 1_000_000;
pub const MIDI_EARLY_THRESHOLD : SyncTime = 2_000;
pub const NON_MIDI_LOOKAHEAD : SyncTime = 20_000;

/// High-precision Link ↔ SystemTime conversion calibration
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct TimebaseCalibration {
    /// SystemTime - LinkTime offset at calibration point
    link_to_system_offset: i64,
    /// When we last calibrated (Link time in microseconds)
    last_calibration: u64,
}

impl TimebaseCalibration {
    fn new() -> Self {
        Self::default()
    }
}

pub struct World {
    queue: BinaryHeap<TimedMessage>,
    message_source: Receiver<TimedMessage>,
    next_timeout: Duration,
    clock: Clock,
    audio_engine_tx: Option<Sender<ScheduledEngineMessage>>,
    voice_id_counter: u32,
    registry: ModuleRegistry,
    timebase_calibration: TimebaseCalibration,
    timebase_calibration_interval: SyncTime,
    // MIDI interface latency compensation (2ms)
    midi_early_threshold: SyncTime,
    // Lookahead for non-MIDI messages (OSC, AudioEngine) - send early for internal scheduling
    non_midi_lookahead: SyncTime,
    notification_sender: Option<watch::Sender<SovaNotification>>,
}

impl World {
    pub fn create(
        clock_server: Arc<ClockServer>,
        audio_engine_tx: Option<Sender<ScheduledEngineMessage>>,
        registry: ModuleRegistry,
        engine_log_rx: Option<Receiver<EngineLogMessage>>,
        notification_sender: Option<watch::Sender<SovaNotification>>,
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
                    audio_engine_tx,
                    voice_id_counter: 0,
                    registry,
                    timebase_calibration: TimebaseCalibration::new(),
                    timebase_calibration_interval: TIMEBASE_CAIBRATION_INTERVAL,    // 1s calibration interval
                    midi_early_threshold: MIDI_EARLY_THRESHOLD,                     // 2ms for MIDI interface compensation
                    non_midi_lookahead: NON_MIDI_LOOKAHEAD,                         // 20ms lookahead for OSC/AudioEngine
                    notification_sender,
                };
                world.live(engine_log_rx);
            })
            .expect("Unable to start World");
        (handle, tx)
    }

    pub fn live(&mut self, engine_log_rx: Option<Receiver<EngineLogMessage>>) {
        let start_date = self.clock.micros();
        // Initialize timebase calibration
        self.calibrate_timebase();
        log_println!("[+] Starting world at {start_date}");
        loop {
            // Process engine logs without blocking
            if let Some(ref log_rx) = engine_log_rx {
                while let Ok(engine_log) = log_rx.try_recv() {
                    self.handle_engine_log(engine_log);
                }
            }

            let remaining = self.next_timeout.saturating_sub(Duration::from_micros(ACTIVE_WAITING_SWITCH_MICROS)); // Reduced for better precision
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

    fn handle_timed_message(&mut self, timed_message: TimedMessage) {
        // Regular message - add to queue for timed execution
        self.add_message(timed_message);
    }

    pub fn add_message(&mut self, msg: TimedMessage) {
        self.queue.push(msg);
    }

    fn handle_engine_log(&mut self, engine_log: EngineLogMessage) {
        let log_msg = match engine_log {
            EngineLogMessage::Info(msg) => LogMessage::info(msg),
            EngineLogMessage::Warning(msg) => LogMessage::warn(msg),
            EngineLogMessage::Error(msg) => LogMessage::error(msg),
            EngineLogMessage::Debug(msg) => LogMessage::debug(msg),
        };
        
        // Forward log message to server notifications for client broadcast
        // TODO: Changer pour faire un truc beau
        if let Some(ref sender) = self.notification_sender {
            // Wrap the LogMessage in a TimedMessage for the notification system
            let timed_message = TimedMessage {
                message: crate::protocol::message::ProtocolMessage {
                    device: std::sync::Arc::new(crate::protocol::device::ProtocolDevice::Log),
                    payload: crate::protocol::payload::ProtocolPayload::LOG(log_msg.clone()),
                },
                time: self.clock.micros(),
            };
            let notification = SovaNotification::Log(timed_message);
            let _ = sender.send(notification);
        }
        
        // Also send the log message to slot 0 (log device) for local handling
        let timed_message = TimedMessage {
            message: crate::protocol::message::ProtocolMessage {
                device: std::sync::Arc::new(crate::protocol::device::ProtocolDevice::Log),
                payload: crate::protocol::payload::ProtocolPayload::LOG(log_msg),
            },
            time: self.clock.micros(),
        };
        
        self.add_message(timed_message);
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
        let TimedMessage { message, time } = msg;
        match message.payload {
            ProtocolPayload::LOG(log_message) => {
                let log_output = match log_message.event {
                    Some(event) => match event {
                        ConcreteEvent::MidiNote(note, vel, chan, dur_micros, dev_id) => {
                            let dur_ms = dur_micros as f64 / 1000.0;
                            let dur_beats = self.clock.micros_to_beats(dur_micros);
                            format!(
                                "MidiNote(Note: {}, Vel: {}, Chan: {}, Dur: {:.1}ms / {:.2} beats, Dev: {})",
                                note, vel, chan, dur_ms, dur_beats, dev_id
                            )
                        }
                        _ => format!("{:?}", event),
                    },
                    None => log_message.msg,
                };

                let mut clock_time = self.clock.micros();
                let drift = clock_time.abs_diff(time);
                clock_time %= 60 * 1000 * 1000;
                let time = time % (60 * 1000 * 1000);

                log_println!(
                    "{} {} | Time : {clock_time} ; Wanted : {time} ; Drift : {drift}",
                    log_message.level, log_output,
                );
            }
            ProtocolPayload::AudioEngine(audio_payload) => {
                // Handle timebase calibration first, outside of any borrows
                let current_link_time = self.clock.micros();
                if current_link_time - self.timebase_calibration.last_calibration
                    > self.timebase_calibration_interval
                {
                    self.calibrate_timebase();
                }

                if let Some(ref tx) = self.audio_engine_tx {
                    let (engine_message, new_voice_id_counter) = self
                        .convert_audio_engine_payload_to_engine_message(
                            &audio_payload,
                            self.voice_id_counter,
                        );
                    self.voice_id_counter = new_voice_id_counter;

                    // LOOKAHEAD SCHEDULING: Send ALL messages early with their timestamp
                    // Use the actual musical time + lookahead for proper scheduling
                    let engine_due_time = time + self.non_midi_lookahead;
                    let scheduled_msg = ScheduledEngineMessage::Scheduled(ScheduledMessage {
                        due_time_micros: engine_due_time,
                        message: engine_message,
                    });
                    
                    let _ = tx.send(scheduled_msg);
                }
            }
            ProtocolPayload::MIDI(_) => {
                // MIDI early dispatch optimization - send early for interface compensation
                let current_sync_time = self.clock.micros();
                let time_until_execution = time.saturating_sub(current_sync_time);

                if time <= current_sync_time || time_until_execution <= self.midi_early_threshold {
                    // Send immediately for past messages or within MIDI threshold
                    let _ = message.send(time);
                } else {
                    // For future MIDI messages, send early to compensate for interface latency
                    let early_send_time = time.saturating_sub(self.midi_early_threshold);
                    let _ = message.send(early_send_time);
                }
            }
            ProtocolPayload::OSC(ref osc_msg) => {
                // Send OSC messages immediately for external system scheduling
                // External systems (SuperDirt, etc.) handle timing internally using the provided timestamp
                if osc_msg.addr.starts_with("/dirt/") || osc_msg.addr.contains("play") {
                    let current_sync_time = self.clock.micros();

                    // Calculate precise temporal context for SuperDirt
                    let cycle_duration_micros = 60_000_000.0 / self.clock.tempo(); // microseconds per cycle
                    let current_cycle = current_sync_time as f64 / cycle_duration_micros;
                    let target_cycle = time as f64 / cycle_duration_micros;
                    let _delta_cycles = target_cycle - current_cycle; // Future: can be used for cps/cycle/delta parameters

                    // Send immediately with original timestamp for SuperDirt internal scheduling
                    let _ = message.send(time);
                } else {
                    // Regular OSC: Send immediately with original timestamp
                    let _ = message.send(time);
                }
            }
            _ => {
                // Other protocols: Send with precise target timestamp
                let _ = message.send(time);
            }
        }
    }

    /// Calibrate the Link↔SystemTime offset with maximum precision
    fn calibrate_timebase(&mut self) {
        // Multi-sample calibration to minimize race condition uncertainty
        let mut best_offset = 0i64;
        let mut min_latency = u64::MAX;

        // Take 8 samples and use the one with minimum measurement latency
        for _ in 0..8 {
            let before_system = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64;
            let link_time = self.clock.micros();
            let after_system = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64;

            let measurement_latency = after_system - before_system;
            if measurement_latency < min_latency {
                min_latency = measurement_latency;
                // Interpolate to midpoint to minimize race condition
                let interpolated_system_time = before_system + (measurement_latency / 2);
                best_offset = interpolated_system_time as i64 - link_time as i64;
            }
        }

        self.timebase_calibration.link_to_system_offset = best_offset;
        self.timebase_calibration.last_calibration = self.clock.micros();
    }

    fn convert_audio_engine_payload_to_engine_message(
        &self,
        payload: &AudioEnginePayload,
        voice_id_counter: u32,
    ) -> (EngineMessage, u32) {
        use crate::protocol::osc::Argument;

        // Convert Argument array to string array for unified parser
        let mut string_args: Vec<String> = Vec::with_capacity(payload.0.len());
        
        for arg in &payload.0 {
            match arg {
                Argument::String(s) => string_args.push(s.clone()),
                Argument::Int(i) => string_args.push(i.to_string()),
                Argument::Float(f) => string_args.push(f.to_string()),
                Argument::Blob(_) | Argument::Timetag(_) => continue,
            }
        }
        
        // Convert to &str references
        let str_args: Vec<&str> = string_args.iter().map(|s| s.as_str()).collect();
        
        // Use the unified parser from registry
        if let Some((engine_msg, new_counter)) = self.registry.parse_unified_message(&str_args, voice_id_counter) {
            (engine_msg, new_counter)
        } else {
            // Fallback: create a no-op message if parsing fails
            (
                EngineMessage::Update {
                    voice_id: 0,
                    track_id: 0,
                    parameters: HashMap::new(),
                },
                voice_id_counter,
            )
        }
    }
}
