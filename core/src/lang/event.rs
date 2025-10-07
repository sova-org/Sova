use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::clock::SyncTime;
use crate::lang::Program;
use crate::protocol::osc::{Argument, OSCMessage};
use crate::util::decimal_operations::float64_from_decimal;
use crate::log_eprintln;

use super::variable::VariableValue;
use super::{evaluation_context::EvaluationContext, variable::Variable};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        args: Vec<Argument>,
        device_id: usize,
    },
    Osc {
        message: OSCMessage,
        device_id: usize,
    },
    StartProgram(Program)
}

impl ConcreteEvent {
    pub fn device_id(&self) -> usize {
        match self {
            ConcreteEvent::MidiNote(_, _, _, _, device_id) 
            | ConcreteEvent::MidiControl(_, _, _, device_id) 
            | ConcreteEvent::MidiProgram(_, _, device_id) 
            | ConcreteEvent::MidiAftertouch(_, _, _, device_id) 
            | ConcreteEvent::MidiChannelPressure(_, _, device_id) 
            | ConcreteEvent::MidiSystemExclusive(_, device_id) 
            | ConcreteEvent::MidiStart(device_id) 
            | ConcreteEvent::MidiStop(device_id) 
            | ConcreteEvent::MidiReset(device_id) 
            | ConcreteEvent::MidiContinue(device_id) 
            | ConcreteEvent::MidiClock(device_id) 
            | ConcreteEvent::Dirt { args: _, device_id } 
            | ConcreteEvent::Osc { message: _, device_id } 
                => *device_id,
            ConcreteEvent::Nop 
            | ConcreteEvent::StartProgram(_) 
                => usize::MAX,
        }
    }
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
        sound: Variable,
        params: HashMap<String, Variable>,
        device_id: Variable,
    }, // corresponding ConcreteEvent is directly Osc
    Osc {
        addr: Variable,
        args: Vec<Variable>,
        device_id: Variable,
    },
    StartProgram(Variable)
}

