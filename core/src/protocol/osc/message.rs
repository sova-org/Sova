use std::{collections::HashMap, fmt::Display};

use rosc::OscTime;
use serde::{Deserialize, Serialize};

use crate::{clock::{Clock, SyncTime}, vm::{event::ConcreteEvent, variable::VariableValue}, protocol::ProtocolPayload};

/// Represents a single OSC message, consisting of an address pattern and a list of arguments.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OSCMessage {
    /// The OSC address pattern (e.g., "/synth/play").
    pub addr: String,
    /// The list of arguments associated with the message.
    pub args: Vec<VariableValue>,
    /// An optional Timetag
    pub timetag: Option<(u32,u32)>
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
    pub fn new(addr: String, args: Vec<VariableValue>) -> Self {
        OSCMessage { addr, args, timetag: None }
    }

    /// Utility function to chain creation and date assignement of an OSCMessage
    pub fn at_date(mut self, date: Option<(u32,u32)>) -> Self {
        self.timetag = date;
        self
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
    /// * `date` - The date of the event
    /// * `duration` - The duration of the event
    /// * `clock` - The VM clock in order to perform conversions.
    ///
    /// # Returns
    /// An `OSCMessage` with `addr` set to "/dirt/play" and `args` containing the flattened key-value pairs.
    pub fn dirt(
        mut args: Vec<VariableValue>,
        date: SyncTime,
        duration: SyncTime,
        clock: &Clock,
    ) -> Self {
        // Calculate SuperDirt context using the clock
        let tempo_bpm = clock.tempo();
        let cps = tempo_bpm / 60.0;
        let cycle = clock.beat_at_date(date); // Beat at the event's specific time
        let delta = duration as f64 / 1_000_000.0;

        // Optional: Sort keys for deterministic argument order, though usually not required by SuperDirt.

        // Add temporal information required by SuperDirt
        args.push(VariableValue::Str("cps".to_string()));
        args.push(VariableValue::Float(cps));
        args.push(VariableValue::Str("cycle".to_string()));
        args.push(VariableValue::Float(cycle));
        args.push(VariableValue::Str("delta".to_string()));
        args.push(VariableValue::Float(delta));

        OSCMessage {
            addr: "/dirt/play".to_string(),
            args,
            timetag: None
        }
    }

    pub fn generate_messages(event: ConcreteEvent, date: SyncTime, clock: &Clock) 
        -> Vec<(ProtocolPayload, SyncTime)>
    {        
        let timetag = match OscTime::try_from(clock.to_system_time(date)) {
            Ok(t) => Some(t.into()),
            _ => None
        };
        match event {
            // Handle Generic OSC Event (pass-through)
            ConcreteEvent::Osc {
                mut message,
                device_id: _,
            } => {
                if message.timetag.is_none() {
                    message.timetag = timetag;
                }
                vec![(message.into(), date)]
            }
            // Handle Dirt Event (map to /dirt/play with context)
            ConcreteEvent::Dirt { args, device_id: _ } => {
                let mut flat_args = Vec::new();
                for (key, value) in args.into_iter() {
                    flat_args.push(VariableValue::Str(key));
                    flat_args.push(value);
                }

                let duration = clock.beats_to_micros(1.0);
                let dirt_msg = Self::dirt(flat_args, date, duration, clock)
                    .at_date(timetag);

                vec![(dirt_msg.into(), date)]
            }
            // Legacy MIDI-to-OSC mappings (consider removal/refinement)
            ConcreteEvent::MidiNote(note, vel, chan, _dur, _device_id) => {
                vec![(OSCMessage {
                    addr: "/midi/noteon".to_string(),
                    args: vec![
                        VariableValue::Integer(note as i64),
                        VariableValue::Integer(vel as i64),
                        VariableValue::Integer(chan as i64),
                    ],
                    timetag: timetag
                }.into(), date)]
            }
            ConcreteEvent::MidiControl(control, value, chan, _device_id) => {
                vec![(OSCMessage {
                    addr: "/midi/cc".to_string(),
                    args: vec![
                        VariableValue::Integer(control as i64),
                        VariableValue::Integer(value as i64),
                        VariableValue::Integer(chan as i64),
                    ],
                    timetag: timetag
                }.into(), date)]
            }
            ConcreteEvent::MidiProgram(program, chan, _device_id) => {
                vec![(OSCMessage {
                    addr: "/midi/program".to_string(),
                    args: vec![
                        VariableValue::Integer(program as i64),
                        VariableValue::Integer(chan as i64),
                    ],
                    timetag: timetag
                }.into(), date)]
            }
            ConcreteEvent::Generic(args, duration, channel, _device_id) => {
                let mut flat_args = Vec::new();
                let mut args = match args {
                    VariableValue::Map(map) => map,
                    value => {
                        let mut map = HashMap::new();
                        map.insert("sound".to_owned(), value);
                        map
                    }
                };
                if (args.contains_key("s") || args.contains_key("sound")) && !args.contains_key("sustain") {
                    let dur_s = (duration as f64) / 1_000_000.0;
                    args.insert("sustain".to_owned(), dur_s.into());
                }
                for (key, value) in args.into_iter() {
                    flat_args.push(VariableValue::Str(key));
                    flat_args.push(value);
                }
                let mut dirt_msg = Self::dirt(flat_args, date, duration, clock)
                    .at_date(timetag);

                if !channel.is_empty() {
                    dirt_msg.addr = channel;
                }

                vec![(dirt_msg.into(), date)]
            }
            _ => Vec::new(), // Ignore other events for OSC for now
        }
    }

}
