use std::{
    sync::{Arc, Mutex},
    usize,
};

use serde::{Deserialize, Serialize};

use crate::{device_map::DeviceMap, lang::interpreter::Interpreter};
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
    pub instance_vars: VariableStore,
    pub stack: Vec<VariableValue>,
    pub scheduled_time: SyncTime,
    pub interpreter: Box<dyn Interpreter>
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
            index: self.index,
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
    pub fn execute_at(
        script: Arc<Script>, 
        interpreter: Box<dyn Interpreter>,
        line_index: usize, 
        date: SyncTime,
    ) -> Self {
        let mut instance_vars = VariableStore::new();
        instance_vars.insert_no_cast(
            "_current_midi_device_id".to_string(),
            VariableValue::Integer(1),
        );
        ScriptExecution {
            script,
            line_index,
            scheduled_time: date,
            instance_vars,
            stack: Vec::new(),
            interpreter,
        }
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
        let (opt_ev, opt_wait) = self.interpreter.execute_next(&mut ctx);
        let wait = match opt_wait {
            Some(dt) => dt,
            _ => 0
        };
        if let Some(ev) = opt_ev {

        };
        self.scheduled_time += wait;
        if let Some((ev, wait)) = self.interpreter.execute_next(&mut ctx) {
            let res = Some((ev, self.scheduled_time));
            
            res
        } else {
            None
        }
    }

    #[inline]
    pub fn stop(&mut self) {
        self.interpreter.stop();
    }

    #[inline]
    pub fn has_terminated(&self) -> bool {
        self.interpreter.has_terminated()
    }

    #[inline]
    pub fn is_ready(&self, date: SyncTime) -> bool {
        self.scheduled_time <= date
    }

    #[inline]
    pub fn remaining_before(&self, date: SyncTime) -> SyncTime {
        self.scheduled_time.saturating_sub(date)
    }

}
