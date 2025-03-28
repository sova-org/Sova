use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogMessage {
    pub level: Severity,
    pub msg : String
}

impl LogMessage {

    pub fn new(level : Severity, msg : String) -> Self {
        LogMessage { level, msg }
    }

    pub fn fatal(msg : String) -> Self {
        LogMessage { level : Severity::Fatal, msg }
    }

    pub fn error(msg : String) -> Self {
        LogMessage { level : Severity::Error, msg }
    }

    pub fn warn(msg : String) -> Self {
        LogMessage { level : Severity::Warn, msg }
    }

    pub fn info(msg : String) -> Self {
        LogMessage { level : Severity::Info, msg }
    }

    pub fn debug(msg : String) -> Self {
        LogMessage { level : Severity::Debug, msg }
    }

}
