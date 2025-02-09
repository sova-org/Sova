use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::clock::TimeSpan;

use super::variable::{Variable, VariableStore, VariableValue};

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
        globals : &VariableStore, 
        persistents : &VariableStore, 
        ephemeral : &VariableStore
    ) {
        match self {
            EventValue::Reference(var) => { 
                let value = var.evaluate(globals, persistents, ephemeral).unwrap();
                *self = EventValue::Value(value);
            },
            EventValue::Value(_) => (),
        }
    }

}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Nop,
    Chord(Vec<u64>, TimeSpan),
    Timed(Box<Event>, TimeSpan),
    #[serde(untagged)]
    Value(EventValue),
    #[serde(untagged)]
    List(Vec<Event>),
    #[serde(untagged)]
    Map(HashMap<String, Event>)
}

impl Event {

    pub fn map_values(
        &mut self, 
        globals : &VariableStore, 
        persistents : &VariableStore, 
        ephemeral : &VariableStore
    ) {
        match self {
            Event::Timed(event, _) => event.map_values(globals, persistents, ephemeral),
            Event::Value(value) => value.map_value(globals, persistents, ephemeral),
            Event::List(events) => {
                for event in events.iter_mut() {
                    event.map_values(globals, persistents, ephemeral);
                }
            },
            Event::Map(hash_map) => {
                for (_, event) in hash_map.iter_mut() {
                    event.map_values(globals, persistents, ephemeral);
                }
            },
            _ => ()
        }
    }

}