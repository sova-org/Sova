use std::{collections::HashMap, sync::{Arc, Mutex}};

use serde::{Deserialize, Serialize};

use crate::{clock::{Clock, SyncTime}, lang::{event::Event, variable::VariableStore, Instruction, Program}};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Script {
    pub content : String,
    pub compiled : Program,
    pub step_vars : Mutex<VariableStore>,
}

pub enum ReturnInfo {
    None,
    IndexChange(usize),
    ProgChange(usize, Program),
}

pub struct ScriptExecution {
    pub script : Arc<Script>,
    pub prog: Program,
    pub arg_prog: Program,
    pub instance_vars : VariableStore,
    pub instruction_index : usize,
    pub return_stack : Vec<ReturnInfo>,
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
            script: script.clone(),
            prog: script.compiled.clone(),
            arg_prog: script.compiled.clone(),
            instance_vars: HashMap::new(),
            instruction_index: 0,
            return_stack: Vec::new(),
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
        &self.prog[self.instruction_index]
    }

    pub fn execute_next(&mut self, environment_vars : &mut VariableStore, global_vars : &mut VariableStore, sequence_vars : &mut VariableStore, clock : &Clock) -> Option<(Event, SyncTime)> {
        if self.has_terminated() {
            return None;
        }
        let current = &self.prog[self.instruction_index];
        match current {
            Instruction::Control(_) => {
                self.execute_control(environment_vars, global_vars, sequence_vars, clock);
                None
            },
            Instruction::Effect(event, time_span) => {
                self.instruction_index += 1;
                let wait = time_span.as_micros(clock);
                let mut generated = event.clone();
                generated.map_values(environment_vars, global_vars, sequence_vars, &self.script.step_vars.lock().unwrap(), &self.instance_vars);
                let res = (generated, self.scheduled_time); // TODO: si les instructions de contrôle font dépasser scheduled_time ?
                self.scheduled_time += wait;
                Some(res)
            },
        }
    }

    fn execute_control(&mut self, environment_vars : &mut VariableStore, global_vars : &mut VariableStore, sequence_vars : &mut VariableStore, clock : &Clock) {
        let Instruction::Control(control) =  &self.prog[self.instruction_index] else {
            return;
        };
        // Less performant than to do everything in one single check, but easier to read and write ?
        let mut step_vars = self.script.step_vars.lock().unwrap();
        let instance_vars = &mut self.instance_vars;
        /*
        if !ensure_executability(control, environment_vars, global_vars, sequence_vars, &mut step_vars, instance_vars) {
            return;
        }
        */
        match control.execute(environment_vars, global_vars, sequence_vars, &mut step_vars, instance_vars, clock, &mut self.return_stack, self.instruction_index, &self.arg_prog) {
            ReturnInfo::None => self.instruction_index += 1,
            ReturnInfo::IndexChange(index) => self.instruction_index = index,
            ReturnInfo::ProgChange(index, prog) => {
                self.instruction_index = index;
                self.prog = prog.clone();
                self.arg_prog = prog.clone();
            },
        };
    }    
}

/*
fn ensure_executability(
    control : &ControlASM,
    environment_vars : &mut VariableStore,
    global_vars : &mut VariableStore,
    sequence_vars : &mut VariableStore,
    step_vars : &mut VariableStore,
    instance_vars : &mut VariableStore
) -> bool {
    match control {
        ControlASM::Add(x, y, _) | ControlASM::Sub(x, y, _) |
        ControlASM::And(x, y, _) | ControlASM::Or(x, y, _) 
    => {
            Variable::ensure_existing(x, y, environment_vars, global_vars, sequence_vars, step_vars, instance_vars) && x.is_mutable()
        },
        ControlASM::JumpIfLess(x, y, _) => {
            Variable::ensure_existing(x, y, environment_vars, global_vars, sequence_vars, step_vars, instance_vars)
        },
        ControlASM::Mov(_, var) | ControlASM::JumpIf(var, _) | ControlASM::Not(var, _) => {
            var.exists(environment_vars, global_vars, sequence_vars, step_vars, instance_vars)
        },
        _ => true
    }
}
    */
