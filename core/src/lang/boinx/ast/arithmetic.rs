use std::{cmp, fmt::Display};

use crate::{clock::{NEVER, TimeSpan}, lang::boinx::ast::BoinxItem, vm::{EvaluationContext, variable::VariableValue,}
};

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

fn internal_vm_op(
    ctx: &EvaluationContext, 
    i1: BoinxItem, 
    op: BoinxArithmeticOp, 
    i2: BoinxItem
) -> BoinxItem {
    use BoinxArithmeticOp::*;
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

pub fn arithmetic_op(
    ctx: &mut EvaluationContext,
    i1: BoinxItem,
    op: BoinxArithmeticOp,
    i2: BoinxItem,
) -> BoinxItem {
    use BoinxItem::*;
    match (i1, op, i2) {
        (Stop, _, _) | (_, _, Stop) => {
            Stop
        }
        (Escape(e), op, i) => {
            let res = arithmetic_op(ctx, *e, op, i);
            Escape(Box::new(res.unescape()))
        }
        (i, op, Escape(e)) => {
            let res = arithmetic_op(ctx, i, op, *e);
            Escape(Box::new(res.unescape()))
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
        (i1, op, i2) => {
            let mut date = 0;
            let (mut pos1, mut dur1) = i1.position(ctx, date);
            let (mut pos2, mut dur2) = i2.position(ctx, date);
            let len = ctx.clock.beats_to_micros(ctx.frame_len);
            let mut res = Vec::new();

            while dur1 < NEVER || dur2 < NEVER {
                let dur = cmp::min(dur1, dur2);
                
                let items1 = i1.untimed_at(pos1);
                let items2 = i2.untimed_at(pos2);

                for item1 in items1 {
                    for item2 in items2.iter() {
                        let item = internal_vm_op(ctx, item1.clone(), op, item2.clone());
                        let item = WithDuration(Box::new(item), TimeSpan::Micros(dur));
                        res.push(item);
                    }
                }
                
                date += dur;
                (pos1, dur1) = i1.position(ctx, date);
                (pos2, dur2) = i2.position(ctx, date);
            }

            if res.len() == 1 {
                match res.pop().unwrap() {
                    WithDuration(item, ts) if ts.as_micros(ctx.clock, ctx.frame_len) == len => {
                        *item
                    }
                    item => item
                }
            } else {
                Sequence(res)
            }
        }
    }
}
