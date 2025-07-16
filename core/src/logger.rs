use std::sync::{Arc, Mutex, OnceLock};
use crossbeam_channel::{Sender, Receiver, unbounded};
use crate::protocol::log::{LogMessage, Severity};


/// Global logger instance
static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

/// Logger operating mode
#[derive(Debug, Clone)]
pub enum LoggerMode {
    /// Standalone mode: logs directly to terminal
    Standalone,
    /// Embedded mode: logs through channel communication
    Embedded(Sender<LogMessage>),
}

/// Core logging system that supports both standalone and embedded modes
pub struct Logger {
    mode: Arc<Mutex<LoggerMode>>,
}

impl Logger {
    /// Create a new logger in standalone mode
    pub fn new_standalone() -> Self {
        Logger {
            mode: Arc::new(Mutex::new(LoggerMode::Standalone)),
        }
    }

    /// Create a new logger in embedded mode with a channel sender
    pub fn new_embedded(sender: Sender<LogMessage>) -> Self {
        Logger {
            mode: Arc::new(Mutex::new(LoggerMode::Embedded(sender))),
        }
    }

    /// Switch to embedded mode with the provided channel sender
    pub fn set_embedded_mode(&self, sender: Sender<LogMessage>) {
        if let Ok(mut mode) = self.mode.lock() {
            *mode = LoggerMode::Embedded(sender);
        }
    }

    /// Switch to standalone mode
    pub fn set_standalone_mode(&self) {
        if let Ok(mut mode) = self.mode.lock() {
            *mode = LoggerMode::Standalone;
        }
    }

    /// Log a message with the specified severity
    pub fn log(&self, level: Severity, msg: String) {
        let log_msg = LogMessage::new(level, msg);
        
        if let Ok(mode) = self.mode.lock() {
            match &*mode {
                LoggerMode::Standalone => {
                    match log_msg.level {
                        Severity::Fatal | Severity::Error => {
                            eprintln!("{}", log_msg);
                        }
                        _ => {
                            println!("{}", log_msg);
                        }
                    }
                }
                LoggerMode::Embedded(sender) => {
                    if let Err(_) = sender.try_send(log_msg.clone()) {
                        // Fallback to terminal if channel is full/closed
                        eprintln!("Logger channel error: {}", log_msg);
                    }
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

/// Switch the global logger to standalone mode
pub fn set_standalone_mode() {
    get_logger().set_standalone_mode();
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