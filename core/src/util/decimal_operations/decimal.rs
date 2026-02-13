use std::{cmp::Ordering, fmt::Display, ops::{Add, Div, Mul, Neg, Rem, Sub}};

use serde::{Deserialize, Serialize};

use crate::util::decimal_operations::{add_decimal, decimal_from_float64, div_decimal, eq_decimal, float64_from_decimal, lt_decimal, mul_decimal, rem_decimal, simplify_decimal, string_from_decimal, sub_decimal};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Decimal {
    pub sign: i8,
    pub num: u64,
    pub den: u64
}

impl Decimal {
    pub fn simplified(self) -> Self {
        simplify_decimal(self.sign, self.num, self.den).into()
    }
    pub fn one() -> Self {
        Self { sign: 1, num: 1, den: 1 }
    }
    pub fn zero() -> Self {
        Self { sign: 1, num: 0, den: 1 }
    }
    pub fn is_zero(&self) -> bool {
        self.num == 0
    }
}

impl Default for Decimal {
    fn default() -> Self {
        Self::zero()
    }
}

impl Neg for Decimal {
    type Output = Decimal;
    fn neg(self) -> Self::Output {
        Decimal { sign: -self.sign, num: self.num, den: self.den }
    }
}

impl Add for Decimal {
    type Output = Decimal;
    fn add(self, rhs: Self) -> Self::Output {
        add_decimal(
            self.sign, self.num, self.den, 
            rhs.sign, rhs.num, rhs.den
        ).into()
    }
}

impl Sub for Decimal {
    type Output = Decimal;
    fn sub(self, rhs: Self) -> Self::Output {
        sub_decimal(
            self.sign, self.num, self.den, 
            rhs.sign, rhs.num, rhs.den
        ).into()
    }
}

impl Mul for Decimal {
    type Output = Decimal;
    fn mul(self, rhs: Self) -> Self::Output {
        mul_decimal(
            self.sign, self.num, self.den, 
            rhs.sign, rhs.num, rhs.den
        ).into()
    }
}

impl Div for Decimal {
    type Output = Decimal;
    fn div(self, rhs: Self) -> Self::Output {
        div_decimal(
            self.sign, self.num, self.den, 
            rhs.sign, rhs.num, rhs.den
        ).into()
    }
}

impl Rem for Decimal {
    type Output = Decimal;
    fn rem(self, rhs: Self) -> Self::Output {
        rem_decimal(
            self.sign, self.num, self.den, 
            rhs.sign, rhs.num, rhs.den
        ).into()
    }
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        eq_decimal(
            self.sign, self.num, self.den, 
            other.sign, other.num, other.den
        )
    }
}
impl Eq for Decimal { }

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if *self == *other {
            Ordering::Equal
        } else if lt_decimal(
            self.sign, self.num, self.den, 
            self.sign, self.num, self.den
        ) {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

impl From<f64> for Decimal {
    fn from(value: f64) -> Self {
        decimal_from_float64(value).into()
    }
}

impl From<u64> for Decimal {
    fn from(value: u64) -> Self {
        Decimal {
            sign: 1,
            num: value as u64,
            den: 1
        }
    }
}

impl From<i64> for Decimal {
    fn from(value: i64) -> Self {
        Decimal {
            sign: value.signum() as i8,
            num: value.abs() as u64,
            den: 1
        }
    }
}

impl From<Decimal> for f64 {
    fn from(value: Decimal) -> Self {
        float64_from_decimal(value.sign, value.num, value.den)
    }
}

impl From<Decimal> for i64 {
    fn from(value: Decimal) -> Self {
        (value.sign as i64) * (value.den / value.num) as i64
    }
}

impl From<(i8, u64, u64)> for Decimal {
    fn from(value: (i8, u64, u64)) -> Self {
        Decimal { 
            sign: value.0, 
            num: value.1, 
            den: value.2
        }
    }
}

impl Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", string_from_decimal(self.sign, self.num, self.den))
    }
}