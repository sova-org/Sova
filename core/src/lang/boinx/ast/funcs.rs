use rand::seq::SliceRandom;

use crate::{
    clock::TimeSpan,
    vm::{
        EvaluationContext, 
        variable::VariableValue,
    },
    lang::boinx::ast::BoinxItem,
    log_warn,
};

fn unpack_if_one(mut args: Vec<BoinxItem>) -> Vec<BoinxItem> {
    use BoinxItem::*;
    if args.len() > 1 {
        return args;
    }
    match args.pop().unwrap() {
        Sequence(items) | Simultaneous(items) => items,
        a => vec![a],
    }
}

pub fn execute_boinx_function(
    ctx: &EvaluationContext,
    name: &str,
    mut args: Vec<BoinxItem>,
) -> BoinxItem {
    use BoinxItem::*;
    match name {
        "choice" => {
            args = unpack_if_one(args);
            let i = rand::random_range(0..args.len());
            args.remove(i)
        }
        "shuffle" => {
            args = unpack_if_one(args);
            args.shuffle(&mut rand::rng());
            Sequence(args)
        }
        "rev" => {
            args = unpack_if_one(args);
            args = args.into_iter().rev().collect();
            Sequence(args)
        }
        "range" => {
            let (i1, i2) = if args.len() >= 2 {
                let mut iter = args.into_iter();
                let a = VariableValue::from(iter.next().unwrap());
                let b = VariableValue::from(iter.next().unwrap());
                let a = a.as_integer(ctx.clock, ctx.frame_len);
                let b = b.as_integer(ctx.clock, ctx.frame_len);
                (a, b)
            } else {
                let a = VariableValue::from(args.pop().unwrap());
                let a = a.as_integer(ctx.clock, ctx.frame_len);
                (0, a)
            };
            Sequence((i1..i2).map(|i| Note(i)).collect())
        }
        "randrange" => {
            let (i1, i2) = if args.len() >= 2 {
                let mut iter = args.into_iter();
                let a = VariableValue::from(iter.next().unwrap());
                let b = VariableValue::from(iter.next().unwrap());
                let a = a.as_float(ctx.clock, ctx.frame_len);
                let b = b.as_float(ctx.clock, ctx.frame_len);
                (a, b)
            } else {
                let a = VariableValue::from(args.pop().unwrap());
                let a = a.as_float(ctx.clock, ctx.frame_len);
                (0.0, a)
            };
            Number(rand::random_range(i1..i2))
        }
        "irandrange" => {
            let (i1, i2) = if args.len() >= 2 {
                let mut iter = args.into_iter();
                let a = VariableValue::from(iter.next().unwrap());
                let b = VariableValue::from(iter.next().unwrap());
                let a = a.as_integer(ctx.clock, ctx.frame_len);
                let b = b.as_integer(ctx.clock, ctx.frame_len);
                (a, b)
            } else {
                let a = VariableValue::from(args.pop().unwrap());
                let a = a.as_integer(ctx.clock, ctx.frame_len);
                (0, a)
            };
            Note(rand::random_range(i1..i2))
        }
        "after" => {
            if args.len() > 1 {
                log_warn!("Too many arguments for 'after' function, taking only last !");
            }
            let dur = match args.pop().unwrap().unescape() {
                Duration(d) => d,
                Number(f) => TimeSpan::Frames(f),
                _ => {
                    log_warn!("Argument for 'after' is not a duration !");
                    TimeSpan::default()
                }
            };
            Sequence(vec![WithDuration(Box::new(Mute), dur), Placeholder])
        }
        _ => {
            log_warn!("Boinx function '{name}' does not exist !");
            BoinxItem::Mute
        }
    }
}
