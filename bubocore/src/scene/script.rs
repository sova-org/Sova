use std::{
    sync::{Arc, Mutex},
    usize,
};

use serde::{Deserialize, Serialize};

use crate::device_map::DeviceMap;
use crate::{
    clock::{Clock, SyncTime},
    lang::{
        Instruction, Program,
        evaluation_context::EvaluationContext,
        event::ConcreteEvent,
        variable::{VariableStore, VariableValue},
    },
};

use super::Line;

#[derive(Debug, Serialize, Deserialize)]
pub struct Script {
    pub content: String,
    pub lang: String,
    #[serde(skip_serializing, default)]
    pub compiled: Program,
    #[serde(skip_serializing, default)]
    pub frame_vars: Mutex<VariableStore>,
    pub index: usize,
}

impl Default for Script {
    fn default() -> Self {
        Script {
            content: String::default(),
            lang: "bali".to_string(),
            compiled: Program::default(),
            frame_vars: Mutex::new(VariableStore::default()),
            index: 0,
        }
    }
}

impl Script {
    pub fn new(content: String, compiled: Program, lang: String, index: usize) -> Self {
        Self {
            content,
            lang,
            compiled,
            frame_vars: Mutex::new(VariableStore::new()),
            index,
        }
    }
}

pub enum ReturnInfo {
    None,
    IndexChange(usize),
    RelIndexChange(i64),
    ProgChange(usize, Program),
}

pub struct ScriptExecution {
    pub script: Arc<Script>,
    pub line_index: usize,
    pub prog: Program,
    pub instance_vars: VariableStore,
    pub stack: Vec<VariableValue>,
    pub instruction_index: usize,
    pub return_stack: Vec<ReturnInfo>,
    pub scheduled_time: SyncTime,
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

/// Warning : this implementation of clone is very time intensive as it requires to lock a mutex !
impl Clone for Script {
    fn clone(&self) -> Self {
        Self {
            lang: self.lang.clone(),
            content: self.content.clone(),
            compiled: self.compiled.clone(),
            frame_vars: Mutex::new(self.frame_vars.lock().unwrap().clone()),
            index: self.index.clone(),
        }
    }
}

impl From<Program> for Script {
    fn from(compiled: Program) -> Self {
        Script {
            compiled,
            ..Default::default()
        }
    }
}
impl From<String> for Script {
    fn from(content: String) -> Self {
        Script {
            content,
            ..Default::default()
        }
    }
}

impl ScriptExecution {
    pub fn execute_at(script: Arc<Script>, line_index: usize, date: SyncTime) -> Self {
        let prog = script.compiled.clone();
        let mut instance_vars = VariableStore::new();
        instance_vars.insert_no_cast(
            "_current_midi_device_id".to_string(),
            VariableValue::Integer(1),
        );

        ScriptExecution {
            script,
            prog,
            line_index,
            instruction_index: 0,
            scheduled_time: date,
            return_stack: Vec::new(),
            instance_vars,
            stack: Vec::new(),
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
    pub fn is_ready(&self, date: SyncTime) -> bool {
        self.scheduled_time <= date
    }

    #[inline]
    pub fn remaining_before(&self, date: SyncTime) -> SyncTime {
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

    pub fn execute_next(
        &mut self,
        clock: &Clock,
        globals: &mut VariableStore,
        lines: &mut [Line],
        device_map: Arc<DeviceMap>,
    ) -> Option<(ConcreteEvent, SyncTime)> {
        if self.has_terminated() {
            return None;
        }
        let current = &self.prog[self.instruction_index];
        match current {
            Instruction::Control(_) => {
                self.execute_control(clock, globals, lines, device_map);
                None
            }
            Instruction::Effect(event, var_time_span) => {
                self.instruction_index += 1;
                let mut ctx = EvaluationContext {
                    global_vars: globals,
                    frame_vars: &mut self.script.frame_vars.lock().unwrap(),
                    instance_vars: &mut self.instance_vars,
                    stack: &mut self.stack,
                    lines,
                    current_scene: self.line_index,
                    script: &self.script,
                    clock,
                    device_map,
                };
                let wait = ctx
                    .evaluate(var_time_span)
                    .as_dur()
                    .as_micros(clock, ctx.frame_len());
                let c_event = event.make_concrete(&mut ctx);
                let res = (c_event, self.scheduled_time);
                self.scheduled_time += wait;
                Some(res)
            }
        }
    }

    fn execute_control(
        &mut self,
        clock: &Clock,
        globals: &mut VariableStore,
        lines: &mut [Line],
        device_map: Arc<DeviceMap>,
    ) {
        let Instruction::Control(control) = &self.prog[self.instruction_index] else {
            return;
        };
        let mut ctx = EvaluationContext {
            global_vars: globals,
            frame_vars: &mut self.script.frame_vars.lock().unwrap(),
            instance_vars: &mut self.instance_vars,
            stack: &mut self.stack,
            lines,
            current_scene: self.line_index,
            script: &self.script,
            clock,
            device_map,
        };
        match control.execute(
            &mut ctx,
            &mut self.return_stack,
            self.instruction_index,
            &self.prog,
        ) {
            ReturnInfo::None => self.instruction_index += 1,
            ReturnInfo::IndexChange(index) => self.instruction_index = index,
            ReturnInfo::RelIndexChange(index_change) => {
                let mut index = self.instruction_index as i64;
                index += index_change;
                if index < 0 {
                    index = 0
                };
                self.instruction_index = index as usize;
            }
            ReturnInfo::ProgChange(index, prog) => {
                self.instruction_index = index;
                self.prog = prog.clone();
            }
        };
    }
}
