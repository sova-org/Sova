use std::{collections::HashMap, ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr}};

use serde::{Deserialize, Serialize};

use crate::clock::{Clock, TimeSpan};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum VariableValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Dur(TimeSpan),
}

impl BitAnd for VariableValue {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => VariableValue::Integer(i1 & i2),
            _ => panic!("Bitwise and with wrong types, this should never happen"),
        }
    }
}

impl BitOr for VariableValue {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => VariableValue::Integer(i1 | i2),
            _ => panic!("Bitwise or with wrong types, this should never happen"),
        }
    }
}

impl BitXor for VariableValue {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => VariableValue::Integer(i1 ^ i2),
            _ => panic!("Bitwise xor with wrong types, this should never happen"),
        }
    }
}

impl Shl for VariableValue {
    type Output = Self;
    fn shl(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                if i2 < 0 {
                    VariableValue::Integer(i1)
                } else {
                    VariableValue::Integer(i1 << i2)
                }
            },
            _ => panic!("Left shift with wrong types, this should never happen"),
        }
    }
}

impl Shr for VariableValue {
    type Output = Self;
    fn shr(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                if i2 < 0 {
                    VariableValue::Integer(i1)
                } else {
                    VariableValue::Integer(i1 >> i2)
                }
            }
            _ => panic!("Right shift (arithmetic) with wrong types, this should never happen"),
        }
    }
}

