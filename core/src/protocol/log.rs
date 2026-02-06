use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::hash::{Hash, Hasher};

use crate::clock::SyncTime;
use crate::protocol::payload::ProtocolPayload;
use crate::vm::event::ConcreteEvent;

/// Represents the severity level of a log message.
///
/// Used to categorize log messages for filtering and display purposes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Indicates a critical error that prevents the application from continuing.
    Fatal,
    /// Indicates a significant error that affects functionality but may allow continuation.
    Error,
    /// Indicates a potential issue or unexpected situation.
    Warn,
    /// Indicates informational messages about the application's state or progress.
    Info,
    /// Indicates detailed messages useful for debugging.
    Debug,
}

impl Display for Severity {
    /// Formats the `Severity` level with a text label for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Fatal => write!(f, "[FATAL]"),
            Severity::Error => write!(f, "[ERROR]"),
            Severity::Warn => write!(f, "[WARN]"),
            Severity::Info => write!(f, "[INFO]"),
            Severity::Debug => write!(f, "[DEBUG]"),
        }
    }
}

/// The standard name used to identify the internal logging device.
///
/// See `ProtocolDevice::Log`.
pub const LOG_NAME: &str = "log";

/// Represents a structured log message.
///
/// Contains a severity level, an optional associated event, and the log message text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogMessage {
    /// The severity level of the log message.
    pub level: Severity,
    /// An optional `ConcreteEvent` associated with this log message.
    /// Can provide context about the operation that generated the log.
    pub event: Option<ConcreteEvent>,
    /// The main text content of the log message.
    pub msg: String,
}

impl Hash for LogMessage {
    /// Hashes the `LogMessage` based on its severity level and message content.
    ///
    /// Note: The associated `event` is not included in the hash calculation.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.level.hash(state);
        self.msg.hash(state);
    }
}

impl Display for LogMessage {
    /// Formats the `LogMessage` for display, showing the severity icon and the message text.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let log_event = self
            .event
            .as_ref()
            .map(|event| format!("{:?}", event))
            .unwrap_or_default();
        write!(f, "{} {} {}", self.level, self.msg, log_event)
    }
}

impl LogMessage {
    /// Creates a new `LogMessage` with the specified severity and message text.
    ///
    /// The associated `event` is initialized to `None`.
    pub fn new(level: Severity, msg: String) -> Self {
        LogMessage {
            level,
            event: None,
            msg,
        }
    }

    /// Creates a new `LogMessage` with `Severity::Fatal`.
    pub fn fatal(msg: String) -> Self {
        LogMessage {
            level: Severity::Fatal,
            event: None,
            msg,
        }
    }

    /// Creates a new `LogMessage` with `Severity::Error`.
    pub fn error(msg: String) -> Self {
        LogMessage {
            level: Severity::Error,
            event: None,
            msg,
        }
    }

    /// Creates a new `LogMessage` with `Severity::Warn`.
    pub fn warn(msg: String) -> Self {
        LogMessage {
            level: Severity::Warn,
            event: None,
            msg,
        }
    }

    /// Creates a new `LogMessage` with `Severity::Info`.
    pub fn info(msg: String) -> Self {
        LogMessage {
            level: Severity::Info,
            event: None,
            msg,
        }
    }

    /// Creates a new `LogMessage` with `Severity::Debug`.
    pub fn debug(msg: String) -> Self {
        LogMessage {
            level: Severity::Debug,
            event: None,
            msg,
        }
    }

    /// Creates a new `LogMessage` from a `ConcreteEvent` and severity level.
    ///
    /// The message text is derived from the event's debug representation.
    pub fn from_event(level: Severity, event: ConcreteEvent) -> Self {
        LogMessage {
            level,
            event: None,
            msg: format!("{:?}", event),
        }
    }

    pub fn generate_messages(
        event: ConcreteEvent,
        date: SyncTime,
    ) -> Vec<(ProtocolPayload, SyncTime)> {
        match event {
            ConcreteEvent::Print(msg) => vec![(Self::info(msg).into(), date)],
            _ => vec![(Self::from_event(Severity::Info, event).into(), date)],
        }
    }
}
