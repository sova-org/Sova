use crate::memory::{SampleLibrary, VoiceMemory};
use crate::registry::ModuleRegistry;
use crate::types::{EngineMessage, ScheduledMessage, TrackId, VoiceId};

// Real-time safe logging - local macro
#[cfg(feature = "rt-safe")]
macro_rules! rt_eprintln {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "rt-safe"))]
macro_rules! rt_eprintln {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    };
}
use crossbeam_channel::Sender;
use rosc::{OscMessage, OscPacket, OscType};
use std::any::Any;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

pub enum ScheduledEngineMessage {
    Immediate(EngineMessage),
    Scheduled(ScheduledMessage),
}

pub enum EngineChannelMessage {
    Command(ScheduledEngineMessage),
    StatusRequest,
}

pub struct OscServer {
    socket: UdpSocket,
    next_voice_id: VoiceId,
    registry: ModuleRegistry,
    #[allow(dead_code)]
    voice_memory: Arc<VoiceMemory>,
    sample_library: Arc<SampleLibrary>,
    receive_buffer: [u8; 4096],
    string_buffer: [u8; 1024],
}

impl OscServer {
    pub fn new(
        host: &str,
        port: u16,
        registry: ModuleRegistry,
        voice_memory: Arc<VoiceMemory>,
        sample_library: Arc<SampleLibrary>,
    ) -> Result<Self, String> {
        let addr = format!("{}:{}", host, port);
        let socket = UdpSocket::bind(&addr)
            .map_err(|e| format!("Failed to bind OSC server to {}: {}", addr, e))?;

        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| format!("Failed to set socket timeout: {}", e))?;

        println!("OSC server listening on {}", addr);