impl Not for VariableValue {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            VariableValue::Integer(i) => VariableValue::Integer(!i),
            VariableValue::Bool(b) => VariableValue::Bool(!b),
            _ => panic!("Not or bitwise not with wrong types, this should never happen"),
        }
    }
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
            VariableValue::Dur(_) => Self::Dur(TimeSpan::Micros(0)),
        }
    }

    pub fn is_true(self, clock : &Clock) -> bool {
        match self {
            VariableValue::Bool(b) => b,
            _ => self.cast_as_bool(clock).is_true(clock), // peut-être que ce serait mieux de ne pas autoriser à utiliser is_true sur autre chose que des Bool ?
        }
    }

    pub fn add(self, other : VariableValue, clock : &Clock) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => VariableValue::Integer(i1 + i2),
            (VariableValue::Float(f1), VariableValue::Float(f2)) => VariableValue::Float(f1 + f2),
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => VariableValue::Dur(d1.add(d2, clock)),
            _ => panic!("Addition with wrong types, this should never happen"),
        }
    }
    
    pub fn div(self, other : VariableValue, clock : &Clock) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                if i2 != 0 {
                    VariableValue::Integer(i1 / i2)
                } else {
                    VariableValue::Integer(0)
                }
            },
            (VariableValue::Float(f1), VariableValue::Float(f2)) => {
                if f2 != 0.0 {
                    VariableValue::Float(f1 / f2)
                } else {
                    VariableValue::Float(0.0)
                }
            },
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => VariableValue::Dur(d1.div(d2, clock)),
            _ => panic!("Division with wrong types, this should never happen"),
        }
    }
    
    pub fn rem(self, other : VariableValue, clock : &Clock) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                if i2 != 0 {
                    VariableValue::Integer(i1 % i2)
                } else {
                    VariableValue::Integer(i1)
                }
            },
            (VariableValue::Float(_), VariableValue::Float(_)) => VariableValue::Float(0.0),
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => VariableValue::Dur(d1.rem(d2, clock)),
            _ => panic!("Reminder (modulo) with wrong types, this should never happen"),
        }
    }
    
    pub fn mul(self, other : VariableValue, clock : &Clock) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => VariableValue::Integer(i1 * i2),
            (VariableValue::Float(f1), VariableValue::Float(f2)) => VariableValue::Float(f1 * f2),
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => VariableValue::Dur(d1.mul(d2, clock)),
            _ => panic!("Multiplication with wrong types, this should never happen"),
        }
    }
    
    pub fn sub(self, other : VariableValue, clock : &Clock) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => VariableValue::Integer(i1 - i2),
            (VariableValue::Float(f1), VariableValue::Float(f2)) => VariableValue::Float(f1 - f2),
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => VariableValue::Dur(d1.sub(d2, clock)),
            _ => panic!("Subtraction with wrong types, this should never happen"),
        }
    }

    pub fn and(self, other : VariableValue) -> VariableValue {
        match (self, other) {
            (VariableValue::Bool(b1), VariableValue::Bool(b2)) => VariableValue::Bool(b1 && b2),
            _ => panic!("Logical and with wrong types, this should never happen"),
        }
    }

    pub fn or(self, other : VariableValue) -> VariableValue {
        match (self, other) {
            (VariableValue::Bool(b1), VariableValue::Bool(b2)) => VariableValue::Bool(b1 || b2),
            _ => panic!("Logical or with wrong types, this should never happen"),
        }
    }
 
    pub fn xor(self, other : VariableValue) -> VariableValue {
        match (self, other) {
            (VariableValue::Bool(b1), VariableValue::Bool(b2)) => VariableValue::Bool((b1 && !b2) || (!b1 && b2)),
            _ => panic!("Logical xor with wrong types, this should never happen"),
        }
    }

    pub fn logical_shift(self, other : VariableValue) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                if i2 < 0 {
                   VariableValue::Integer(i1)
                } else {
                    VariableValue::Integer((i1 as u64 >> i2 as u64) as i64)
                }
            }
            _ => panic!("Right shift (logical) with wrong types, this should never happen"),
        }
    }

    pub fn cast_as_integer(&self, clock : &Clock) -> VariableValue {
        match self {
        VariableValue::Integer(i) => VariableValue::Integer(*i),
        VariableValue::Float(f) => VariableValue::Integer(f.round() as i64),
        VariableValue::Bool(b) => if *b { VariableValue::Integer(1) } else { VariableValue::Integer(0) }
        VariableValue::Str(s) => match s.parse::<i64>() {
            Ok(n) => VariableValue::Integer(n),
            Err(_) => VariableValue::Integer(0),
          }
        VariableValue::Dur(d) => VariableValue::Integer(d.as_micros(clock).try_into().unwrap()),
        }
    }

    pub fn cast_as_float(&self, clock : &Clock) -> VariableValue {
        match self {
        VariableValue::Integer(i) => VariableValue::Float(*i as f64),
        VariableValue::Float(f) => VariableValue::Float(*f),
        VariableValue::Bool(b) => if *b { VariableValue::Float(1.0) } else { VariableValue::Float(0.0) }
        VariableValue::Str(s) => match s.parse::<f64>() {
            Ok(n) => VariableValue::Float(n),
            Err(_) => VariableValue::Float(0.0),
          }
        VariableValue::Dur(d) => VariableValue::Float(d.as_micros(clock) as f64),
        }
    }

    pub fn cast_as_bool(&self, clock : &Clock) -> VariableValue {
        match self {
            VariableValue::Integer(i) => VariableValue::Bool(*i != 0),
            VariableValue::Float(f) => VariableValue::Bool(*f != 0.0),
            VariableValue::Bool(b) => VariableValue::Bool(*b),
            VariableValue::Str(s) => VariableValue::Bool(s.len() > 0), 
            VariableValue::Dur(d) => VariableValue::Bool(d.as_micros(clock) != 0),
        }
    }

    pub fn cast_as_str(&self, clock : &Clock) -> VariableValue {
        match self {
            VariableValue::Integer(i) => VariableValue::Str(i.to_string()),
            VariableValue::Float(f) => VariableValue::Str(f.to_string()),
            VariableValue::Bool(b) => if *b { VariableValue::Str("True".to_string()) } else { VariableValue::Str("False".to_string()) },
            VariableValue::Str(s) => VariableValue::Str(s.to_string()),
            VariableValue::Dur(d) => VariableValue::Str(d.as_micros(clock).to_string()),
        }
    }

    pub fn cast_as_dur(&self) -> VariableValue {
        match self {
            VariableValue::Integer(i) => VariableValue::Dur(TimeSpan::Micros(i.unsigned_abs())),
            VariableValue::Float(f) => VariableValue::Dur(TimeSpan::Micros((f.round() as i64).unsigned_abs())),
            VariableValue::Bool(_) => VariableValue::Dur(TimeSpan::Micros(0)), // TODO décider comment caster booléen vers durée
            VariableValue::Str(_) => VariableValue::Dur(TimeSpan::Micros(0)), // TODO parser la chaîne de caractères
            VariableValue::Dur(d) => VariableValue::Dur(*d),
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
        -> VariableValue
    {
        let res = (match self {
            Variable::Environment(name) => environment_vars.get(name),
            Variable::Global(name) => global_vars.get(name),
            Variable::Sequence(name) => sequence_vars.get(name),
            Variable::Step(name) => step_vars.get(name),
            Variable::Instance(name) => instance_vars.get(name),
            Variable::Constant(value) => Some(value),
        }).map(VariableValue::clone);

        match res {
            Some(vv) => vv,
            None => VariableValue::Bool(false), 
        }
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
        let value = other.evaluate(environment_vars, global_vars, sequence_vars, step_vars, instance_vars);
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
