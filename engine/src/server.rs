use crate::memory::{SampleLibrary, VoiceMemory};
use crate::modulation::Modulation;
use crate::registry::{ENGINE_PARAM_DESCRIPTORS, ModuleRegistry};
use crate::types::{EngineMessage, ScheduledMessage, TrackId, VoiceId};
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
    sample_library: Arc<std::sync::Mutex<SampleLibrary>>,
    receive_buffer: [u8; 4096],
    string_buffer: [u8; 1024],
}

impl OscServer {
    pub fn new(
        host: &str,
        port: u16,
        registry: ModuleRegistry,
        voice_memory: Arc<VoiceMemory>,
        sample_library: Arc<std::sync::Mutex<SampleLibrary>>,
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

                let due_timestamp =
                    if let Ok(timestamp) = self.registry.validate_timestamp(&parameters) {
                        parameters.remove("due");
                        Some(timestamp)
                    } else if parameters.contains_key("due") {
                        println!("Message rejected - timestamp validation failed");
                        return None;
                    } else {
                        None
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

                if let Some(due_time_ms) = due_timestamp {
                    let scheduled_msg = ScheduledMessage {
                        due_time_ms,
                        message: engine_message,
                    };
                    Some(ScheduledEngineMessage::Scheduled(scheduled_msg))
                } else {
                    Some(ScheduledEngineMessage::Immediate(engine_message))
                }
            }
            "/update" => {
                let mut parameters = self.parse_osc_parameters(&msg.args);

                let due_timestamp =
                    if let Ok(timestamp) = self.registry.validate_timestamp(&parameters) {
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

                if let Some(due_time_ms) = due_timestamp {
                    Some(ScheduledEngineMessage::Scheduled(ScheduledMessage {
                        due_time_ms,
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
                            let param_value = self.parse_parameter_value(val);
                            raw_parameters.insert(key.clone(), param_value);
                        }
                    }
                    _ => {}
                }
            }
            i += 2;
        }

        let mut normalized_parameters = HashMap::with_capacity(raw_parameters.len());
        for (key, value) in raw_parameters {
            if key == "s" {
                normalized_parameters.insert(key, value);
            } else {
                let normalized_key = self.normalize_parameter_name(&key, source_name.as_ref());
                if self.is_valid_parameter(normalized_key, source_name.as_ref()) {
                    normalized_parameters.insert(normalized_key.to_string(), value);
                    println!("  {} -> {} (normalized)", key, normalized_key);
                } else {
                    println!("  {} = <invalid parameter>", key);
                }
            }
        }

        normalized_parameters
    }

    fn parse_play_message(&mut self, parts: &[&str]) -> Option<ScheduledEngineMessage> {
        if parts.is_empty() {
            return None;
        }

        let mut track_id = 1;
        let mut param_start = 0;

        if !parts[0].chars().all(|c| c.is_alphabetic()) {
            if let Ok(tid) = parts[0].parse() {
                track_id = tid;
                param_start = 1;
            }
        }

        let voice_id = self.next_voice_id;
        self.next_voice_id += 1;

        let mut parameters = self.parse_parameters(&parts[param_start..]);
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

        let due_timestamp = if let Ok(timestamp) = self.registry.validate_timestamp(&parameters) {
            parameters.remove("due");
            Some(timestamp)
        } else if parameters.contains_key("due") {
            return None;
        } else {
            None
        };

        let engine_message = EngineMessage::Play {
            voice_id,
            track_id,
            source_name,
            parameters,
        };

        if let Some(due_time_ms) = due_timestamp {
            Some(ScheduledEngineMessage::Scheduled(ScheduledMessage {
                due_time_ms,
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

        let due_timestamp = if let Ok(timestamp) = self.registry.validate_timestamp(&parameters) {
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

        if let Some(due_time_ms) = due_timestamp {
            Some(ScheduledEngineMessage::Scheduled(ScheduledMessage {
                due_time_ms,
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
                let param_value = self.parse_parameter_value(value_str);
                raw_parameters.insert(key.to_string(), param_value);
            }

            i += 2;
        }

        let mut normalized_parameters = HashMap::with_capacity(raw_parameters.len());
        for (key, value) in raw_parameters {
            if key == "s" {
                normalized_parameters.insert(key, value);
            } else {
                let normalized_key = self.normalize_parameter_name(&key, source_name.as_ref());
                if self.is_valid_parameter(normalized_key, source_name.as_ref()) {
                    normalized_parameters.insert(normalized_key.to_string(), value);
                } else {
                    println!("  {} = <invalid parameter>", key);
                }
            }
        }

        normalized_parameters
    }

    fn normalize_parameter_name(&self, param: &str, source_name: Option<&String>) -> &'static str {
        for desc in &ENGINE_PARAM_DESCRIPTORS {
            if desc.name == param {
                return desc.name;
            }
            for alias in desc.aliases {
                if *alias == param {
                    return desc.name;
                }
            }
        }

        if let Some(source) = source_name {
            if self.registry.sources.contains_key(source) {
                let module = self.registry.sources.get(source).unwrap()();
                for desc in module.get_parameter_descriptors() {
                    if desc.name == param {
                        return desc.name;
                    }
                    for alias in desc.aliases {
                        if *alias == param {
                            return desc.name;
                        }
                    }
                }
            }
        }

        for factory in self.registry.local_effects.values() {
            let module = factory();
            for desc in module.get_parameter_descriptors() {
                if desc.name == param {
                    return desc.name;
                }
                for alias in desc.aliases {
                    if *alias == param {
                        return desc.name;
                    }
                }
            }
        }

        for factory in self.registry.global_effects.values() {
            let module = factory();
            for desc in module.get_parameter_descriptors() {
                if desc.name == param {
                    return desc.name;
                }
                for alias in desc.aliases {
                    if *alias == param {
                        return desc.name;
                    }
                }
            }
        }

        Box::leak(param.to_string().into_boxed_str())
    }

    fn is_valid_parameter(&self, param_name: &str, source_name: Option<&String>) -> bool {
        for desc in &ENGINE_PARAM_DESCRIPTORS {
            if desc.name == param_name {
                return true;
            }
            for alias in desc.aliases {
                if *alias == param_name {
                    return true;
                }
            }
        }

        if let Some(source) = source_name {
            if self.registry.sources.contains_key(source) {
                let module = self.registry.sources.get(source).unwrap()();
                for desc in module.get_parameter_descriptors() {
                    if desc.name == param_name {
                        return true;
                    }
                    for alias in desc.aliases {
                        if *alias == param_name {
                            return true;
                        }
                    }
                }
            }
        }

        for factory in self.registry.local_effects.values() {
            let module = factory();
            for desc in module.get_parameter_descriptors() {
                if desc.name == param_name {
                    return true;
                }
                for alias in desc.aliases {
                    if *alias == param_name {
                        return true;
                    }
                }
            }
        }

        for factory in self.registry.global_effects.values() {
            let module = factory();
            for desc in module.get_parameter_descriptors() {
                if desc.name == param_name {
                    return true;
                }
                for alias in desc.aliases {
                    if *alias == param_name {
                        return true;
                    }
                }
            }
        }

        // Check generic wet parameters for global effects
        if self.registry.is_global_effect_wet_parameter(param_name).is_some() {
            return true;
        }

        false
    }

    fn parse_parameter_value(&self, value: &str) -> Box<dyn Any + Send> {
        if value.contains(':') {
            Box::new(Modulation::parse(value))
        } else if let Ok(float_val) = value.parse::<f32>() {
            Box::new(float_val)
        } else {
            Box::new(value.to_string())
        }
    }

    fn print_samples(&self) {
        println!("=== Sample Library Status ===");

        if let Ok(sample_lib) = self.sample_library.try_lock() {
            let folders = sample_lib.get_all_folders();

            if folders.is_empty() {
                println!("No sample directories found");
                return;
            }

            println!("Found {} sample directories:", folders.len());

            for (index, (folder_name, total_samples, loaded_samples)) in folders.iter().enumerate()
            {
                println!(
                    "  [{}] {} ({}/{} loaded)",
                    index, folder_name, loaded_samples, total_samples
                );

                let folder_contents = sample_lib.get_folder_contents(folder_name);
                if folder_contents.is_empty() {
                    println!("      (no .wav files)");
                } else {
                    for (sample_idx, file_name) in folder_contents.iter().enumerate() {
                        println!("      [{}] {}", sample_idx, file_name);
                    }
                }
            }
        } else {
            println!("Sample library is currently locked");
        }

        println!("=== Usage Examples ===");
        println!("  /play s sample sample_name kick sample_number 0");
        println!("  /play s sample sample_name bass sample_number 2.5");
        println!("==============================");
    }
}
