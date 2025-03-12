use std::{collections::HashMap, ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Sub, SubAssign}, fmt::Write};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum VariableValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

impl PartialOrd for VariableValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (VariableValue::Integer(x), VariableValue::Integer(y)) => x.partial_cmp(y),

            (VariableValue::Float(x), VariableValue::Float(y)) => x.partial_cmp(y),
            (VariableValue::Integer(x), VariableValue::Float(y)) => (*x as f64).partial_cmp(y),
            (VariableValue::Float(x), VariableValue::Integer(y)) => x.partial_cmp(&(*y as f64)),

            (VariableValue::Bool(x), VariableValue::Bool(y)) => x.partial_cmp(y),
            (VariableValue::Bool(x), VariableValue::Integer(y)) => (*x as i64).partial_cmp(y),
            (VariableValue::Integer(x), VariableValue::Bool(y)) => x.partial_cmp(&(*y as i64)),

            (VariableValue::Str(x), VariableValue::Str(y)) => x.partial_cmp(y),
            _ => None
        }
    }
}

impl Add for VariableValue {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}
impl AddAssign for VariableValue {
    fn add_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (VariableValue::Integer(x), VariableValue::Integer(y)) => *x += y,

            (VariableValue::Float(x), VariableValue::Float(y)) => *x += y,
            (VariableValue::Integer(x), VariableValue::Float(y)) => *x += y as i64,
            (VariableValue::Float(x), VariableValue::Integer(y)) => *x += y as f64,

            (VariableValue::Bool(x), VariableValue::Bool(y)) => *x |= y,
            (VariableValue::Bool(x), VariableValue::Integer(y)) => *x |= y > 0,
            (VariableValue::Integer(x), VariableValue::Bool(y)) => *x |= y as i64,

            (VariableValue::Str(x), VariableValue::Str(y)) => *x += &y,
            (VariableValue::Str(x), VariableValue::Integer(y)) => { let _ = write!(x, "{y}"); },
            (VariableValue::Str(x), VariableValue::Float(y)) => { let _ = write!(x, "{y}") ; },
            (VariableValue::Str(x), VariableValue::Bool(y)) => { let _ = write!(x, "{y}") ; },
            _ => ()
        };
    }
}
impl Sub for VariableValue {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}
impl SubAssign for VariableValue {
    fn sub_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (VariableValue::Integer(x), VariableValue::Integer(y)) => *x -= y,
            (VariableValue::Float(x), VariableValue::Float(y)) => *x -= y,
            (VariableValue::Integer(x), VariableValue::Float(y)) => *x -= y as i64,
            (VariableValue::Float(x), VariableValue::Integer(y)) => *x -= y as f64,
            (VariableValue::Integer(x), VariableValue::Bool(y)) => *x -= y as i64,
            _ => ()
        };
    }
}
impl BitAnd for VariableValue {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl BitAndAssign for VariableValue {
    fn bitand_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (VariableValue::Integer(x), VariableValue::Integer(y)) => *x &= y,
            (VariableValue::Integer(x), VariableValue::Float(y)) => *x &= y as i64,
            (VariableValue::Bool(x), VariableValue::Bool(y)) => *x &= y,
            (VariableValue::Bool(x), VariableValue::Integer(y)) => *x &= y > 0,
            (VariableValue::Integer(x), VariableValue::Bool(y)) => *x &= y as i64,
            _ => ()
        };
    }
}
impl BitOr for VariableValue {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl BitOrAssign for VariableValue {
    fn bitor_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (VariableValue::Integer(x), VariableValue::Integer(y)) => *x |= y,
            (VariableValue::Integer(x), VariableValue::Float(y)) => *x |= y as i64,
            (VariableValue::Bool(x), VariableValue::Bool(y)) => *x |= y,
            (VariableValue::Bool(x), VariableValue::Integer(y)) => *x |= y > 0,
            (VariableValue::Integer(x), VariableValue::Bool(y)) => *x |= y as i64,
            _ => ()
        };
    }
}
impl Not for VariableValue {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            VariableValue::Integer(i) => VariableValue::Integer(!i),
            VariableValue::Bool(b) => VariableValue::Bool(!b),
            _ => self
        }
    }
}

impl From<i64> for VariableValue {
    fn from(value: i64) -> Self {
        VariableValue::Integer(value)
    }
}
impl From<f64> for VariableValue {
    fn from(value: f64) -> Self {
        VariableValue::Float(value)
    }
}
impl From<bool> for VariableValue {
    fn from(value: bool) -> Self {
        VariableValue::Bool(value)
    }
}
impl From<String> for VariableValue {
    fn from(value: String) -> Self {
        VariableValue::Str(value)
    }
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

}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Variable {
    Environment(String),
    Global(String),
    Sequence(String), // not fully handled
    Step(String),
    Instance(String),
    Constant(VariableValue),
}

