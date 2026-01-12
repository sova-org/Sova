use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    ops::{BitAnd, BitOr, BitXor, Neg, Not, Shl, Shr},
};

use serde::{Deserialize, Serialize};

use crate::{
    clock::{Clock, SyncTime, TimeSpan},
    vm::Program,
};

use crate::util::decimal_operations::{
    add_decimal, decimal_from_float64, div_decimal, eq_decimal, float64_from_decimal, leq_decimal,
    lt_decimal, mul_decimal, neq_decimal, rem_decimal, string_from_decimal, sub_decimal,
};

use super::{environment_func::EnvironmentFunc, EvaluationContext};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VariableValue {
    Decimal(i8, u64, u64), // sign, numerator, denominator
    Func(Program),
    Blob(Vec<u8>),
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

impl BitAnd for VariableValue {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
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
            _ => panic!("Bitwise and with wrong types, this should never happen"),
        }
    }
}

impl BitOr for VariableValue {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
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
            _ => panic!("Bitwise or with wrong types, this should never happen"),
        }
    }
}

impl BitXor for VariableValue {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
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
            }
            (VariableValue::Vec(mut v), VariableValue::Integer(i)) => {
                v.rotate_left(i as usize);
                VariableValue::Vec(v)
            }
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
            (VariableValue::Vec(mut v), VariableValue::Integer(i)) => {
                v.rotate_right(i as usize);
                VariableValue::Vec(v)
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

impl Neg for VariableValue {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            VariableValue::Integer(i) => VariableValue::Integer(-i),
            VariableValue::Float(f) => VariableValue::Float(-f),
            VariableValue::Decimal(s, p, q) => VariableValue::Decimal(-s, p, q),
            VariableValue::Bool(b) => {
                if b {
                    VariableValue::Integer(-1)
                } else {
                    VariableValue::Bool(false)
                }
            }
            VariableValue::Str(s) => VariableValue::Str(s.chars().rev().collect()),
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

            (
                VariableValue::Decimal(x_sign, x_num, x_den),
                VariableValue::Decimal(y_sign, y_num, y_den),
            ) => {
                if *x_sign < 0 && *y_sign >= 0 {
                    return Some(Ordering::Less);
                }

                if *x_sign >= 0 && *y_sign < 0 {
                    return Some(Ordering::Greater);
                }

                let x_for_cmp = *x_num * *y_den;
                let y_for_cmp = *y_num * *x_den;

                // both positive
                if *x_sign >= 0 {
                    if x_for_cmp < y_for_cmp {
                        return Some(Ordering::Less);
                    }

                    if x_for_cmp > y_for_cmp {
                        return Some(Ordering::Greater);
                    }

                    return Some(Ordering::Equal);
                }

                // both negative
                if x_for_cmp < y_for_cmp {
                    return Some(Ordering::Greater);
                }

                if x_for_cmp > y_for_cmp {
                    return Some(Ordering::Less);
                }

                Some(Ordering::Equal)
            }
            (VariableValue::Integer(x), VariableValue::Decimal(_, _, _)) => {
                let x_sign = if *x < 0 { -1 } else { 1 };
                let x_num = if *x < 0 { (-*x) as u64 } else { *x as u64 };
                let x_den = 1;
                VariableValue::Decimal(x_sign, x_num, x_den).partial_cmp(other)
            }
            (VariableValue::Decimal(_, _, _), VariableValue::Integer(y)) => {
                let y_sign = if *y < 0 { -1 } else { 1 };
                let y_num = if *y < 0 { (-*y) as u64 } else { *y as u64 };
                let y_den = 1;
                self.partial_cmp(&VariableValue::Decimal(y_sign, y_num, y_den))
            }
            (VariableValue::Float(x), VariableValue::Decimal(y_sign, y_num, y_den)) => {
                let mut y = (*y_num as f64) / (*y_den as f64);
                if *y_sign < 0 {
                    y = -y;
                }
                x.partial_cmp(&y)
            }
            (VariableValue::Decimal(x_sign, x_num, x_den), VariableValue::Float(y)) => {
                let mut x = (*x_num as f64) / (*x_den as f64);
                if *x_sign < 0 {
                    x = -x;
                }
                x.partial_cmp(y)
            }

            (VariableValue::Bool(x), VariableValue::Bool(y)) => x.partial_cmp(y),
            (VariableValue::Bool(x), VariableValue::Integer(y)) => (*x as i64).partial_cmp(y),
            (VariableValue::Integer(x), VariableValue::Bool(y)) => x.partial_cmp(&(*y as i64)),

            (VariableValue::Str(x), VariableValue::Str(y)) => x.partial_cmp(y),
            _ => None,
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
impl From<Program> for VariableValue {
    fn from(value: Program) -> Self {
        VariableValue::Func(value)
    }
}

impl VariableValue {
    pub fn clone_type(&self) -> VariableValue {
        match self {
            VariableValue::Integer(_) => Self::Integer(0),
            VariableValue::Float(_) => Self::Float(0.0),
            VariableValue::Decimal(_, _, _) => Self::Decimal(1, 0, 1),
            VariableValue::Bool(_) => Self::Bool(false),
            VariableValue::Str(_) => Self::Str("".to_owned()),
            VariableValue::Dur(_) => Self::Dur(TimeSpan::Micros(0)),
            VariableValue::Func(_) => todo!(),
            VariableValue::Map(_) => Self::Map(HashMap::new()),
            VariableValue::Vec(_) => Self::Vec(Vec::new()),
            VariableValue::Blob(_) => Self::Blob(Vec::new()),
        }
    }

    pub fn compatible_cast(&mut self, other: &mut VariableValue, ctx: &EvaluationContext) {
        // cast to correct types
        match self {
            VariableValue::Integer(_) => {
                *other = other.cast_as_integer(ctx.clock, ctx.frame_len);
            }
            VariableValue::Float(_) => {
                *other = other.cast_as_float(ctx.clock, ctx.frame_len);
            }
            VariableValue::Decimal(_, _, _) => {
                *other = other.cast_as_decimal(ctx.clock, ctx.frame_len);
            }
            VariableValue::Dur(_) => {
                *other = other.cast_as_dur();
            }
            VariableValue::Map(_) => {
                *other = other.cast_as_map();
            }
            _ => match other {
                VariableValue::Integer(_) => {
                    *self = self.cast_as_integer(ctx.clock, ctx.frame_len);
                }
                VariableValue::Float(_) => {
                    *self = self.cast_as_float(ctx.clock, ctx.frame_len);
                }
                VariableValue::Decimal(_, _, _) => {
                    *self = self.cast_as_decimal(ctx.clock, ctx.frame_len);
                }
                VariableValue::Dur(_) => {
                    *self = self.cast_as_dur();
                }
                _ => {
                    *self = self.cast_as_integer(ctx.clock, ctx.frame_len);
                    *other = self.cast_as_integer(ctx.clock, ctx.frame_len);
                }
            },
        }
    }

    pub fn is_true(self, ctx: &EvaluationContext) -> bool {
        match self {
            VariableValue::Bool(b) => b,
            _ => self.cast_as_bool(ctx.clock, ctx.frame_len).is_true(ctx), // peut-être que ce serait mieux de ne pas autoriser à utiliser is_true sur autre chose que des Bool ?
        }
    }

    pub fn lt(self, other: VariableValue) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Bool(i1 < i2)
            }
            (VariableValue::Float(f1), VariableValue::Float(f2)) => VariableValue::Bool(f1 < f2),
            (
                VariableValue::Decimal(x_sign, x_num, x_den),
                VariableValue::Decimal(y_sign, y_num, y_den),
            ) => VariableValue::Bool(lt_decimal(x_sign, x_num, x_den, y_sign, y_num, y_den)),
            (VariableValue::Dur(_d1), VariableValue::Dur(_d2)) => todo!(),
            _ => panic!("Comparison (lt or gt) with wrong types, this should never happen"),
        }
    }

    pub fn gt(self, other: VariableValue) -> VariableValue {
        other.lt(self)
    }

    pub fn leq(self, other: VariableValue) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Bool(i1 <= i2)
            }
            (
                VariableValue::Decimal(x_sign, x_num, x_den),
                VariableValue::Decimal(y_sign, y_num, y_den),
            ) => VariableValue::Bool(leq_decimal(x_sign, x_num, x_den, y_sign, y_num, y_den)),
            (VariableValue::Float(f1), VariableValue::Float(f2)) => VariableValue::Bool(f1 <= f2),
            (VariableValue::Dur(_d1), VariableValue::Dur(_d2)) => todo!(),
            _ => panic!("Comparison (leq or geq) with wrong types, this should never happen"),
        }
    }

    pub fn geq(self, other: VariableValue) -> VariableValue {
        other.leq(self)
    }

    pub fn eq(self, other: VariableValue) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Bool(i1 == i2)
            }
            (
                VariableValue::Decimal(x_sign, x_num, x_den),
                VariableValue::Decimal(y_sign, y_num, y_den),
            ) => VariableValue::Bool(eq_decimal(x_sign, x_num, x_den, y_sign, y_num, y_den)),
            (VariableValue::Float(f1), VariableValue::Float(f2)) => VariableValue::Bool(f1 == f2),
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => VariableValue::Bool(d1 == d2),
            (VariableValue::Map(m1), VariableValue::Map(m2)) => VariableValue::Bool(m1 == m2),
            (VariableValue::Vec(v1), VariableValue::Vec(v2)) => VariableValue::Bool(v1 == v2),
            _ => panic!("Comparison (eq) with wrong types, this should never happen"),
        }
    }

    pub fn neq(self, other: VariableValue) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Bool(i1 != i2)
            }
            (
                VariableValue::Decimal(x_sign, x_num, x_den),
                VariableValue::Decimal(y_sign, y_num, y_den),
            ) => VariableValue::Bool(neq_decimal(x_sign, x_num, x_den, y_sign, y_num, y_den)),
            (VariableValue::Float(f1), VariableValue::Float(f2)) => VariableValue::Bool(f1 != f2),
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => VariableValue::Bool(d1 != d2),
            (VariableValue::Map(m1), VariableValue::Map(m2)) => VariableValue::Bool(m1 != m2),
            (VariableValue::Vec(v1), VariableValue::Vec(v2)) => VariableValue::Bool(v1 != v2),
            _ => panic!("Comparison (neq) with wrong types, this should never happen"),
        }
    }

    pub fn add(self, other: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Integer(i1 + i2)
            }
            (VariableValue::Float(f1), VariableValue::Float(f2)) => VariableValue::Float(f1 + f2),
            (
                VariableValue::Decimal(x_sign, x_num, x_den),
                VariableValue::Decimal(y_sign, y_num, y_den),
            ) => {
                let (z_sign, z_num, z_den) =
                    add_decimal(x_sign, x_num, x_den, y_sign, y_num, y_den);
                VariableValue::Decimal(z_sign, z_num, z_den)
            }
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => {
                VariableValue::Dur(d1.add(d2, ctx.clock, ctx.frame_len))
            }
            (VariableValue::Map(mut m1), VariableValue::Map(m2)) => {
                for (key, value) in m2 {
                    if !m1.contains_key(&key) {
                        m1.insert(key, value);
                    } else {
                        let x1 = m1.get(&key).cloned().unwrap();
                        m1.insert(key, x1.add(value, ctx));
                    }
                }
                VariableValue::Map(m1)
            }
            _ => panic!("Addition with wrong types, this should never happen"),
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
                VariableValue::Decimal(x_sign, x_num, x_den),
                VariableValue::Decimal(y_sign, y_num, y_den),
            ) => {
                let (z_sign, z_num, z_den) =
                    div_decimal(x_sign, x_num, x_den, y_sign, y_num, y_den);
                VariableValue::Decimal(z_sign, z_num, z_den)
            }
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => {
                VariableValue::Dur(d1.div(d2, ctx.clock, ctx.frame_len))
            }
            (VariableValue::Map(mut m1), VariableValue::Map(m2)) => {
                for (key, value) in m2 {
                    if !m1.contains_key(&key) {
                        m1.insert(key, value);
                    } else {
                        let x1 = m1.get(&key).cloned().unwrap();
                        m1.insert(key, x1.div(value, ctx));
                    }
                }
                VariableValue::Map(m1)
            }
            _ => panic!("Division with wrong types, this should never happen"),
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
                VariableValue::Decimal(x_sign, x_num, x_den),
                VariableValue::Decimal(y_sign, y_num, y_den),
            ) => {
                let (z_sign, z_num, z_den) =
                    rem_decimal(x_sign, x_num, x_den, y_sign, y_num, y_den);
                VariableValue::Decimal(z_sign, z_num, z_den)
            }
            (VariableValue::Map(mut m1), VariableValue::Map(m2)) => {
                for (key, value) in m2 {
                    if !m1.contains_key(&key) {
                        m1.insert(key, value);
                    } else {
                        let x1 = m1.get(&key).cloned().unwrap();
                        m1.insert(key, x1.rem(value, ctx));
                    }
                }
                VariableValue::Map(m1)
            }
            _ => panic!("Reminder (modulo) with wrong types, this should never happen"),
        }
    }

    pub fn mul(self, other: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Integer(i1 * i2)
            }
            (VariableValue::Float(f1), VariableValue::Float(f2)) => VariableValue::Float(f1 * f2),
            (
                VariableValue::Decimal(x_sign, x_num, x_den),
                VariableValue::Decimal(y_sign, y_num, y_den),
            ) => {
                let (z_sign, z_num, z_den) =
                    mul_decimal(x_sign, x_num, x_den, y_sign, y_num, y_den);
                VariableValue::Decimal(z_sign, z_num, z_den)
            }
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => {
                VariableValue::Dur(d1.mul(d2, ctx.clock, ctx.frame_len))
            }
            (VariableValue::Map(mut m1), VariableValue::Map(m2)) => {
                for (key, value) in m2 {
                    if !m1.contains_key(&key) {
                        m1.insert(key, value);
                    } else {
                        let x1 = m1.get(&key).cloned().unwrap();
                        m1.insert(key, x1.mul(value, ctx));
                    }
                }
                VariableValue::Map(m1)
            }
            _ => panic!("Multiplication with wrong types, this should never happen"),
        }
    }

    pub fn sub(self, other: VariableValue, ctx: &EvaluationContext) -> VariableValue {
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Integer(i1 - i2)
            }
            (VariableValue::Float(f1), VariableValue::Float(f2)) => VariableValue::Float(f1 - f2),
            (
                VariableValue::Decimal(x_sign, x_num, x_den),
                VariableValue::Decimal(y_sign, y_num, y_den),
            ) => {
                let (z_sign, z_num, z_den) =
                    sub_decimal(x_sign, x_num, x_den, y_sign, y_num, y_den);
                VariableValue::Decimal(z_sign, z_num, z_den)
            }
            (VariableValue::Dur(d1), VariableValue::Dur(d2)) => {
                VariableValue::Dur(d1.sub(d2, ctx.clock, ctx.frame_len))
            }
            (VariableValue::Map(mut m1), VariableValue::Map(m2)) => {
                for (key, value) in m2 {
                    if !m1.contains_key(&key) {
                        m1.insert(key, value);
                    } else {
                        let x1 = m1.get(&key).cloned().unwrap();
                        m1.insert(key, x1.sub(value, ctx));
                    }
                }
                VariableValue::Map(m1)
            }
            _ => panic!("Subtraction with wrong types, this should never happen"),
        }
    }

    pub fn pow(self, other: VariableValue, _ctx: &EvaluationContext) -> VariableValue {
        // TODO: Add support for other types !
        match (self, other) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => {
                VariableValue::Integer(i1.pow(i2 as u32))
            }
            (VariableValue::Float(f1), VariableValue::Float(f2)) => {
                VariableValue::Float(f1.powf(f2))
            }
            _ => panic!("Power with wrong types, this should never happen"),
        }
    }

    pub fn and(self, other: VariableValue) -> VariableValue {
        match (self, other) {
            (VariableValue::Bool(b1), VariableValue::Bool(b2)) => VariableValue::Bool(b1 && b2),
            _ => panic!("Logical and with wrong types, this should never happen"),
        }
    }

    pub fn or(self, other: VariableValue) -> VariableValue {
        match (self, other) {
            (VariableValue::Bool(b1), VariableValue::Bool(b2)) => VariableValue::Bool(b1 || b2),
            _ => panic!("Logical or with wrong types, this should never happen"),
        }
    }

    pub fn xor(self, other: VariableValue) -> VariableValue {
        match (self, other) {
            (VariableValue::Bool(b1), VariableValue::Bool(b2)) => {
                VariableValue::Bool((b1 && !b2) || (!b1 && b2))
            }
            _ => panic!("Logical xor with wrong types, this should never happen"),
        }
    }

    pub fn logical_shift(self, other: VariableValue) -> VariableValue {
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

    pub fn cast_as_integer(&self, clock: &Clock, frame_len: f64) -> VariableValue {
        VariableValue::Integer(self.as_integer(clock, frame_len))
    }

    pub fn cast_as_float(&self, clock: &Clock, frame_len: f64) -> VariableValue {
        VariableValue::Float(self.as_float(clock, frame_len))
    }

    pub fn cast_as_decimal(&self, clock: &Clock, frame_len: f64) -> VariableValue {
        let (sign, num, den) = self.as_decimal(clock, frame_len);
        VariableValue::Decimal(sign, num, den)
    }

    pub fn cast_as_bool(&self, clock: &Clock, frame_len: f64) -> VariableValue {
        VariableValue::Bool(self.as_bool(clock, frame_len))
    }

    pub fn cast_as_str(&self, clock: &Clock, frame_len: f64) -> VariableValue {
        VariableValue::Str(self.as_str(clock, frame_len))
    }

    pub fn cast_as_dur(&self) -> VariableValue {
        VariableValue::Dur(self.as_dur())
    }

    pub fn cast_as_map(&self) -> VariableValue {
        VariableValue::Map(self.as_map())
    }

    pub fn cast_as_vec(&self) -> VariableValue {
        VariableValue::Vec(self.as_vec())
    }

    pub fn cast_as_blob(&self) -> VariableValue {
        VariableValue::Blob(self.as_blob())
    }

    pub fn as_integer(&self, clock: &Clock, frame_len: f64) -> i64 {
        match self {
            VariableValue::Integer(i) => *i,
            VariableValue::Float(f) => f.round() as i64,
            VariableValue::Decimal(sign, num, den) => {
                let mut as_int = (*num / *den) as i64;
                if *sign < 0 {
                    as_int = -as_int;
                }
                as_int
            }
            VariableValue::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            VariableValue::Str(s) => s.parse::<i64>().unwrap_or(0),
            VariableValue::Dur(d) => d.as_micros(clock, frame_len).try_into().unwrap(),
            VariableValue::Func(_) => todo!(),
            VariableValue::Map(_) | VariableValue::Vec(_) => 0,
            VariableValue::Blob(b) => {
                let mut arr = [0u8; 8];
                for i in 0..std::cmp::min(b.len(), 8) {
                    arr[i] = b[i];
                }
                i64::from_le_bytes(arr)
            }
        }
    }

    pub fn as_float(&self, clock: &Clock, frame_len: f64) -> f64 {
        match self {
            VariableValue::Integer(i) => *i as f64,
            VariableValue::Float(f) => *f,
            VariableValue::Decimal(sign, num, den) => float64_from_decimal(*sign, *num, *den),
            VariableValue::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            VariableValue::Str(s) => s.parse::<f64>().unwrap_or(0.0),
            VariableValue::Dur(d) => d.as_micros(clock, frame_len) as f64,
            VariableValue::Func(_) => todo!(),
            VariableValue::Map(_) | VariableValue::Vec(_) => 0.0,
            VariableValue::Blob(b) => {
                let mut arr = [0u8; 8];
                for i in 0..std::cmp::min(b.len(), 8) {
                    arr[i] = b[i];
                }
                f64::from_le_bytes(arr)
            }
        }
    }

    pub fn as_decimal(&self, clock: &Clock, frame_len: f64) -> (i8, u64, u64) {
        match self {
            VariableValue::Integer(i) => {
                let sign = if *i < 0 { -1 } else { 1 };
                let num = if *i < 0 { (-*i) as u64 } else { *i as u64 };
                (sign, num, 1)
            }
            VariableValue::Float(f) => decimal_from_float64(*f),
            VariableValue::Decimal(sign, num, den) => (*sign, *num, *den),
            VariableValue::Bool(b) => {
                if *b {
                    (1, 1, 1)
                } else {
                    (1, 0, 1)
                }
            }
            VariableValue::Str(s) => match s.parse::<f64>() {
                Ok(n) => decimal_from_float64(n),
                Err(_) => (1, 0, 1),
            },
            VariableValue::Dur(d) => (1, d.as_micros(clock, frame_len) as u64, 1),
            VariableValue::Func(_) => todo!(),
            VariableValue::Map(_) | VariableValue::Blob(_) | VariableValue::Vec(_) => (1, 0, 1),
        }
    }

    pub fn as_bool(&self, clock: &Clock, frame_len: f64) -> bool {
        match self {
            VariableValue::Integer(i) => *i != 0,
            VariableValue::Float(f) => *f != 0.0,
            VariableValue::Decimal(_, num, _) => *num != 0,
            VariableValue::Bool(b) => *b,
            VariableValue::Str(s) => !s.is_empty(),
            VariableValue::Dur(d) => d.as_micros(clock, frame_len) != 0,
            VariableValue::Func(_) => todo!(),
            VariableValue::Map(map) => !map.is_empty(),
            VariableValue::Vec(vec) => !vec.is_empty(),
            VariableValue::Blob(b) => !b.is_empty(),
        }
    }

    pub fn as_str(&self, clock: &Clock, frame_len: f64) -> String {
        match self {
            VariableValue::Integer(i) => i.to_string(),
            VariableValue::Float(f) => f.to_string(),
            VariableValue::Decimal(sign, num, den) => string_from_decimal(*sign, *num, *den),
            VariableValue::Bool(b) => {
                if *b {
                    "True".to_string()
                } else {
                    "False".to_string()
                }
            }
            VariableValue::Str(s) => s.to_string(),
            VariableValue::Dur(d) => d.as_micros(clock, frame_len).to_string(),
            VariableValue::Func(_) => todo!(),
            VariableValue::Map(_) => "[map]".to_string(),
            VariableValue::Vec(_) => "[vec]".to_string(),
            VariableValue::Blob(b) => String::from_utf8(b.clone()).unwrap_or_default(),
        }
    }

    pub fn as_dur(&self) -> TimeSpan {
        match self {
            VariableValue::Integer(i) => TimeSpan::Micros(i.unsigned_abs()),
            VariableValue::Float(f) => TimeSpan::Micros((f.round() as i64).unsigned_abs()),
            VariableValue::Decimal(_, num, den) => TimeSpan::Micros((num / den) as u64),
            VariableValue::Bool(_) => TimeSpan::Micros(0), // TODO décider comment caster booléen vers durée
            VariableValue::Str(_) => TimeSpan::Micros(0),  // TODO parser la chaîne de caractères
            VariableValue::Dur(d) => *d,
            VariableValue::Func(_) => todo!(),
            VariableValue::Map(_) | VariableValue::Vec(_) => TimeSpan::Micros(0),
            VariableValue::Blob(b) => TimeSpan::Micros(b.len() as SyncTime),
        }
    }

    pub fn as_map(&self) -> HashMap<String, VariableValue> {
        match self {
            VariableValue::Map(map) => map.clone(),
            x => {
                let mut map = HashMap::new();
                map.insert("s".to_owned(), x.clone());
                map
            }
        }
    }

    pub fn as_blob(&self) -> Vec<u8> {
        match self {
            VariableValue::Integer(i) => Vec::from(i.to_le_bytes()),
            VariableValue::Float(f) => Vec::from(f.to_le_bytes()),
            VariableValue::Decimal(_, _, _) => Vec::new(),
            VariableValue::Bool(b) => {
                if *b {
                    vec![1]
                } else {
                    Vec::new()
                }
            }
            VariableValue::Str(s) => Vec::from(s.as_bytes()),
            VariableValue::Dur(_) => Vec::new(),
            VariableValue::Func(_) => Vec::new(),
            VariableValue::Map(_) => Vec::new(),
            VariableValue::Vec(v) => v.iter().map(VariableValue::as_blob).flatten().collect(),
            VariableValue::Blob(b) => b.clone(),
        }
    }

    pub fn as_vec(&self) -> Vec<VariableValue> {
        match self {
            VariableValue::Map(m) => {
                let mut res = Vec::new();
                for (key, value) in m.iter() {
                    res.push(VariableValue::Str(key.clone()));
                    res.push(value.clone());
                }
                res
            }
            VariableValue::Vec(v) => v.clone(),
            item => vec![item.clone()],
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(self, VariableValue::Integer(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, VariableValue::Float(_))
    }

    pub fn is_decimal(&self) -> bool {
        matches!(self, VariableValue::Decimal(_, _, _))
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Variable {
    Environment(EnvironmentFunc),
    Global(String),
    Line(String), // not fully handled
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

    pub fn insert(
        &mut self,
        key: String,
        value: VariableValue
    ) -> Option<VariableValue> {
        if self.watchers.len() > 0 {
            self.delta.push(key.clone());
        }
        self.content.insert(key, value)
    }

    pub fn insert_cast(&mut self, key: String, mut value: VariableValue, clock: &Clock, frame_len: f64) -> Option<VariableValue> {
        if let Some(old_value) = self.content.get(&key) {
            match old_value {
                VariableValue::Integer(_) => value = value.cast_as_integer(clock, frame_len),
                VariableValue::Float(_) => value = value.cast_as_float(clock, frame_len),
                VariableValue::Decimal(_, _, _) => value = value.cast_as_decimal(clock, frame_len),
                VariableValue::Bool(_) => value = value.cast_as_bool(clock, frame_len),
                VariableValue::Str(_) => value = value.cast_as_str(clock, frame_len),
                VariableValue::Dur(_) => value = value.cast_as_dur(),
                VariableValue::Func(_) => { /* Do nothing, allow overwrite */ }
                VariableValue::Map(_) => { /* Do nothing, allow overwrite */ }
                VariableValue::Vec(_) => { /* Do nothing, allow overwrite */ }
                VariableValue::Blob(_) => { /* Do nothing, allow overwrite */ }
            }
        }
        self.insert(key, value)
    }

    pub fn get(&self, key: &str) -> Option<&VariableValue> {
        self.content.get(key)
    }

    pub fn has(&self, key: &str) -> bool {
        self.content.contains_key(key)
    }

    pub fn get_create(&mut self, key: &str) -> &VariableValue {
        if !self.content.contains_key(key) {
            self.content.insert(key.to_owned(), VariableValue::default());
        }
        self.content.get(key).unwrap()
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut VariableValue> {
        self.content.get_mut(key)
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
