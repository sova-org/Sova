use crate::lang::Program;

pub type BaliProgram = Vec<TopLevelStatement>;

#[derive(Debug)]
pub enum TopLevelStatement {
    AtStatement(Box<Expression>, Vec<Statement>),
    Statement(Statement),
}

#[derive(Debug)]
pub enum Statement {
    AfterFrac(Box<Expression>, Effect),
    After(Effect),
    BeforeFrac(Box<Expression>, Effect),
    Before(Effect),
    Effect(Effect),
}

#[derive(Debug)]
pub enum Effect {
    Definition(String, Box<Expression>),
    Note(Box<Expression>, Box<Expression>, Box<Expression>, Box<Expression>, String),
    ProgramChange(Box<Expression>, Box<Expression>, String),
    ControlChange(Box<Expression>, Box<Expression>, Box<Expression>, String),
}

#[derive(Debug)]
pub enum Expression {
    Addition(Box<Expression>, Box<Expression>),
    Multiplication(Box<Expression>, Box<Expression>),
    Subtraction(Box<Expression>, Box<Expression>),
    Division(Box<Expression>, Box<Expression>),
    Modulo(Box<Expression>, Box<Expression>),
    Value(Value),
}

#[derive(Debug)]
pub enum Value {
    Number(u8),
    Variable(String),
    Fraction(Box<Expression>, Box<Expression>),
}

pub fn bali_as_asm(prog: BaliProgram) -> Program {
    Vec::new()
}