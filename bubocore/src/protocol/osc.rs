use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

/// Represents the different types of arguments an OSC message can have.
/// We start with the basics: Int, Float, String.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Argument {
    Int(i32),
    Float(f32),
    String(String),
    Blob(Vec<u8>),
    Timetag(u64),
    // Add other types like Timetag, etc., if needed later
}

impl Eq for Argument {}

impl std::hash::Hash for Argument {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OSCMessage {
    pub addr: String,
    pub args: Vec<Argument>,
}

impl Display for OSCMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OSCMessage {{ addr: \"{}\", args: {:?} }}",
            self.addr, self.args
        )
    }
}

impl OSCMessage {
    /// Creates a standard OSC message.
    pub fn new(addr: String, args: Vec<Argument>) -> Self {
        OSCMessage { addr, args }
    }

    /// Creates an OSC message specifically formatted for SuperDirt's /dirt/play address.
    ///
    /// Takes a HashMap where keys are SuperDirt parameter names (e.g., "s", "n", "amp")
    /// and values are the corresponding `Argument` types (Int, Float, String).
    /// The arguments in the resulting OSC message will be flattened into [key1, val1, key2, val2, ...].
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
