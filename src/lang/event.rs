use serde::{Deserialize, Serialize};

use crate::clock::TimeSpan;

use super::{evaluation_context::EvaluationContext, variable::Variable};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConcreteEvent {
    Nop,
    MidiNote(u64, u64, u64, TimeSpan, String),
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Nop,
    MidiNote(Variable, Variable, Variable, Variable, Variable),
}


impl Event {

    pub fn make_concrete(
        &self,
        ctx : &mut EvaluationContext
    ) -> ConcreteEvent {
        match &self {
            Event::Nop => ConcreteEvent::Nop,
            Event::MidiNote(note, vel, chan, time, dev) => {
                let note = ctx.evaluate(note);
                let note = note.as_integer(ctx.clock) as u64;
                let time = ctx.evaluate(time);
                let time = time.as_dur();
                let chan = ctx.evaluate(chan);
                let chan = chan.as_integer(ctx.clock) as u64;
                let vel = ctx.evaluate(vel);
                let vel = vel.as_integer(ctx.clock) as u64;
                let dev = ctx.evaluate(dev);
                let dev = dev.as_str(ctx.clock);
                ConcreteEvent::MidiNote(note, vel, chan, time, dev)
            },
        }
    }

}
