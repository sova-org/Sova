// Basic operations on decimal numbers represented by triplets (sign, num, den) where
// sign is a signe (should be -1 or 1), num is the numerator of a fraction, den is the
// denominator of a fraction. Fractions as results of operations are always simplified.

#[cfg(test)]
mod tests;
mod decimal;

pub use decimal::Decimal;

// addition
pub fn add_decimal(
    x_sign: i8,
    x_num: u64,
    x_den: u64,
    y_sign: i8,
    y_num: u64,
    y_den: u64,
) -> (i8, u64, u64) {
    let x_num_for_add = x_num * y_den;
    let y_num_for_add = y_num * x_den;
    let res_sign = if (x_sign < 0 && y_sign < 0) || (x_sign >= 0 && y_sign >= 0) {
        x_sign
    } else if x_num_for_add > y_num_for_add {
        x_sign
    } else {
        y_sign
    };
    let res_num = if (x_sign < 0 && y_sign < 0) || (x_sign >= 0 && y_sign >= 0) {
        x_num_for_add + y_num_for_add
    } else if x_num_for_add > y_num_for_add {
        x_num_for_add - y_num_for_add
    } else {
        y_num_for_add - x_num_for_add
    };
    let res_den = x_den * y_den;
    simplify_decimal(res_sign, res_num, res_den)
}

// subtraction
pub fn sub_decimal(
    x_sign: i8,
    x_num: u64,
    x_den: u64,
    y_sign: i8,
    y_num: u64,
    y_den: u64,
) -> (i8, u64, u64) {
    let y_sign_for_add = if y_sign < 0 { 1 } else { -1 };
    add_decimal(x_sign, x_num, x_den, y_sign_for_add, y_num, y_den)
}

// multiplication
pub fn mul_decimal(
    x_sign: i8,
    x_num: u64,
    x_den: u64,
    y_sign: i8,
    y_num: u64,
    y_den: u64,
) -> (i8, u64, u64) {
    let sign = x_sign * y_sign;
    let num = x_num * y_num;
    let den = x_den * y_den;
    simplify_decimal(sign, num, den)
}

// division, dividing by 0 returns 0
pub fn div_decimal(
    x_sign: i8,
    x_num: u64,
    x_den: u64,
    y_sign: i8,
    y_num: u64,
    y_den: u64,
) -> (i8, u64, u64) {
    let sign = x_sign * y_sign;
    let num = x_num * y_den;
    let den = y_num * x_den;
    if den == 0 {
        return (1, 0, 1);
    }
    simplify_decimal(sign, num, den)
}

// (strictly) lower than test
pub fn lt_decimal(x_sign: i8, x_num: u64, x_den: u64, y_sign: i8, y_num: u64, y_den: u64) -> bool {
    let x_for_cmp = x_num * y_den;
    let y_for_cmp = y_num * x_den;
    (x_sign < 0 && y_sign >= 0)
        || (x_sign < 0 && y_sign < 0 && x_for_cmp > y_for_cmp)
        || (x_sign >= 0 && y_sign >= 0 && x_for_cmp < y_for_cmp)
}

// lower or equal than test
pub fn leq_decimal(x_sign: i8, x_num: u64, x_den: u64, y_sign: i8, y_num: u64, y_den: u64) -> bool {
    let x_for_cmp = x_num * y_den;
    let y_for_cmp = y_num * x_den;
    (x_for_cmp == 0 && y_for_cmp == 0)
        || (x_sign < 0 && y_sign >= 0)
        || (x_sign < 0 && y_sign < 0 && x_for_cmp >= y_for_cmp)
        || (x_sign >= 0 && y_sign >= 0 && x_for_cmp <= y_for_cmp)
}

// equality test
pub fn eq_decimal(x_sign: i8, x_num: u64, x_den: u64, y_sign: i8, y_num: u64, y_den: u64) -> bool {
    let x_for_cmp = x_num * y_den;
    let y_for_cmp = y_num * x_den;
    (x_sign < 0 && y_sign < 0 && x_for_cmp == y_for_cmp)
        || (x_sign >= 0 && y_sign >= 0 && x_for_cmp == y_for_cmp)
}

