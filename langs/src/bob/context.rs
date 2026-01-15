//! Compilation context and label-based jump system for the Bob compiler.
//!
//! This module provides:
//! - `Label` and `LabeledInstr` for forward/backward jumps without offset calculation
//! - `resolve_labels()` to convert labeled instructions to relative jumps
//! - `CompileContext` for tracking functions, temporaries, and labels during compilation

use sova_core::vm::Instruction;
use sova_core::vm::control_asm::ControlASM;
use sova_core::vm::variable::Variable;
use std::collections::HashMap;

// ============================================================================
// Label-Based Jump System
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Label(usize);

pub(crate) enum LabeledInstr {
    Instr(Instruction),
    Jump(Label),
    JumpIf(Variable, Label),
    JumpIfNot(Variable, Label),
    Mark(Label),
}

/// Resolves labeled instructions into concrete instructions with relative jumps.
///
/// Two-pass algorithm:
/// 1. First pass: Record positions of all `Mark` labels
/// 2. Second pass: Replace `Jump*` with `RelJump*` using computed offsets
pub(crate) fn resolve_labels(labeled: Vec<LabeledInstr>) -> Vec<Instruction> {
    let mut positions: HashMap<Label, usize> = HashMap::new();
    let mut pos = 0;
    for instr in &labeled {
        match instr {
            LabeledInstr::Mark(label) => {
                positions.insert(label.clone(), pos);
            }
            _ => pos += 1,
        }
    }

    let mut result = Vec::new();
    let mut current = 0;
    for instr in labeled {
        match instr {
            LabeledInstr::Instr(i) => {
                result.push(i);
                current += 1;
            }
            LabeledInstr::Jump(label) => {
                let offset = positions[&label] as i64 - current as i64;
                result.push(Instruction::Control(ControlASM::RelJump(offset)));
                current += 1;
            }
            LabeledInstr::JumpIf(var, label) => {
                let offset = positions[&label] as i64 - current as i64;
                result.push(Instruction::Control(ControlASM::RelJumpIf(var, offset)));
                current += 1;
            }
            LabeledInstr::JumpIfNot(var, label) => {
                let offset = positions[&label] as i64 - current as i64;
                result.push(Instruction::Control(ControlASM::RelJumpIfNot(var, offset)));
                current += 1;
            }
            LabeledInstr::Mark(_) => {}
        }
    }
    result
}

// ============================================================================
// Compilation Context
// ============================================================================

#[derive(Debug, Clone)]
pub(crate) struct FunctionInfo {
    pub arg_names: Vec<String>,
}

pub(crate) struct CompileContext {
    pub functions: HashMap<String, FunctionInfo>,
    pub default_dev: i64,
    pub temp_counter: usize,
    pub label_counter: usize,
}

impl CompileContext {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            default_dev: 1,
            temp_counter: 0,
            label_counter: 0,
        }
    }

    pub fn temp(&mut self, prefix: &str) -> Variable {
        let id = self.temp_counter;
        self.temp_counter += 1;
        Variable::Instance(format!("{prefix}{id}"))
    }

    pub fn line_temp(&mut self, prefix: &str) -> Variable {
        let id = self.temp_counter;
        self.temp_counter += 1;
        Variable::Line(format!("{prefix}{id}"))
    }

    pub fn new_label(&mut self) -> Label {
        let id = self.label_counter;
        self.label_counter += 1;
        Label(id)
    }
}
