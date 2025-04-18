use serde::{Deserialize, Serialize};

use crate::clock::SyncTime;

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
                let note = ctx.evaluate(note).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let time = ctx.evaluate(time).as_dur().as_micros(ctx.clock, ctx.frame_len());
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
                let pressure = ctx.evaluate(pressure).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let channel = ctx.evaluate(channel).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let dev_id = ctx.evaluate(dev).as_integer(ctx.clock, ctx.frame_len()) as usize;
                ConcreteEvent::MidiAftertouch(note, pressure, channel, dev_id)
            }
            Event::MidiChannelPressure(pressure, channel, dev) => {
                let channel = ctx.evaluate(channel).as_integer(ctx.clock, ctx.frame_len()) as u64;
                let pressure = ctx.evaluate(pressure).as_integer(ctx.clock, ctx.frame_len()) as u64;
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
        }
    }
}
