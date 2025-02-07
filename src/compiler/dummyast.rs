use crate::lang::{Program, Instruction, Event};
use crate::clock::TimeSpan;

mod dummytranslator;
pub use dummytranslator::DummyCompiler;

#[derive(Debug)]
pub struct Prog {
    pub instructions : Vec<Inst>,
}

#[derive(Debug, Copy, Clone)]
pub enum Inst {
    EventPlayNote(u64, u64, u64),
}

impl Prog {
    pub fn new(i: Inst) -> Prog {
        Prog {
            instructions: vec![i]
        }
    }

    pub fn add_instruction(&mut self, i: Inst) {
        self.instructions.push(i);
    }

    pub fn as_asm(self) -> Program {
        self.instructions.iter().map(|i| i.as_asm()).collect()
    }
}

impl Inst {
    pub fn as_asm(self) -> Instruction {
        use self::Inst::*;
        match self {
            EventPlayNote(n, d, p) => Instruction::Effect(
                Event::Note(n, TimeSpan::Micros(d*100000)),
                TimeSpan::Micros(p*100000)
            ),
        }
    }
}
