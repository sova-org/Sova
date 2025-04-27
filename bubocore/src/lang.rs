use control_asm::ControlASM;
use event::Event;
use serde::{Deserialize, Serialize};
use variable::Variable;

pub mod control_asm;
pub mod environment_func;
pub mod evaluation_context;
pub mod event;
pub mod variable;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Instruction {
    Control(ControlASM),
    Effect(Event, Variable),
}

impl Instruction {
    pub fn is_control(&self) -> bool {
        match self {
            Instruction::Control(_) => true,
            _ => false,
        }
    }

    pub fn is_effect(&self) -> bool {
        match self {
            Instruction::Effect(_, _) => true,
            _ => false,
        }
    }
}

pub type Program = Vec<Instruction>;
