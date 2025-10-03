use std::{
    collections::{HashMap, VecDeque}
};

use serde::{Deserialize, Serialize};

use crate::{
    clock::SyncTime, compiler::CompilationError, lang::{
        evaluation_context::PartialContext, event::ConcreteEvent, interpreter::asm_interpreter::ASMInterpreter, variable::{VariableStore, VariableValue}, Program
    }
};
use crate::lang::interpreter::Interpreter;

pub enum CompilationState {
    NotCompiled,
    Compiled(Program),
    Error(CompilationError)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Script {
    content: String,
    lang: String,
    #[serde(skip_serializing, default)]
    pub compiled: Program,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub args: HashMap<String, String>,
}

impl Default for Script {
    fn default() -> Self {
        Script {
            content: String::default(),
            lang: "bali".to_string(),
            compiled: Program::default(),
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
            args: HashMap::default(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty() && !self.is_compiled()
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
    pub instance_vars: VariableStore,
    pub stack: VecDeque<VariableValue>,
    pub scheduled_time: SyncTime,
    pub interpreter: Box<dyn Interpreter>,
}

impl ScriptExecution {
    pub fn execute_at(
        interpreter: Box<dyn Interpreter>,
        date: SyncTime,
    ) -> Self {
        let mut instance_vars = VariableStore::new();
        instance_vars.insert_no_cast(
            "_current_midi_device_id".to_string(),
            VariableValue::Integer(1),
        );
        ScriptExecution {
            scheduled_time: date,
            instance_vars,
            stack: VecDeque::new(),
            interpreter,
        }
    }

    pub fn execute_program_at(
        program: Program,
        date: SyncTime
    ) -> Self {
        let interpreter = Box::new(ASMInterpreter::new(program));
        Self::execute_at(interpreter, date)
    }

    pub fn execute_next<'a>(
        &'a mut self,
        mut partial: PartialContext<'a>
    ) -> Option<(ConcreteEvent, SyncTime)> {
        if self.has_terminated() {
            return None;
        }
        partial.instance_vars = Some(&mut self.instance_vars);
        partial.stack = Some(&mut self.stack);
        let Some(mut ctx) = partial.to_context() else {
            return None;
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
        self.scheduled_time = self.scheduled_time.saturating_add(wait);
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