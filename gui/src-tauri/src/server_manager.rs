use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::io::{BufReader, AsyncBufReadExt};
use std::process::Stdio;
use regex::Regex;
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    // Network
    pub ip: String,
    pub port: u16,
    
    // Audio Engine
    pub audio_engine: bool,
    pub sample_rate: u32,
    pub block_size: u32,
    pub buffer_size: u32,
    pub max_audio_buffers: u32,
    pub max_voices: u32,
    pub output_device: Option<String>,
    
    // OSC
    pub osc_port: u16,
    pub osc_host: String,
    
    // Advanced
    pub timestamp_tolerance_ms: u64,
    pub audio_files_location: String,
    pub audio_priority: u8,
    
    // Relay
    pub relay: Option<String>,
    pub instance_name: String,
    pub relay_token: Option<String>,
    
    // Special flags (not typically saved in config)
    pub list_devices: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port: 8080,
            audio_engine: false, // Default to false since it's a flag
            sample_rate: 44100,
            block_size: 512,
            buffer_size: 1024,
            max_audio_buffers: 2048,
            max_voices: 128,
            output_device: None,
            osc_port: 12345,
            osc_host: "127.0.0.1".to_string(),
            timestamp_tolerance_ms: 1000,
            audio_files_location: "./samples".to_string(),
            audio_priority: 80,
            relay: None,
            instance_name: "local".to_string(),
            relay_token: None,
            list_devices: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    pub status: ServerStatus,
    pub process_id: Option<u32>,
    pub config: ServerConfig,
    pub logs: Vec<LogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
}

pub struct ServerManager {
    state: Arc<Mutex<ServerState>>,
    process: Arc<Mutex<Option<Child>>>,
    log_sender: mpsc::UnboundedSender<LogEntry>,
    log_receiver: Arc<Mutex<mpsc::UnboundedReceiver<LogEntry>>>,
    app_handle: Arc<Mutex<Option<tauri::AppHandle>>>,
}

impl ServerManager {
    pub fn new() -> Self {
        Self::new_with_config(ServerConfig::default())
    }
    
    pub fn new_with_config(config: ServerConfig) -> Self {
        let (log_sender, log_receiver) = mpsc::unbounded_channel();
        
        Self {
            state: Arc::new(Mutex::new(ServerState {
                status: ServerStatus::Stopped,
                process_id: None,
                config,
                logs: Vec::new(),
            })),
            process: Arc::new(Mutex::new(None)),
            log_sender,
            log_receiver: Arc::new(Mutex::new(log_receiver)),
            app_handle: Arc::new(Mutex::new(None)),
        }
    }
    
    pub fn set_app_handle(&self, handle: tauri::AppHandle) {
        if let Ok(mut app_handle) = self.app_handle.lock() {
            *app_handle = Some(handle);
        }
    }
    
    pub fn get_state(&self) -> ServerState {
        self.state.lock().unwrap().clone()
    }
    
    pub fn get_local_log_file_path(&self) -> Option<String> {
        let state = self.state.lock().unwrap();
        if matches!(state.status, ServerStatus::Running | ServerStatus::Starting) {
            // When the server is running locally, we can provide the log file path
            // The core server writes logs to ~/.config/bubocore/logs/bubocore.log
            if let Some(config_dir) = dirs::config_dir() {
                let log_path = config_dir.join("bubocore").join("logs").join("bubocore.log");
                return Some(log_path.to_string_lossy().to_string());
            }
        }
        None
    }
    
    pub fn update_config(&self, config: ServerConfig) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        
        // Don't allow config changes while server is running
        if matches!(state.status, ServerStatus::Running | ServerStatus::Starting) {
            return Err(anyhow::anyhow!("Cannot change configuration while server is running"));
        }
        
