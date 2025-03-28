use serde::{Deserialize, Serialize};

use crate::clock::SyncTime;

use super::{evaluation_context::EvaluationContext, variable::Variable};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConcreteEvent {
    Nop,
    MidiNote(u64, u64, u64, SyncTime, String),
    // TODO: MIDI Pitchbend
    MidiControl(u64, u64, u64, String),
    MidiProgram(u64, u64, String),
    MidiAftertouch(u64, u64, u64, String),
    MidiChannelPressure(u64, u64, String),
    MidiSystemExclusive(Vec<u64>, String),
    MidiStart(String),
    MidiStop(String),
    MidiReset(String),
    MidiContinue(String),
    MidiClock(String),
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
}

impl Event {
    pub fn make_concrete(&self, ctx: &mut EvaluationContext) -> ConcreteEvent {
        match &self {
            Event::Nop => ConcreteEvent::Nop,
            Event::MidiNote(note, vel, chan, time, dev) => {
                let note = ctx.evaluate(note).as_integer(ctx) as u64;
                let time = ctx.evaluate(time).as_dur().as_micros(ctx.clock, ctx.step_len());
                let chan = ctx.evaluate(chan).as_integer(ctx) as u64;
                let vel = ctx.evaluate(vel).as_integer(ctx) as u64;
                let dev = ctx.evaluate(dev).as_str(ctx);
                ConcreteEvent::MidiNote(note, vel, chan, time, dev)
            }
            Event::MidiControl(control, value, channel, dev) => {
                let control = ctx.evaluate(control).as_integer(ctx) as u64;
                let value = ctx.evaluate(value).as_integer(ctx) as u64;
                let channel = ctx.evaluate(channel).as_integer(ctx) as u64;
                let dev = ctx.evaluate(dev).as_str(ctx);
                ConcreteEvent::MidiControl(control, value, channel, dev)
            }
            Event::MidiProgram(program, channel, dev) => {
                let program = ctx.evaluate(program).as_integer(ctx) as u64;
                let channel = ctx.evaluate(channel).as_integer(ctx) as u64;
                let dev = ctx.evaluate(dev).as_str(ctx);
                ConcreteEvent::MidiProgram(program, channel, dev)
            }
            Event::MidiAftertouch(note, pressure, channel, dev) => {
                let note = ctx.evaluate(note).as_integer(ctx) as u64;
                let pressure = ctx.evaluate(pressure).as_integer(ctx) as u64;
                let channel = ctx.evaluate(channel).as_integer(ctx) as u64;
                let dev = ctx.evaluate(dev).as_str(ctx);
                ConcreteEvent::MidiAftertouch(note, pressure, channel, dev)
            }
            Event::MidiChannelPressure(pressure, channel, dev) => {
                let channel = ctx.evaluate(channel).as_integer(ctx) as u64;
                let pressure = ctx.evaluate(pressure).as_integer(ctx) as u64;
                let dev = ctx.evaluate(dev).as_str(ctx);
                ConcreteEvent::MidiChannelPressure(pressure, channel, dev)
            }
            Event::MidiSystemExclusive(data, dev) => {
                let d: Vec<u64> = data
                    .iter()
                    .map(|v| ctx.evaluate(v).as_integer(ctx) as u64)
                    .collect();
                let dev = ctx.evaluate(dev).as_str(ctx);
                ConcreteEvent::MidiSystemExclusive(d, dev)
            }
            Event::MidiStart(dev) => {
                let dev = ctx.evaluate(dev).as_str(ctx);
                ConcreteEvent::MidiStart(dev)
            }
            Event::MidiStop(dev) => {
                let dev = ctx.evaluate(dev).as_str(ctx);
                ConcreteEvent::MidiStop(dev)
            }
            Event::MidiReset(dev) => {
                let dev = ctx.evaluate(dev).as_str(ctx);
                ConcreteEvent::MidiReset(dev)
            }
            Event::MidiContinue(dev) => {
                let dev = ctx.evaluate(dev).as_str(ctx);
                ConcreteEvent::MidiContinue(dev)
            }
            Event::MidiClock(dev) => {
                let dev = ctx.evaluate(dev).as_str(ctx);
                ConcreteEvent::MidiClock(dev)
            }
        }
    }
}
