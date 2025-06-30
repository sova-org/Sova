//! Defines the core language elements, instructions, and program structure.

use control_asm::ControlASM;
use event::Event;
use serde::{Deserialize, Serialize};
use variable::{Variable, VariableValue};

/// Module related to control flow instructions.
pub mod control_asm;
/// Module defining functions available in the execution environment.
pub mod environment_func;
/// Module for the context during program evaluation.
pub mod evaluation_context;
/// Module defining events that can be triggered as effects.
pub mod event;
/// Module defining the variable types and values used in the language.
pub mod variable;

/// Represents a single instruction in a program's execution flow.
///
/// An instruction is the fundamental unit of execution. Programs are sequences of these instructions.
/// Instructions are either control flow/computation operations (`ControlASM`) or side effects (`Event`).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Instruction {
    /// A control flow or computation instruction.
    ///
    /// These instructions handle operations like arithmetic, logic, comparisons,
    /// memory access (moving values, stack operations), conditional and unconditional jumps,
    /// function calls, and interacting with built-in oscillators or MIDI CC values.
    /// See `ControlASM` for specific operations.
    Control(ControlASM),

    /// A side effect instruction, representing an interaction with the outside world.
    ///
    /// These instructions typically send messages via protocols like MIDI, OSC, or Dirt/TidalCycles.
    /// They are defined symbolically using `Event` and `Variable`s, which are evaluated
    /// into `ConcreteEvent`s at runtime using the `EvaluationContext`.
    /// The second `Variable` often specifies the target device ID for the event.
    Effect(Event, Variable),
}

impl Instruction {
    /// Returns `true` if the instruction is a control flow instruction.
    pub fn is_control(&self) -> bool {
        matches!(self, Instruction::Control(_))
    }

    /// Returns `true` if the instruction is an effect instruction.
    pub fn is_effect(&self) -> bool {
        matches!(self, Instruction::Effect(_, _))
    }
}

/// Represents a sequence of instructions forming a complete program or function body.
pub type Program = Vec<Instruction>;

pub fn debug_print(prog: &Program, about: String, begin: String) {
    let info = format!("INTERNAL {} CONTENT", about);
    println!("{}BEGIN: {}", begin, info);
    for (count, inst) in prog.iter().enumerate() {
        match inst {
            Instruction::Control(ControlASM::RelJump(x))
            | Instruction::Control(ControlASM::RelJumpIf(_, x))
            | Instruction::Control(ControlASM::RelJumpIfNot(_, x))
            | Instruction::Control(ControlASM::RelJumpIfDifferent(_, _, x))
            | Instruction::Control(ControlASM::RelJumpIfEqual(_, _, x))
            | Instruction::Control(ControlASM::RelJumpIfLess(_, _, x))
            | Instruction::Control(ControlASM::RelJumpIfLessOrEqual(_, _, x)) => {
                println!(
                    "{}{}: {:?} ➡️  {}",
                    begin,
                    count,
                    inst,
                    (count as i64 + x) as usize
                )
            }
            Instruction::Control(ControlASM::Jump(x))
            | Instruction::Control(ControlASM::JumpIf(_, x))
            | Instruction::Control(ControlASM::JumpIfNot(_, x))
            | Instruction::Control(ControlASM::JumpIfDifferent(_, _, x))
            | Instruction::Control(ControlASM::JumpIfEqual(_, _, x))
            | Instruction::Control(ControlASM::JumpIfLess(_, _, x))
            | Instruction::Control(ControlASM::JumpIfLessOrEqual(_, _, x)) => {
                println!("{}{}: {:?} ➡️  {}", begin, count, inst, x)
            }
            Instruction::Control(ControlASM::Mov(
                Variable::Constant(VariableValue::Func(f)),
                f_content,
            )) => {
                println!("{}{}: Control(Mov(", begin, count);
                debug_print(f, "FUNCTION".to_string(), "   ".to_string());
                println!("{}   {:?}))", begin, f_content);
            }
            _ => println!("{}{}: {:?}", begin, count, inst),
        };
    }
    println!("{}END: {}", begin, info);
}
