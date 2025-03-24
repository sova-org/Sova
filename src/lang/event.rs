use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::clock::{TimeSpan, Clock};

use super::variable::{Variable, VariableStore, VariableValue};

/*
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventValue {
    Reference(Variable),
    #[serde(untagged)]
    Value(VariableValue),
}

impl EventValue {

    pub fn is_mapped(&self) -> bool {
        if let Self::Value(_) = self {
            true
        } else {
            false
        }
    }

    pub fn map_value(
        &mut self, 
        environment_vars : &VariableStore,
        global_vars : &VariableStore, 
        sequence_vars : &VariableStore,
        step_vars : &VariableStore, 
        instance_vars : &VariableStore
    ) {
        match self {
            EventValue::Reference(var) => { 
                let value = var.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars).unwrap();
                *self = EventValue::Value(value);
            },
            EventValue::Value(_) => (),
        }
    }

}
*/

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConcreteEventPayload {
    Nop,
    Chord(Vec<u64>, TimeSpan),
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
    Chord(Vec<Variable>, Variable),
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
            EventPayload::Chord(elements, time) => {
                let vals: Vec<u64> = elements.into_iter().map(|elem| {
                    let mut val = elem.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars).unwrap();
                    val = val.cast_as_integer(clock);
                    match val {
                        VariableValue::Integer(n) => n as u64,
                        _ => unreachable!(),
                    }
                }).collect();
                let mut t = time.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars).unwrap();
                t = t.cast_as_dur();
                let duration = match t {
                    VariableValue::Dur(d) => d,
                    _ => unreachable!(),
                };
                ConcreteEventPayload::Chord(vals, duration)
            }
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

    /*
    pub fn map_values(
        &mut self,
        environment_vars : &VariableStore,
        global_vars : &VariableStore,
        sequence_vars : &VariableStore,
        step_vars : &VariableStore,
        instance_vars : &VariableStore
    ) {
        match self {
            Event::Timed(event, _) => event.map_values(environment_vars, global_vars, sequence_vars, step_vars, instance_vars),
            Event::Value(value) => value.map_value(environment_vars, global_vars, sequence_vars, step_vars, instance_vars),
            Event::List(events) => {
                for event in events.iter_mut() {
                    event.map_values(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                }
            },
            Event::Map(hash_map) => {
                for (_, event) in hash_map.iter_mut() {
                    event.map_values(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                }
            },
            _ => ()
        }
    */

}