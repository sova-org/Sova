use std::collections::HashMap;

pub enum VariableValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Str(String)
}

pub enum Variable {
    Global(String),
    Persistent(String),
    Ephemeral(String),
    Constant(VariableValue)
}

pub type VariableStore = HashMap<String, VariableValue>;
