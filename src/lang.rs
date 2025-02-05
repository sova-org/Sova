use variable::Variable;

use crate::clock::MusicTime;

pub mod variable;

#[derive(Debug)]
pub enum Event {
    Nop,
    Note(u64, MusicTime),
    Break,
    Exit
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Instruction {
    Control(ControlASM),
    Effect(Event, MusicTime),
}

pub type Program = Vec<Instruction>;