        Ok(Self {
            socket,
            next_voice_id: 0,
            registry,
            voice_memory,
            sample_library,
            receive_buffer: [0u8; 4096],
            string_buffer: [0u8; 1024],
        })
    }

    /// Lock-free OSC server using crossbeam channels
    pub fn run_lockfree(&mut self, engine_tx: Sender<ScheduledEngineMessage>) {
        println!("Starting lock-free OSC server...");

        loop {
            match self.socket.recv_from(&mut self.receive_buffer) {
                Ok((size, addr)) => {
                    println!("OSC message received from {}: {} bytes", addr, size);

                    let mut temp_buffer = [0u8; 1024];
                    temp_buffer[..size].copy_from_slice(&self.receive_buffer[..size]);

                    if let Some(message) = self.parse_osc_message(&temp_buffer[..size]) {
                        // Send to audio thread (bounded channel prevents blocking)
                        if let Err(_) = engine_tx.try_send(message) {
                            rt_eprintln!("[OSC WARNING] Command queue full - dropping message");
                        }
                    }
                }
                Err(err) => {
                    if err.kind() != std::io::ErrorKind::TimedOut {
                        rt_eprintln!("[OSC ERROR] Failed to receive: {}", err);
                    }
                }
            }
        }
    }

    pub fn run(&mut self, engine_tx: mpsc::Sender<ScheduledEngineMessage>) {
        loop {
            match self.socket.recv_from(&mut self.receive_buffer) {
                Ok((size, addr)) => {
                    println!("OSC message received from {}: {} bytes", addr, size);

                    let mut temp_buffer = [0u8; 1024];
                    temp_buffer[..size].copy_from_slice(&self.receive_buffer[..size]);

                    if let Some(message) = self.parse_osc_message(&temp_buffer[..size]) {
                        let _ = engine_tx.send(message);
                    }
                }
                Err(_) => continue,
            }
        }
    }

    fn parse_osc_packet(&mut self, data: &[u8]) -> Option<ScheduledEngineMessage> {
        self.parse_osc_message(data)
    }

    fn parse_osc_message(&mut self, data: &[u8]) -> Option<ScheduledEngineMessage> {
        match rosc::decoder::decode_udp(data) {
            Ok((_, packet)) => match packet {
                OscPacket::Message(msg) => {
                    println!("OSC Message - Address: {}, Args: {:?}", msg.addr, msg.args);
                    self.handle_osc_message(msg)
                }
                OscPacket::Bundle(bundle) => {
                    println!("OSC Bundle with {} messages", bundle.content.len());
                    for packet in bundle.content {
                        if let OscPacket::Message(msg) = packet {
                            println!(
                                "  Bundle Message - Address: {}, Args: {:?}",
                                msg.addr, msg.args
                            );
                            if let Some(scheduled_msg) = self.handle_osc_message(msg) {
                                return Some(scheduled_msg);
                            }
                        }
                    }
                    None
                }
            },
            Err(_) => {
                let text_copy = if let Some(message_str) = self.parse_as_text(data) {
                    message_str.to_string()
                } else {
                    return None;
                };

                let parts = self.split_string(&text_copy);
                if parts.is_empty() {
                    return None;
                }

                match parts[0] {
                    "/play" => self.parse_play_message(&parts[1..]),
                    "/update" => self.parse_update_message(&parts[1..]),
                    "/stop" => Some(ScheduledEngineMessage::Immediate(EngineMessage::Stop)),
                    "/panic" => Some(ScheduledEngineMessage::Immediate(EngineMessage::Panic)),
                    "/samples" => {
                        self.print_samples();
                        None
                    }
                    _ => None,
                }
            }
        }
    }

    fn parse_as_text(&mut self, data: &[u8]) -> Option<&str> {
        let len = data.len().min(self.string_buffer.len());
        self.string_buffer[..len].copy_from_slice(&data[..len]);
        std::str::from_utf8(&self.string_buffer[..len]).ok()
    }

    fn split_string<'a>(&self, input: &'a str) -> [&'a str; 32] {
        let mut parts = [""; 32];
        let mut count = 0;
        for part in input.split_whitespace() {
            if count < 32 {
                parts[count] = part;
                count += 1;
            } else {
                break;
            }
        }
        parts
    }

    fn handle_osc_message(&mut self, msg: OscMessage) -> Option<ScheduledEngineMessage> {
        match msg.addr.as_str() {
            "/play" => {
                let voice_id = self.next_voice_id;
                self.next_voice_id += 1;

                let mut parameters = self.parse_osc_parameters(&msg.args);

                let due_timestamp = if let Ok(timestamp) = self
                    .registry
                    .validate_timestamp_deterministic(&parameters, 0)
                {
                    parameters.remove("due");
                    Some(timestamp)
                } else if parameters.contains_key("due") {
                    println!("Message rejected - timestamp validation failed");
                    return None;
                } else {
                    None
                };

                let track_id = parameters
                    .get("track")
                    .and_then(|t| t.downcast_ref::<f32>())
                    .map(|&f| f as TrackId)
                    .unwrap_or(1);

                let source_name = if let Some(s) = parameters.remove("s") {
                    if let Some(s_str) = s.downcast_ref::<String>() {
                        println!(
                            "OSC: Playing source '{}' with {} parameters",
                            s_str,
                            parameters.len()
                        );
                        s_str.clone()
                    } else {
                        println!("Error: Invalid source parameter type");
                        return None;
                    }
                } else {
                    println!("Error: No source specified in /play message");
                    return None;
                };

                if !parameters.contains_key("dur") {
                    parameters.insert("dur".to_string(), Box::new(1.0f32) as Box<dyn Any + Send>);
                }

                let engine_message = EngineMessage::Play {
                    voice_id,
                    track_id,
                    source_name,
                    parameters,
                };

                if let Some(due_time_micros) = due_timestamp {
                    let scheduled_msg = ScheduledMessage {
                        due_time_micros,
                        message: engine_message,
                    };
                    Some(ScheduledEngineMessage::Scheduled(scheduled_msg))
                } else {
                    Some(ScheduledEngineMessage::Immediate(engine_message))
                }
            }
            "/update" => {
                let mut parameters = self.parse_osc_parameters(&msg.args);

                let due_timestamp = if let Ok(timestamp) = self
                    .registry
                    .validate_timestamp_deterministic(&parameters, 0)
                {
                    parameters.remove("due");
                    Some(timestamp)
                } else if parameters.contains_key("due") {
                    println!("Update message rejected - timestamp validation failed");
                    return None;
                } else {
                    None
                };

                let voice_id = if let Some(voice) = parameters.remove("voice") {
                    if let Some(voice_val) = voice.downcast_ref::<f32>() {
                        *voice_val as VoiceId
                    } else {
                        return None;
                    }
                } else {
                    return None;
                };

                let track_id = if let Some(track) = parameters.remove("track") {
                    if let Some(track_val) = track.downcast_ref::<f32>() {
                        *track_val as TrackId
                    } else {
                        1
                    }
                } else {
                    1
                };

                let engine_message = EngineMessage::Update {
                    voice_id,
                    track_id,
                    parameters,
                };

                if let Some(due_time_micros) = due_timestamp {
                    Some(ScheduledEngineMessage::Scheduled(ScheduledMessage {
                        due_time_micros,
                        message: engine_message,
                    }))
                } else {
                    Some(ScheduledEngineMessage::Immediate(engine_message))
                }
            }
            "/stop" => Some(ScheduledEngineMessage::Immediate(EngineMessage::Stop)),
            "/panic" => Some(ScheduledEngineMessage::Immediate(EngineMessage::Panic)),
            "/samples" => {
                self.print_samples();
                None
            }
            _ => {
                println!("Unknown OSC address: {}", msg.addr);
                None
            }
        }
    }

    fn parse_osc_parameters(&self, args: &[OscType]) -> HashMap<String, Box<dyn Any + Send>> {
        let mut raw_parameters = HashMap::with_capacity(16);
        let mut source_name = None;

        let mut i = 0;
        while i + 1 < args.len() {
            if let (OscType::String(key), value) = (&args[i], &args[i + 1]) {
                match value {
                    OscType::Int(val) => {
                        raw_parameters
                            .insert(key.clone(), Box::new(*val as f32) as Box<dyn Any + Send>);
                    }
                    OscType::Float(val) => {
                        raw_parameters.insert(key.clone(), Box::new(*val) as Box<dyn Any + Send>);
                    }
                    OscType::String(val) => {
                        if key == "s" {
                            source_name = Some(val.clone());
                            raw_parameters
                                .insert(key.clone(), Box::new(val.clone()) as Box<dyn Any + Send>);
                        } else {
                            let param_value = self.registry.parse_parameter_value(val);
                            raw_parameters.insert(key.clone(), param_value);
                        }
                    }
                    _ => {}
                }
            }
            i += 2;
        }

        self.registry
            .normalize_parameters(raw_parameters, source_name.as_ref())
    }

    fn parse_play_message(&mut self, parts: &[&str]) -> Option<ScheduledEngineMessage> {
        if parts.is_empty() {
            return None;
        }

        let mut parameters = self.parse_parameters(parts);

        let voice_id = if let Some(voice_param) = parameters
            .remove("id")
            .or_else(|| parameters.remove("voice"))
            .or_else(|| parameters.remove("v"))
        {
            if let Some(voice_str) = voice_param.downcast_ref::<String>() {
                if voice_str == "s" {
                    // Auto-assign using server's voice counter
                    let id = self.next_voice_id;
                    self.next_voice_id += 1;
                    id
                } else if let Ok(explicit_id) = voice_str.parse::<u32>() {
                    explicit_id
                } else {
                    // Invalid voice ID, use auto-assignment
                    let id = self.next_voice_id;
                    self.next_voice_id += 1;
                    id
                }
            } else {
                // Invalid type, use auto-assignment
                let id = self.next_voice_id;
                self.next_voice_id += 1;
                id
            }
        } else {
            // No voice ID specified, use auto-assignment
            let id = self.next_voice_id;
            self.next_voice_id += 1;
            id
        };
        let source_name = if let Some(s) = parameters.remove("s") {
            if let Some(s_str) = s.downcast_ref::<String>() {
                match s_str.as_str() {
                    "sine" => "sine_oscillator".to_string(),
                    name => name.to_string(),
                }
            } else {
                return None;
            }
        } else {
            return None;
        };

        if !parameters.contains_key("dur") {
            parameters.insert("dur".to_string(), Box::new(1.0f32) as Box<dyn Any + Send>);
        }

        let due_timestamp = if let Ok(timestamp) = self
            .registry
            .validate_timestamp_deterministic(&parameters, 0)
        {
            parameters.remove("due");
            Some(timestamp)
        } else if parameters.contains_key("due") {
            return None;
        } else {
            None
        };

        let track_id = parameters
            .get("track")
            .and_then(|t| t.downcast_ref::<f32>())
            .map(|&f| f as TrackId)
            .unwrap_or(1);

        let engine_message = EngineMessage::Play {
            voice_id,
            track_id,
            source_name,
            parameters,
        };

        if let Some(due_time_micros) = due_timestamp {
            Some(ScheduledEngineMessage::Scheduled(ScheduledMessage {
                due_time_micros,
                message: engine_message,
            }))
        } else {
            Some(ScheduledEngineMessage::Immediate(engine_message))
        }
    }

    fn parse_update_message(&self, parts: &[&str]) -> Option<ScheduledEngineMessage> {
        if parts.len() < 3 {
            return None;
        }

        let voice_id: VoiceId = parts[0].parse().ok()?;
        let track_id: TrackId = parts[1].parse().ok()?;

        let mut parameters = self.parse_parameters(&parts[2..]);

        let due_timestamp = if let Ok(timestamp) = self
            .registry
            .validate_timestamp_deterministic(&parameters, 0)
        {
            parameters.remove("due");
            Some(timestamp)
        } else if parameters.contains_key("due") {
            return None;
        } else {
            None
        };

        let engine_message = EngineMessage::Update {
            voice_id,
            track_id,
            parameters,
        };

        if let Some(due_time_micros) = due_timestamp {
            Some(ScheduledEngineMessage::Scheduled(ScheduledMessage {
                due_time_micros,
                message: engine_message,
            }))
        } else {
            Some(ScheduledEngineMessage::Immediate(engine_message))
        }
    }

    fn parse_parameters(&self, parts: &[&str]) -> HashMap<String, Box<dyn Any + Send>> {
        let mut raw_parameters = HashMap::with_capacity(16);
        let mut source_name = None;

        let mut i = 0;
        while i + 1 < parts.len() {
            let key = parts[i];
            let value_str = parts[i + 1];

            if key == "s" {
                source_name = Some(value_str.to_string());
                raw_parameters.insert(
                    key.to_string(),
                    Box::new(value_str.to_string()) as Box<dyn Any + Send>,
                );
            } else {
                let param_value = self.registry.parse_parameter_value(value_str);
                raw_parameters.insert(key.to_string(), param_value);
            }

            i += 2;
        }

        self.registry
            .normalize_parameters(raw_parameters, source_name.as_ref())
    }

    fn print_samples(&self) {
        println!("=== Sample Library Status ===");

        let folders = self.sample_library.get_all_folders();

        if folders.is_empty() {
            println!("No sample directories found");
            return;
        }

        println!("Found {} sample directories:", folders.len());

        for (index, (folder_name, total_samples, loaded_samples)) in folders.iter().enumerate() {
            println!(
                "  [{}] {} ({}/{} loaded)",
                index, folder_name, loaded_samples, total_samples
            );

            let folder_contents = self.sample_library.get_folder_contents(folder_name);
            if folder_contents.is_empty() {
                println!("      (no .wav files)");
            } else {
                for (sample_idx, file_name) in folder_contents.iter().enumerate() {
                    println!("      [{}] {}", sample_idx, file_name);
                }
            }
        }

        println!("=== Usage Examples ===");
        println!("  /play s sample sample_name kick sample_number 0");
        println!("  /play s sample sample_name bass sample_number 2.5");
        println!("==============================");
    }
}
