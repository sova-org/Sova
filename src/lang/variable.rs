use std::collections::HashMap;

#[derive(Debug)]
pub enum VariableValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Str(String)
}

#[derive(Debug)]
pub enum Variable {
    Global(String),
    Persistent(String),
    Ephemeral(String),
    Constant(VariableValue)
}

pub type VariableStore = HashMap<String, VariableValue>;
