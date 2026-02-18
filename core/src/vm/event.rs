use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::clock::SyncTime;
use crate::protocol::osc::OSCMessage;
use crate::vm::Program;

use super::variable::VariableValue;
use super::{EvaluationContext, variable::Variable};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConcreteEvent {
    Nop,
    Print(String),
    MidiNote(u64, u64, u64, SyncTime, usize),
    MidiPitchBend(u16, u64, usize),
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
        args: HashMap<String, VariableValue>,
        device_id: usize,
    },
    Osc {
        message: OSCMessage,
        device_id: usize,
    },
    StartProgram(Program),
    Generic(VariableValue, SyncTime, String, usize),
}

impl ConcreteEvent {
    pub fn device_id(&self) -> Option<usize> {
        match self {
            ConcreteEvent::MidiNote(_, _, _, _, device_id)
            | ConcreteEvent::MidiPitchBend(_, _, device_id)
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
            | ConcreteEvent::Osc {
                message: _,
                device_id,
            }
            | ConcreteEvent::Generic(_, _, _, device_id) => Some(*device_id),
            ConcreteEvent::Print(_) => Some(0),
            ConcreteEvent::Nop | ConcreteEvent::StartProgram(_) => None,
        }
    }
}

impl fmt::Display for ConcreteEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConcreteEvent::Nop => write!(f, "nop"),
            ConcreteEvent::Print(s) => write!(f, "{s}"),
            ConcreteEvent::MidiNote(note, vel, ch, dur, dev) =>
                write!(f, "note {note} vel {vel} ch {ch} dur {dur}us dev {dev}"),
            ConcreteEvent::MidiPitchBend(value, ch, dev) =>
                write!(f, "bend {value} ch {ch} dev {dev}"),
            ConcreteEvent::MidiControl(cc, val, ch, dev) =>
                write!(f, "cc {cc} val {val} ch {ch} dev {dev}"),
            ConcreteEvent::MidiProgram(pg, ch, dev) =>
                write!(f, "pgm {pg} ch {ch} dev {dev}"),
            ConcreteEvent::MidiAftertouch(note, pressure, ch, dev) =>
                write!(f, "at {note} pressure {pressure} ch {ch} dev {dev}"),
            ConcreteEvent::MidiChannelPressure(pressure, ch, dev) =>
                write!(f, "cp {pressure} ch {ch} dev {dev}"),
            ConcreteEvent::MidiSystemExclusive(data, dev) =>
                write!(f, "sysex {:?} dev {dev}", data),
            ConcreteEvent::MidiStart(dev) => write!(f, "midi-start dev {dev}"),
            ConcreteEvent::MidiStop(dev) => write!(f, "midi-stop dev {dev}"),
            ConcreteEvent::MidiReset(dev) => write!(f, "midi-reset dev {dev}"),
            ConcreteEvent::MidiContinue(dev) => write!(f, "midi-continue dev {dev}"),
            ConcreteEvent::MidiClock(dev) => write!(f, "midi-clock dev {dev}"),
            ConcreteEvent::Dirt { args, device_id } =>
                write!(f, "dirt {args:?} dev {device_id}"),
            ConcreteEvent::Osc { message, device_id } =>
                write!(f, "osc {} dev {device_id}", message.addr),
            ConcreteEvent::StartProgram(_) => write!(f, "start-program"),
            ConcreteEvent::Generic(val, dur, ch, dev) =>
                write!(f, "generic {val:?} dur {dur}us ch {ch} dev {dev}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Nop,
    Print(Variable),
    /// MidiNote(note, velocity, channel, duration, device_id)
    MidiNote(Variable, Variable, Variable, Variable, Variable),
    /// MidiPitchBend(value, channel, device_id)
    MidiPitchBend(Variable, Variable, Variable),
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
    },
    Osc {
        addr: Variable,
        args: Vec<Variable>,
        device_id: Variable,
    },
    StartProgram(Variable),

    /// ----- Generic events -----

    /// Generic event: value, duration, channel, device
    Generic(Variable, Variable, Variable, Variable),
}

