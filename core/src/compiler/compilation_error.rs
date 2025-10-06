use std::{error, fmt, string::FromUtf8Error};

use serde::{Deserialize, Serialize};

/// Represents an error that occurred during the compilation process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationError {
    /// The name of the language or compiler stage where the error occurred.
    pub lang: String,
    /// A detailed message describing the error.
    pub info: String,
    /// The starting position in the source code related to the error, if applicable.
    pub from: usize,
    /// The ending position in the source code related to the error, if applicable.
    pub to: usize,
}

impl CompilationError {
    pub fn default_error(lang: String) -> Self {
        Self {
            lang,
            info: "unknown error (todo)".to_string(),
            from: 0,
            to: 0,
        }
    }
}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} error: {}", self.lang, self.info)
    }
}

impl error::Error for CompilationError {}

/// Converts an I/O error into a `CompilationError`.
impl From<std::io::Error> for CompilationError {
    fn from(_: std::io::Error) -> Self {
        CompilationError::default_error("io".to_string())
    }
}

/// Converts a process output error (often from `wait_with_output`) into a `CompilationError`.
/// Note: Specific details from the process output are lost in this conversion.
impl From<std::process::Output> for CompilationError {
    fn from(_: std::process::Output) -> Self {
        CompilationError::default_error("process".to_string())
    }
}

/// Converts a UTF-8 conversion error into a `CompilationError`.
impl From<FromUtf8Error> for CompilationError {
    fn from(_: FromUtf8Error) -> Self {
        CompilationError::default_error("FromUtf8".to_string())
    }
}

/// Converts a Serde JSON deserialization error into a `CompilationError`.
impl From<serde_json::Error> for CompilationError {
    fn from(_: serde_json::Error) -> Self {
        CompilationError::default_error("serde_json".to_string())
    }
}