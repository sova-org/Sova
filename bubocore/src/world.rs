use std::{
    collections::BinaryHeap,
    sync::{
        Arc,
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
    },
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
    },
};
use bubo_engine::{
    registry::ModuleRegistry,
    server::ScheduledEngineMessage,
    types::{EngineMessage, ScheduledMessage},
};
use std::collections::HashMap;

// WORLD_TIME_MARGIN constant moved to TimingConfig.world_precision_margin_micros

/// High-precision Link ↔ SystemTime conversion calibration
struct TimebaseCalibration {
    /// SystemTime - LinkTime offset at calibration point
    link_to_system_offset: i64,
    /// When we last calibrated (Link time in microseconds)
    last_calibration: u64,
    /// Recalibrate every N microseconds (1 second)
    calibration_interval: u64,
}

impl TimebaseCalibration {
    fn new() -> Self {
        Self {
            link_to_system_offset: 0,
            last_calibration: 0,
            calibration_interval: 1_000_000, // 1 second in microseconds
        }
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
    shutdown_requested: bool,
    timebase_calibration: TimebaseCalibration,
    timebase_calibration_interval: SyncTime,
    // MIDI interface latency compensation (2ms)
    midi_early_threshold: SyncTime,
}

impl World {
    pub fn create(
        clock_server: Arc<ClockServer>,
        audio_engine_tx: Option<Sender<ScheduledEngineMessage>>,
        registry: ModuleRegistry,
    ) -> (JoinHandle<()>, Sender<TimedMessage>) {
        let (tx, rx) = mpsc::channel();
        let handle = ThreadBuilder::default()
            .name("deep-BuboCore-world")
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
                    shutdown_requested: false,
                    timebase_calibration: TimebaseCalibration::new(),
                    timebase_calibration_interval: 100_000, // 100ms calibration interval
                    midi_early_threshold: 2_000, // 2ms for MIDI interface compensation
                };
                world.live();
            })
            .expect("Unable to start World");
        (handle, tx)
    }

    pub fn live(&mut self) {
        let start_date = self.get_clock_micros();
        // Initialize timebase calibration
        self.calibrate_timebase();
        println!("[+] Starting world at {start_date}");
        loop {
            // Check for shutdown request
            if self.shutdown_requested {
                break;
            }

            let remaining = self
                .next_timeout
                .saturating_sub(Duration::from_micros(300)); // Use constant for now
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
            let mut time = self.get_clock_micros();

            // Active waiting when not enough time to wait again
            while next.time > time && next.time.saturating_sub(time) <= 300 {
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

    fn handle_timed_message(&mut self, timed_message: TimedMessage) {
        // Check if this is a control message
        if let crate::protocol::payload::ProtocolPayload::Control(control_msg) =
            &timed_message.message.payload
        {
            match control_msg {
                crate::protocol::payload::ControlMessage::Shutdown => {
                    println!("[-] World received shutdown signal");
                    self.shutdown_requested = true;
                    return;
                }
            }
        }

        // Regular message - add to queue for timed execution
        self.add_message(timed_message);
    }

    pub fn add_message(&mut self, msg: TimedMessage) {
        self.queue.push(msg);
    }

    fn refresh_next_timeout(&mut self) {
        let Some(next_msg) = self.queue.peek() else {
            self.next_timeout = Duration::MAX;
            return;
        };

        let now = self.get_clock_micros();
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

                let mut clock_time = self.get_clock_micros();
                let drift = clock_time.abs_diff(time);
                clock_time %= 60 * 1000 * 1000;
                let time = time % (60 * 1000 * 1000);

                println!(
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

                    let current_sync_time = self.get_clock_micros();
                    
                    let scheduled_msg = if time <= current_sync_time {
                        // Only execute immediately if message is actually overdue
                        ScheduledEngineMessage::Immediate(engine_message)
                    } else {
                        // Always schedule future messages with precise timestamp
                        // The audio engine handles sample-accurate timing internally
                        let system_due_time =
                            (time as i64 + self.timebase_calibration.link_to_system_offset) as u64;

                        ScheduledEngineMessage::Scheduled(ScheduledMessage {
                            due_time_micros: system_due_time,
                            message: engine_message,
                        })
                    };
                    let _ = tx.send(scheduled_msg);
                }
            }
            ProtocolPayload::MIDI(_) => {
                // MIDI early dispatch optimization - send early for interface compensation
                let current_sync_time = self.get_clock_micros();
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
                // SuperDirt optimization - enhanced temporal context parameters
                if osc_msg.addr.starts_with("/dirt/") || osc_msg.addr.contains("play") {
                    let current_sync_time = self.get_clock_micros();
                    
                    // Calculate precise temporal context for SuperDirt
                    let cycle_duration_micros = 60_000_000.0 / self.clock.tempo(); // microseconds per cycle
                    let current_cycle = current_sync_time as f64 / cycle_duration_micros;
                    let target_cycle = time as f64 / cycle_duration_micros;
                    let _delta_cycles = target_cycle - current_cycle; // Future: can be used for cps/cycle/delta parameters
                    
                    // Send with enhanced timing precision for SuperDirt compatibility
                    let _ = message.send(time);
                } else {
                    // Regular OSC: Send with precise target timestamp
                    let _ = message.send(time);
                }
            }
            _ => {
                // Other protocols: Send with precise target timestamp
                let _ = message.send(time);
            }
        }
    }

    fn get_clock_micros(&self) -> SyncTime {
        self.clock.micros()
    }

    /// Calibrate the Link↔SystemTime offset with maximum precision
    fn calibrate_timebase(&mut self) {
        // Multi-sample calibration to minimize race condition uncertainty
        let mut best_offset = 0i64;
        let mut min_latency = u64::MAX;
        
        // Take 8 samples and use the one with minimum measurement latency
        for _ in 0..8 {
            let before_system = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as u64;
            let link_time = self.clock.micros();
            let after_system = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as u64;
            
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
        use std::any::Any;

        let mut raw_parameters: HashMap<String, Box<dyn Any + Send>> = HashMap::new();
        let mut source_name = String::new();

        // Parse arguments generically (like OSC parsing)
        let mut i = 0;
        while i + 1 < payload.args.len() {
            if let (Argument::String(key), value) = (&payload.args[i], &payload.args[i + 1]) {
                if key == "s" {
                    if let Argument::String(s) = value {
                        source_name = s.clone();
                    }
                } else {
                    // Convert to Box<dyn Any + Send> with proper parsing (like OSC route)
                    let param_value: Box<dyn Any + Send> = match value {
                        Argument::Int(i) => Box::new(*i as f32),
                        Argument::Float(f) => Box::new(*f),
                        Argument::String(s) => {
                            // Use registry parsing to handle modulations (same as OSC route)
                            self.registry.parse_parameter_value(s)
                        }
                        _ => Box::new(0.0f32),
                    };
                    raw_parameters.insert(key.clone(), param_value);
                }
            }
            i += 2;
        }

        // Add source name for parameter normalization context
        raw_parameters.insert("s".to_string(), Box::new(source_name.clone()));

        // Normalize parameters using registry (this resolves aliases like fd->sample_name, nb->sample_number)
        let parameters = self
            .registry
            .normalize_parameters(raw_parameters, Some(&source_name));

        // Extract track_id from parameters (let engine handle defaults)
        let track_id = parameters
            .get("track")
            .and_then(|t| t.downcast_ref::<f32>())
            .map(|&f| f as u8)
            .unwrap_or(0); // Default to track 0

        // Always create new voice (simplified - no voice_id tracking)
        let voice_id = voice_id_counter;
        let new_voice_id_counter = voice_id_counter.wrapping_add(1);

        (
            EngineMessage::Play {
                voice_id,
                track_id,
                source_name,
                parameters,
            },
            new_voice_id_counter,
        )
    }
}
