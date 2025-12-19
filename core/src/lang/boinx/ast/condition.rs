use std::{collections::BTreeSet, fmt::Display};

use crate::{vm::{EvaluationContext, variable::VariableValue}, lang::boinx::ast::{BoinxIdent, BoinxItem}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoinxConditionOp {
    Less,
    LessEq,
    #[default]
    Equal,
    NotEqual,
    GreaterEq,
    Greater,
}

impl BoinxConditionOp {
    pub fn parse(txt: &str) -> Self {
        match txt {
            "<" => Self::Less,
            "<=" => Self::LessEq,
            "==" => Self::Equal,
            "!=" => Self::NotEqual,
            ">=" => Self::GreaterEq,
            ">" => Self::Greater,
            _ => Self::Equal,
        }
    }
}

impl Display for BoinxConditionOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoinxConditionOp::Less => write!(f, "<"),
            BoinxConditionOp::LessEq => write!(f, "<="),
            BoinxConditionOp::Equal => write!(f, "=="),
            BoinxConditionOp::NotEqual => write!(f, "!="),
            BoinxConditionOp::GreaterEq => write!(f, ">="),
            BoinxConditionOp::Greater => write!(f, ">"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BoinxCondition(pub Box<BoinxItem>, pub BoinxConditionOp, pub Box<BoinxItem>);

impl BoinxCondition {

    pub fn slots<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut BoinxItem> + 'a> {
        Box::new(self.0.slots().chain(self.2.slots()))
    }

    pub fn is_true(&self, ctx: &EvaluationContext) -> bool {
        let mut x1 = VariableValue::from((*self.0).clone());
        let mut x2 = VariableValue::from((*self.2).clone());
        x1.compatible_cast(&mut x2, ctx);
        (match self.1 {
            BoinxConditionOp::Less => x1.lt(x2),
            BoinxConditionOp::LessEq => x1.leq(x2),
            BoinxConditionOp::Equal => x1.eq(x2),
            BoinxConditionOp::NotEqual => x1.neq(x2),
            BoinxConditionOp::GreaterEq => x1.geq(x2),
            BoinxConditionOp::Greater => x1.gt(x2),
        })
        .is_true(ctx)
    }

    pub fn evaluate(&self, ctx: &mut EvaluationContext) -> BoinxCondition {
        BoinxCondition(
            Box::new(self.0.evaluate(ctx)),
            self.1,
            Box::new(self.2.evaluate(ctx))
        )
    }

    pub fn evaluate_vars(&self, ctx: &EvaluationContext, forbidden: &mut BTreeSet<BoinxIdent>) -> BoinxCondition {
        BoinxCondition(
            Box::new(self.0.evaluate_vars(ctx, forbidden)),
            self.1,
            Box::new(self.2.evaluate_vars(ctx, forbidden))
        )
    }

    pub fn has_vars(&self) -> bool {
        self.0.has_vars() || self.2.has_vars()
    }
}