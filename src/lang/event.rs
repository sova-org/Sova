use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::clock::{TimeSpan, Clock};

use super::variable::{Variable, VariableStore, VariableValue};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConcreteEvent {
    Nop,
    MidiNote(u64, u64, u64, TimeSpan, String),
    //Timed(Box<Event>, TimeSpan),
    //#[serde(untagged)]
    //Value(EventValue),
    //#[serde(untagged)]
    //List(Vec<Event>),
    //#[serde(untagged)]
    //Map(HashMap<String, Event>)
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Nop,
    MidiNote(Variable, Variable, Variable, Variable, Variable),
    //Timed(Box<Event>, TimeSpan),
    //#[serde(untagged)]
    //Value(EventValue),
    //#[serde(untagged)]
    //List(Vec<Event>),
    //#[serde(untagged)]
    //Map(HashMap<String, Event>)
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
        match &self {
            Event::Nop => ConcreteEvent::Nop,
            Event::MidiNote(note, vel, chan, time, dev) => {
                let note = note.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                let note = note.as_integer(clock) as u64;
                let time = time.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                let time = time.as_dur();
                let chan = chan.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                let chan = chan.as_integer(clock) as u64;
                let vel = vel.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                let vel = vel.as_integer(clock) as u64;
                let dev = dev.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
                let dev = dev.as_str(clock);
                ConcreteEvent::MidiNote(note, vel, chan, time, dev)
            },
        }
    }

}
