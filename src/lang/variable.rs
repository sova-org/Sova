use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Str(String)
}

impl VariableValue {

    pub fn clone_type(&self) -> VariableValue {
        match self {
            VariableValue::Integer(_) => Self::Integer(0),
            VariableValue::Float(_) => Self::Float(0.0),
            VariableValue::Bool(_) => Self::Bool(false),
            VariableValue::Str(_) => Self::Str("".to_owned()),
        }
    }

    pub fn is_true(&self) -> bool {
        match self {
            VariableValue::Integer(i) => *i > 0,
            VariableValue::Float(f) => *f > 0.0,
            VariableValue::Bool(b) => *b,
            VariableValue::Str(s) => s.len() > 0,
        }
    }

    pub fn make_consistents(value1 : &mut VariableValue, value2 : &mut VariableValue) -> bool {

        true
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Variable {
    Global(String),
    Persistent(String),
    Ephemeral(String),
    Constant(VariableValue)
}

pub type VariableStore = HashMap<String, VariableValue>;

impl Variable {

    pub fn evaluate(&self, globals : &VariableStore, persistents : &VariableStore, ephemer : &VariableStore)
        -> Option<VariableValue>
    {
        (match self {
            Variable::Global(name) => globals.get(name),
            Variable::Persistent(name) => persistents.get(name),
            Variable::Ephemeral(name) => ephemer.get(name),
            Variable::Constant(value) => Some(value),
        }).map(VariableValue::clone)
    }

    pub fn set(
        &self, value : VariableValue, globals : &mut VariableStore, persistents : &mut VariableStore, ephemer : &mut VariableStore
    ) {
        match self {
            Variable::Global(name) => { globals.insert(name.clone(), value); },
            Variable::Persistent(name) => { persistents.insert(name.clone(), value); },
            Variable::Ephemeral(name) => { ephemer.insert(name.clone(), value); },
            Variable::Constant(_) => (),
        };
    }

    pub fn exists(&self, globals : &VariableStore, persistents : &VariableStore, ephemer : &VariableStore)
        -> bool
    {
        match self {
            Variable::Global(name) => globals.contains_key(name),
            Variable::Persistent(name) => persistents.contains_key(name),
            Variable::Ephemeral(name) => ephemer.contains_key(name),
            Variable::Constant(_) => true,
        }
    }

    pub fn make_as(&self, other : &Variable, globals : &mut VariableStore, persistents : &mut VariableStore, ephemer : &mut VariableStore) {
        let Some(value) = other.evaluate(globals, persistents, ephemer) else {
            return;
        };
        let value = value.clone_type();
        match self {
            Variable::Global(name) => { globals.insert(name.clone(), value); },
            Variable::Persistent(name) => { persistents.insert(name.clone(), value); },
            Variable::Ephemeral(name) => { ephemer.insert(name.clone(), value); },
            Variable::Constant(_) => (),
        };
    }

    pub fn is_mutable(&self) -> bool {
        match self {
            Variable::Constant(_) => false,
            _ => true
        }
    }

    pub fn ensure_existing(
            var1 : &Variable, var2 : &Variable,
            globals : &mut VariableStore, persistents : &mut VariableStore, ephemer : &mut VariableStore
    ) -> bool {
        let mut res = true;
        match (var1.exists(globals, persistents, ephemer), var2.exists(globals, persistents, ephemer)) {
            (true, false) => var2.make_as(var1, globals, persistents, ephemer),
            (false, true) => var1.make_as(var2, globals, persistents, ephemer),
            (false, false) => res = false,
            _ => ()
        }
        res
    }

}
