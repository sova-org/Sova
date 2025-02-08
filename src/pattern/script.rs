use core::time;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{clock::{Clock, SyncTime, TimeSpan}, lang::{control_asm::ControlASM, variable::{Variable, VariableStore}, Event, Instruction, Program}};

#[derive(Debug, Default)]
pub struct Script {
    pub content : String,
    pub compiled : Program,
    pub persistents : RefCell<VariableStore>,
}

pub struct ScriptExecution {
    pub script : Rc<Script>,
    pub ephemeral : VariableStore,
    pub current_instruction : usize,
    pub scheduled_time : SyncTime
}

impl Script {

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn is_compiled(&self) -> bool {
        !self.compiled.is_empty()
    }

}

impl ScriptExecution {

    pub fn execute_at(script : Rc<Script>, date : SyncTime) -> Self {
        ScriptExecution { 
            script, 
            ephemeral: HashMap::new(), 
            current_instruction: 0, 
            scheduled_time: date 
        }
    }

    pub fn stop(&mut self) {
        self.current_instruction = usize::MAX;
    }

    pub fn has_terminated(&self) -> bool {
        self.current_instruction < self.script.compiled.len()
    }

    pub fn is_ready(&self, date : SyncTime) -> bool {
        self.scheduled_time <= date
    }

    pub fn execute_next(&mut self, globals : &mut VariableStore, clock : &Clock) -> Option<Event> {
        if self.current_instruction >= self.script.compiled.len() {
            return None;
        }
        let current = &self.script.compiled[self.current_instruction];
        match current {
            Instruction::Control(_) => {
                self.execute_control(globals);
                None
            },
            Instruction::Effect(event, time_span) => {
                self.current_instruction += 1;
                let micros = time_span.as_micros(clock);
                self.scheduled_time += micros;
                Some(event.clone())
            },
        }
    }

    fn execute_control(&mut self, globals : &mut VariableStore) {
        let Instruction::Control(control) =  &self.script.compiled[self.current_instruction] else {
            return;
        };
        // Less performance than to do everything in one single loop, but easier to read and write ?
        let mut persistents = self.script.persistents.borrow_mut();
        let ephemer = &mut self.ephemeral;
        match control {
            ControlASM::Add(x, y) | ControlASM::Sub(x, y) |
            ControlASM::And(x, y) | ControlASM::Or(x, y) |
            ControlASM::JumpIfLess(x, y, _) => {
                if !Variable::ensure_existing(x, y, globals, &mut *persistents, ephemer) {
                    return;
                }
            },
            ControlASM::Mov(_, var) | ControlASM::JumpIf(var, _) | ControlASM::Not(var) => {
                if !var.exists(globals, &mut *persistents, ephemer) {
                    return;
                }
            },
        }
        self.current_instruction += 1;
        match control {
            ControlASM::Mov(x, y) => {
                let value = y.evaluate(globals, & *persistents, ephemer).unwrap();
                x.set(value, globals, &mut *persistents, ephemer);
            },
            ControlASM::JumpIf(variable, index) => {
                let value = variable.evaluate(globals, & *persistents, ephemer).unwrap();
                if value.is_true() {
                    self.current_instruction = *index;
                }
            },
            ControlASM::JumpIfLess(x, y, index) => {
                if x.evaluate(globals, & *persistents, ephemer) < y.evaluate(globals, & *persistents, ephemer) {
                    self.current_instruction = *index;
                }
            },
            ControlASM::Add(x, y) => todo!(),
            ControlASM::Sub(x, y) => todo!(),
            ControlASM::And(x, y) => todo!(),
            ControlASM::Or(x, y) => todo!(),
            ControlASM::Not(variable) => todo!(),
        }
    }

}