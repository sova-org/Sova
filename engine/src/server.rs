use crate::constants::{OSC_STRING_BUFFER_SIZE};
use crate::memory::{SampleLibrary, VoiceMemory};
use crate::registry::ModuleRegistry;
use crate::types::{EngineMessage, ScheduledMessage, TrackId, VoiceId, LoggerHandle};

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
use std::sync::mpsc;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

#[derive(Debug)]
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
    voice_id_counter: VoiceId,
    registry: ModuleRegistry,
    #[allow(dead_code)]
    voice_memory: Arc<VoiceMemory>,
    sample_library: Arc<SampleLibrary>,
    receive_buffer: [u8; 4096],
    string_buffer: [u8; 1024],
    shutdown_flag: Arc<AtomicBool>,
    logger: LoggerHandle,
}

impl OscServer {
    pub fn new(
        host: &str,
        port: u16,
        registry: ModuleRegistry,
        voice_memory: Arc<VoiceMemory>,
        sample_library: Arc<SampleLibrary>,
        shutdown_flag: Arc<AtomicBool>,
        logger: LoggerHandle,
    ) -> Result<Self, String> {
        let addr = format!("{}:{}", host, port);
        let socket = UdpSocket::bind(&addr)
            .map_err(|e| format!("Failed to bind OSC server to {}: {}", addr, e))?;

        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| format!("Failed to set socket timeout: {}", e))?;

        logger.log_info(&format!("OSC server listening on {}", addr));

