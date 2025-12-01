use std::fmt::Display;

use crate::lang::{evaluation_context::EvaluationContext, interpreter::boinx::ast::BoinxItem, variable::VariableValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoinxArithmeticOp {
    #[default]
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    Pow,
}

impl BoinxArithmeticOp {
    pub fn parse(txt: &str) -> Self {
        match txt {
            "+" => Self::Add,
            "-" => Self::Sub,
            "*" => Self::Mul,
            "/" => Self::Div,
            "%" => Self::Rem,
            "<<" => Self::Shl,
            ">>" => Self::Shr,
            "^" => Self::Pow,
            _ => Self::Add,
        }
    }
}

impl Display for BoinxArithmeticOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoinxArithmeticOp::Add => write!(f, "+"),
            BoinxArithmeticOp::Sub => write!(f, "-"),
            BoinxArithmeticOp::Mul => write!(f, "*"),
            BoinxArithmeticOp::Div => write!(f, "/"),
            BoinxArithmeticOp::Rem => write!(f, "%"),
            BoinxArithmeticOp::Shl => write!(f, "<<"),
            BoinxArithmeticOp::Shr => write!(f, ">>"),
            BoinxArithmeticOp::Pow => write!(f, "^"),
        }
    }
}

pub fn arithmetic_op(
    ctx: &EvaluationContext,
    i1: BoinxItem, 
    op: BoinxArithmeticOp, 
    i2: BoinxItem
) -> BoinxItem {
    use BoinxItem::*;
    use BoinxArithmeticOp::*;
    match (i1, op, i2) {
        (Escape(e), op, i) => {
            let res = arithmetic_op(ctx, *e, op, i);
            Escape(Box::new(res.unescape()))
        }
        (i, op, Escape(e)) => {
            let res = arithmetic_op(ctx, i, op, *e);
            Escape(Box::new(res.unescape()))
        }
        (Sequence(v1), op, Sequence(v2)) => {
            todo!()
        }
        (Simultaneous(v1), op, Simultaneous(v2)) => {
            let mut res = Vec::new();
            for item1 in v1 {
                for item2 in v2.iter() {
                    res.push(arithmetic_op(ctx, item1.clone(), op, item2.clone()));
                }
            }
            Simultaneous(res)
        }
        (Sequence(v1), op, Simultaneous(v2)) => {
            let mut res = Vec::new();
            for item1 in v1 {
                let mut inner = Vec::new();
                for item2 in v2.iter() {
                    inner.push(arithmetic_op(ctx, item1.clone(), op, item2.clone()));
                }
                res.push(Simultaneous(inner));
            }
            Sequence(res)
        }
        (Simultaneous(v1), op, Sequence(v2)) => {
            let mut res = Vec::new();
            for item1 in v1 {
                let mut inner = Vec::new();
                for item2 in v2.iter() {
                    inner.push(arithmetic_op(ctx, item1.clone(), op, item2.clone()));
                }
                res.push(Sequence(inner));
            }
            Simultaneous(res)
        }
        (Simultaneous(v1), op, item2) => {
            let mut res = Vec::new();
            for item1 in v1 {
                res.push(arithmetic_op(ctx, item1, op, item2.clone()));
            }
            Simultaneous(res)
        }
        (item1, op, Simultaneous(v2)) => {
            let mut res = Vec::new();
            for item2 in v2 {
                res.push(arithmetic_op(ctx, item1.clone(), op, item2));
            }
            Simultaneous(res)
        }
        (Sequence(v1), op, item2) => {
            let mut res = Vec::new();
            for item1 in v1 {
                res.push(arithmetic_op(ctx, item1, op, item2.clone()));
            }
            Sequence(res)
        }
        (item1, op, Sequence(v2)) => {
            let mut res = Vec::new();
            for item2 in v2 {
                res.push(arithmetic_op(ctx, item1.clone(), op, item2));
            }
            Sequence(res)
        }
        (WithDuration(i1, d1), op, WithDuration(i2, d2)) => {
            todo!()
        }
        (WithDuration(i1, d1), op, i2) => {
            todo!()
        }
        (i1, op, WithDuration(i2, d2)) => {
            todo!()
        }
        (i1, op, i2) => {
            let mut i1 = VariableValue::from(i1);
            let mut i2 = VariableValue::from(i2);
            i1.compatible_cast(&mut i2, ctx);
            let res = match op {
                Add => i1.add(i2, ctx),
                Sub => i1.sub(i2, ctx),
                Mul => i1.mul(i2, ctx),
                Div => i1.div(i2, ctx),
                Rem => i1.rem(i2, ctx),
                Shl => {
                    i1 = i1.cast_as_integer(&ctx.clock, ctx.frame_len);
                    i2 = i2.cast_as_integer(&ctx.clock, ctx.frame_len);
                    i1 << i2
                }
                Shr => {
                    i1 = i1.cast_as_integer(&ctx.clock, ctx.frame_len);
                    i2 = i2.cast_as_integer(&ctx.clock, ctx.frame_len);
                    i1 >> i2
                }
                Pow => i1.pow(i2, ctx),
            };
            BoinxItem::from(res)
        }
    }
}