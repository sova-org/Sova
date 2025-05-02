use std::fmt::{self, Display};
use crate::protocol::midi::MidiError;

/// A general error type for operations within the `protocol` module.
///
/// This struct wraps a descriptive error message as a `String`.
/// It serves as a unified error type, often created by converting
/// more specific errors (like IO, MIDI, or OSC errors) using the `From` trait.
#[derive(Debug)]
pub struct ProtocolError(pub String);

impl Display for ProtocolError {
    /// Formats the `ProtocolError` for display.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Protocol Error: {}", self.0)
    }
}

/// Marks `ProtocolError` as a standard Rust error.
///
/// This allows `ProtocolError` to be used with error handling mechanisms
/// like the `?` operator and ensures compatibility with other error types.
impl std::error::Error for ProtocolError {}

impl From<MidiError> for ProtocolError {
    /// Converts a `MidiError` into a `ProtocolError`.
    ///
    /// This facilitates propagating errors from MIDI-specific operations.
    fn from(value: MidiError) -> Self {
        // Extracts the inner string message from MidiError.
        ProtocolError(value.0)
    }
}

impl From<std::io::Error> for ProtocolError {
    /// Converts a standard `std::io::Error` into a `ProtocolError`.
    ///
    /// Useful for wrapping errors related to network operations (like UDP sockets)
    /// or potential future file operations within the protocol module.
    fn from(e: std::io::Error) -> Self {
        ProtocolError(format!("IO Error: {}", e))
    }
}

impl From<rosc::OscError> for ProtocolError {
    /// Converts an error from the `rosc` library (`rosc::OscError`) into a `ProtocolError`.
    ///
    /// Handles errors related to OSC message encoding or decoding.
    fn from(e: rosc::OscError) -> Self {
        ProtocolError(format!("OSC Error: {}", e))
    }
}