        Ok(Self {
            socket,
            voice_id_counter: 0,
            registry,
            voice_memory,
            sample_library,
            receive_buffer: [0u8; 4096],
            string_buffer: [0u8; 1024],
            shutdown_flag,
            logger,
        })
    }

    /// Lock-free OSC server using crossbeam channels
    pub fn run_lockfree(&mut self, engine_tx: Sender<ScheduledEngineMessage>) {
        loop {
            if self.shutdown_flag.load(Ordering::Relaxed) {
                break;
            }

            match self.socket.recv_from(&mut self.receive_buffer) {
                Ok((size, _addr)) => {
                    let mut temp_buffer = [0u8; OSC_STRING_BUFFER_SIZE];
                    temp_buffer[..size].copy_from_slice(&self.receive_buffer[..size]);

                    if let Some(message) = self.parse_osc_message(&temp_buffer[..size]) {
                        if engine_tx.try_send(message).is_err() {
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
                Ok((size, _)) => {
                    let mut temp_buffer = [0u8; OSC_STRING_BUFFER_SIZE];
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
                OscPacket::Message(msg) => self.handle_unified_message(&msg),
                OscPacket::Bundle(bundle) => {
                    for packet in bundle.content {
                        if let OscPacket::Message(msg) = packet {
                            if let Some(scheduled_msg) = self.handle_unified_message(&msg) {
                                return Some(scheduled_msg);
                            }
                        }
                    }
                    None
                }
            },
            Err(_) => {
                let parsed_text = self.parse_as_text(data);
                let text_copy = parsed_text?.to_string();

                let parts = self.split_string(&text_copy);
                if parts.is_empty() {
                    return None;
                }

                // Use unified command handling for text commands too
                self.handle_unified_command(parts[0], &parts[1..])
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

    /// Convert OSC arguments to string parts for unified parsing
    fn osc_args_to_string_parts(&self, args: &[OscType]) -> Vec<String> {
        args.iter()
            .filter_map(|arg| match arg {
                OscType::String(s) => Some(s.clone()),
                OscType::Int(i) => Some(i.to_string()),
                OscType::Float(f) => Some(f.to_string()),
                OscType::Double(d) => Some(d.to_string()),
                OscType::Bool(b) => Some(b.to_string()),
                OscType::Long(l) => Some(l.to_string()),
                _ => {
                    self.logger.log_warning(&format!("Unsupported OSC argument type: {:?}", arg));
                    None
                }
            })
            .collect()
    }

    /// Unified message handler for both OSC and text commands
    fn handle_unified_message(&mut self, msg: &OscMessage) -> Option<ScheduledEngineMessage> {
        let string_args = self.osc_args_to_string_parts(&msg.args);
        let str_refs: Vec<&str> = string_args.iter().map(|s| s.as_str()).collect();
        self.handle_unified_command(&msg.addr, &str_refs)
    }

    /// Unified command handler that processes all commands the same way
    fn handle_unified_command(&mut self, command: &str, args: &[&str]) -> Option<ScheduledEngineMessage> {
        match command {
            "/play" => {
                // Use the unified parser from registry
                if let Some((engine_msg, new_counter)) = self.registry.parse_unified_message(args, self.voice_id_counter) {
                    self.voice_id_counter = new_counter;
                    
                    // NOTE: For standalone OSC testing, we always treat messages as immediate.
                    // Timestamps are handled at the engine level for sample-accurate playback.
                    // This differs from library mode where messages can be scheduled for future execution.
                    Some(ScheduledEngineMessage::Immediate(engine_msg))
                } else {
                    self.logger.log_error("Failed to parse /play command");
                    None
                }
            }
            "/update" => self.parse_update_unified(args),
            "/stop" => Some(ScheduledEngineMessage::Immediate(EngineMessage::Stop)),
            "/panic" => Some(ScheduledEngineMessage::Immediate(EngineMessage::Panic)),
            "/samples" => {
                self.print_samples();
                None
            }
            _ => {
                self.logger.log_warning(&format!("Unknown command: {}", command));
                None
            }
        }
    }


    /// Unified /update command parser
    fn parse_update_unified(&self, args: &[&str]) -> Option<ScheduledEngineMessage> {
        if args.len() < 4 {
            self.logger.log_error("Invalid /update command: need at least voice_id track_id param_key param_value");
            return None;
        }

        let voice_id: VoiceId = args[0].parse().ok()?;
        let track_id: TrackId = args[1].parse().ok()?;

        // Parse parameters using registry's unified approach
        let mut raw_parameters: HashMap<String, Box<dyn Any + Send>> = HashMap::new();
        let mut i = 2;
        while i + 1 < args.len() {
            let key = args[i];
            let value = args[i + 1];
            let param_value = self.registry.parse_parameter_value(value);
            raw_parameters.insert(key.to_string(), param_value);
            i += 2;
        }

        // Normalize parameters (no source context for updates)
        let parameters = self.registry.normalize_parameters(raw_parameters, None);

        // NOTE: For standalone OSC testing, updates are always immediate
        let engine_message = EngineMessage::Update {
            voice_id,
            track_id,
            parameters,
        };

        Some(ScheduledEngineMessage::Immediate(engine_message))
    }


    fn print_samples(&self) {
        self.logger.log_info("=== Sample Library Status ===");

        let folders = self.sample_library.get_all_folders();

        if folders.is_empty() {
            self.logger.log_info("No sample directories found");
            return;
        }

        self.logger.log_info(&format!("Found {} sample directories:", folders.len()));

        for (index, (folder_name, total_samples, loaded_samples)) in folders.iter().enumerate() {
            self.logger.log_info(&format!(
                "  [{}] {} ({}/{} loaded)",
                index, folder_name, loaded_samples, total_samples
            ));

            let folder_contents = self.sample_library.get_folder_contents(folder_name);
            if folder_contents.is_empty() {
                self.logger.log_info("      (no .wav files)");
            } else {
                for (sample_idx, file_name) in folder_contents.iter().enumerate() {
                    self.logger.log_info(&format!("      [{}] {}", sample_idx, file_name));
                }
            }
        }

        self.logger.log_info("=== Usage Examples ===");
        self.logger.log_info("  /play s sample sample_name kick sample_number 0");
        self.logger.log_info("  /play s sample sample_name bass sample_number 2.5");
        self.logger.log_info("==============================");
    }
}
