use std::fmt::Display;
use serde::{Deserialize, Serialize};

use crate::lang::event::ConcreteEvent;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    Fatal,
    Error,
    Warn,
    Info,
    Debug,
}

impl Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Fatal => write!(f, "[☠️ ]"),
            Severity::Error => write!(f, "[⛔️]"),
            Severity::Warn => write!(f, "[⚠️ ]"),
            Severity::Info => write!(f, "[🤟]"),
            Severity::Debug => write!(f, "[🔩]"),
        }
    }
}

pub const LOG_NAME: &str = "log";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: Severity,
    pub event: Option<ConcreteEvent>,
    pub msg : String
}

impl Display for LogMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.level, self.msg)
    }
}

impl LogMessage {

    pub fn new(level : Severity, msg : String) -> Self {
        LogMessage { level, event: None, msg }
    }

    pub fn fatal(msg : String) -> Self {
        LogMessage { level : Severity::Fatal, event: None, msg }
    }

    pub fn error(msg : String) -> Self {
        LogMessage { level : Severity::Error, event: None, msg }
    }

    pub fn warn(msg : String) -> Self {
        LogMessage { level : Severity::Warn, event: None, msg }
    }

    pub fn info(msg : String) -> Self {
        LogMessage { level : Severity::Info, event: None, msg }
    }

    pub fn debug(msg : String) -> Self {
        LogMessage { level : Severity::Debug, event: None, msg }
    }

    pub fn from_event(level: Severity, event: ConcreteEvent) -> Self {
        LogMessage { level, event: Some(event), msg: String::new() }
    }

}
