use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::clock::SyncTime;
use crate::vm::Program;
use crate::protocol::osc::OSCMessage;

use super::variable::VariableValue;
use super::{EvaluationContext, variable::Variable};

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
        args: Vec<VariableValue>,
        device_id: usize,
    },
    Osc {
        message: OSCMessage,
        device_id: usize,
    },
    StartProgram(Program),
    Sound(SyncTime, SyncTime, VariableValue, VariableValue, usize),
    VoiceSound(VariableValue, SyncTime, SyncTime, VariableValue, VariableValue, usize),
    VoiceSetting(VariableValue, SyncTime, VariableValue, VariableValue, usize),
}

impl ConcreteEvent {
    pub fn device_id(&self) -> Option<usize> {
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
            | ConcreteEvent::Sound(_, _, _, _, device_id)
            | ConcreteEvent::VoiceSound(_, _, _, _, _, device_id) 
            | ConcreteEvent::VoiceSetting(_, _, _, _, device_id) 
                => Some(*device_id),
            ConcreteEvent::Nop 
            | ConcreteEvent::StartProgram(_) 
                => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Nop,
    /// MidiNote(note, velocity, channel, duration, device_id)
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
    },
    Osc {
        addr: Variable,
        args: Vec<Variable>,
        device_id: Variable,
    },
    StartProgram(Variable),
    
    /// ----- Generic events -----

    /// Play a sound : Sound(delay_before, duration, instrument, value, device)
    /// Supported by : Midi, Dough, Dirt 
    Sound(Variable, Variable, Variable, Variable, Variable),
    /// Play a sound on a voice : Sound(voice, delay_before, duration, instrument, value, device)
    /// Supported by : Midi, Dough, Dirt 
    VoiceSound(Variable, Variable, Variable, Variable, Variable, Variable),
    /// Change voice setting : Setting(voice, delay_before, setting, value, device)
    /// Supported by : Midi, Dough 
    VoiceSetting(Variable, Variable, Variable, Variable, Variable),
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
                args.push(VariableValue::Str("s".to_string()));
                args.push(ctx.evaluate(sound));

                // add params to args
                for (key, value) in params {
                    args.push(VariableValue::Str(key.clone()));
                    args.push(ctx.evaluate(value));
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
            Event::Sound(delay_before, duration, instrument, value, dev) => {
                ConcreteEvent::Sound(
                    ctx.evaluate(delay_before).as_dur().as_micros(ctx.clock, ctx.frame_len), 
                    ctx.evaluate(duration).as_dur().as_micros(ctx.clock, ctx.frame_len), 
                    ctx.evaluate(instrument), 
                    ctx.evaluate(value), 
                    ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize
                )
            }
            Event::VoiceSound(voice, delay_before, duration, instrument, value, dev) => {
                ConcreteEvent::VoiceSound(
                    ctx.evaluate(voice), 
                    ctx.evaluate(delay_before).as_dur().as_micros(ctx.clock, ctx.frame_len), 
                    ctx.evaluate(duration).as_dur().as_micros(ctx.clock, ctx.frame_len), 
                    ctx.evaluate(instrument), 
                    ctx.evaluate(value), 
                    ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize
                )
            }
            Event::VoiceSetting(voice, delay_before, setting, value, dev) => {
                ConcreteEvent::VoiceSetting(
                    ctx.evaluate(voice), 
                    ctx.evaluate(delay_before).as_dur().as_micros(ctx.clock, ctx.frame_len), 
                    ctx.evaluate(setting), 
                    ctx.evaluate(value), 
                    ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len) as usize
                )
            }
        }
    }
}
