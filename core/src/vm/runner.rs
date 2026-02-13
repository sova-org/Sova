//! Standalone runner for executing Sova programs without the full runtime.
//!
//! This module provides a minimal execution environment for testing and debugging
//! Sova languages. Use [`Runner`] to configure the environment, or the convenience
//! functions [`execute_program`] and [`execute_interpreter`] for quick execution
//! with defaults.
//!
//! # Examples
//!
//! Simple execution with defaults:
//! ```ignore
//! use sova_core::vm::runner::execute_program;
//!
//! let result = execute_program(prog);
//! ```
//!
//! Custom environment setup:
//! ```ignore
//! use sova_core::vm::runner::Runner;
//! use sova_core::vm::variable::VariableValue;
//!
//! let mut runner = Runner::new();
//! runner.tempo = 140.0;
//! runner.global_vars.insert("X".to_string(), VariableValue::Integer(10));
//! let result = runner.run_program(prog);
//! ```

use std::collections::VecDeque;
use std::sync::Arc;

use crate::clock::{Clock, ClockServer, SyncTime};
use crate::device_map::DeviceMap;
use crate::vm::event::ConcreteEvent;
use crate::vm::interpreter::Interpreter;
use crate::vm::interpreter::asm_interpreter::ASMInterpreter;
use crate::vm::variable::VariableStore;
use crate::vm::{EvaluationContext, Program};

/// Result of executing a program to completion.
#[derive(Debug)]
pub struct ExecutionResult {
    /// Events emitted during execution, paired with their scheduled time (microseconds).
    pub events: Vec<(ConcreteEvent, SyncTime)>,
    /// Global variables after execution.
    pub global_vars: VariableStore,
    /// Frame variables after execution.
    pub frame_vars: VariableStore,
    /// Line variables after execution.
    pub line_vars: VariableStore,
    /// Instance variables after execution.
    pub instance_vars: VariableStore,
    /// Total accumulated time in microseconds.
    pub total_time: SyncTime,
}

/// Configurable runner for executing Sova programs.
///
/// Create a runner, configure the environment, then call [`run_program`](Runner::run_program)
/// or [`run_interpreter`](Runner::run_interpreter).
pub struct Runner {
    // --- Variables (pre-populated state) ---
    /// Global variables, shared across all scripts (single uppercase letters in Bob).
    pub global_vars: VariableStore,
    /// Frame-scoped variables, persist within a frame.
    pub frame_vars: VariableStore,
    /// Line-scoped variables, persist within a line.
    pub line_vars: VariableStore,

    // --- Timing ---
    /// Tempo in beats per minute.
    pub tempo: f64,
    /// Musical quantum (beats per bar).
    pub quantum: f64,
    /// Frame length in beats.
    pub frame_len: f64,

    // --- Scene context ---
    /// Current line index in the scene.
    pub line_index: usize,
    pub line_iterations: usize,
    /// Current frame index within the line.
    pub frame_index: usize,
    pub frame_triggers: usize,
    /// Scene structure: frame lengths for each line. `structure[line][frame] = length in beats`.
    pub structure: Vec<Vec<f64>>,
}

impl Default for Runner {
    fn default() -> Self {
        Self {
            global_vars: VariableStore::new(),
            frame_vars: VariableStore::new(),
            line_vars: VariableStore::new(),
            tempo: 120.0,
            quantum: 4.0,
            frame_len: 1.0,
            line_index: 0,
            line_iterations: 0,
            frame_index: 0,
            frame_triggers: 0,
            structure: vec![vec![1.0]],
        }
    }
}

impl Runner {
    /// Create a new runner with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Execute a compiled program (bytecode) and collect results.
    pub fn run_program(self, prog: Program) -> ExecutionResult {
        let interp = Box::new(ASMInterpreter::new(prog));
        self.run_interpreter(interp)
    }

    /// Execute an interpreter and collect results.
    pub fn run_interpreter(self, mut interp: Box<dyn Interpreter>) -> ExecutionResult {
        let clock_server = Arc::new(ClockServer::new(self.tempo, self.quantum));
        let clock: Clock = clock_server.into();
        let device_map = DeviceMap::new();

        let mut global_vars = self.global_vars;
        let mut frame_vars = self.frame_vars;
        let mut line_vars = self.line_vars;
        let mut instance_vars = VariableStore::new();
        let mut stack = VecDeque::new();

        let mut events = Vec::new();
        let mut total_time: SyncTime = 0;

        while !interp.has_terminated() {
            let mut ctx = EvaluationContext {
                logic_date: total_time,
                global_vars: &mut global_vars,
                line_vars: &mut line_vars,
                frame_vars: &mut frame_vars,
                instance_vars: &mut instance_vars,
                stack: &mut stack,
                line_index: self.line_index,
                line_iterations: self.line_iterations,
                frame_index: self.frame_index,
                frame_len: self.frame_len,
                frame_triggers: self.frame_triggers,
                structure: &self.structure,
                clock: &clock,
                device_map: &device_map,
            };

            let (event_opt, wait_time) = interp.execute_next(&mut ctx);

            if let Some(event) = event_opt {
                events.push((event, total_time));
            }
            if wait_time != crate::clock::NEVER {
                total_time = total_time.saturating_add(wait_time);
            }
        }

        ExecutionResult {
            events,
            global_vars,
            frame_vars,
            line_vars,
            instance_vars,
            total_time,
        }
    }
}

// --- Convenience functions for simple cases ---

/// Execute a compiled program with default configuration.
pub fn execute_program(prog: Program) -> ExecutionResult {
    Runner::new().run_program(prog)
}

/// Execute an interpreter with default configuration.
pub fn execute_interpreter(interp: Box<dyn Interpreter>) -> ExecutionResult {
    Runner::new().run_interpreter(interp)
}