impl Event {
    pub fn make_concrete(&self, ctx: &mut EvaluationContext) -> ConcreteEvent {
        match &self {
            Event::Nop => ConcreteEvent::Nop,
            Event::MidiNote(note, vel, chan, time, dev) => {
                let note = ctx.evaluate(note).as_integer(ctx.clock, ctx.frame_len) as u64;
                let time = ctx
                    .evaluate(time)
                    .as_dur()
                    .as_micros(ctx.clock, ctx.frame_len);
                let chan = ctx.evaluate(chan).as_integer(ctx.clock, ctx.frame_len) as u64;
                let vel = ctx.evaluate(vel).as_integer(ctx.clock, ctx.frame_len) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize;
                ConcreteEvent::MidiNote(note, vel, chan, time, dev_id)
            }
            Event::MidiControl(control, value, channel, dev) => {
                let control = ctx.evaluate(control).as_integer(ctx.clock, ctx.frame_len) as u64;
                let value = ctx.evaluate(value).as_integer(ctx.clock, ctx.frame_len) as u64;
                let channel = ctx.evaluate(channel).as_integer(ctx.clock, ctx.frame_len) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize;
                ConcreteEvent::MidiControl(control, value, channel, dev_id)
            }
            Event::MidiProgram(program, channel, dev) => {
                let program = ctx.evaluate(program).as_integer(ctx.clock, ctx.frame_len) as u64;
                let channel = ctx.evaluate(channel).as_integer(ctx.clock, ctx.frame_len) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize;
                ConcreteEvent::MidiProgram(program, channel, dev_id)
            }
            Event::MidiAftertouch(note, pressure, channel, dev) => {
                let note = ctx.evaluate(note).as_integer(ctx.clock, ctx.frame_len) as u64;
                let pressure = ctx
                    .evaluate(pressure)
                    .as_integer(ctx.clock, ctx.frame_len) as u64;
                let channel = ctx.evaluate(channel).as_integer(ctx.clock, ctx.frame_len) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize;
                ConcreteEvent::MidiAftertouch(note, pressure, channel, dev_id)
            }
            Event::MidiChannelPressure(pressure, channel, dev) => {
                let channel = ctx.evaluate(channel).as_integer(ctx.clock, ctx.frame_len) as u64;
                let pressure = ctx
                    .evaluate(pressure)
                    .as_integer(ctx.clock, ctx.frame_len) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize;
                ConcreteEvent::MidiChannelPressure(pressure, channel, dev_id)
            }
            Event::MidiSystemExclusive(data, dev) => {
                let d: Vec<u64> = data
                    .iter()
                    .map(|v| ctx.evaluate(v).as_integer(ctx.clock, ctx.frame_len) as u64)
                    .collect();
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize;
                ConcreteEvent::MidiSystemExclusive(d, dev_id)
            }
            Event::MidiStart(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize;
                ConcreteEvent::MidiStart(dev_id)
            }
            Event::MidiStop(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize;
                ConcreteEvent::MidiStop(dev_id)
            }
            Event::MidiReset(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize;
                ConcreteEvent::MidiReset(dev_id)
            }
            Event::MidiContinue(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize;
                ConcreteEvent::MidiContinue(dev_id)
            }
            Event::MidiClock(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize;
                ConcreteEvent::MidiClock(dev_id)
            }
            Event::Dirt {
                sound,
                params,
                device_id,
            } => {
                // get device
                let device_id =
                    ctx.evaluate(device_id)
                        .as_integer(ctx.clock, ctx.frame_len) as usize;

                // get args
                let mut args = Vec::new();

                // add sound to args
                args.push(Argument::String("s".to_string()));
                let sound = ctx.evaluate(sound);
                let sound = match sound {
                    VariableValue::Integer(i) => Argument::Int(i as i32),
                    VariableValue::Float(f) => Argument::Float(f as f32),
                    VariableValue::Decimal(sig, num, den) => {
                        Argument::Float(float64_from_decimal(sig, num, den) as f32)
                    }
                    VariableValue::Str(s) => Argument::String(s),
                    _ => todo!(),
                };
                args.push(sound);

                // add params to args
                for (key, value) in params {
                    args.push(Argument::String(key.clone()));
                    let param_arg = match ctx.evaluate(value) {
                        VariableValue::Integer(i) => Argument::Int(i as i32),
                        VariableValue::Float(f) => Argument::Float(f as f32),
                        VariableValue::Decimal(sig, num, den) => {
                            Argument::Float(float64_from_decimal(sig, num, den) as f32)
                        }
                        VariableValue::Str(s) => Argument::String(s),
                        VariableValue::Bool(b) => Argument::Int(if b { 1 } else { 0 }),
                        _ => {
                            log_eprintln!(
                                "[WARN] Dirt to OSC: Unsupported param type {:?} for key '{}'. Sending Int 0.",
                                value, key
                            );
                            Argument::Int(0)
                        }
                    };
                    args.push(param_arg);
                }

                ConcreteEvent::Dirt { args, device_id }
            }
            Event::Osc {
                addr,
                args,
                device_id,
            } => {
                let dev_id = ctx
                    .evaluate(device_id)
                    .as_integer(ctx.clock, ctx.frame_len) as usize;
                let addr = ctx.evaluate(addr).as_str(ctx.clock, ctx.frame_len);
                let mut osc_args = Vec::new();
                for arg in args.iter() {
                    let arg = ctx.evaluate(arg);
                    let arg = match arg {
                        VariableValue::Integer(i) => Argument::Int(i as i32),
                        VariableValue::Float(f) => Argument::Float(f as f32),
                        VariableValue::Decimal(sig, num, den) => {
                            Argument::Float(float64_from_decimal(sig, num, den) as f32)
                        }
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
            Event::StartProgram(var) => {
                if let VariableValue::Func(fun) = ctx.evaluate(var) {
                    ConcreteEvent::StartProgram(fun)
                } else {
                    ConcreteEvent::StartProgram(Program::default())
                }
            }
        }
    }
}
