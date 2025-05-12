use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::clock::SyncTime;
use crate::protocol::osc::{OSCMessage, Argument};
use crate::util::decimal_operations::float64_from_decimal;


use super::variable::VariableValue;
use super::{evaluation_context::EvaluationContext, variable::Variable};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConcreteEvent {
    Nop,
    MidiNote(u64, u64, u64, SyncTime, usize),
    // TODO: MIDI Pitchbend
    MidiControl(u64, u64, u64, usize),
    MidiProgram(u64, u64, usize),
    MidiAftertouch(u64, u64, u64, usize),
    MidiChannelPressure(u64, u64, usize),
    MidiSystemExclusive(Vec<u64>, usize),
    MidiStart(usize),
    MidiStop(usize),
    MidiReset(usize),
    MidiContinue(usize),
    MidiClock(usize),
    Dirt {
        sound: VariableValue,
        params: HashMap<String, VariableValue>,
        device_id: usize,
    },
    Osc {
        message: OSCMessage,
        device_id: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Nop,
    MidiNote(Variable, Variable, Variable, Variable, Variable),
    // TODO: MIDI Pitchbend
    MidiControl(Variable, Variable, Variable, Variable),
    MidiProgram(Variable, Variable, Variable),
    MidiAftertouch(Variable, Variable, Variable, Variable),
    MidiChannelPressure(Variable, Variable, Variable),
    MidiSystemExclusive(Vec<Variable>, Variable),
    MidiStart(Variable),
    MidiStop(Variable),
    MidiReset(Variable),
    MidiContinue(Variable),
    MidiClock(Variable),
    Dirt {
        data: Variable,
        device_id: Variable,
    },
    Osc {
        addr: Variable,
        args: Vec<Variable>,
        device_id: Variable,
    },
}

impl Event {
    pub fn make_concrete(&self, ctx: &mut EvaluationContext) -> ConcreteEvent {
        match &self {
            Event::Nop => ConcreteEvent::Nop,
            Event::MidiNote(note, vel, chan, time, dev) => {
                let note = ctx.evaluate(note).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let time = ctx
                    .evaluate(time)
                    .as_dur()
                    .as_micros(ctx.clock, ctx.frame_len());
                let chan = ctx.evaluate(chan).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let vel = ctx.evaluate(vel).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len()) as usize;
                ConcreteEvent::MidiNote(note, vel, chan, time, dev_id)
            }
            Event::MidiControl(control, value, channel, dev) => {
                let control = ctx.evaluate(control).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let value = ctx.evaluate(value).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let channel = ctx.evaluate(channel).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len()) as usize;
                ConcreteEvent::MidiControl(control, value, channel, dev_id)
            }
            Event::MidiProgram(program, channel, dev) => {
                let program = ctx.evaluate(program).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let channel = ctx.evaluate(channel).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len()) as usize;
                ConcreteEvent::MidiProgram(program, channel, dev_id)
            }
            Event::MidiAftertouch(note, pressure, channel, dev) => {
                let note = ctx.evaluate(note).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let pressure = ctx
                    .evaluate(pressure)
                    .as_integer(ctx.clock, ctx.frame_len()) as u64;
                let channel = ctx.evaluate(channel).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len()) as usize;
                ConcreteEvent::MidiAftertouch(note, pressure, channel, dev_id)
            }
            Event::MidiChannelPressure(pressure, channel, dev) => {
                let channel = ctx.evaluate(channel).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let pressure = ctx
                    .evaluate(pressure)
                    .as_integer(ctx.clock, ctx.frame_len()) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len()) as usize;
                ConcreteEvent::MidiChannelPressure(pressure, channel, dev_id)
            }
            Event::MidiSystemExclusive(data, dev) => {
                let d: Vec<u64> = data
                    .iter()
                    .map(|v| ctx.evaluate(v).as_integer(ctx.clock, ctx.frame_len()) as u64)
                    .collect();
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len()) as usize;
                ConcreteEvent::MidiSystemExclusive(d, dev_id)
            }
            Event::MidiStart(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len()) as usize;
                ConcreteEvent::MidiStart(dev_id)
            }
            Event::MidiStop(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len()) as usize;
                ConcreteEvent::MidiStop(dev_id)
            }
            Event::MidiReset(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len()) as usize;
                ConcreteEvent::MidiReset(dev_id)
            }
            Event::MidiContinue(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len()) as usize;
                ConcreteEvent::MidiContinue(dev_id)
            }
            Event::MidiClock(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len()) as usize;
                ConcreteEvent::MidiClock(dev_id)
            }
            Event::Dirt { data, device_id } => {
                let dev_id = ctx
                    .evaluate(device_id)
                    .as_integer(ctx.clock, ctx.frame_len()) as usize;
                let evaluated_data_var = ctx.evaluate(data);

                // Initialize default sound and empty params
                let mut concrete_sound = VariableValue::Str("default".to_string()); // Default sound
                let mut concrete_params = HashMap::new();

                match evaluated_data_var.as_map() {
                    Some(map_var) => {
                        for (key, value_var) in map_var {
                            // Convert VariableValue to OscArgument
                            let concrete_value = match value_var {
                                VariableValue::Integer(i) => VariableValue::Integer(*i),
                                VariableValue::Float(f) => VariableValue::Float(*f),
                                VariableValue::Str(s) => VariableValue::Str(s.clone()),
                                // Add other necessary conversions if Map can hold more types
                                _ => {
                                    eprintln!(
                                        "[!] Warning: Unsupported value type ({:?}) in Dirt event data map for key '{}'. Skipping.",
                                        value_var, key
                                    );
                                    continue;
                                }
                            };

                            // Separate sound ('s') from other params
                            if key == "s" {
                                concrete_sound = concrete_value;
                            } else {
                                concrete_params.insert(key.clone(), concrete_value);
                            }
                        }
                    }
                    None => {
                        eprintln!(
                            "[!] Warning: Dirt event data did not evaluate to a Map. Using default sound and empty params. Evaluated to: {:?}",
                            evaluated_data_var
                        );
                        // Keep default sound and empty params
                    }
                };

                ConcreteEvent::Dirt {
                    sound: concrete_sound,   // Use the separated sound value
                    params: concrete_params, // Use the separated params map
                    device_id: dev_id,
                }
            }
            Event::Osc { addr, args, device_id } => {
                let dev_id = ctx
                    .evaluate(device_id)
                    .as_integer(ctx.clock, ctx.frame_len()) as usize;
                let addr = ctx.evaluate(addr).as_str(ctx.clock, ctx.frame_len());
                let mut osc_args = Vec::new();
                for arg in args.iter() {
                    let arg = ctx.evaluate(arg);
                    let arg = match arg {
                        VariableValue::Integer(i) => Argument::Int(i as i32),
                        VariableValue::Float(f) => Argument::Float(f as f32),
                        VariableValue::Decimal(sig, num, den) => Argument::Float(float64_from_decimal(sig, num, den) as f32),
                        VariableValue::Str(s) => Argument::String(s),
                        _ => todo!(),
                    };
                    osc_args.push(arg);
                }
                let message = OSCMessage {
                    addr,
                    args: osc_args,
                };
                ConcreteEvent::Osc {
                    message,
                    device_id: dev_id,
                }
            }
        }
    }
}
