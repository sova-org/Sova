use std::{
    collections::BinaryHeap,
    sync::{
        Arc,
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
    },
    thread::JoinHandle,
    time::Duration,
};
use thread_priority::{ThreadBuilder, ThreadPriority};

use crate::lang::event::ConcreteEvent;
use crate::{
    clock::{Clock, ClockServer, SyncTime},
    protocol::{
        payload::{ProtocolPayload, AudioEnginePayload},
        message::TimedMessage,
    },
};
use bubo_engine::{types::EngineMessage, server::ScheduledEngineMessage, registry::ModuleRegistry};
use std::collections::HashMap;

const WORLD_TIME_MARGIN: u64 = 300;

pub struct World {
    queue: BinaryHeap<TimedMessage>,
    message_source: Receiver<TimedMessage>,
    next_timeout: Duration,
    clock: Clock,
    audio_engine_tx: Option<Sender<ScheduledEngineMessage>>,
    voice_id_counter: u32,
    registry: ModuleRegistry,
    shutdown_requested: bool,
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
                };
                world.live();
            })
            .expect("Unable to start World");
        (handle, tx)
    }

    pub fn live(&mut self) {
        let start_date = self.get_clock_micros();
        println!("[+] Starting world at {start_date}");
        loop {
            // Check for shutdown request
            if self.shutdown_requested {
                break;
            }
            
            let remaining = self
                .next_timeout
                .saturating_sub(Duration::from_micros(WORLD_TIME_MARGIN));
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

    fn handle_timed_message(&mut self, timed_message: TimedMessage) {
        // Check if this is a control message
        if let crate::protocol::payload::ProtocolPayload::Control(control_msg) = &timed_message.message.payload {
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

        // New time duration
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
                // DEBUG: Print all audio engine messages with their arguments
                println!("[DEBUG AUDIO ENGINE] Device: {}, Args count: {}", 
                    audio_payload.device_id, 
                    audio_payload.args.len());
                println!("  Arguments:");
                for (i, arg) in audio_payload.args.iter().enumerate() {
                    println!("    [{}] {:?}", i, arg);
                }
                
                if let Some(ref tx) = self.audio_engine_tx {
                    let (engine_message, new_voice_id_counter) = self.convert_audio_engine_payload_to_engine_message(
                        &audio_payload,
                        self.voice_id_counter,
                    );
                    self.voice_id_counter = new_voice_id_counter;
                    let scheduled_msg = ScheduledEngineMessage::Immediate(engine_message);
                    let _ = tx.send(scheduled_msg);
                }
            }
            _ => {
                let _ = message.send(self.get_clock_micros());
            }
        }
    }

    fn get_clock_micros(&self) -> SyncTime {
        self.clock.micros()
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
                        },
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
        let parameters = self.registry.normalize_parameters(raw_parameters, Some(&source_name));

        // Extract track_id from parameters (let engine handle defaults)
        let track_id = parameters.get("track")
            .and_then(|t| t.downcast_ref::<f32>())
            .map(|&f| f as u8)
            .unwrap_or(0);  // Default to track 0
        
        // Always create new voice (simplified - no voice_id tracking)
        let voice_id = voice_id_counter;
        let new_voice_id_counter = voice_id_counter.wrapping_add(1);
        
        (EngineMessage::Play {
            voice_id,
            track_id,
            source_name,
            parameters,
        }, new_voice_id_counter)
    }
}