impl Event {
    pub fn make_concrete(&self, ctx: &mut EvaluationContext) -> ConcreteEvent {
        match &self {
            Event::Nop => ConcreteEvent::Nop,
            Event::Print(var) => ConcreteEvent::Print(ctx.evaluate(var).as_str(ctx)),
            Event::MidiNote(note, vel, chan, time, dev) => {
                let note = ctx.evaluate(note).as_integer(ctx) as u64;
                let time = ctx
                    .evaluate(time)
                    .as_dur(ctx)
                    .as_micros(ctx.clock, ctx.frame_len);
                let chan = ctx.evaluate(chan).as_integer(ctx) as u64;
                let vel = ctx.evaluate(vel).as_integer(ctx) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx) as usize;
                ConcreteEvent::MidiNote(note, vel, chan, time, dev_id)
            }
            Event::MidiPitchBend(value, channel, dev) => {
                let value = ctx.evaluate(value).as_integer(ctx) as u16;
                let channel = ctx.evaluate(channel).as_integer(ctx) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx) as usize;
                ConcreteEvent::MidiPitchBend(value, channel, dev_id)
            }
            Event::MidiControl(control, value, channel, dev) => {
                let control = ctx.evaluate(control).as_integer(ctx) as u64;
                let value = ctx.evaluate(value).as_integer(ctx) as u64;
                let channel = ctx.evaluate(channel).as_integer(ctx) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx) as usize;
                ConcreteEvent::MidiControl(control, value, channel, dev_id)
            }
            Event::MidiProgram(program, channel, dev) => {
                let program = ctx.evaluate(program).as_integer(ctx) as u64;
                let channel = ctx.evaluate(channel).as_integer(ctx) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx) as usize;
                ConcreteEvent::MidiProgram(program, channel, dev_id)
            }
            Event::MidiAftertouch(note, pressure, channel, dev) => {
                let note = ctx.evaluate(note).as_integer(ctx) as u64;
                let pressure = ctx.evaluate(pressure).as_integer(ctx) as u64;
                let channel = ctx.evaluate(channel).as_integer(ctx) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx) as usize;
                ConcreteEvent::MidiAftertouch(note, pressure, channel, dev_id)
            }
            Event::MidiChannelPressure(pressure, channel, dev) => {
                let channel = ctx.evaluate(channel).as_integer(ctx) as u64;
                let pressure = ctx.evaluate(pressure).as_integer(ctx) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx) as usize;
                ConcreteEvent::MidiChannelPressure(pressure, channel, dev_id)
            }
            Event::MidiSystemExclusive(data, dev) => {
                let d: Vec<u64> = data
                    .iter()
                    .map(|v| ctx.evaluate(v).as_integer(ctx) as u64)
                    .collect();
                let dev_id = ctx.evaluate(dev).as_integer(ctx) as usize;
                ConcreteEvent::MidiSystemExclusive(d, dev_id)
            }
            Event::MidiStart(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx) as usize;
                ConcreteEvent::MidiStart(dev_id)
            }
            Event::MidiStop(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx) as usize;
                ConcreteEvent::MidiStop(dev_id)
            }
            Event::MidiReset(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx) as usize;
                ConcreteEvent::MidiReset(dev_id)
            }
            Event::MidiContinue(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx) as usize;
                ConcreteEvent::MidiContinue(dev_id)
            }
            Event::MidiClock(dev) => {
                let dev_id = ctx.evaluate(dev).as_integer(ctx) as usize;
                ConcreteEvent::MidiClock(dev_id)
            }
            Event::Dirt {
                sound,
                params,
                device_id,
            } => {
                let device_id = ctx.evaluate(device_id).as_integer(ctx) as usize;

                let mut params: HashMap<String, VariableValue> = params
                    .iter()
                    .map(|(key, value)| (key.clone(), ctx.evaluate(value)))
                    .collect();
                params.insert("sound".to_string(), ctx.evaluate(sound));

                ConcreteEvent::Dirt {
                    args: params,
                    device_id,
                }
            }
            Event::Osc {
                addr,
                args,
                device_id,
            } => {
                let dev_id = ctx.evaluate(device_id).as_integer(ctx) as usize;
                let addr = ctx.evaluate(addr).as_str(ctx);
                let osc_args = args.iter().map(|var| ctx.evaluate(var)).collect();
                let message = OSCMessage::new(addr, osc_args);
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
            Event::Generic(value, duration, channel, device) => ConcreteEvent::Generic(
                ctx.evaluate(value),
                ctx.evaluate(duration)
                    .as_dur(ctx)
                    .as_micros(ctx.clock, ctx.frame_len),
                ctx.evaluate(channel).as_str(ctx),
                ctx.evaluate(device).as_integer(ctx) as usize,
            ),
        }
    }
}
