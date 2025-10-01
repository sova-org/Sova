use std::sync::{Arc, Mutex, OnceLock};
use std::io::Write;
use std::fs::{File, OpenOptions, create_dir_all};
use std::path::PathBuf;
use crossbeam_channel::{Sender, Receiver, unbounded};
use tokio::sync::watch;
use crate::protocol::log::{LogMessage, Severity};
use crate::protocol::message::{TimedMessage, ProtocolMessage};
use crate::protocol::payload::ProtocolPayload;
use crate::protocol::device::ProtocolDevice;
use crate::schedule::notification::SchedulerNotification;


/// Global logger instance
static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

/// Log file configuration
const LOG_FILE_MAX_SIZE: u64 = 1024 * 1024; // 1MB
const LOG_FILE_MAX_COUNT: usize = 5;
const LOG_FILE_NAME: &str = "sova.log";

/// File-based log writer with rotation
#[derive(Debug)]
pub struct LogFileWriter {
    log_dir: PathBuf,
    current_file: Option<File>,
    current_size: u64,
}

impl LogFileWriter {
    pub fn new() -> Result<Self, std::io::Error> {
        let log_dir = Self::get_log_directory()?;
        create_dir_all(&log_dir)?;
        
        Ok(LogFileWriter {
            log_dir,
            current_file: None,
            current_size: 0,
        })
    }
    
    fn get_log_directory() -> Result<PathBuf, std::io::Error> {
        let mut path = dirs::config_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        path.push("sova");
        path.push("logs");
        Ok(path)
    }
    
    fn get_current_log_path(&self) -> PathBuf {
        self.log_dir.join(LOG_FILE_NAME)
    }
    
    fn rotate_logs(&mut self) -> Result<(), std::io::Error> {
        let current_path = self.get_current_log_path();
        
        // Close current file
        self.current_file = None;
        
        // Rotate existing log files
        for i in (1..LOG_FILE_MAX_COUNT).rev() {
            let old_path = self.log_dir.join(format!("{}.{}", LOG_FILE_NAME, i));
            let new_path = self.log_dir.join(format!("{}.{}", LOG_FILE_NAME, i + 1));
            
            if old_path.exists() {
                if i == LOG_FILE_MAX_COUNT - 1 {
                    // Delete oldest file
                    std::fs::remove_file(&old_path)?;
                } else {
                    // Rename to next number
                    std::fs::rename(&old_path, &new_path)?;
                }
            }
        }
        
        // Move current log to .1
        if current_path.exists() {
            let archived_path = self.log_dir.join(format!("{}.1", LOG_FILE_NAME));
            std::fs::rename(&current_path, &archived_path)?;
        }
        
        self.current_size = 0;
        Ok(())
    }
    
    fn ensure_file_open(&mut self) -> Result<(), std::io::Error> {
        if self.current_file.is_none() {
            let path = self.get_current_log_path();
            self.current_file = Some(OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?);
            
            // Get current file size
            if let Ok(metadata) = std::fs::metadata(&path) {
                self.current_size = metadata.len();
            }
        }
        Ok(())
    }
    
    pub fn write_log(&mut self, log_msg: &LogMessage) -> Result<(), std::io::Error> {
        self.ensure_file_open()?;
        
        let formatted_log = format!("{}\n", log_msg);
        let log_bytes = formatted_log.as_bytes();
        
        // Check if rotation is needed
        if self.current_size + log_bytes.len() as u64 > LOG_FILE_MAX_SIZE {
            self.rotate_logs()?;
            self.ensure_file_open()?;
        }
        
        if let Some(ref mut file) = self.current_file {
            file.write_all(log_bytes)?;
            file.flush()?;
            self.current_size += log_bytes.len() as u64;
        }
        
        Ok(())
    }
    
    pub fn get_log_file_path(&self) -> PathBuf {
        self.get_current_log_path()
    }
}

