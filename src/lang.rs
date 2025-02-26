use control_asm::ControlASM;
use event::Event;
use serde::{Deserialize, Serialize};

use crate::clock::TimeSpan;

pub mod control_asm;
pub mod event;
pub mod variable;

#[derive(Debug, Serialize, Deserialize)]
pub enum Instruction {
    Control(ControlASM),
    Effect(Event, TimeSpan),
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
