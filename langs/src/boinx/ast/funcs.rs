use std::{cell::LazyCell, collections::{BTreeMap, HashMap}, sync::LazyLock};

use rand::seq::SliceRandom;

use sova_core::{
    clock::TimeSpan, log_warn, vm::{EvaluationContext, variable::VariableValue}
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

pub type ItemGen = fn(&EvaluationContext, args: Vec<BoinxItem>) -> BoinxItem;
pub struct ItemFunc {
    pub doc: String,
    pub func: ItemGen
}

impl ItemFunc {

    pub fn define(doc: &str, f: ItemGen) -> Self {
        Self {
            doc: doc.to_owned(),
            func: f
        }
    }

    pub fn evaluate(&self, ctx: &EvaluationContext, args: Vec<BoinxItem>) -> BoinxItem {
        (self.func)(ctx, args)
    }
    
}

const FUNCS : LazyCell<BTreeMap<String, ItemFunc>> = LazyCell::new(|| {
    use BoinxItem::*;
    let mut funcs = BTreeMap::new();
    funcs.insert("choice".to_owned(), ItemFunc::define(
        "Uniformly *samples* one item amongst the arguments",
        |_, mut args| {
            args = unpack_if_one(args);
            let i = rand::random_range(0..args.len());
            args.remove(i)
        }
    ));
    funcs.insert("shuffle".to_owned(), ItemFunc::define(
        "Shuffles args into a sequence",
        |_, mut args| {
            args = unpack_if_one(args);
            args.shuffle(&mut rand::rng());
            Sequence(args)
        }
    ));
    funcs.insert("rev".to_owned(), ItemFunc::define(
        "Reverses args into a sequence",
        |_, mut args| {
            args = unpack_if_one(args);
            args = args.into_iter().rev().collect();
            Sequence(args)
        }
    ));
    funcs.insert("range".to_owned(), ItemFunc::define(
        "Generates the sequence of integers between the first and the second argument (or starting from 0 if there is only one argument)",
        |ctx, mut args| {
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
    ));
    funcs.insert("randrange".to_owned(), ItemFunc::define(
        "Samples a random float in the range given",
        |ctx, mut args| {
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
    ));
    funcs.insert("irandrange".to_owned(), ItemFunc::define(
        "Samples a random int in the range given",
        |ctx, mut args| {
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
    ));
    funcs.insert("maybe".to_owned(), ItemFunc::define(
        "Returns the first argument with probability 0.5 (or using second argument as the probability), else returns a mute",
        |ctx, mut args| {
            if args.len() > 2 { 
                log_warn!("Too many arguments for 'maybe' function, taking only two last !");
            }
            let prob = if args.len() > 1 { 
                VariableValue::from(args.pop().unwrap()).as_float(ctx)
            } else {
                0.5
            };
            let item = args.pop().unwrap();
            if rand::random_bool(prob) {
                item
            } else {
                Mute
            }
        }
    ));
    funcs
});

pub fn execute_boinx_function(
    ctx: &mut EvaluationContext,
    name: &str,
    mut args: Vec<BoinxItem>,
) -> BoinxItem {
    use BoinxItem::*;
    match name {
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
            let index = ctx.frame_triggers % len;
            args.swap_remove(index)
        }
        _ => {
            log_warn!("Boinx function '{name}' does not exist !");
            BoinxItem::Mute
        }
    }
}
