use std::{
    cmp::Ordering, collections::{HashMap, HashSet}, mem, ops::Neg
};

use serde::{Deserialize, Serialize};

use crate::{
    clock::{SyncTime, TimeSpan}, error::SovaError, util::decimal_operations::Decimal, vm::{Program, ValueGenerator}
};

use super::{EvaluationContext, environment_func::EnvironmentFunc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VariableValue {
    Decimal(Decimal), 
    Func(Program),
    Blob(Vec<u8>),
    Generator(ValueGenerator),
    #[serde(untagged)]
    Integer(i64),
    #[serde(untagged)]
    Float(f64),
    #[serde(untagged)]
    Bool(bool),
    #[serde(untagged)]
    Str(String),
    #[serde(untagged)]
    Dur(TimeSpan),
    #[serde(untagged)]
    Map(HashMap<String, VariableValue>),
    #[serde(untagged)]
    Vec(Vec<VariableValue>),
}

impl Default for VariableValue {
    fn default() -> Self {
        Self::Integer(0)
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
impl From<TimeSpan> for VariableValue {
    fn from(value: TimeSpan) -> Self {
        VariableValue::Dur(value)
    }
}
impl From<HashMap<String, VariableValue>> for VariableValue {
    fn from(value: HashMap<String, VariableValue>) -> Self {
        VariableValue::Map(value)
    }
}
impl From<Vec<VariableValue>> for VariableValue {
    fn from(value: Vec<VariableValue>) -> Self {
        VariableValue::Vec(value)
    }
}
impl From<Vec<u8>> for VariableValue {
    fn from(value: Vec<u8>) -> Self {
        VariableValue::Blob(value)
    }
}
impl From<Program> for VariableValue {
    fn from(value: Program) -> Self {
        VariableValue::Func(value)
    }
}
impl From<ValueGenerator> for VariableValue {
    fn from(value: ValueGenerator) -> Self {
        VariableValue::Generator(value)
    }
}

impl VariableValue {
    pub fn clone_type(&self) -> VariableValue {
        match self {
            VariableValue::Integer(_) => Self::Integer(0),
            VariableValue::Float(_) => Self::Float(0.0),
            VariableValue::Decimal(_) => Self::Decimal(Default::default()),
            VariableValue::Bool(_) => Self::Bool(false),
            VariableValue::Str(_) => Self::Str("".to_owned()),
            VariableValue::Dur(_) => Self::Dur(TimeSpan::Micros(0)),
            VariableValue::Func(_) => todo!(),
            VariableValue::Map(_) => Self::Map(HashMap::new()),
            VariableValue::Vec(_) => Self::Vec(Vec::new()),
            VariableValue::Blob(_) => Self::Blob(Vec::new()),
            VariableValue::Generator(_) => Self::Generator(Default::default())
        }
    }

    pub fn as_type(&mut self, other: &VariableValue, ctx: &EvaluationContext) {
        // cast to correct types
        match other {
            VariableValue::Integer(_) => {
                self.cast_as_integer(ctx);
            }
            VariableValue::Float(_) => {
                self.cast_as_float(ctx);
            }
            VariableValue::Bool(_) => {
                self.cast_as_bool(ctx);
            }
            VariableValue::Str(_) => {
                self.cast_as_str(ctx);
            }
            VariableValue::Decimal(_) => {
                self.cast_as_decimal(ctx);
            }
            VariableValue::Dur(_) => {
                self.cast_as_dur(ctx);
            }
            VariableValue::Map(_) => {
                self.cast_as_map(ctx);
            }
            VariableValue::Vec(_) => {
                self.cast_as_vec(ctx);
            }
            VariableValue::Blob(_) => {
                self.cast_as_blob(ctx);
            }
            VariableValue::Generator(g) => {
                self.as_type(&g.get_current(ctx), ctx);
            }
            VariableValue::Func(_) => *self = other.clone_type(),
        }
    }

    pub fn compatible_cast(&mut self, other: &mut VariableValue, ctx: &EvaluationContext) {
        // cast to correct types
        match self {
            VariableValue::Integer(_) => {
                other.cast_as_integer(ctx);
            }
            VariableValue::Float(_) => {
                other.cast_as_float(ctx);
            }
            VariableValue::Decimal(_) => {
                other.cast_as_decimal(ctx);
            }
            VariableValue::Dur(_) => {
                other.cast_as_dur(ctx);
            }
            VariableValue::Map(_) => {
                other.cast_as_map(ctx);
            }
            VariableValue::Vec(_) => {
                other.cast_as_vec(ctx);
            }
            VariableValue::Str(_) => {
                other.cast_as_str(ctx);
            }
            _ => match other {
                VariableValue::Integer(_) => {
                    self.cast_as_integer(ctx);
                }
                VariableValue::Float(_) => {
                    self.cast_as_float(ctx);
                }
                VariableValue::Decimal(_) => {
                    self.cast_as_decimal(ctx);
                }
                VariableValue::Dur(_) => {
                    self.cast_as_dur(ctx);
                }
                VariableValue::Str(_) => {
                    self.cast_as_str(ctx);
                }
                VariableValue::Map(_) => {
                    self.cast_as_map(ctx);
                }
                _ => {
                    self.cast_as_integer(ctx);
                    other.cast_as_integer(ctx);
                }
            },
        }
    }

    pub fn is_true(self, ctx: &EvaluationContext) -> bool {
        match self {
            VariableValue::Bool(b) => b,
            _ => self.as_bool(ctx), // peut-être que ce serait mieux de ne pas autoriser à utiliser is_true sur autre chose que des Bool ?
        }
    }

    pub fn cmp(&self, other : &VariableValue, ctx: &EvaluationContext) -> Option<Ordering> {
        match (self, other) {
            (VariableValue::Integer(x), VariableValue::Integer(y)) => x.partial_cmp(y),
            (VariableValue::Float(x), VariableValue::Float(y)) => x.partial_cmp(y),
            (VariableValue::Integer(x), VariableValue::Float(y)) => (*x as f64).partial_cmp(y),
            (VariableValue::Float(x), VariableValue::Integer(y)) => x.partial_cmp(&(*y as f64)),
            (
                VariableValue::Decimal(d1),
                VariableValue::Decimal(d2),
            ) => {
                d1.partial_cmp(d2)
            }
            (VariableValue::Integer(x), VariableValue::Decimal(d)) => {
                Decimal::from(*x).partial_cmp(d)
            }
            (VariableValue::Decimal(d), VariableValue::Integer(y)) => {
                d.partial_cmp(&Decimal::from(*y))
            }
            (VariableValue::Float(x), VariableValue::Decimal(d)) => {
                x.partial_cmp(&f64::from(*d))
            }
            (VariableValue::Decimal(d), VariableValue::Float(y)) => {
                f64::from(*d).partial_cmp(y)
            }

            (VariableValue::Str(x), VariableValue::Str(y)) => x.partial_cmp(y),

            (VariableValue::Vec(x), VariableValue::Vec(y)) => {
                for (x,y) in x.iter().zip(y.iter()) {
                    let comp = x.cmp(y, ctx);
                    if comp.is_none() || comp == Some(Ordering::Equal) {
                        continue;
                    }
                    return comp;
                }
                if x.len() < y.len() {
                    Some(Ordering::Less)
                } else if x.len() > y.len() {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Equal)
                }
            }

            (x, y) => {
                x.yield_integer(ctx).partial_cmp(&y.yield_integer(ctx))
            }
        }
    }

    pub fn lt(&self, other: &VariableValue, ctx: &EvaluationContext) -> VariableValue {
        (self.cmp(&other, ctx) == Some(Ordering::Less)).into()
    }

    pub fn gt(&self, other: &VariableValue, ctx: &EvaluationContext) -> VariableValue {
        (self.cmp(&other, ctx) == Some(Ordering::Greater)).into()
    }

    pub fn leq(&self, other: &VariableValue, ctx: &EvaluationContext) -> VariableValue {
        let cmp = self.cmp(&other, ctx);
        (cmp == Some(Ordering::Less) || cmp == Some(Ordering::Equal)).into()
    }

    pub fn geq(&self, other: &VariableValue, ctx: &EvaluationContext) -> VariableValue {
        let cmp = self.cmp(&other, ctx);
        (cmp == Some(Ordering::Greater) || cmp == Some(Ordering::Equal)).into()
    }

    pub fn eq(&self, other: &VariableValue, ctx: &EvaluationContext) -> VariableValue {
        (self.cmp(other, ctx) == Some(Ordering::Equal)).into()
    }

    pub fn neq(&self, other: &VariableValue, ctx: &EvaluationContext) -> VariableValue {
        self.eq(other, ctx).not(ctx)
    }

    pub fn add(self, other: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Integer(i1 + i2)
            }
            (VariableValue::Float(f1), VariableValue::Float(f2)) => VariableValue::Float(f1 + f2),
            (
                VariableValue::Decimal(x),
                VariableValue::Decimal(y),
            ) => {
                VariableValue::Decimal(x + y)
            }
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => {
                VariableValue::Dur(d1.add(d2, ctx.clock, ctx.frame_len))
            }
            (VariableValue::Map(mut m1), VariableValue::Map(m2)) => {
                let mut res = HashMap::new();
                for (key, value) in m2 {
                    if m1.contains_key(&key) {
                        let x1 = m1.remove(&key).unwrap();
                        res.insert(key, x1.add(value, ctx));
                    }
                }
                VariableValue::Map(res)
            }
            (VariableValue::Vec(v1), VariableValue::Vec(v2)) => {
                let (mut v1, mut v2) = if v1.len() <= v2.len() {
                    (v1,v2)
                } else {
                    (v2,v1)
                };
                for (i, x) in v1.iter_mut().enumerate() {
                    let value = mem::take(x);
                    *x = value.add(mem::take(&mut v2[i]), ctx);
                }
                v1.into()
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "Addition with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.add(y, ctx)
            }
        }
    }

    pub fn div(self, other: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                if i2 != 0 {
                    VariableValue::Integer(i1 / i2)
                } else {
                    VariableValue::Integer(0)
                }
            }
            (VariableValue::Float(f1), VariableValue::Float(f2)) => {
                if f2 != 0.0 {
                    VariableValue::Float(f1 / f2)
                } else {
                    VariableValue::Float(0.0)
                }
            }
            (
                VariableValue::Decimal(x),
                VariableValue::Decimal(y),
            ) => {
                if !y.is_zero() {
                    VariableValue::Decimal(x / y)
                } else {
                    VariableValue::Decimal(Decimal::zero())
                }
            }
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => {
                VariableValue::Dur(d1.div(d2, ctx.clock, ctx.frame_len))
            }
            (VariableValue::Map(mut m1), VariableValue::Map(m2)) => {
                let mut res = HashMap::new();
                for (key, value) in m2 {
                    if m1.contains_key(&key) {
                        let x1 = m1.remove(&key).unwrap();
                        res.insert(key, x1.div(value, ctx));
                    }
                }
                VariableValue::Map(res)
            }
            (VariableValue::Vec(mut v1), VariableValue::Vec(v2)) => {
                if v1.len() > v2.len() {
                    v1.resize(v2.len(), Default::default());
                }
                for (i, y) in v2.into_iter().enumerate() {
                    if v1.len() <= i {
                        break;
                    }
                    let x = mem::take(&mut v1[i]);
                    v1[i] = x.div(y, ctx)
                }
                v1.into()
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "Division with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.div(y, ctx)
            }
        }
    }

    pub fn rem(self, other: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                if i2 != 0 {
                    VariableValue::Integer(i1.rem_euclid(i2))
                } else {
                    VariableValue::Integer(i1)
                }
            }
            (VariableValue::Float(f1), VariableValue::Float(f2)) => {
                if f2 != 0.0 {
                    VariableValue::Float(f1.rem_euclid(f2))
                } else {
                    VariableValue::Float(f1)
                }
            }
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => {
                VariableValue::Dur(d1.rem(d2, ctx.clock, ctx.frame_len))
            }
            (
                VariableValue::Decimal(x),
                VariableValue::Decimal(y),
            ) => {
                VariableValue::Decimal(x % y)
            }
            (VariableValue::Map(mut m1), VariableValue::Map(m2)) => {
                let mut res = HashMap::new();
                for (key, value) in m2 {
                    if m1.contains_key(&key) {
                        let x1 = m1.remove(&key).unwrap();
                        res.insert(key, x1.rem(value, ctx));
                    }
                }
                VariableValue::Map(res)
            }
            (VariableValue::Vec(mut v1), VariableValue::Vec(v2)) => {
                if v1.len() > v2.len() {
                    v1.resize(v2.len(), Default::default());
                }
                for (i, y) in v2.into_iter().enumerate() {
                    if v1.len() <= i {
                        break;
                    }
                    let x = mem::take(&mut v1[i]);
                    v1[i] = x.rem(y, ctx)
                }
                v1.into()
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "Remainder with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.rem(y, ctx)
            }
        }
    }

    pub fn mul(self, other: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Integer(i1 * i2)
            }
            (VariableValue::Float(f1), VariableValue::Float(f2)) => VariableValue::Float(f1 * f2),
            (
                VariableValue::Decimal(x),
                VariableValue::Decimal(y),
            ) => {
                VariableValue::Decimal(x * y)
            }
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => {
                VariableValue::Dur(d1.mul(d2, ctx.clock, ctx.frame_len))
            }
            (VariableValue::Map(mut m1), VariableValue::Map(m2)) => {
                let mut res = HashMap::new();
                for (key, value) in m2 {
                    if m1.contains_key(&key) {
                        let x1 = m1.remove(&key).unwrap();
                        res.insert(key, x1.mul(value, ctx));
                    }
                }
                VariableValue::Map(res)
            }
            (VariableValue::Vec(v1), VariableValue::Vec(v2)) => {
                let (mut v1, mut v2) = if v1.len() <= v2.len() {
                    (v1,v2)
                } else {
                    (v2,v1)
                };
                for (i, x) in v1.iter_mut().enumerate() {
                    let value = mem::take(x);
                    *x = value.mul(mem::take(&mut v2[i]), ctx);
                }
                v1.into()
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "Multiplication with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.mul(y, ctx)
            }
        }
    }

    pub fn sub(self, other: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Integer(i1 - i2)
            }
            (VariableValue::Float(f1), VariableValue::Float(f2)) => VariableValue::Float(f1 - f2),
            (
                VariableValue::Decimal(x),
                VariableValue::Decimal(y),
            ) => {
                VariableValue::Decimal(x - y)
            }
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => {
                VariableValue::Dur(d1.sub(d2, ctx.clock, ctx.frame_len))
            }
            (VariableValue::Map(mut m1), VariableValue::Map(m2)) => {
                let mut res = HashMap::new();
                for (key, value) in m2 {
                    if m1.contains_key(&key) {
                        let x1 = m1.remove(&key).unwrap();
                        res.insert(key, x1.sub(value, ctx));
                    }
                }
                VariableValue::Map(res)
            }
            (VariableValue::Vec(mut v1), VariableValue::Vec(v2)) => {
                if v1.len() > v2.len() {
                    v1.resize(v2.len(), Default::default());
                }
                for (i, y) in v2.into_iter().enumerate() {
                    if v1.len() <= i {
                        break;
                    }
                    let x = mem::take(&mut v1[i]);
                    v1[i] = x.sub(y, ctx)
                }
                v1.into()
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "Subtraction with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.sub(y, ctx)
            }
        }
    }

    pub fn pow(self, other: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        // TODO: Add support for other types !
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Integer(i1.pow(i2 as u32))
            }
            (VariableValue::Float(f1), VariableValue::Float(f2)) => {
                VariableValue::Float(f1.powf(f2))
            }
            (VariableValue::Map(mut m1), VariableValue::Map(m2)) => {
                let mut res = HashMap::new();
                for (key, value) in m2 {
                    if m1.contains_key(&key) {
                        let x1 = m1.remove(&key).unwrap();
                        res.insert(key, x1.sub(value, ctx));
                    }
                }
                VariableValue::Map(res)
            }
            (VariableValue::Vec(mut v1), VariableValue::Vec(v2)) => {
                if v1.len() > v2.len() {
                    v1.resize(v2.len(), Default::default());
                }
                for (i, y) in v2.into_iter().enumerate() {
                    if v1.len() <= i {
                        break;
                    }
                    let x = mem::take(&mut v1[i]);
                    v1[i] = x.pow(y, ctx)
                }
                v1.into()
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "Pow with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.pow(y, ctx)
            }
        }
    }

    pub fn concat(self, other: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, other) {
            (VariableValue::Map(m1), VariableValue::Map(mut m2)) => {
                for (key, value) in m1.into_iter() {
                    m2.insert(key, value);
                }
                VariableValue::Map(m2)
            }
            (VariableValue::Vec(mut v1), VariableValue::Vec(mut v2)) => {
                v1.append(&mut v2);
                VariableValue::Vec(v1)
            }
            (VariableValue::Str(mut s1), VariableValue::Str(s2)) => {
                s1.push_str(s2.as_str());
                VariableValue::Str(s1)
            }
            (VariableValue::Blob(mut v1), VariableValue::Blob(mut v2)) => {
                v1.append(&mut v2);
                VariableValue::Blob(v1)
            }
            (x, y) => {
                let mut v1 = x.as_vec(ctx);
                let mut v2 = y.as_vec(ctx);
                v1.append(&mut v2);
                VariableValue::Vec(v1)
            }
        }
    }

    pub fn and(self, other: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        (self.as_bool(ctx) && other.as_bool(ctx)).into()
    }

    pub fn or(self, other: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        (self.as_bool(ctx) || other.as_bool(ctx)).into()
    }

    pub fn xor(self, other: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        let b1 = self.as_bool(ctx);
        let b2 = other.as_bool(ctx);
        ((b1 && !b2) || (!b1 && b2)).into()
    }

    pub fn bitand(self, rhs: Self, ctx: &EvaluationContext) -> VariableValue {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Integer(i1 & i2)
            }
            (VariableValue::Map(mut m1), VariableValue::Map(m2)) => {
                let keys1 : HashSet<String> = m1.keys().cloned().collect();
                let keys2 : HashSet<String> = m2.keys().cloned().collect();
                let to_remove = keys1.symmetric_difference(&keys2);
                for key in to_remove {
                    let _ = m1.remove(key);
                }
                VariableValue::Map(m1)
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "BitAnd with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.bitand(y, ctx)
            }
        }
    }

    pub fn bitor(self, rhs: Self, ctx: &EvaluationContext) -> VariableValue {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Integer(i1 | i2)
            }
            (VariableValue::Map(mut m1), VariableValue::Map(m2)) => {
                for (key, value) in m2 {
                    if m1.contains_key(&key) {
                        continue;
                    }
                    m1.insert(key, value);
                }
                VariableValue::Map(m1)
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "BitOr with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.bitor(y, ctx)
            }
        }
    }

    pub fn bitxor(self, rhs: Self, ctx: &EvaluationContext) -> VariableValue {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Integer(i1 ^ i2)
            }
            (VariableValue::Map(mut m1), VariableValue::Map(mut m2)) => {
                let keys1 : HashSet<String> = m1.keys().cloned().collect();
                let keys2 : HashSet<String> = m2.keys().cloned().collect();
                let to_keep : HashSet<String> = keys1.symmetric_difference(&keys2).cloned().collect();
                let mut res = HashMap::new();
                for key in keys1 {
                    if to_keep.contains(&key) {
                        let x = m1.remove(&key).unwrap();
                        res.insert(key, x);
                    }
                }
                for key in keys2 {
                    if to_keep.contains(&key) {
                        let x = m2.remove(&key).unwrap();
                        res.insert(key, x);
                    }
                }
                VariableValue::Map(res)
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "BitXor with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.bitxor(y, ctx)
            }
        }
    }

    pub fn shr(self, rhs: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                if i2 < 0 {
                    VariableValue::Integer(i1)
                } else {
                    VariableValue::Integer(i1 >> i2)
                }
            }
            (VariableValue::Vec(mut v), VariableValue::Integer(i)) => {
                v.rotate_right(i as usize);
                VariableValue::Vec(v)
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "ShiftRightL with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.shr(y, ctx)
            }
        }
    }

    pub fn shl(self, rhs: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                if i2 < 0 {
                    VariableValue::Integer(i1)
                } else {
                    VariableValue::Integer(i1 << i2)
                }
            }
            (VariableValue::Vec(mut v), VariableValue::Integer(i)) => {
                v.rotate_left(i as usize);
                VariableValue::Vec(v)
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "ShiftLeftL with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.shl(y, ctx)
            }
        }
    }

    pub fn arithmetic_shr(self, rhs: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                if i2 < 0 {
                    VariableValue::Integer(i1)
                } else {
                    VariableValue::Integer(i1 >> i2)
                }
            }
            (VariableValue::Vec(mut v), VariableValue::Integer(i)) => {
                v.rotate_right(i as usize);
                VariableValue::Vec(v)
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "ShiftRightA with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.arithmetic_shr(y, ctx)
            }
        }
    }

    pub fn arithmetic_shl(self, rhs: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                if i2 < 0 {
                    VariableValue::Integer(i1)
                } else {
                    VariableValue::Integer(i1 << i2)
                }
            }
            (VariableValue::Vec(mut v), VariableValue::Integer(i)) => {
                v.rotate_left(i as usize);
                VariableValue::Vec(v)
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "ShiftLeftA with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.arithmetic_shl(y, ctx)
            }
        }
    }

    pub fn circular_shr(self, rhs: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                if i2 < 0 {
                    VariableValue::Integer(i1)
                } else {
                    VariableValue::Integer(i1.rotate_right(i2 as u32))
                }
            }
            (VariableValue::Vec(mut v), VariableValue::Integer(i)) => {
                v.rotate_left(i as usize);
                VariableValue::Vec(v)
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "Circular ShiftRight with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.circular_shr(y, ctx)
            }
        }
    }

    pub fn circular_shl(self, rhs: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, rhs) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                if i2 < 0 {
                    VariableValue::Integer(i1)
                } else {
                    VariableValue::Integer(i1.rotate_left(i2 as u32))
                }
            }
            (VariableValue::Vec(mut v), VariableValue::Integer(i)) => {
                v.rotate_left(i as usize);
                VariableValue::Vec(v)
            }
            (mut x, mut y) => {
                ctx.errors.throw(
                    SovaError::from(ctx).message(format!(
                        "Circular ShiftLeft with wrong types : {x:?}/{y:?}"
                    ))
                );
                x.cast_as_integer(ctx);
                y.cast_as_integer(ctx);
                x.circular_shl(y, ctx)
            }
        }
    }

    pub fn not(self, ctx: &EvaluationContext) -> VariableValue {
        match self {
            VariableValue::Integer(i) => VariableValue::Integer(!i),
            VariableValue::Bool(b) => VariableValue::Bool(!b),
            VariableValue::Decimal(decimal) => todo!(),
            VariableValue::Func(instructions) => todo!(),
            VariableValue::Blob(mut items) => {
                VariableValue::Blob(items.iter_mut().map(|x| !*x).collect())
            }
            VariableValue::Generator(g) => g.get_current(ctx).not(ctx),
            VariableValue::Float(f) => {
                if f == 0.0 {
                    1.0
                } else {
                    0.0
                }.into()
            }
            VariableValue::Str(s) => todo!(),
            VariableValue::Dur(time_span) => todo!(),
            VariableValue::Map(hash_map) => todo!(),
            VariableValue::Vec(variable_values) => todo!(),
        }
    }

    pub fn neg(self, ctx: &EvaluationContext) -> VariableValue {
        match self {
            VariableValue::Integer(i) => VariableValue::Integer(-i),
            VariableValue::Float(f) => VariableValue::Float(-f),
            VariableValue::Decimal(d) => VariableValue::Decimal(-d),
            VariableValue::Bool(b) => {
                if b {
                    VariableValue::Integer(-1)
                } else {
                    VariableValue::Bool(false)
                }
            }
            VariableValue::Str(s) => VariableValue::Str(s.chars().rev().collect()),
            VariableValue::Vec(mut v) => {
                for x in v.iter_mut() {
                    let value = mem::take(x);
                    *x = value.neg(ctx);
                }
                v.into()
            }
            VariableValue::Map(mut m) => {
                for x in m.values_mut() {
                    let value = mem::take(x);
                    *x = value.neg(ctx);
                }
                m.into()
            }
            VariableValue::Func(p) => (p.len() as i64).neg().into(),
            VariableValue::Blob(items) => todo!(),
            VariableValue::Generator(g) => g.get_current(ctx).neg(ctx),
            VariableValue::Dur(time_span) => todo!(),
        }
    }

    pub fn cast_as_integer(&mut self, ctx: &EvaluationContext) {
        let value = mem::take(self);
        *self = VariableValue::Integer(value.as_integer(ctx))
    }

    pub fn cast_as_float(&mut self, ctx: &EvaluationContext) {
        let value = mem::take(self);
        *self = VariableValue::Float(value.as_float(ctx))
    }

    pub fn cast_as_decimal(&mut self, ctx: &EvaluationContext) {
        let value = mem::take(self);
        let d = value.as_decimal(ctx);
        *self = VariableValue::Decimal(d)
    }

    pub fn cast_as_bool(&mut self, ctx: &EvaluationContext) {
        let value = mem::take(self);
        *self = VariableValue::Bool(value.as_bool(ctx))
    }

    pub fn cast_as_str(&mut self, ctx: &EvaluationContext) {
        let value = mem::take(self);
        *self = VariableValue::Str(value.as_str(ctx))
    }

    pub fn cast_as_dur(&mut self, ctx: &EvaluationContext) {
        let value = mem::take(self);
        *self = VariableValue::Dur(value.as_dur(ctx))
    }

    pub fn cast_as_map(&mut self, ctx: &EvaluationContext) {
        let value = mem::take(self);
        *self = VariableValue::Map(value.as_map(ctx))
    }

    pub fn cast_as_vec(&mut self, ctx: &EvaluationContext) {
        let value = mem::take(self);
        *self = VariableValue::Vec(value.as_vec(ctx))
    }

    pub fn cast_as_blob(&mut self, ctx: &EvaluationContext) {
        let value = mem::take(self);
        *self = VariableValue::Blob(value.as_blob(ctx))
    }

    pub fn as_integer(self, ctx: &EvaluationContext) -> i64 {
        self.yield_integer(ctx)
    }

    pub fn as_float(self, ctx: &EvaluationContext) -> f64 {
        self.yield_float(ctx)
    }

    pub fn as_decimal(self, ctx: &EvaluationContext) -> Decimal {
        self.yield_decimal(ctx)
    }

    pub fn as_bool(self, ctx: &EvaluationContext) -> bool {
        self.yield_bool(ctx)
    }

    pub fn as_str(self, ctx: &EvaluationContext) -> String {
        match self {
            VariableValue::Integer(i) => i.to_string(),
            VariableValue::Float(f) => f.to_string(),
            VariableValue::Decimal(d) => f64::from(d).to_string(),
            VariableValue::Bool(b) => b.to_string(),
            VariableValue::Str(s) => s.to_string(),
            VariableValue::Dur(d) => d.as_micros(ctx.clock, ctx.frame_len).to_string(),
            VariableValue::Func(f) => serde_json::to_string(&f).unwrap_or_default(),
            VariableValue::Map(m) => serde_json::to_string(&m).unwrap_or_default(),
            VariableValue::Vec(v) => serde_json::to_string(&v).unwrap_or_default(),
            VariableValue::Blob(b) => String::from_utf8(b).unwrap_or_default(),
            VariableValue::Generator(g) => g.get_current(ctx).as_str(ctx)
        }
    }

    pub fn as_dur(self, ctx: &EvaluationContext) -> TimeSpan {
        self.yield_dur(ctx)
    }

    pub fn as_map(self, ctx: &EvaluationContext) -> HashMap<String, VariableValue> {
        match self {
            VariableValue::Map(map) => map,
            VariableValue::Generator(g) => g.get_current(ctx).as_map(ctx),
            VariableValue::Str(x) => {
                let mut map = HashMap::new();
                map.insert("sound".to_owned(), x.into());
                map
            }
            VariableValue::Integer(x) => {
                let mut map = HashMap::new();
                map.insert("i".to_owned(), x.into());
                map
            }
            VariableValue::Float(x) => {
                let mut map = HashMap::new();
                map.insert("freq".to_owned(), x.into());
                map
            }
            VariableValue::Bool(x) => {
                let mut map = HashMap::new();
                if x {
                    map.insert("i".to_owned(), 0.into());
                }
                map
            }
            VariableValue::Dur(x) => {
                let mut map = HashMap::new();
                map.insert("duration".to_owned(), x.into());
                map
            }
            VariableValue::Decimal(d) => {
                let mut map = HashMap::new();
                map.insert("freq".to_owned(), VariableValue::Decimal(d));
                map
            }
            VariableValue::Vec(v) => {
                let mut i = v.into_iter();
                let mut res = HashMap::new();
                while let (Some(key), Some(value)) = (i.next(), i.next()) {
                    res.insert(key.as_str(ctx), value);
                }
                res
            }
            VariableValue::Blob(b) => {
                let mut map = HashMap::new();
                map.insert("data".to_owned(), b.into());
                map
            }
            VariableValue::Func(_) => HashMap::new(),
        }
    }

    pub fn as_blob(self, ctx: &EvaluationContext) -> Vec<u8> {
        match self {
            VariableValue::Integer(i) => Vec::from(i.to_le_bytes()),
            VariableValue::Float(f) => Vec::from(f.to_le_bytes()),
            VariableValue::Decimal(d) => Vec::from(f64::from(d).to_le_bytes()),
            VariableValue::Bool(b) => {
                if b {
                    vec![1]
                } else {
                    Vec::new()
                }
            }
            VariableValue::Str(s) => Vec::from(s.as_bytes()),
            VariableValue::Dur(d) => Vec::from(d.as_beats(ctx.clock, ctx.frame_len).to_le_bytes()),
            VariableValue::Func(_) => Vec::new(),
            VariableValue::Map(_) => Vec::new(),
            VariableValue::Vec(v) => v.into_iter().map(|x| VariableValue::as_blob(x, ctx)).flatten().collect(),
            VariableValue::Blob(b) => b,
            VariableValue::Generator(g) => g.get_current(ctx).as_blob(ctx)
        }
    }

    pub fn as_vec(self, ctx: &EvaluationContext) -> Vec<VariableValue> {
        match self {
            VariableValue::Map(m) => {
                let mut res = Vec::new();
                for (key, value) in m.into_iter() {
                    res.push(VariableValue::Str(key));
                    res.push(value);
                }
                res
            }
            VariableValue::Bool(b) => { 
                if b {
                    vec![ 1.into() ]
                } else {
                    Vec::new()
                }
            }
            VariableValue::Generator(g) => g.get_current(ctx).as_vec(ctx),
            VariableValue::Vec(v) => v,
            item => vec![item],
        }
    }

    pub fn yield_integer(&self, ctx: &EvaluationContext) -> i64 {
        match self {
            VariableValue::Integer(i) => *i,
            VariableValue::Float(f) => f.round() as i64,
            VariableValue::Decimal(d) => (*d).into(),
            VariableValue::Bool(b) => *b as i64,
            VariableValue::Str(s) => s.parse::<i64>().unwrap_or(0),
            VariableValue::Dur(d) => d.as_beats(ctx.clock, ctx.frame_len).round() as i64,
            VariableValue::Func(p) => p.len() as i64,
            VariableValue::Map(m) => m.len() as i64,
            VariableValue::Vec(v) => v.len() as i64,
            VariableValue::Blob(b) => {
                let mut arr = [0u8; 8];
                for i in 0..std::cmp::min(b.len(), 8) {
                    arr[i] = b[i];
                }
                i64::from_le_bytes(arr)
            }
            VariableValue::Generator(g) => g.get_current(ctx).as_integer(ctx)
        }
    }

    pub fn yield_float(&self, ctx: &EvaluationContext) -> f64 {
        match self {
            VariableValue::Integer(i) => *i as f64,
            VariableValue::Float(f) => *f,
            VariableValue::Decimal(d) => (*d).into(),
            VariableValue::Bool(b) => *b as i8 as f64,
            VariableValue::Str(s) => s.parse::<f64>().unwrap_or(0.0),
            VariableValue::Dur(d) => d.as_beats(ctx.clock, ctx.frame_len),
            VariableValue::Func(p) => p.len() as f64,
            VariableValue::Map(m) => m.len() as f64, 
            VariableValue::Vec(v) => v.len() as f64,
            VariableValue::Blob(b) => {
                let mut arr = [0u8; 8];
                for i in 0..std::cmp::min(b.len(), 8) {
                    arr[i] = b[i];
                }
                f64::from_le_bytes(arr)
            }
            VariableValue::Generator(g) => g.get_current(ctx).as_float(ctx)
        }
    }

    pub fn yield_decimal(&self, ctx: &EvaluationContext) -> Decimal {
        match self {
            VariableValue::Integer(i) => Decimal::from(*i),
            VariableValue::Float(f) => Decimal::from(*f),
            VariableValue::Decimal(d) => *d,
            VariableValue::Bool(b) => {
                if *b {
                    Decimal::one()
                } else {
                    Decimal::zero()
                }
            }
            VariableValue::Str(s) => match s.parse::<f64>() {
                Ok(n) => Decimal::from(n),
                Err(_) => Decimal::zero(),
            },
            VariableValue::Dur(d) => Decimal::from(d.as_beats(ctx.clock, ctx.frame_len)),
            VariableValue::Func(p) => Decimal::from(p.len() as u64),
            VariableValue::Map(m) => Decimal::from(m.len() as u64),
            VariableValue::Vec(v) => Decimal::from(v.len() as u64),
            VariableValue::Generator(g) => g.get_current(ctx).as_decimal(ctx),
            x if matches!(x, VariableValue::Blob(_)) => Decimal::from(x.yield_float(ctx)),
            VariableValue::Blob(_) => unreachable!()
        }
    }

    pub fn yield_bool(&self, ctx: &EvaluationContext) -> bool {
        match self {
            VariableValue::Integer(i) => *i != 0,
            VariableValue::Float(f) => *f != 0.0,
            VariableValue::Decimal(d) => !d.is_zero(),
            VariableValue::Bool(b) => *b,
            VariableValue::Str(s) => !s.is_empty(),
            VariableValue::Dur(d) => d.as_micros(ctx.clock, ctx.frame_len) != 0,
            VariableValue::Func(p) => !p.is_empty(),
            VariableValue::Map(map) => !map.is_empty(),
            VariableValue::Vec(vec) => !vec.is_empty(),
            VariableValue::Blob(b) => b.iter().any(|byte| *byte > 0),
            VariableValue::Generator(g) => g.get_current(ctx).as_bool(ctx)
        }
    }

    pub fn yield_dur(&self, ctx: &EvaluationContext) -> TimeSpan {
        match self {
            VariableValue::Integer(i) => TimeSpan::Beats(*i as f64),
            VariableValue::Float(f) => TimeSpan::Beats(*f),
            VariableValue::Decimal(d) => TimeSpan::Beats(f64::from(*d)),
            VariableValue::Bool(b) => TimeSpan::Beats(*b as i8 as f64),
            VariableValue::Str(s) => if let Ok(i) = s.parse::<SyncTime>() {
                TimeSpan::Micros(i)
            } else if let Ok(f) = s.parse::<f64>() {
                TimeSpan::Beats(f)
            } else {
                TimeSpan::Micros(0)
            }
            VariableValue::Dur(d) => *d,
            VariableValue::Func(p) => TimeSpan::Beats(p.len() as f64),
            VariableValue::Map(m) => TimeSpan::Beats(m.len() as f64),
            VariableValue::Vec(v) => TimeSpan::Beats(v.len() as f64),
            VariableValue::Generator(g) => g.get_current(ctx).as_dur(ctx),
            x if x.is_blob() => TimeSpan::Beats(x.yield_float(ctx)),
            VariableValue::Blob(_) => unreachable!()
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(self, VariableValue::Integer(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, VariableValue::Float(_))
    }

    pub fn is_decimal(&self) -> bool {
        matches!(self, VariableValue::Decimal(_))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, VariableValue::Bool(_))
    }

    pub fn is_str(&self) -> bool {
        matches!(self, VariableValue::Str(_))
    }

    pub fn is_dur(&self) -> bool {
        matches!(self, VariableValue::Dur(_))
    }

    pub fn is_func(&self) -> bool {
        matches!(self, VariableValue::Func(_))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, VariableValue::Map(_))
    }

    pub fn is_vec(&self) -> bool {
        matches!(self, VariableValue::Vec(_))
    }

    pub fn is_blob(&self) -> bool {
        matches!(self, VariableValue::Blob(_))
    }

    pub fn is_generator(&self) -> bool {
        matches!(self, VariableValue::Generator(_))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Variable {
    Environment(EnvironmentFunc),
    Global(String),
    Line(String),
    Frame(String),
    Instance(String),
    Constant(VariableValue),
    StackBack,
    StackFront
}

impl Default for Variable {
    fn default() -> Self {
        VariableValue::default().into()
    }
}

impl Variable {
    pub fn is_mutable(&self) -> bool {
        match self {
            Variable::Constant(_) | Variable::Environment(_) => false,
            _ => true,
        }
    }

    /// Simple way to access register variables : instance variables with integer names.
    pub fn reg(n: usize) -> Self {
        Variable::Instance(n.to_string())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct VariableStore {
    content: HashMap<String, VariableValue>,
    delta: Vec<String>,
    watchers: Vec<usize>,
}

impl VariableStore {
    pub fn new() -> VariableStore {
        Default::default()
    }

    pub fn insert(&mut self, key: String, value: VariableValue) -> Option<VariableValue> {
        if self.watchers.len() > 0 {
            self.delta.push(key.clone());
        }
        self.content.insert(key, value)
    }

    pub fn get(&self, key: &str) -> Option<&VariableValue> {
        self.content.get(key)
    }

    pub fn has(&self, key: &str) -> bool {
        self.content.contains_key(key)
    }

    pub fn get_create(&mut self, key: &str, default: VariableValue) -> &VariableValue {
        if !self.content.contains_key(key) {
            self.content
                .insert(key.to_owned(), default);
        }
        self.content.get(key).unwrap()
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut VariableValue> {
        self.content.get_mut(key)
    }

    pub fn get_mut_create(&mut self, key: &str, default: VariableValue) -> &mut VariableValue {
        if !self.content.contains_key(key) {
            self.content
                .insert(key.to_owned(), default);
        }
        self.content.get_mut(key).unwrap()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &VariableValue)> {
        self.content.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut VariableValue)> {
        self.content.iter_mut()
    }

    pub fn one_letter_vars(&self) -> impl Iterator<Item = (&String, &VariableValue)> {
        self.iter().filter(|(k, _)| k.len() == 1)
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.reset_changes();
    }

    pub fn watch(&mut self) -> usize {
        let new_id = self.watchers.len();
        self.watchers.push(self.delta.len());
        new_id
    }

    pub fn reset_changes(&mut self) {
        self.delta.clear();
        for i in self.watchers.iter_mut() {
            *i = 0;
        }
    }

    pub fn changes(&mut self, watcher: usize) -> impl Iterator<Item = (&String, &VariableValue)> {
        let start = self.watchers[watcher];
        self.watchers[watcher] = self.delta.len();
        self.delta[start..].iter().map(|s| (s, &self.content[s]))
    }

    pub fn clean_changes(&mut self) {
        let min = self
            .watchers
            .iter()
            .min()
            .map(|m| *m)
            .unwrap_or(self.delta.len());
        self.delta.drain(0..min);
        for i in self.watchers.iter_mut() {
            *i -= min;
        }
    }

    pub fn has_changed(&self, watcher: usize) -> bool {
        if watcher >= self.watchers.len() {
            return false;
        }
        self.watchers[watcher] < self.delta.len()
    }

    pub fn apply_changes<I>(&mut self, watcher: usize, changes: I)
    where
        I: Iterator<Item = (String, VariableValue)>,
    {
        let mut changed = 0;
        for (name, value) in changes {
            self.insert(name, value);
            changed += 1;
        }
        if watcher < self.watchers.len() {
            self.watchers[watcher] += changed;
        }
    }
}

impl From<HashMap<String, VariableValue>> for VariableStore {
    fn from(content: HashMap<String, VariableValue>) -> Self {
        VariableStore {
            content,
            ..Default::default()
        }
    }
}

impl<'a> FromIterator<(&'a String, &'a VariableValue)> for VariableStore {
    fn from_iter<T: IntoIterator<Item = (&'a String, &'a VariableValue)>>(iter: T) -> Self {
        VariableStore {
            content: iter
                .into_iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            ..Default::default()
        }
    }
}

impl From<VariableStore> for HashMap<String, VariableValue> {
    fn from(value: VariableStore) -> Self {
        value.content
    }
}

impl From<EnvironmentFunc> for Variable {
    fn from(value: EnvironmentFunc) -> Self {
        Variable::Environment(value)
    }
}

impl<T : Into<VariableValue>> From<T> for Variable {
    fn from(value: T) -> Self {
        Variable::Constant(value.into())
    }
}
