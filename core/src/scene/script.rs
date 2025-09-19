use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    usize,
};

use serde::{Deserialize, Serialize};

use crate::{
    clock::{Clock, SyncTime},
    lang::{
        Program,
        evaluation_context::EvaluationContext,
        event::ConcreteEvent,
        variable::{VariableStore, VariableValue},
    },
};
use crate::{device_map::DeviceMap, lang::interpreter::Interpreter};

use super::Line;

#[derive(Debug, Serialize, Deserialize)]
pub struct Script {
    content: String,
    lang: String,
    #[serde(skip_serializing, default)]
    pub compiled: Program,
    #[serde(skip_serializing, default)]
    pub frame_vars: Mutex<VariableStore>,
    pub index: usize,
    pub line_index: usize,
    pub args: HashMap<String, String>,
}

impl Default for Script {
    fn default() -> Self {
        Script {
            content: String::default(),
            lang: "bali".to_string(),
            compiled: Program::default(),
            frame_vars: Mutex::new(VariableStore::default()),
            index: usize::MAX,
            line_index: usize::MAX,
            args: HashMap::default(),
        }
    }
}

impl Script {
    pub fn new(content: String, lang: String) -> Self {
        Self {
            content,
            lang,
            compiled: Program::default(),
            frame_vars: Mutex::new(VariableStore::new()),
            index: usize::MAX,
            line_index: usize::MAX,
            args: HashMap::default(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    #[inline]
    pub fn is_compiled(&self) -> bool {
        !self.compiled.is_empty()
    }

    pub fn set_content(&mut self, content: String) {
        self.compiled.clear();
        self.content = content;
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn lang(&self) -> &str {
        &self.lang
    }

    pub fn set_lang(&mut self, lang: String) {
        self.compiled.clear();
        self.lang = lang;
    }
}

/// Warning : this implementation of clone is very time intensive as it requires to lock a mutex !
impl Clone for Script {
    fn clone(&self) -> Self {
        Self {
            lang: self.lang.clone(),
            content: self.content.clone(),
            compiled: self.compiled.clone(),
            frame_vars: Default::default(), // Mutex::new(self.frame_vars.lock().unwrap().clone()),
            index: self.index,
            line_index: self.line_index,
            args: self.args.clone(),
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

#[derive(Debug, Default, Clone)]
pub enum ReturnInfo {
    #[default]
    None,
    IndexChange(usize),
    RelIndexChange(i64),
    ProgChange(usize, Program),
}

pub struct ScriptExecution {
    pub script: Arc<Script>,
    pub instance_vars: VariableStore,
    pub stack: Vec<VariableValue>,
    pub scheduled_time: SyncTime,
    pub interpreter: Box<dyn Interpreter>,
}

impl ScriptExecution {
    pub fn execute_at(
        script: Arc<Script>,
        interpreter: Box<dyn Interpreter>,
        date: SyncTime,
    ) -> Self {
        let mut instance_vars = VariableStore::new();
        instance_vars.insert_no_cast(
            "_current_midi_device_id".to_string(),
            VariableValue::Integer(1),
        );
        ScriptExecution {
            script,
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
            current_scene: self.script.line_index,
            script: &self.script,
            clock,
            device_map,
        };
        let (opt_ev, opt_wait) = self.interpreter.execute_next(&mut ctx);
        if opt_ev.is_none() && opt_wait.is_none() {
            return None;
        }
        let wait = match opt_wait {
            Some(dt) => dt,
            _ => 0,
        };
        let mut res = (ConcreteEvent::Nop, self.scheduled_time);
        if let Some(ev) = opt_ev {
            res.0 = ev;
        };
        self.scheduled_time += wait;
        Some(res)
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