pub type VariableStore = HashMap<String, VariableValue>;

impl Variable {

    pub fn evaluate(&self, environment_vars : &VariableStore, global_vars : &VariableStore, sequence_vars : &VariableStore, step_vars : &VariableStore, instance_vars : &VariableStore)
        -> Option<VariableValue>
    {
        (match self {
            Variable::Environment(name) => environment_vars.get(name),
            Variable::Global(name) => global_vars.get(name),
            Variable::Sequence(name) => sequence_vars.get(name),
            Variable::Step(name) => step_vars.get(name),
            Variable::Instance(name) => instance_vars.get(name),
            Variable::Constant(value) => Some(value),
        }).map(VariableValue::clone)
    }

    pub fn set(
        &self,
        value : VariableValue,
        environment_vars : &mut VariableStore,
        global_vars : &mut VariableStore,
        sequence_vars : &mut VariableStore,
        step_vars : &mut VariableStore,
        instance_vars : &mut VariableStore
    ) {
        match self {
            Variable::Environment(name) => { environment_vars.insert(name.clone(), value); },
            Variable::Global(name) => { global_vars.insert(name.clone(), value); },
            Variable::Sequence(name) => { sequence_vars.insert(name.clone(), value); },
            Variable::Step(name) => { step_vars.insert(name.clone(), value); },
            Variable::Instance(name) => { instance_vars.insert(name.clone(), value); },
            Variable::Constant(_) => (),
        };
    }

    pub fn mut_value<'a>(
        &'a self,
        environment_vars : &'a mut VariableStore,
        global_vars : &'a mut VariableStore,
        step_vars : &'a mut VariableStore,
        sequence_vars : &'a mut VariableStore,
        instance_vars : &'a mut VariableStore
    ) -> Option<&'a mut VariableValue> {
        match self {
            Variable::Environment(name) => environment_vars.get_mut(name),
            Variable::Global(name) => global_vars.get_mut(name),
            Variable::Sequence(name) => sequence_vars.get_mut(name),
            Variable::Step(name) => step_vars.get_mut(name),
            Variable::Instance(name) => instance_vars.get_mut(name),
            _ => None
        }
    }

    pub fn exists(&self, environment_vars : &VariableStore, global_vars : &VariableStore, sequence_vars : &VariableStore, step_vars : &VariableStore, instance_vars : &VariableStore)
        -> bool
    {
        match self {
            Variable::Environment(name) => environment_vars.contains_key(name),
            Variable::Global(name) => global_vars.contains_key(name),
            Variable::Sequence(name) => sequence_vars.contains_key(name),
            Variable::Step(name) => step_vars.contains_key(name),
            Variable::Instance(name) => instance_vars.contains_key(name),
            Variable::Constant(_) => true,
        }
    }

    pub fn make_as(
        &self,
        other : &Variable,
        environment_vars : &mut VariableStore,
        global_vars : &mut VariableStore,
        sequence_vars : &mut VariableStore,
        step_vars : &mut VariableStore,
        instance_vars : &mut VariableStore
    ) {
        let Some(value) = other.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars) else {
            return;
        };
        let value = value.clone_type();
        match self {
            Variable::Environment(name) => { environment_vars.insert(name.clone(), value); },
            Variable::Global(name) => { global_vars.insert(name.clone(), value); },
            Variable::Sequence(name) => { sequence_vars.insert(name.clone(), value); },
            Variable::Step(name) => { step_vars.insert(name.clone(), value); },
            Variable::Instance(name) => { instance_vars.insert(name.clone(), value); },
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
            environment_vars : &mut VariableStore, global_vars : &mut VariableStore, sequence_vars : &mut VariableStore, step_vars : &mut VariableStore, instance_vars : &mut VariableStore
    ) -> bool {
        let mut res = true;
        match (var1.exists(environment_vars, global_vars, sequence_vars, step_vars, instance_vars), var2.exists(environment_vars, global_vars, sequence_vars, step_vars, instance_vars)) {
            (true, false) => var2.make_as(var1, environment_vars, global_vars, sequence_vars, step_vars, instance_vars),
            (false, true) => var1.make_as(var2, environment_vars, global_vars, sequence_vars, step_vars, instance_vars),
            (false, false) => res = false,
            _ => ()
        }
        res
    }

}