        state.config = config;
        Ok(())
    }
    
    pub async fn start_server(&self) -> Result<()> {
        let config = {
            let mut state = self.state.lock().unwrap();
            
            if matches!(state.status, ServerStatus::Running | ServerStatus::Starting) {
                return Err(anyhow::anyhow!("Server is already running or starting"));
            }
            
            state.status = ServerStatus::Starting;
            state.config.clone()
        };
        
        // Check if port is available before trying to start
        if !self.is_port_available(config.port).await {
            let mut state = self.state.lock().unwrap();
            state.status = ServerStatus::Error(format!("Port {} is already in use", config.port));
            return Err(anyhow::anyhow!("Port {} is already in use", config.port));
        }
        
        // Find core binary
        let core_path = self.find_core_binary()?;
        
        // Build command arguments
        let mut cmd = Command::new(&core_path);
        cmd.arg("--ip").arg(&config.ip);
        cmd.arg("--port").arg(config.port.to_string());
        cmd.arg("--sample-rate").arg(config.sample_rate.to_string());
        cmd.arg("--block-size").arg(config.block_size.to_string());
        cmd.arg("--buffer-size").arg(config.buffer_size.to_string());
        cmd.arg("--max-audio-buffers").arg(config.max_audio_buffers.to_string());
        cmd.arg("--max-voices").arg(config.max_voices.to_string());
        cmd.arg("--osc-port").arg(config.osc_port.to_string());
        cmd.arg("--osc-host").arg(&config.osc_host);
        cmd.arg("--timestamp-tolerance-ms").arg(config.timestamp_tolerance_ms.to_string());
        cmd.arg("--audio-files-location").arg(&config.audio_files_location);
        cmd.arg("--audio-priority").arg(config.audio_priority.to_string());
        cmd.arg("--instance-name").arg(&config.instance_name);
        
        if config.audio_engine {
            cmd.arg("--audio-engine");
        }
        
        if let Some(device) = &config.output_device {
            cmd.arg("--output-device").arg(device);
        }
        
        if let Some(relay) = &config.relay {
            cmd.arg("--relay").arg(relay);
        }
        
        if let Some(token) = &config.relay_token {
            cmd.arg("--relay-token").arg(token);
        }
        
        // Configure stdio and force unbuffered output
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        // Force unbuffered output from spawned process
        cmd.env("RUST_LOG_SPAN_EVENTS", "full");
        cmd.env("RUST_BACKTRACE", "1");
        
        // Start the process
        let mut child = cmd.spawn().map_err(|e| {
            let mut state = self.state.lock().unwrap();
            state.status = ServerStatus::Error(format!("Failed to start server process: {}", e));
            e
        })?;
        let process_id = child.id();
        
        // Extract stdout and stderr for monitoring
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        
        // Store process reference
        *self.process.lock().unwrap() = Some(child);
        
        // Update state to Starting first
        {
            let mut state = self.state.lock().unwrap();
            state.status = ServerStatus::Starting;
            state.process_id = process_id;
        }
        
        // Start log monitoring with extracted handles
        self.start_log_monitoring_with_handles(stdout, stderr).await;
        
        self.add_log("info", "Server process started, waiting for TCP server to be ready...");
        
        // Wait for the TCP server to actually be ready to accept connections
        let config_port = config.port;
        let max_wait_time = 300; // 30 seconds max wait (300 * 100ms = 30s)
        let mut waited = 0;
        
        while waited < max_wait_time {
            if !self.is_port_available(config_port).await {
                // Port is in use, server is listening
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
            waited += 1;
        }
        
        if waited >= max_wait_time {
            // Check if process is still running
            let mut process_guard = self.process.lock().unwrap();
            if let Some(child) = process_guard.as_mut() {
                if let Ok(Some(exit_status)) = child.try_wait() {
                    // Process has exited
                    let mut state = self.state.lock().unwrap();
                    state.status = ServerStatus::Error(format!("Server process exited with status: {}", exit_status));
                    return Err(anyhow::anyhow!("Server process exited with status: {}", exit_status));
                }
            }
            
            let mut state = self.state.lock().unwrap();
            state.status = ServerStatus::Error("Server failed to start listening on port".to_string());
            return Err(anyhow::anyhow!("Server failed to start listening on port {} within {} seconds", config_port, max_wait_time / 10));
        }
        
        // Now mark as truly running
        {
            let mut state = self.state.lock().unwrap();
            state.status = ServerStatus::Running;
        }
        
        self.add_log("info", "Server is now listening and ready for connections");
        
        Ok(())
    }
    
    pub async fn stop_server(&self) -> Result<()> {
        {
            let mut state = self.state.lock().unwrap();
            
            if matches!(state.status, ServerStatus::Stopped | ServerStatus::Stopping) {
                return Err(anyhow::anyhow!("Server is already stopped or stopping"));
            }
            
            state.status = ServerStatus::Stopping;
        }
        
        let mut process_guard = self.process.lock().unwrap();
        if let Some(mut child) = process_guard.take() {
            // Try graceful shutdown first
            if let Err(e) = child.terminate() {
                eprintln!("Failed to terminate process gracefully: {}", e);
                // Force kill if graceful termination fails
                let _ = child.kill();
            }
            
            // Wait for process to exit
            let _ = child.wait();
        }
        
        // Update state
        {
            let mut state = self.state.lock().unwrap();
            state.status = ServerStatus::Stopped;
            state.process_id = None;
        }
        
        self.add_log("info", "Server stopped");
        
        Ok(())
    }
    
    pub async fn restart_server(&self) -> Result<()> {
        self.stop_server().await?;
        
        // Wait a bit for cleanup
        tokio::time::sleep(Duration::from_millis(1000)).await;
        
        self.start_server().await
    }
    
    pub fn get_recent_logs(&self, limit: usize) -> Vec<LogEntry> {
        let state = self.state.lock().unwrap();
        let logs = &state.logs;
        
        if logs.len() <= limit {
            logs.clone()
        } else {
            logs[logs.len() - limit..].to_vec()
        }
    }
    
    pub fn list_audio_devices(&self) -> Result<Vec<String>> {
        // Find core binary
        let core_path = self.find_core_binary()?;
        
        // Run core with --list-devices flag
        let output = std::process::Command::new(&core_path)
            .arg("--list-devices")
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to list audio devices: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        // Parse the output to extract device names
        let output_str = String::from_utf8_lossy(&output.stdout);
        let devices: Vec<String> = output_str
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                // Look for device lines that start with validation marks (✓ or ✗)
                let device_name = if trimmed.starts_with("✓ ") {
                    trimmed.strip_prefix("✓ ").unwrap_or("").trim()
                } else if trimmed.starts_with("✗ ") {
                    trimmed.strip_prefix("✗ ").unwrap_or("").trim()
                } else {
                    return None;
                };
                
                // Skip empty device names
                if device_name.is_empty() {
                    return None;
                }
                
                // Remove [DEFAULT] suffix if present
                let clean_name = if device_name.ends_with(" [DEFAULT]") {
                    device_name.trim_end_matches(" [DEFAULT]")
                } else {
                    device_name
                };
                
                Some(clean_name.to_string())
            })
            .collect();
        
        let result = devices;
        
        Ok(result)
    }
    
    fn find_core_binary(&self) -> Result<String> {
        // Try to find the core binary in common locations
        let possible_paths = [
            "../../target/release/core",
            "../../target/debug/core",
            "./core",
            "core",
        ];
        
        for path in &possible_paths {
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }
        
        Err(anyhow::anyhow!("Core binary not found. Please build the project first with 'cargo build'"))
    }
    
    async fn is_port_available(&self, port: u16) -> bool {
        use std::net::TcpListener;
        
        TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
    }
    
    pub async fn detect_running_server(&self) -> Result<bool> {
        let config = {
            let state = self.state.lock().unwrap();
            state.config.clone()
        };
        
        // Simply check if port is in use
        if self.is_port_available(config.port).await {
            return Ok(false); // Port is available, no server running
        }
        
        // Port is occupied, assume it's our server and update state
        let mut state = self.state.lock().unwrap();
        state.status = ServerStatus::Running;
        state.process_id = None; // We don't know the PID of external process
        self.add_log("info", "Detected server running on configured port");
        
        Ok(true)
    }
    
    async fn start_log_monitoring_with_handles(&self, stdout: Option<tokio::process::ChildStdout>, stderr: Option<tokio::process::ChildStderr>) {
        let log_sender = self.log_sender.clone();
        let app_handle = self.app_handle.clone();
        
        
        if let (Some(stdout), Some(stderr)) = (stdout, stderr) {
            let log_sender_stdout = log_sender.clone();
            let log_sender_stderr = log_sender.clone();
            let app_handle_stdout = app_handle.clone();
            let app_handle_stderr = app_handle.clone();
            
            // Spawn task for stdout monitoring
            tokio::spawn(async move {
                let stdout_reader = BufReader::new(stdout);
                let mut lines = stdout_reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if !line.trim().is_empty() {
                        Self::parse_and_send_log(&log_sender_stdout, &line, "info", &app_handle_stdout);
                    }
                }
            });
            
            // Spawn task for stderr monitoring
            tokio::spawn(async move {
                let stderr_reader = BufReader::new(stderr);
                let mut lines = stderr_reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if !line.trim().is_empty() {
                        Self::parse_and_send_log(&log_sender_stderr, &line, "error", &app_handle_stderr);
                    }
                }
            });
        }
        
        self.add_log("info", "Core log monitoring started");
    }
    
    fn parse_and_send_log(log_sender: &mpsc::UnboundedSender<LogEntry>, line: &str, default_level: &str, app_handle: &Arc<Mutex<Option<tauri::AppHandle>>>) {
        // Parse multiple log formats:
        // 1. Core format: [LEVEL] message  
        // 2. Timestamped format: YYYY-MM-DDTHH:MM:SS.sssssssZ [LEVEL] message
        // 3. Simple format: LEVEL: message
        let core_regex = Regex::new(r"^\[(\w+)\]\s*(.*)$").unwrap();
        let timestamped_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z\s*\[(\w+)\]\s*(.*)$").unwrap();
        let simple_regex = Regex::new(r"^(\w+):\s*(.*)$").unwrap();
        
        let (level, message) = if let Some(captures) = timestamped_regex.captures(line) {
            let level = captures.get(1).map_or(default_level, |m| m.as_str()).to_lowercase();
            let message = captures.get(2).map_or(line, |m| m.as_str());
            (level, message.to_string())
        } else if let Some(captures) = core_regex.captures(line) {
            let level = captures.get(1).map_or(default_level, |m| m.as_str()).to_lowercase();
            let message = captures.get(2).map_or(line, |m| m.as_str());
            (level, message.to_string())
        } else if let Some(captures) = simple_regex.captures(line) {
            let level = captures.get(1).map_or(default_level, |m| m.as_str()).to_lowercase();
            let message = captures.get(2).map_or(line, |m| m.as_str());
            (level, message.to_string())
        } else {
            // If no log format detected, use the whole line as message
            (default_level.to_string(), line.to_string())
        };
        
        let log_entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level,
            message,
        };
        
        // Send to local log channel
        let _ = log_sender.send(log_entry.clone());
        
        // Emit to frontend immediately
        if let Ok(app_handle_guard) = app_handle.lock() {
            if let Some(app_handle) = app_handle_guard.as_ref() {
                let _ = app_handle.emit("server-log", &log_entry);
            }
        }
    }
    
    fn add_log(&self, level: &str, message: &str) {
        let log_entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level: level.to_string(),
            message: message.to_string(),
        };
        
        let mut state = self.state.lock().unwrap();
        state.logs.push(log_entry.clone());
        
        // Keep only last 1000 logs
        if state.logs.len() > 1000 {
            state.logs.remove(0);
        }
        
        // Send to log channel
        let _ = self.log_sender.send(log_entry);
    }
}

// Extension trait for Child process to add terminate method
trait ChildExt {
    fn terminate(&mut self) -> std::io::Result<()>;
}

impl ChildExt for Child {
    fn terminate(&mut self) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            if let Some(pid) = self.id() {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }
            Ok(())
        }
        
        #[cfg(windows)]
        {
            self.kill()
        }
    }
}