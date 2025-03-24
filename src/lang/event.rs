use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::clock::{TimeSpan, Clock};

use super::variable::{Variable, VariableStore, VariableValue};

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


#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub payload: EventPayload,
    pub device: Variable,
}


impl Event {

    pub fn make_concrete(
        &self,
        environment_vars : &VariableStore,
        global_vars : &VariableStore,
        sequence_vars : &VariableStore,
        step_vars : &VariableStore,
        instance_vars : &VariableStore,
        clock : &Clock,
    ) -> ConcreteEvent {
        let res_event = match &self.payload {
            EventPayload::Nop => ConcreteEventPayload::Nop,
            EventPayload::Note(n, time, chan, vel) => {
                let val = n.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars).unwrap();
                let val = val.as_integer(clock) as u64;
                let t = time.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars).unwrap();
                let duration = t.as_dur();
                let chan = chan.as_ref().map(|x| {
                    let x = x.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars).unwrap();
                    x.as_integer(clock) as u64
                });
                let vel = vel.as_ref().map(|x| {
                    let x = x.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars).unwrap();
                    x.as_integer(clock) as u64
                });
                ConcreteEventPayload::Note(val, duration, chan, vel)
            },
            // EventPayload::Chord(elements, time, chan) => {
            //     let vals: Vec<u64> = elements.into_iter().map(|elem| {
            //         let val = elem.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars).unwrap();
            //         val.as_integer(clock) as u64
            //     }).collect();
            //     let t = time.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars).unwrap();
            //     let duration = t.as_dur();
            //     let chan = chan.as_ref().map(|x| {
            //         let x = x.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars).unwrap();
            //         x.as_integer(clock) as u64
            //     });
            //     ConcreteEventPayload::Chord(vals, duration, chan)
            // }
        };
        let mut res_device = self.device.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars).unwrap();
        res_device = res_device.cast_as_str(clock);
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
