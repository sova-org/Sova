use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

use crate::clock::{Clock, SyncTime};
use crate::lang::event::ConcreteEvent;
use crate::protocol::payload::ProtocolPayload;

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
    /// * `cps` - The cps (cycles per second) parameter.
    /// * `cycle` - The cycle parameter.
    /// * `delta` - The delta parameter.
    /// * `orbit` - The orbit parameter.
    ///
    /// # Returns
    /// An `OSCMessage` with `addr` set to "/dirt/play" and `args` containing the flattened key-value pairs.
    pub fn dirt(
        data: HashMap<String, Argument>,
        cps: f32,
        cycle: f32,
        delta: f32,
        orbit: i32,
    ) -> Self {
        let mut args = Vec::with_capacity(data.len() * 2 + 8); // +8 for the 4 temporal key-value pairs
        // Optional: Sort keys for deterministic argument order, though usually not required by SuperDirt.

        // Unordered iteration is fine for SuperDirt:
        for (key, value) in data {
            args.push(Argument::String(key));
            args.push(value);
        }

        // Add temporal information required by SuperDirt
        args.push(Argument::String("cps".to_string()));
        args.push(Argument::Float(cps));
        args.push(Argument::String("cycle".to_string()));
        args.push(Argument::Float(cycle));
        args.push(Argument::String("delta".to_string()));
        args.push(Argument::Float(delta));
        args.push(Argument::String("orbit".to_string()));
        args.push(Argument::Int(orbit));

        OSCMessage {
            addr: "/dirt/play".to_string(),
            args,
        }
    }

    pub fn generate_messages(event: ConcreteEvent, date: SyncTime, clock: &Clock) 
        -> Vec<(ProtocolPayload, SyncTime)>
    {
        match event {
            // Handle Generic OSC Event (pass-through)
            ConcreteEvent::Osc {
                message,
                device_id: _,
            } => vec![(message.into(), date)],
            // Handle Dirt Event (map to /dirt/play with context)
            ConcreteEvent::Dirt { args, device_id: _ } => {
                // Calculate SuperDirt context using the clock
                let tempo_bpm = clock.tempo();
                let cps_val = tempo_bpm / 60.0;
                let cycle_val = clock.beat_at_date(date); // Beat at the event's specific time
                let delta_micros = clock.beats_to_micros(1.0); // Use 1 beat for delta
                let delta_val = delta_micros as f64 / 1_000_000.0;
                let orbit_val = 0i32; // Default orbit

                let capacity = 4 * 2 + 2 + args.len();
                let mut full_args: Vec<Argument> = Vec::with_capacity(capacity);

                // Add context parameters
                full_args.push(Argument::String("cps".to_string()));
                full_args.push(Argument::Float(cps_val as f32));
                full_args.push(Argument::String("cycle".to_string()));
                full_args.push(Argument::Float(cycle_val as f32));
                full_args.push(Argument::String("delta".to_string()));
                full_args.push(Argument::Float(delta_val as f32));
                full_args.push(Argument::String("orbit".to_string()));
                full_args.push(Argument::Int(orbit_val));

                // Add other parameters
                full_args.extend(args);

                vec![(OSCMessage {
                    addr: "/dirt/play".to_string(),
                    args: full_args,
                }.into(), date)]
            }
            // Legacy MIDI-to-OSC mappings (consider removal/refinement)
            ConcreteEvent::MidiNote(note, vel, chan, _dur, _device_id) => {
                vec![(OSCMessage {
                    addr: "/midi/noteon".to_string(),
                    args: vec![
                        Argument::Int(note as i32),
                        Argument::Int(vel as i32),
                        Argument::Int(chan as i32),
                    ],
                }.into(), date)]
            }
            ConcreteEvent::MidiControl(control, value, chan, _device_id) => {
                vec![(OSCMessage {
                    addr: "/midi/cc".to_string(),
                    args: vec![
                        Argument::Int(control as i32),
                        Argument::Int(value as i32),
                        Argument::Int(chan as i32),
                    ],
                }.into(), date)]
            }
            ConcreteEvent::MidiProgram(program, chan, _device_id) => {
                vec![(OSCMessage {
                    addr: "/midi/program".to_string(),
                    args: vec![
                        Argument::Int(program as i32),
                        Argument::Int(chan as i32),
                    ],
                }.into(), date)]
            }
            _ => Vec::new(), // Ignore other events for OSC for now
        }
    }

}
