use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

/// Represents the different types of arguments an OSC (Open Sound Control) message can contain.
///
/// This enum covers common OSC argument types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Argument {
    /// An OSC 32-bit integer (`i`).
    Int(i32),
    /// An OSC 32-bit float (`f`).
    Float(f32),
    /// An OSC string (`s`).
    String(String),
    /// An OSC blob (binary data) (`b`).
    Blob(Vec<u8>),
    /// An OSC 64-bit timetag (`t`), usually representing NTP time.
    Timetag(u64),
    // Other types like Double(f64), Char(char), RGBA(u32), Midi(Vec<u8>), etc.,
    // can be added here if needed in the future.
}

// Manual implementation of Eq because f32 doesn't derive Eq.
// PartialEq is already derived and handles f32 comparison appropriately (within tolerance).
// This Eq implementation relies on the PartialEq logic.
impl Eq for Argument {}

// Manual implementation of Hash because f32 doesn't derive Hash.
impl std::hash::Hash for Argument {
    /// Hashes the `Argument`.
    ///
    /// Distinguishes between variants and hashes their inner values.
    /// For `Float`, it hashes the underlying bits (`to_bits()`).
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Argument::Int(i) => {
                0.hash(state);
                i.hash(state);
            }
            Argument::Float(f) => {
                1.hash(state);
                f.to_bits().hash(state);
            }
            Argument::String(s) => {
                2.hash(state);
                s.hash(state);
            }
            Argument::Blob(b) => {
                3.hash(state);
                b.hash(state);
            }
            Argument::Timetag(t) => {
                4.hash(state);
                t.hash(state);
            }
        }
    }
}

/// Represents a single OSC message, consisting of an address pattern and a list of arguments.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OSCMessage {
    /// The OSC address pattern (e.g., "/synth/play").
    pub addr: String,
    /// The list of arguments associated with the message.
    pub args: Vec<Argument>,
}

impl Display for OSCMessage {
    /// Formats the `OSCMessage` for display, showing the address and arguments.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OSCMessage {{ addr: \"{}\", args: {:?} }}",
            self.addr, self.args
        )
    }
}

impl OSCMessage {
    /// Creates a new `OSCMessage` with the given address and arguments.
    ///
    /// # Arguments
    /// * `addr` - The OSC address pattern string.
    /// * `args` - A vector containing the `Argument` values for the message.
    pub fn new(addr: String, args: Vec<Argument>) -> Self {
        OSCMessage { addr, args }
    }

    /// Creates an OSC message specifically formatted for SuperDirt's `/dirt/play` address.
    ///
    /// SuperDirt expects arguments in a flattened key-value format.
    /// This function takes a `HashMap` where keys are SuperDirt parameter names
    /// (e.g., "s", "n", "amp") and values are the corresponding `Argument` types.
    /// The arguments in the resulting `OSCMessage` will be ordered as `[key1, val1, key2, val2, ...]`, though the specific order
    /// might vary due to HashMap iteration order (which is usually acceptable for SuperDirt).
    ///
    /// # Arguments
    /// * `data` - A `HashMap` mapping SuperDirt parameter names (String) to their values (`Argument`).
    ///
    /// # Returns
    /// An `OSCMessage` with `addr` set to "/dirt/play" and `args` containing the flattened key-value pairs.
    pub fn dirt(data: HashMap<String, Argument>) -> Self {
        let mut args = Vec::with_capacity(data.len() * 2);
        // Optional: Sort keys for deterministic argument order, though usually not required by SuperDirt.

        // Unordered iteration is fine for SuperDirt:
        for (key, value) in data {
            args.push(Argument::String(key));
            args.push(value);
        }
        // TODO: add temporal information

        OSCMessage {
            addr: "/dirt/play".to_string(),
            args,
        }
    }
}
