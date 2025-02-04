use control_asm::ControlASM;
use serde::{Deserialize, Serialize};

use crate::clock::TimeSpan;

pub mod variable;
pub mod control_asm;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Nop,
    Note(i64, TimeSpan),
    Break,
    Exit
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Instruction {
    Control(ControlASM),
    Effect(Event, TimeSpan),
}

impl Instruction {

    pub fn is_control(&self) -> bool {
        match self {
            Instruction::Control(_) => true,
            _ => false
        }
    }

    pub fn is_effect(&self) -> bool {
        match self {
            Instruction::Effect(_,_) => true,
            _ => false
        }
    }

    pub fn yield_effect(&self) -> Option<(Event, TimeSpan)> {
        match self {
            Instruction::Effect(a,b) => Some((a.clone(), b.clone())),
            _ => None
        }
    }

}

pub type Program = Vec<Instruction>;
