#[derive(Debug, Clone)]
pub struct ConcreteFraction {
    pub signe: i64,
    pub numerator: i64,
    pub denominator: i64,
}

impl ConcreteFraction {
    pub fn from_dec_string(dec: String) -> ConcreteFraction {
        let parts: Vec<&str> = dec.split('.').collect();
        let int_part = match parts[0].parse::<i64>() {
            Ok(n) => n,
            Err(_) => 0,
        };
        let dec_part = match parts[1].parse::<i64>() {
            Ok(n) => n,
            Err(_) => 0,
        };
        let num_dec = parts[1].len();
        let mut denominator = 1;
        for _i in 0..num_dec {
            denominator = denominator * 10;
        }
        let signe = if int_part < 0 {
            -1
        } else {
            1
        };
        let int_part = if int_part < 0 {
            -int_part
        } else {
            int_part
        };
        let numerator = int_part * denominator + dec_part;
        ConcreteFraction {
            signe,
            numerator,
            denominator,
        }
        .simplify()
    }

    pub fn tof64(&self) -> f64 {
        (self.signe * self.numerator) as f64 / self.denominator as f64
    }

    pub fn add(&self, other: &Self) -> ConcreteFraction {
        ConcreteFraction {
            signe: 1,
            numerator: self.signe * self.numerator * other.denominator
                + other.signe * other.numerator * self.denominator,
            denominator: self.denominator * other.denominator,
        }
        .simplify()
    }

    pub fn sub(&self, other: &Self) -> ConcreteFraction {
        ConcreteFraction {
            signe: 1,
            numerator: self.signe * self.numerator * other.denominator
                - other.signe * other.numerator * self.denominator,
            denominator: self.denominator * other.denominator,
        }
        .simplify()
    }

    pub fn mult(&self, other: &Self) -> ConcreteFraction {
        ConcreteFraction {
            signe: 1,
            numerator: self.signe * self.numerator * other.signe * other.numerator,
            denominator: self.denominator * other.denominator,
        }
        .simplify()
    }

    pub fn multbyint(&self, mult: i64) -> ConcreteFraction {
        ConcreteFraction {
            signe: 1,
            numerator: self.signe * self.numerator * mult,
            denominator: self.denominator,
        }
        .simplify()
    }

    pub fn divbyint(&self, div: i64) -> ConcreteFraction {
        ConcreteFraction {
            signe: 1,
            numerator: self.signe * self.numerator,
            denominator: self.denominator * div,
        }
    }

    fn simplify(&self) -> ConcreteFraction {
        let signe = if self.numerator * self.denominator < 0 {
            -self.signe
        } else {
            self.signe
        };
        let numerator = if self.numerator < 0 {
            -self.numerator
        } else {
            self.numerator
        };
        let denominator = if self.denominator < 0 {
            -self.denominator
        } else {
            self.denominator
        };
        let gcd = Self::gcd(numerator, denominator);
        let numerator = numerator / gcd;
        let denominator = denominator / gcd;
        ConcreteFraction {
            signe,
            numerator,
            denominator,
        }
    }

    fn gcd(a: i64, b: i64) -> i64 {
        let mut max = if a > b { a } else { b };

        let mut min = if a > b { b } else { a };

        while min != 0 {
            let r = max % min;
            max = min;
            min = r;
        }

        max
    }
}
