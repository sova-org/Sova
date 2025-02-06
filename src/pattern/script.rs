use crate::{clock::TimeSpan, lang::{control_asm::ControlASM, variable::{Variable, VariableStore}, Event, Instruction, Program}};

#[derive(Debug, Default)]
pub struct Script {
    pub content : String,
    pub compiled : Program,
    pub persistents : VariableStore,
    pub ephemeral : VariableStore,
    pub current_instruction : usize
}

impl Script {

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn is_compiled(&self) -> bool {
        !self.compiled.is_empty()
    }

    pub fn start(&mut self) {
        self.current_instruction = 0;
    }

    pub fn stop(&mut self) {
        self.ephemeral.clear();
        self.current_instruction = usize::MAX;
    }

    pub fn is_executing(&mut self) -> bool {
        self.current_instruction < self.compiled.len()
    }

    pub fn execute_next(&mut self, globals : &mut VariableStore) -> Option<(Event, TimeSpan)> {
        if self.current_instruction >= self.compiled.len() {
            return None;
        }
        let current = &self.compiled[self.current_instruction];
        match current {
            Instruction::Control(_) => {
                self.execute_control(globals);
                None
            },
            Instruction::Effect(event, time_span) => {
                self.current_instruction += 1;
                Some((event.clone(), time_span.clone()))
            },
        }
    }

    pub fn execute_control(&mut self, globals : &mut VariableStore) {
        let Instruction::Control(control) =  &self.compiled[self.current_instruction] else {
            return;
        };
        // Less performance than to do everything in one single loop, but easier to read ?
        let persistents = &mut self.persistents;
        let ephemer = &mut self.ephemeral;
        match control {
            ControlASM::Add(x, y) | ControlASM::Sub(x, y) |
            ControlASM::And(x, y) | ControlASM::Or(x, y) |
            ControlASM::JumpIfLess(x, y, _) => {
                if !Variable::ensure_existing(x, y, globals, persistents, ephemer) {
                    return;
                }
            },
            ControlASM::Mov(_, var) | ControlASM::JumpIf(var, _) | ControlASM::Not(var) => {
                if !var.exists(globals, persistents, ephemer) {
                    return;
                }
            }
            _ => ()
        }
        self.current_instruction += 1;
        match control {
            ControlASM::Mov(x, y) => {
                let value = y.evaluate(globals, persistents, ephemer).unwrap();
                x.set(value, globals, persistents, ephemer);
            },
            ControlASM::JumpIf(variable, index) => {
                let value = variable.evaluate(globals, persistents, ephemer).unwrap();
                if value.is_true() {
                    self.current_instruction = *index;
                }
            },
            ControlASM::JumpIfLess(x, y, _) => todo!(),
            ControlASM::Add(x, y) => todo!(),
            ControlASM::Sub(x, y) => todo!(),
            ControlASM::And(x, y) => todo!(),
            ControlASM::Or(x, y) => todo!(),
            ControlASM::Not(variable) => todo!(),
        }
    }



}
