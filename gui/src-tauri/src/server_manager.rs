use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    // Network
    pub ip: String,
    pub port: u16,
    
    // Audio Engine
    pub audio_engine: bool,
    pub sample_rate: u32,
    pub block_size: u32,
    pub buffer_size: usize,
    pub max_audio_buffers: usize,
    pub max_voices: usize,
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
        }
    }
    
    pub fn get_state(&self) -> ServerState {
        self.state.lock().unwrap().clone()
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
        
        // Check if port is available
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
        
        // Configure stdio
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        // Start the process
        let child = cmd.spawn()?;
        let process_id = child.id();
        
        // Store process reference
        *self.process.lock().unwrap() = Some(child);
        
        // Update state
        {
            let mut state = self.state.lock().unwrap();
            state.status = ServerStatus::Running;
            state.process_id = Some(process_id);
        }
        
        // Start log monitoring
        self.start_log_monitoring().await;
        
        self.add_log("info", "Server started successfully");
        
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
        let output = Command::new(&core_path)
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
        
        let mut result = devices;
        
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
    
    async fn start_log_monitoring(&self) {
        // This would monitor stdout/stderr from the child process
        // For now, we'll just add a placeholder log
        self.add_log("info", &format!("Server process started"));
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
            unsafe {
                libc::kill(self.id() as i32, libc::SIGTERM);
            }
            Ok(())
        }
        
        #[cfg(windows)]
        {
            self.kill()
        }
    }
}