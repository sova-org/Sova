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

}
