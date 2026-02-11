use std::collections::HashMap;

use rand::seq::SliceRandom;

use sova_core::{
    clock::TimeSpan, log_warn, 
    vm::{EvaluationContext, variable::{Variable, VariableValue}}
};

use crate::boinx::ast::{BoinxArithmeticOp, BoinxItem};

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

pub fn explode_map(ctx: &mut EvaluationContext, map: HashMap<String, BoinxItem>) -> BoinxItem {
    let mut items = None;
    for (key, value) in map.into_iter() {
        let mut to_add = value;
        for atom in to_add.atomic_items_mut() {
            let obj = std::mem::replace(atom, BoinxItem::Str(key.clone()));
            atom.receive(obj);
        }
        if let Some(i) = &mut items {
            let value = std::mem::take(i);
            *i = BoinxItem::Arithmetic(Box::new(to_add), BoinxArithmeticOp::Add, Box::new(value));
        } else {
            items = Some(to_add)
        }
    }
    items.unwrap_or_default().evaluate(ctx)
}

pub fn execute_boinx_function(
    ctx: &mut EvaluationContext,
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
                let a = a.as_integer(ctx);
                let b = b.as_integer(ctx);
                (a, b)
            } else {
                let a = VariableValue::from(args.pop().unwrap());
                let a = a.as_integer(ctx);
                (0, a)
            };
            Sequence((i1..i2).map(|i| Note(i)).collect())
        }
        "randrange" => {
            let (i1, i2) = if args.len() >= 2 {
                let mut iter = args.into_iter();
                let a = VariableValue::from(iter.next().unwrap());
                let b = VariableValue::from(iter.next().unwrap());
                let a = a.as_float(ctx);
                let b = b.as_float(ctx);
                (a, b)
            } else {
                let a = VariableValue::from(args.pop().unwrap());
                let a = a.as_float(ctx);
                (0.0, a)
            };
            Number(rand::random_range(i1..i2))
        }
        "irandrange" => {
            let (i1, i2) = if args.len() >= 2 {
                let mut iter = args.into_iter();
                let a = VariableValue::from(iter.next().unwrap());
                let b = VariableValue::from(iter.next().unwrap());
                let a = a.as_integer(ctx);
                let b = b.as_integer(ctx);
                (a, b)
            } else {
                let a = VariableValue::from(args.pop().unwrap());
                let a = a.as_integer(ctx);
                (0, a)
            };
            Note(rand::random_range(i1..i2))
        }
        "after" => {
            if args.len() > 1 {
                log_warn!("Too many arguments for 'after' function, taking only last !");
            }
            let dur = match args.pop().unwrap() {
                Duration(d) => d,
                Number(f) => TimeSpan::Frames(f),
                _ => {
                    log_warn!("Argument for 'after' is not a duration !");
                    TimeSpan::default()
                }
            };
            Sequence(vec![WithDuration(Box::new(Mute), dur), Placeholder])
        }
        "secs" => {
            if args.len() > 1 {
                log_warn!("Too many arguments for 'secs' function ! Taking only last !");
            }
            let dur = match args.pop().unwrap() {
                Duration(d) => d,
                Number(f) => TimeSpan::Frames(f),
                _ => {
                    log_warn!("Argument for 'after' is not a duration !");
                    TimeSpan::default()
                }
            };
            Number(dur.as_secs(ctx.clock, ctx.frame_len))
        }
        "len" => {
            if args.len() <= 1 {
                log_warn!("Too few arguments for 'len' ! Ignoring");
            }
            let dur = match args.pop().unwrap() {
                Duration(d) => d,
                Number(f) => TimeSpan::Frames(f),
                _ => {
                    log_warn!("Argument for 'len' is not a duration !");
                    TimeSpan::default()
                }
            };
            WithDuration(Box::new(Simultaneous(args)), dur)
        }
        "at" => {
            if args.len() <= 1 {
                log_warn!("Too few arguments for 'at' ! Ignoring");
            }
            let index = match args.pop().unwrap() {
                Note(i) => i as usize,
                Number(f) => f as usize,
                _ => {
                    log_warn!("Argument for 'at' is not an index !");
                    0
                }
            };
            let mut args = unpack_if_one(args);
            args.swap_remove(index % args.len())
        }
        "ex" => {
            if args.len() > 1 {
                log_warn!("Too many arguments for 'ex' function ! Taking last");
            }
            match args.pop().unwrap() {
                ArgMap(m) => explode_map(ctx, m),
                item => item
            }
        }
        "alt" => {
            let len = args.len();
            let var = format!("alt_{len}");
            let var = Variable::Frame(var); 
            if !ctx.has_var(&var) {
                ctx.redefine(&var, 0);
            }
            let index_value = ctx.evaluate(&var);
            let index = (index_value.as_integer(ctx) as usize) % len;
            ctx.redefine(&var, ((index + 1) % len) as i64);
            args.swap_remove(index)
        }
        _ => {
            log_warn!("Boinx function '{name}' does not exist !");
            BoinxItem::Mute
        }
    }
}
