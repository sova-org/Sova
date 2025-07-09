use crate::compiler::bali::bali_ast::{
    Expression, Value, BooleanExpression, information::TimingInformation, ConcreteFraction, Variable,
};

#[derive(Debug, Clone)]
pub enum AbstractArg {
    Alt(Vec<AbstractArg>, Variable),
    Choice(Vec<AbstractArg>),
    List(Vec<AbstractArg>),
    Concrete(ConcreteArg),
}

#[derive(Debug, Clone)]
pub enum ConcreteArg {
    BoolExpr(Box<BooleanExpression>),
    Expr(Box<Expression>),
    Literal(Value),
    Number(i64),
    TimingInfo(TimingInformation),
}

impl ConcreteArg {
    pub fn to_value(&self) -> Value {
        match self {
            ConcreteArg::Literal(v) => v.clone(),
            _ => Value::Number(0), // should never occur
        }
    }
    
    pub fn to_expression(&self) -> Box<Expression> {
        match self {
            ConcreteArg::Expr(e) => e.clone(),
            _ => Box::new(Expression::Value(Value::Number(0))), // should never occur
        }
    }

    pub fn to_boolean_expression(&self) -> Box<BooleanExpression> {
        match self {
            ConcreteArg::BoolExpr(b) => b.clone(),
            _ => Box::new(BooleanExpression::Equal(
                Box::new(Expression::Value(Value::Number(0))),
                Box::new(Expression::Value(Value::Number(1)))
            )), // should never occur
        }
    }

    pub fn to_integer(&self) -> i64 {
        match self {
            ConcreteArg::Number(i) => i.clone(),
            _ => 0, // should never occur
        }
    }

    pub fn to_timing_information(&self) -> TimingInformation {
        match self {
            ConcreteArg::TimingInfo(t) => t.clone(),
            _ => TimingInformation::PositionRelative(ConcreteFraction{signe: 1, numerator: 0, denominator: 1}),
        }
    }
}