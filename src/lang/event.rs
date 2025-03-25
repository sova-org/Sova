use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::clock::{TimeSpan, Clock};

use super::{evaluation_context::EvaluationContext, variable::{Variable, VariableValue}};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConcreteEventPayload {
    Nop,
    Note(u64, TimeSpan, Option<u64>, Option<u64>),
    //Timed(Box<Event>, TimeSpan),
    //#[serde(untagged)]
    //Value(EventValue),
    //#[serde(untagged)]
    //List(Vec<Event>),
    //#[serde(untagged)]
    //Map(HashMap<String, Event>)
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConcreteEvent {
    pub payload: ConcreteEventPayload,
    pub device: String,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventPayload {
    Nop,
    Note(Variable, Variable, Option<Variable>, Option<Variable>),
    //Timed(Box<Event>, TimeSpan),
    //#[serde(untagged)]
    //Value(EventValue),
    //#[serde(untagged)]
    //List(Vec<Event>),
    //#[serde(untagged)]
    //Map(HashMap<String, Event>)
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub payload: EventPayload,
    pub device: Variable,
}


impl Event {

    pub fn make_concrete(
        &self,
        ctx : &mut EvaluationContext
    ) -> ConcreteEvent {
        let res_event = match &self.payload {
            EventPayload::Nop => ConcreteEventPayload::Nop,
            EventPayload::Note(n, time, chan, vel) => {
                let val = ctx.evaluate(n);
                let val = val.as_integer(ctx.clock) as u64;
                let t = ctx.evaluate(time);
                let duration = t.as_dur();
                let chan = chan.as_ref().map(|x| {
                    let x = ctx.evaluate(x);
                    x.as_integer(ctx.clock) as u64
                });
                let vel = vel.as_ref().map(|x| {
                    let x = ctx.evaluate(x);
                    x.as_integer(ctx.clock) as u64
                });
                ConcreteEventPayload::Note(val, duration, chan, vel)
            },
        };
        let mut res_device = ctx.evaluate(&self.device);
        res_device = res_device.cast_as_str(ctx.clock);
        let dev = match res_device {
            VariableValue::Str(d) => d,
            _ => unreachable!(),
        };
        ConcreteEvent{
            payload: res_event,
            device: dev,
        }
    }

}