// difference test
pub fn neq_decimal(x_sign: i8, x_num: u64, x_den: u64, y_sign: i8, y_num: u64, y_den: u64) -> bool {
    !eq_decimal(x_sign, x_num, x_den, y_sign, y_num, y_den)
}

// fraction simplification
fn simplify_decimal(sign: i8, num: u64, den: u64) -> (i8, u64, u64) {
    if num == 0 {
        return (1, 0, 1);
    }

    // gcd computation
    let gcd = || -> u64 {
        let mut max = if num > den { num } else { den };
        let mut min = if num > den { den } else { num };

        while min != 0 {
            let r = max % min;
            max = min;
            min = r;
        }

        max
    };
    let gcd = gcd();

    // simplification
    let num = num / gcd;
    let den = den / gcd;

    (sign, num, den)
}

// get decimal number from float
// WARNING: due to representation of floats, this will not work for very large floats
pub fn decimal_from_float64(x: f64) -> (i8, u64, u64) {
    let sign = if x < 0.0 { -1 } else { 1 };

    let x = if x < 0.0 { -x } else { x };

    let integer_part = x.trunc() as u64;
    let mut decimal_part = x.fract();
    let mut num_decimal = 10;

    let mut numerator = integer_part;
    let mut denominator = 1;

    while num_decimal > 0 {
        decimal_part *= 10.0;
        let new_num = decimal_part.trunc() as u64;
        decimal_part = decimal_part.fract();

        denominator *= 10;
        numerator = numerator * 10 + new_num;

        num_decimal -= 1;
    }

    simplify_decimal(sign, numerator, denominator)
}

// get float from decimal number
pub fn float64_from_decimal(sign: i8, num: u64, den: u64) -> f64 {
    let mut as_float = (num as f64) / (den as f64);
    if sign < 0 {
        as_float = -as_float;
    }
    as_float
}

// Display decimal number
pub fn string_from_decimal(sign: i8, num: u64, den: u64) -> String {
    let sign = if sign < 0 { "-" } else { "" };
    format!("{sign}{num}/{den}")
}

/// High-precision summation of floating-point values using rational arithmetic.
/// Eliminates cumulative floating-point rounding errors in summation operations.
pub fn precise_sum(values: impl Iterator<Item = f64>) -> f64 {
    let result = values
        .map(decimal_from_float64)
        .fold((1, 0, 1), |acc, val| {
            add_decimal(acc.0, acc.1, acc.2, val.0, val.1, val.2)
        });

    float64_from_decimal(result.0, result.1, result.2)
}

/// High-precision modulo operation for beat positioning using the fraction crate.
/// Preserves exact fractional beat positions (1/4, 1/8, 1/16, etc.) without precision loss.
pub fn precise_multiplication(a: f64, b: f64) -> f64 {
    let a = Decimal::from(a);
    let b = Decimal::from(b);
    (a * b).into()
}

/// High-precision modulo operation for beat positioning using the fraction crate.
/// Preserves exact fractional beat positions (1/4, 1/8, 1/16, etc.) without precision loss.
pub fn precise_modulo(a: f64, b: f64) -> f64 {
    let a = Decimal::from(a);
    let b = Decimal::from(b);
    (a % b).into()
}

/// High-precision division for beat calculations using the fraction crate.
/// Eliminates floating-point precision loss in speed factor and timing divisions.
pub fn precise_division(a: f64, b: f64) -> f64 {
    let a = Decimal::from(a);
    let b = Decimal::from(b);
    (a / b).into()
}

// Reminder of the division of two decimal numbers
pub fn rem_decimal(
    x_sign: i8,
    x_num: u64,
    x_den: u64,
    _y_sign: i8,
    y_num: u64,
    y_den: u64,
) -> (i8, u64, u64) {
    if y_num == 0 {
        return (x_sign, x_num, x_den);
    }

    let sign = 1;
    let num = x_num * y_den - ((x_num * y_den) / (y_num * x_den)) * y_num * x_den; // not simplified on purpose to benefit from the integer division
    let den = x_den * x_den;
    simplify_decimal(sign, num, den)
}