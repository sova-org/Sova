use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{clock::{SyncTime, NEVER}, compiler::CompilationState, lang::{evaluation_context::PartialContext, event::ConcreteEvent, interpreter::InterpreterDirectory, variable::VariableStore}, log_eprintln, scene::script::{Script, ScriptExecution}};

#[derive(Serialize, Deserialize)]
pub struct Frame {
    /// The duration of the frame in beats.
    pub duration: f64,
    /// Specifies how many times each frame should repeat consecutively before moving to the next.
    /// A value of `1` means the frame plays once. Defaults to `1`.
    #[serde(default="default_repetitions", skip_serializing_if="is_default_repetitions")]
    pub repetitions: usize,
    /// Tracks whether the frame in is currently active for playback.
    #[serde(default="default_enabledness", skip_serializing_if="is_default_enabledness")]
    pub enabled: bool,
    /// Scripts associated with the frame. Executed when the frame becomes active.
    script: Script,
    /// Optional user-defined names for each frame. Useful for identification in UIs or debugging.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if="VariableStore::is_empty")]
    pub vars: VariableStore,

    #[serde(skip)]
    script_has_changed: bool,
    #[serde(skip)]
    pub executions: Vec<ScriptExecution>
}

impl Frame {

    pub fn make_consistent(&mut self) {
        if self.repetitions == 0 {
            self.repetitions = 1;
        }
    }

    /// Changes the current value, while preserving executions until the frame is triggered again
    pub fn change(&mut self, other: Frame) {
        let old = std::mem::replace(self, other);
        if old.script().id() != self.script().id() {
            self.mark_script_changed();
        }
        self.executions = old.executions;
    }

    pub fn mark_script_changed(&mut self) {
        self.script_has_changed = true;
    }

    pub fn effective_duration(&self) -> f64 {
        self.duration * (self.repetitions as f64)
    }

    pub fn script(&self) -> &Script {
        &self.script
    }

    pub fn set_script(&mut self, script: Script) {
        if script.id() == self.script.id() {
            return;
        }
        self.script = script;
        self.script_has_changed = true;
    }

    pub fn compilation_state_mut(&mut self) -> &mut CompilationState {
        &mut self.script.compiled
    }

    pub fn update_compilation_state(&mut self, id: u64, state: CompilationState) -> bool {
        if self.script.id() == id {
            self.script.compiled = state;
            true
        } else {
            false
        }
    }

    pub fn trigger(&mut self, date: SyncTime, interpreters: &InterpreterDirectory) {
        if self.script_has_changed {
            self.script_has_changed = false;
            self.executions.clear();
        }
        if !self.enabled || self.script().is_empty() {
            return;
        }
        if let Some(interpreter) = interpreters.get_interpreter(self.script()) {
            let exec = ScriptExecution::execute_at(interpreter, date);
            self.executions.push(exec);
        } else {
            log_eprintln!(
                "Unable to find interpreter to trigger frame (lang={})",
                self.script.lang()
            )
        }
    }

    pub fn kill_executions(&mut self) {
        self.executions.clear();
    }

    pub fn update_executions<'a>(
        &'a mut self,
        date: SyncTime,
        mut partial: PartialContext<'a>
    ) -> (Vec<ConcreteEvent>, Option<SyncTime>) {
        let mut events = Vec::new();
        let mut next_wait: Option<SyncTime> = Some(NEVER);
        let mut new_executions = Vec::new();
        partial.frame_vars = Some(&mut self.vars);
        for exec in self.executions.iter_mut() {
            if !exec.is_ready(date) {
                let wait = exec.remaining_before(date);
                next_wait
                    .as_mut()
                    .map(|value| *value = std::cmp::min(*value, wait));
                continue;
            }
            let Some((event, wait)) = exec.execute_next(partial.child()) else {
                next_wait = None;
                continue;
            };
            match event {
                ConcreteEvent::Nop => (),
                ConcreteEvent::StartProgram(prog) => {
                    let new_exec = ScriptExecution::execute_program_at(prog, date);
                    new_executions.push(new_exec);
                }
                _ => events.push(event)
            }
            next_wait
                .as_mut()
                .map(|value| *value = std::cmp::min(*value, wait));
        }
        self.executions.retain(|exec| !exec.has_terminated());
        self.executions.append(&mut new_executions);
        (events, next_wait)
    }

    pub fn before_next_update(&self, date: SyncTime) -> SyncTime {
        self.executions
            .iter()
            .map(|exec| exec.remaining_before(date))
            .min()
            .unwrap_or(NEVER)
    }

    pub fn has_executions(&self) -> bool {
        !self.executions.is_empty()
    }

}

fn default_repetitions() -> usize {
    1
}

fn is_default_repetitions(value: &usize) -> bool {
    *value == default_repetitions()
}

fn default_enabledness() -> bool {
    true
}

fn is_default_enabledness(value: &bool) -> bool {
    *value == default_enabledness()
}

impl From<f64> for Frame {
    fn from(value: f64) -> Self {
        Frame {
            duration: value,
            ..Default::default()
        }
    }
}

impl From<Script> for Frame {
    fn from(value: Script) -> Self {
        Frame {
            script: value,
            ..Default::default()
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Frame {
            duration: 1.0,
            repetitions: default_repetitions(),
            enabled: default_enabledness(),
            script: Default::default(),
            name: None,
            vars: Default::default(),
            script_has_changed: false,
            executions: Default::default()
        }
    }
}

impl Clone for Frame {
    fn clone(&self) -> Self {
        Self { 
            duration: self.duration.clone(), 
            repetitions: self.repetitions.clone(), 
            enabled: self.enabled.clone(), 
            script: self.script.clone(), 
            name: self.name.clone(), 
            vars: Default::default(), 
            script_has_changed: false, 
            executions: Default::default()
        }
    }
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Frame")
            .field("duration", &self.duration)
            .field("repetitions", &self.repetitions)
            .field("enabled", &self.enabled)
            .field("script", &self.script)
            .field("name", &self.name)
            .field("vars", &self.vars)
            .field("script_has_changed", &self.script_has_changed)
            .field("executions", &self.executions.len())
            .finish()
    }
}