/// Logger operating mode
#[derive(Debug, Clone)]
pub enum LoggerMode {
    /// Standalone mode: logs directly to terminal only
    Standalone,
    /// Embedded mode: logs through channel communication (legacy)
    Embedded(Sender<LogMessage>),
    /// Network mode: logs to clients via notification system (no terminal)
    Network(watch::Sender<SchedulerNotification>),
    /// Dual mode: logs to terminal AND sends to clients (preferred for servers)
    Dual(watch::Sender<SchedulerNotification>),
    /// File mode: logs to file only (for persistent logging)
    File,
    /// Full mode: logs to file, terminal, and clients (complete logging solution)
    Full(watch::Sender<SchedulerNotification>),
}

/// Core logging system that supports both standalone and embedded modes
pub struct Logger {
    mode: Arc<Mutex<LoggerMode>>,
    file_writer: Arc<Mutex<Option<LogFileWriter>>>,
}

impl Logger {
    /// Create a new logger in standalone mode
    pub fn new_standalone() -> Self {
        Logger {
            mode: Arc::new(Mutex::new(LoggerMode::Standalone)),
            file_writer: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a new logger in embedded mode with a channel sender
    pub fn new_embedded(sender: Sender<LogMessage>) -> Self {
        Logger {
            mode: Arc::new(Mutex::new(LoggerMode::Embedded(sender))),
            file_writer: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a new logger in network mode with a notification sender
    pub fn new_network(sender: watch::Sender<SchedulerNotification>) -> Self {
        Logger {
            mode: Arc::new(Mutex::new(LoggerMode::Network(sender))),
            file_writer: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a new logger in file mode (logs to file only)
    pub fn new_file() -> Self {
        let file_writer = match LogFileWriter::new() {
            Ok(writer) => Some(writer),
            Err(e) => {
                eprintln!("Failed to create log file writer: {}", e);
                None
            }
        };
        
        Logger {
            mode: Arc::new(Mutex::new(LoggerMode::File)),
            file_writer: Arc::new(Mutex::new(file_writer)),
        }
    }

    /// Create a new logger in full mode (logs to file, terminal, and clients)
    pub fn new_full(sender: watch::Sender<SchedulerNotification>) -> Self {
        let file_writer = match LogFileWriter::new() {
            Ok(writer) => Some(writer),
            Err(e) => {
                eprintln!("Failed to create log file writer: {}", e);
                None
            }
        };
        
        Logger {
            mode: Arc::new(Mutex::new(LoggerMode::Full(sender))),
            file_writer: Arc::new(Mutex::new(file_writer)),
        }
    }

    /// Switch to embedded mode with the provided channel sender
    pub fn set_embedded_mode(&self, sender: Sender<LogMessage>) {
        if let Ok(mut mode) = self.mode.lock() {
            *mode = LoggerMode::Embedded(sender);
        }
    }

    /// Switch to network mode with the provided notification sender
    pub fn set_network_mode(&self, sender: watch::Sender<SchedulerNotification>) {
        if let Ok(mut mode) = self.mode.lock() {
            *mode = LoggerMode::Network(sender);
        }
    }

    /// Switch to dual mode (terminal + network) with the provided notification sender
    pub fn set_dual_mode(&self, sender: watch::Sender<SchedulerNotification>) {
        if let Ok(mut mode) = self.mode.lock() {
            *mode = LoggerMode::Dual(sender);
        }
    }

    /// Switch to standalone mode
    pub fn set_standalone_mode(&self) {
        if let Ok(mut mode) = self.mode.lock() {
            *mode = LoggerMode::Standalone;
        }
    }

    /// Switch to file mode
    pub fn set_file_mode(&self) {
        if let Ok(mut mode) = self.mode.lock() {
            *mode = LoggerMode::File;
        }
        
        // Initialize file writer if not already present
        if let Ok(mut file_writer) = self.file_writer.lock() {
            if file_writer.is_none() {
                *file_writer = match LogFileWriter::new() {
                    Ok(writer) => Some(writer),
                    Err(e) => {
                        eprintln!("Failed to create log file writer: {}", e);
                        None
                    }
                };
            }
        }
    }

    /// Switch to full mode (file + terminal + network)
    pub fn set_full_mode(&self, sender: watch::Sender<SchedulerNotification>) {
        if let Ok(mut mode) = self.mode.lock() {
            *mode = LoggerMode::Full(sender);
        }
        
        // Initialize file writer if not already present
        if let Ok(mut file_writer) = self.file_writer.lock() {
            if file_writer.is_none() {
                *file_writer = match LogFileWriter::new() {
                    Ok(writer) => Some(writer),
                    Err(e) => {
                        eprintln!("Failed to create log file writer: {}", e);
                        None
                    }
                };
            }
        }
    }

    /// Get the current log file path (if file logging is enabled)
    pub fn get_log_file_path(&self) -> Option<PathBuf> {
        if let Ok(file_writer) = self.file_writer.lock() {
            file_writer.as_ref().map(|w| w.get_log_file_path())
        } else {
            None
        }
    }

    /// Log a message with the specified severity
    pub fn log(&self, level: Severity, msg: String) {
        let log_msg = LogMessage::new(level, msg);
        
        // Helper function to write to file if enabled
        let write_to_file = |log_msg: &LogMessage| {
            if let Ok(mut file_writer) = self.file_writer.lock() {
                if let Some(ref mut writer) = file_writer.as_mut() {
                    if let Err(e) = writer.write_log(log_msg) {
                        eprintln!("Failed to write to log file: {}", e);
                    }
                }
            }
        };
        
        if let Ok(mode) = self.mode.lock() {
            match &*mode {
                LoggerMode::Standalone => {
                    match log_msg.level {
                        Severity::Fatal | Severity::Error => {
                            eprintln!("{}", log_msg);
                            let _ = std::io::stderr().flush();
                        }
                        _ => {
                            println!("{}", log_msg);
                            let _ = std::io::stdout().flush();
                        }
                    }
                }
                LoggerMode::Embedded(sender) => {
                    if let Err(_) = sender.try_send(log_msg.clone()) {
                        // Fallback to terminal if channel is full/closed
                        eprintln!("Logger channel error: {}", log_msg);
                    }
                }
                LoggerMode::Network(sender) => {
                    // Wrap the LogMessage in a TimedMessage for the notification system
                    let timed_message = TimedMessage {
                        message: ProtocolMessage {
                            device: Arc::new(ProtocolDevice::Log),
                            payload: ProtocolPayload::LOG(log_msg.clone()),
                        },
                        time: 0, // Immediate execution
                    };
                    let notification = SchedulerNotification::Log(timed_message);
                    if let Err(_) = sender.send(notification) {
                        // Fallback to terminal if notification channel is closed
                        eprintln!("Logger notification error: {}", log_msg);
                    }
                }
                LoggerMode::Dual(sender) => {
                    // ALWAYS log to terminal first (essential for standalone debugging)
                    match log_msg.level {
                        Severity::Fatal | Severity::Error => {
                            eprintln!("{}", log_msg);
                            let _ = std::io::stderr().flush();
                        }
                        _ => {
                            println!("{}", log_msg);
                            let _ = std::io::stdout().flush();
                        }
                    }
                    
                    // ALWAYS try to send to clients (but don't block if failed)
                    let timed_message = TimedMessage {
                        message: ProtocolMessage {
                            device: Arc::new(ProtocolDevice::Log),
                            payload: ProtocolPayload::LOG(log_msg.clone()),
                        },
                        time: 0, // Immediate execution
                    };
                    let notification = SchedulerNotification::Log(timed_message);
                    // Explicitly ignore errors - terminal logging is the fallback
                    let _ = sender.send(notification);
                }
                LoggerMode::File => {
                    // Only write to file in this mode
                    write_to_file(&log_msg);
                }
                LoggerMode::Full(sender) => {
                    // Write to file first (most important for persistence)
                    write_to_file(&log_msg);
                    
                    // Then log to terminal
                    match log_msg.level {
                        Severity::Fatal | Severity::Error => {
                            eprintln!("{}", log_msg);
                            let _ = std::io::stderr().flush();
                        }
                        _ => {
                            println!("{}", log_msg);
                            let _ = std::io::stdout().flush();
                        }
                    }
                    
                    // Finally send to clients
                    let timed_message = TimedMessage {
                        message: ProtocolMessage {
                            device: Arc::new(ProtocolDevice::Log),
                            payload: ProtocolPayload::LOG(log_msg.clone()),
                        },
                        time: 0, // Immediate execution
                    };
                    let notification = SchedulerNotification::Log(timed_message);
                    let _ = sender.send(notification);
                }
            }
        }
    }

    /// Log with debug severity
    pub fn debug(&self, msg: String) {
        self.log(Severity::Debug, msg);
    }

    /// Log with info severity
    pub fn info(&self, msg: String) {
        self.log(Severity::Info, msg);
    }

    /// Log with warn severity
    pub fn warn(&self, msg: String) {
        self.log(Severity::Warn, msg);
    }

    /// Log with error severity
    pub fn error(&self, msg: String) {
        self.log(Severity::Error, msg);
    }

    /// Log with fatal severity
    pub fn fatal(&self, msg: String) {
        self.log(Severity::Fatal, msg);
    }
}

/// Initialize the global logger in standalone mode
pub fn init_standalone() {
    let _ = GLOBAL_LOGGER.set(Logger::new_standalone());
}

/// Initialize the global logger in embedded mode
pub fn init_embedded(sender: Sender<LogMessage>) {
    let _ = GLOBAL_LOGGER.set(Logger::new_embedded(sender));
}

/// Initialize the global logger in network mode
pub fn init_network(sender: watch::Sender<SchedulerNotification>) {
    let _ = GLOBAL_LOGGER.set(Logger::new_network(sender));
}

/// Create a logging channel pair
pub fn create_log_channel() -> (Sender<LogMessage>, Receiver<LogMessage>) {
    unbounded()
}

/// Get the global logger instance
pub fn get_logger() -> &'static Logger {
    GLOBAL_LOGGER.get_or_init(|| Logger::new_standalone())
}

/// Switch the global logger to embedded mode
pub fn set_embedded_mode(sender: Sender<LogMessage>) {
    get_logger().set_embedded_mode(sender);
}

/// Switch the global logger to network mode
pub fn set_network_mode(sender: watch::Sender<SchedulerNotification>) {
    get_logger().set_network_mode(sender);
}

/// Switch the global logger to dual mode (terminal + network)
pub fn set_dual_mode(sender: watch::Sender<SchedulerNotification>) {
    get_logger().set_dual_mode(sender);
}

/// Switch the global logger to standalone mode
pub fn set_standalone_mode() {
    get_logger().set_standalone_mode();
}

/// Initialize the global logger in file mode
pub fn init_file() {
    let _ = GLOBAL_LOGGER.set(Logger::new_file());
}

/// Initialize the global logger in full mode
pub fn init_full(sender: watch::Sender<SchedulerNotification>) {
    let _ = GLOBAL_LOGGER.set(Logger::new_full(sender));
}

/// Switch the global logger to file mode
pub fn set_file_mode() {
    get_logger().set_file_mode();
}

/// Switch the global logger to full mode (file + terminal + network)
pub fn set_full_mode(sender: watch::Sender<SchedulerNotification>) {
    get_logger().set_full_mode(sender);
}

/// Get the current log file path (if file logging is enabled)
pub fn get_log_file_path() -> Option<PathBuf> {
    get_logger().get_log_file_path()
}

/// Convenience macros for logging
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().debug(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().info(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().warn(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().error(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_fatal {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().fatal(format!($($arg)*))
    };
}

/// Drop-in replacement for println! that goes through the logging system
#[macro_export]
macro_rules! log_println {
    () => {
        $crate::logger::get_logger().info("".to_string())
    };
    ($($arg:tt)*) => {
        $crate::logger::get_logger().info(format!($($arg)*))
    };
}

/// Drop-in replacement for eprintln! that goes through the logging system
#[macro_export]
macro_rules! log_eprintln {
    () => {
        $crate::logger::get_logger().error("".to_string())
    };
    ($($arg:tt)*) => {
        $crate::logger::get_logger().error(format!($($arg)*))
    };
}

/// Drop-in replacement for print! that goes through the logging system
#[macro_export]
macro_rules! log_print {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().info(format!($($arg)*))
    };
}

/// Drop-in replacement for eprint! that goes through the logging system
#[macro_export]
macro_rules! log_eprint {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().error(format!($($arg)*))
    };
}