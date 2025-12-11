use std::{
    collections::{BTreeMap, VecDeque},
    hash::{self, DefaultHasher, Hash, Hasher}, thread::{self, ThreadId},
};

use serde::{Deserialize, Serialize};

use crate::lang::interpreter::Interpreter;
use crate::{
    clock::{NEVER, SyncTime},
    compiler::{CompilationError, CompilationState},
    lang::{
        PartialContext, Program,
        event::ConcreteEvent,
        interpreter::asm_interpreter::ASMInterpreter,
        variable::{VariableStore, VariableValue},
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Script {
    content: String,
    lang: String,
    #[serde(skip_serializing, default)]
    pub compiled: CompilationState,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub args: BTreeMap<String, String>,
}

const ALLOWED_TIME_MARGIN: SyncTime = 10;

impl Default for Script {
    fn default() -> Self {
        Script {
            content: Default::default(),
            lang: "bali".to_string(),
            compiled: Default::default(),
            args: Default::default(),
        }
    }
}

impl Script {
    pub fn new(content: String, lang: String) -> Self {
        Self {
            content,
            lang,
            ..Default::default()
        }
    }

    pub fn is_like(&self, other: &Script) -> bool {
        self.content == other.content && self.lang == other.lang && self.args == other.args
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty() && !self.is_compiled()
    }

    #[inline]
    pub fn is_compiled(&self) -> bool {
        self.compiled.is_compiled()
    }

    pub fn has_compilation_error(&self) -> bool {
        self.compiled.is_err()
    }

    pub fn compilation_state(&self) -> &CompilationState {
        &self.compiled
    }

    pub fn has_not_been_compiled(&self) -> bool {
        self.compiled.has_not_been_compiled()
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

    pub fn set_program(&mut self, prog: Program) {
        self.compiled = CompilationState::Compiled(prog)
    }

    pub fn set_error(&mut self, error: CompilationError) {
        self.compiled = CompilationState::Error(error)
    }

    pub fn id(&self) -> u64 {
        // TODO: possible optimization
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl hash::Hash for Script {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.content.hash(state);
        self.lang.hash(state);
        //self.compiled.hash(state);
        self.args.hash(state);
    }
}

impl From<Program> for Script {
    fn from(compiled: Program) -> Self {
        Script {
            compiled: CompilationState::Compiled(compiled),
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
    interpreter: Option<Box<dyn Interpreter>>,
    thread_id: ThreadId
}

impl ScriptExecution {
    pub fn execute_at(interpreter: Box<dyn Interpreter>, date: SyncTime) -> Self {
        let mut instance_vars = VariableStore::new();
        instance_vars.insert_no_cast(
            "_current_midi_device_id".to_string(),
            VariableValue::Integer(1),
        );
        ScriptExecution {
            scheduled_time: date,
            instance_vars,
            stack: VecDeque::new(),
            interpreter: Some(interpreter),
            thread_id: thread::current().id()
        }
    }

    pub fn interpreter_mut(&mut self) -> Option<&mut Box<dyn Interpreter>> {
        if thread::current().id() == self.thread_id {
            return self.interpreter.as_mut()
        } else {
            return None
        }
    }

    pub fn interpreter(&self) -> Option<&Box<dyn Interpreter>> {
        if thread::current().id() == self.thread_id {
            return self.interpreter.as_ref()
        } else {
            return None
        }
    }

    pub fn execute_program_at(program: Program, date: SyncTime) -> Self {
        let interpreter = Box::new(ASMInterpreter::new(program));
        Self::execute_at(interpreter, date)
    }

    pub fn execute_next<'a>(
        &'a mut self,
        mut partial: PartialContext<'a>,
    ) -> (Option<ConcreteEvent>, SyncTime) {
        if self.has_terminated() {
            return (None, NEVER);
        }
        if thread::current().id() != self.thread_id {
            return (None, NEVER);
        }
        let interpreter = &mut self.interpreter.as_mut().unwrap();
        partial.instance_vars = Some(&mut self.instance_vars);
        partial.stack = Some(&mut self.stack);
        let prev_date = partial.logic_date;
        partial.logic_date = std::cmp::min(
            partial.logic_date,
            self.scheduled_time + ALLOWED_TIME_MARGIN,
        );
        let Some(mut ctx) = partial.to_context() else {
            return (None, NEVER);
        };
        let (opt_ev, wait) = interpreter.execute_next(&mut ctx);
        self.scheduled_time = self.scheduled_time.saturating_add(wait);
        let rem = self.scheduled_time.saturating_sub(prev_date);
        (opt_ev, rem)
    }

    #[inline]
    pub fn stop(&mut self) {
        let Some(interpreter) = self.interpreter_mut() else {
            return;
        };
        interpreter.stop();
    }

    #[inline]
    pub fn has_terminated(&self) -> bool {
        let Some(interpreter) = self.interpreter() else {
            return true;
        };
        interpreter.has_terminated()
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

unsafe impl Send for ScriptExecution {}
unsafe impl Sync for ScriptExecution {}