use std::{collections::HashMap, sync::{Arc, Mutex}};

use serde::{Deserialize, Serialize};

use crate::{clock::{Clock, SyncTime}, lang::{control_asm::ControlASM, event::Event, variable::{Variable, VariableStore, VariableValue}, Instruction, Program}};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Script {
    pub content : String,
    pub compiled : Program,
    pub persistents : Mutex<VariableStore>,
}

pub struct ScriptExecution {
    pub script : Arc<Script>,
    pub ephemeral : VariableStore,
    pub instruction_index : usize,
    pub scheduled_time : SyncTime
}

impl Script {

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    #[inline]
    pub fn is_compiled(&self) -> bool {
        !self.compiled.is_empty()
    }

}

impl From<Program> for Script {
    fn from(compiled: Program) -> Self {
        Script { compiled, ..Default::default() }
    }
}
impl From<String> for Script {
    fn from(content : String) -> Self {
        Script { content, ..Default::default() }
    }
}

impl ScriptExecution {

    pub fn execute_at(script : Arc<Script>, date : SyncTime) -> Self {
        ScriptExecution {
            script,
            ephemeral: HashMap::new(),
            instruction_index: 0,
            scheduled_time: date
        }
    }

    #[inline]
    pub fn stop(&mut self) {
        self.instruction_index = usize::MAX;
    }

    #[inline]
    pub fn has_terminated(&self) -> bool {
        self.instruction_index >= self.script.compiled.len()
    }

    #[inline]
    pub fn is_ready(&self, date : SyncTime) -> bool {
        self.scheduled_time <= date
    }

    #[inline]
    pub fn remaining_before(&self, date : SyncTime) -> SyncTime {
        if date >= self.scheduled_time {
            0
        } else {
            self.scheduled_time - date
        }
    }

    #[inline]
    pub fn current_instruction(&self) -> &Instruction {
        &self.script.compiled[self.instruction_index]
    }

    pub fn execute_next(&mut self, globals : &mut VariableStore, clock : &Clock) -> Option<(Event, SyncTime)> {
        if self.has_terminated() {
            return None;
        }
        let current = &self.script.compiled[self.instruction_index];
        match current {
            Instruction::Control(_) => {
                self.execute_control(globals);
                None
            },
            Instruction::Effect(event, time_span) => {
                self.instruction_index += 1;
                let wait = time_span.as_micros(clock);
                let mut generated = event.clone();
                generated.map_values(globals, &self.script.persistents.lock().unwrap(), &self.ephemeral);
                let res = (generated, self.scheduled_time);
                self.scheduled_time += wait;
                Some(res)
            },
        }
    }

    fn execute_control(&mut self, globals : &mut VariableStore) {
        let Instruction::Control(control) =  &self.script.compiled[self.instruction_index] else {
            return;
        };
        // Less performant than to do everything in one single check, but easier to read and write ?
        let mut persistents = self.script.persistents.lock().unwrap();
        let ephemer = &mut self.ephemeral;
        if !ensure_executability(control, globals, &mut persistents, ephemer) {
            return;
        }
        self.instruction_index += 1;
        match control {
            ControlASM::Mov(x, y) => {
                let value = y.evaluate(globals, &persistents, ephemer).unwrap();
                x.set(value, globals, &mut persistents, ephemer);
            },
            ControlASM::JumpIf(variable, index) => {
                let value = variable.evaluate(globals, &persistents, ephemer).unwrap();
                if value.is_true() {
                    self.instruction_index = *index;
                }
            },
            ControlASM::JumpIfLess(x, y, index) => {
                if x.evaluate(globals, &persistents, ephemer) < y.evaluate(globals, & persistents, ephemer) {
                    self.instruction_index = *index;
                }
            },
            ControlASM::Add(x, y) => {
                let value = y.evaluate(globals, &persistents, ephemer).unwrap();
                let handle = x.mut_value(globals, &mut persistents, ephemer).unwrap();
                *handle += value;
            },
            ControlASM::Sub(x, y) => {
                let value = y.evaluate(globals, &persistents, ephemer).unwrap();
                let handle = x.mut_value(globals, &mut persistents, ephemer).unwrap();
                *handle -= value;
            },
            ControlASM::And(x, y) => {
                let value = y.evaluate(globals, &persistents, ephemer).unwrap();
                let handle = x.mut_value(globals, &mut persistents, ephemer).unwrap();
                *handle &= value;
            },
            ControlASM::Or(x, y) => {
                let value = y.evaluate(globals, &persistents, ephemer).unwrap();
                let handle = x.mut_value(globals, &mut persistents, ephemer).unwrap();
                *handle |= value;
            },
            ControlASM::Cmp(x, y) => {
                let cmp = x.evaluate(globals, &persistents, ephemer) < y.evaluate(globals, &persistents, ephemer);
                x.set(VariableValue::Bool(cmp), globals, &mut persistents, ephemer);
            },
            ControlASM::Not(x) => {
                let value = x.evaluate(globals, &persistents, ephemer).unwrap();
                x.set(!value, globals, &mut persistents, ephemer);
            },
            ControlASM::Exit => {
                self.instruction_index = usize::MAX
            },
            ControlASM::Goto(i) => self.instruction_index = *i,
        }
    }

}

fn ensure_executability(
    control : &ControlASM,
    globals : &mut VariableStore,
    persistents : &mut VariableStore,
    ephemer : &mut VariableStore
) -> bool {
    match control {
        ControlASM::Add(x, y) | ControlASM::Sub(x, y) |
        ControlASM::And(x, y) | ControlASM::Or(x, y)  |
        ControlASM::Cmp(x, y)
    => {
            Variable::ensure_existing(x, y, globals, persistents, ephemer) && x.is_mutable()
        },
        ControlASM::JumpIfLess(x, y, _) => {
            Variable::ensure_existing(x, y, globals, persistents, ephemer)
        },
        ControlASM::Mov(_, var) | ControlASM::JumpIf(var, _) | ControlASM::Not(var) => {
            var.exists(globals, persistents, ephemer)
        },
        _ => true
    }
}
