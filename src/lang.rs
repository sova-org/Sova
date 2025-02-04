use variable::Variable;

use crate::clock::MusicTime;

pub mod variable;

pub enum Event {
    Nop,
    Note(i64, MusicTime),
    Break,
    Exit
}

pub enum ControlASM {
    Mov(Variable, Variable),
    JumpIfLess(Variable, Variable, usize),
    JumpIf(Variable),
    Add(Variable, Variable),
    Sub(Variable, Variable),
    And(Variable, Variable),
    Or(Variable, Variable),
    Not(Variable),
}

pub enum Instruction {
    Control(ControlASM),
    Effect(Event, MusicTime),
}

pub type Program = Vec<Instruction>;
