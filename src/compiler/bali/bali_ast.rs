use crate::lang::Program;

pub type BaliProgram = Vec<TopLevelStatement>;
pub type BaliPreparedProgram = Vec<TimeStatement>;

pub fn bali_as_asm(prog: BaliProgram) -> Program {
    print!("Original prog {:?}\n", prog);
    let prog = expend_prog(prog);
    print!("Expended prog {:?}\n", prog);
    Vec::new()
}

pub fn expend_prog(prog: BaliProgram) -> BaliPreparedProgram {
    prog.into_iter().map(|tls| tls.expend()).flatten().collect()
}

#[derive(Debug)]
pub enum TimeStatement {
    At(f64, Effect),
    JustBefore(f64, Effect),
    JustAfter(f64, Effect),
}

#[derive(Debug)]
pub enum TopLevelStatement {
    AtStatement(Value, Vec<Statement>),
    Statement(Statement),
}

impl TopLevelStatement {

    pub fn expend(self) -> Vec<TimeStatement> {
        match self {
            TopLevelStatement::AtStatement(v, ss) => ss.into_iter().map(|s| s.expend(&v)).collect(),
            TopLevelStatement::Statement(s) => vec![s.expend(&Value::Number(0))],
        }
    }

}

#[derive(Debug)]
pub enum Statement {
    AfterFrac(Value, Effect),
    After(Effect),
    BeforeFrac(Value, Effect),
    Before(Effect),
    Effect(Effect),
}

impl Statement {

    pub fn expend(self, val: &Value) -> TimeStatement {
        match self {
            Statement::AfterFrac(v, e) => TimeStatement::At(v.addf64(val), e),
            Statement::After(e) => TimeStatement::JustAfter(val.tof64(), e),
            Statement::BeforeFrac(v, e) => TimeStatement::At(val.subf64(&v), e),
            Statement::Before(e) => TimeStatement::JustBefore(val.tof64(), e),
            Statement::Effect(e) => TimeStatement::At(val.tof64(), e),
        }
    }

}

#[derive(Debug)]
pub enum Effect {
    Definition(Value, Box<Expression>),
    Note(Box<Expression>, Box<Expression>, Box<Expression>, Box<Expression>, Value),
    ProgramChange(Box<Expression>, Box<Expression>, Value),
    ControlChange(Box<Expression>, Box<Expression>, Box<Expression>, Value),
}

#[derive(Debug)]
pub enum Expression {
    Addition(Box<Expression>, Box<Expression>),
    Multiplication(Box<Expression>, Box<Expression>),
    Subtraction(Box<Expression>, Box<Expression>),
    Division(Box<Expression>, Box<Expression>),
    Modulo(Box<Expression>, Box<Expression>),
    Value(Value),
}

/*
impl Expression {

    pub fn evaluate(&self) -> Box<Expression> {
        match &self {
            Expression::Addition(e1, e2) => {
                let e1 = e1.evaluate();
                let e2 = e2.evaluate();
                match (&*e1, &*e2) {
                    (Expression::Value(Value::Number(n1)), Expression::Value(Value::Number(n2))) => Box::new(Expression::Value(Value::Number(n1 + n2))),
                    _ => Box::new(Expression::Addition(e1, e2)),
                }
            },
            _ => Box::new(Expression::Value(Value::Number(0))),
        }
    }

}
*/

#[derive(Debug)]
pub enum Value {
    Number(u8),
    Variable(String),
    Fraction(Box<Value>, Box<Value>),
}

impl Value {

    pub fn tof64(&self) -> f64 {
        match self {
            Value::Number(n) => *n as f64,
            Value::Fraction(v1, v2) => {
                let v1 = v1.tof64();
                let v2 = v2.tof64();
                if v2 != 0.0 {
                    v1 as f64 / v2 as f64
                } else {
                    v1 as f64
                }
            }
            Value::Variable(_) => 0.0,
        }
    }

    pub fn addf64(&self, other: &Self) -> f64 {
        self.tof64() + other.tof64()
    }

    pub fn subf64(&self, other: &Self) -> f64 {
        self.tof64() - other.tof64()
    }

}
