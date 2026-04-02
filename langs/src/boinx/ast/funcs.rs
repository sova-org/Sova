use std::{
    cell::LazyCell,
    collections::{BTreeMap, HashMap},
};

use rand::seq::SliceRandom;

use sova_core::{
    clock::TimeSpan,
    error::SovaError,
    util::music::rhythm::{bitrhythm, euclid},
    vm::{
        EvaluationContext,
        control_asm::{DEFAULT_CHAN, DEFAULT_DEVICE},
        language::{LanguageDocumentation, LanguageElement, ReferenceEntry},
        variable::VariableValue,
    },
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
            let obj = std::mem::replace(atom, BoinxItem::Str(key.clone(), None));
            atom.receive(obj);
        }
        if let Some(i) = &mut items {
            let value = std::mem::take(i);
            *i = BoinxItem::Arithmetic(
                Box::new(to_add),
                BoinxArithmeticOp::Add,
                Box::new(value),
                None,
            );
        } else {
            items = Some(to_add)
        }
    }
    items.unwrap_or_default().evaluate(ctx)
}

pub fn audio_rate_modulation_string(
    ctx: &EvaluationContext,
    args: Vec<BoinxItem>,
    shape: &str,
    op: &str,
) -> String {
    let mut args = unpack_if_one(args);
    let n_args = args.len();
    let default_period = TimeSpan::Beats(ctx.frame_len).as_secs(ctx.clock, ctx.frame_len);
    let (start, end, period) = match n_args {
        1 => {
            let end = args.pop().unwrap();
            let end = VariableValue::from(end).as_float(ctx);
            (0.0, end, default_period)
        }
        2 => {
            let end = args.pop().unwrap();
            let end = VariableValue::from(end).as_float(ctx);
            let start = args.pop().unwrap();
            let start = VariableValue::from(start).as_float(ctx);
            (start, end, default_period)
        }
        3 => {
            let period = args.pop().unwrap();
            let period = match period {
                BoinxItem::Duration(time_span) => time_span.as_secs(ctx.clock, ctx.frame_len),
                x => VariableValue::from(x).as_float(ctx),
            };
            let end = args.pop().unwrap();
            let end = VariableValue::from(end).as_float(ctx);
            let start = args.pop().unwrap();
            let start = VariableValue::from(start).as_float(ctx);
            (start, end, period)
        }
        _ => {
            ctx.errors.throw(
                SovaError::from(ctx)
                    .message("Too many parameters for audio-rate modulation function ! Ignoring"),
            );
            (0.0, 1.0, default_period)
        }
    };
    format!("{start}{op}{end}:{period}{shape}")
}

pub type ItemGen = fn(&mut EvaluationContext, args: Vec<BoinxItem>) -> BoinxItem;
pub struct ItemFunc {
    pub doc: String,
    pub func: ItemGen,
}

impl ItemFunc {
    pub fn define(doc: &str, f: ItemGen) -> Self {
        Self {
            doc: doc.to_owned(),
            func: f,
        }
    }

    pub fn evaluate(&self, ctx: &mut EvaluationContext, args: Vec<BoinxItem>) -> BoinxItem {
        (self.func)(ctx, args)
    }
}

const FUNCS: LazyCell<BTreeMap<String, ItemFunc>> = LazyCell::new(|| {
    use BoinxItem::*;
    let mut funcs = BTreeMap::new();
    funcs.insert(
        "choice".to_owned(),
        ItemFunc::define(
            "Uniformly *samples* one item amongst the arguments",
            |ctx, mut args| {
                args = unpack_if_one(args);
                let len = args.len();
                if len == 0 {
                    SovaError::from(ctx).message("Trying to choose from empty vec ! Ignoring");
                    return Mute;
                }
                let i = rand::random_range(0..len);
                args.remove(i)
            },
        ),
    );
    funcs.insert(
        "shuffle".to_owned(),
        ItemFunc::define("Shuffles args into a sequence", |_, mut args| {
            args = unpack_if_one(args);
            args.shuffle(&mut rand::rng());
            Sequence(args)
        }),
    );
    funcs.insert(
        "rev".to_owned(),
        ItemFunc::define("Reverses args into a sequence", |_, mut args| {
            args = unpack_if_one(args);
            args = args.into_iter().rev().collect();
            Sequence(args)
        }),
    );
    funcs.insert("range".to_owned(), ItemFunc::define(
        "Generates the sequence of integers between the first and the second argument (or starting from 0 if there is only one argument)",
        |ctx, mut args| {
            let (i1, i2) = if args.len() >= 2 {
                let mut iter = args.into_iter();
                let a = VariableValue::from(iter.next().unwrap());
                let b = VariableValue::from(iter.next().unwrap());
                let a = a.as_integer(ctx);
                let b = b.as_integer(ctx);
                if a <= b { (a,b) } else { (b,a) }
            } else {
                let a = VariableValue::from(args.pop().unwrap());
                let a = a.as_integer(ctx);
                (0, a)
            };
            Sequence((i1..i2).map(|i| Note(i, None)).collect())
        }
    ));
    funcs.insert(
        "rand".to_owned(),
        ItemFunc::define(
            "Samples a random float in the range given",
            |ctx, mut args| {
                if args.len() > 2 {
                    ctx.errors.throw(SovaError::from(&*ctx).message(
                        "Too many arguments for 'randrange' function, taking only two last !",
                    ));
                }
                if args.len() >= 2 {
                    let mut iter = args.into_iter();
                    let a = VariableValue::from(iter.next().unwrap());
                    let b = VariableValue::from(iter.next().unwrap());
                    if a.is_float() || b.is_float() {
                        let a = a.as_float(ctx);
                        let b = b.as_float(ctx);
                        let (a, b) = if a <= b { (a, b) } else { (b, a) };
                        if a == b {
                            return Number(a, None);
                        }
                        Number(rand::random_range(a..b), None)
                    } else {
                        let a = a.as_integer(ctx);
                        let b = b.as_integer(ctx);
                        let (a, b) = if a <= b { (a, b) } else { (b, a) };
                        if a == b {
                            return Note(a, None);
                        }
                        Note(rand::random_range(a..b), None)
                    }
                } else {
                    let a = VariableValue::from(args.pop().unwrap());
                    if a.is_float() {
                        let a = a.as_float(ctx);
                        if a == 0.0 {
                            return Number(0.0, None);
                        }
                        Number(rand::random_range(0.0..a), None)
                    } else {
                        let a = a.as_integer(ctx);
                        if a == 0 {
                            return Note(0, None);
                        }
                        Note(rand::random_range(0..a), None)
                    }
                }
            },
        ),
    );
    funcs.insert("maybe".to_owned(), ItemFunc::define(
        "Returns the first argument with probability 0.5 (or using second argument as the probability), else returns a mute",
        |ctx, mut args| {
            if args.len() > 2 { 
                ctx.errors.throw(SovaError::from(&*ctx).message("Too many arguments for 'maybe' function, taking only two last !"));
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
    funcs.insert("after".to_owned(), ItemFunc::define(
        "Generates a composable sequence with a placeholder after specified duration", 
        |ctx, mut args| {
            if args.len() > 1 {
                ctx.errors.throw(SovaError::from(&*ctx).message("Too many arguments for 'after' function, taking only last !"));
            }
            let dur = match args.pop().unwrap() {
                Duration(d) => d,
                Number(f, _) => TimeSpan::Frames(f),
                _ => {
                    ctx.errors.throw(SovaError::from(&*ctx).message("Argument for 'after' is not a duration !"));
                    TimeSpan::default()
                }
            };
            Sequence(vec![WithDuration(Box::new(Mute), dur), Placeholder])
        }
    ));
    funcs.insert("secs".to_owned(), ItemFunc::define(
        "Converts specified duration into seconds", 
        |ctx, mut args| {
            if args.len() > 1 {
                ctx.errors.throw(SovaError::from(&*ctx).message("Too many arguments for 'secs' function ! Taking only last !"));
            }
            let dur = match args.pop().unwrap() {
                Duration(d) => d,
                _ => {
                    ctx.errors.throw(SovaError::from(&*ctx).message("Argument for 'secs' is not a duration !"));
                    TimeSpan::default()
                }
            };
            Number(dur.as_secs(ctx.clock, ctx.frame_len), None)
        }
    ));
    funcs.insert("frames".to_owned(), ItemFunc::define(
        "Converts specified duration into seconds", 
        |ctx, mut args| {
            if args.len() > 1 {
                ctx.errors.throw(SovaError::from(&*ctx).message("Too many arguments for 'secs' function ! Taking only last !"));
            }
            let dur = match args.pop().unwrap() {
                Duration(d) => {
                    let beats = d.as_beats(ctx.clock, ctx.frame_len);
                    let f_relative = beats / ctx.frame_len;
                    TimeSpan::Frames(f_relative)
                }
                Number(f, _) => TimeSpan::Frames(f),
                Note(i, _) => TimeSpan::Frames(i as f64),
                _ => {
                    ctx.errors.throw(SovaError::from(&*ctx).message("Argument for 'frames' is not a number !"));
                    TimeSpan::Frames(1.0)
                }
            };
            Duration(dur)
        }
    ));
    funcs.insert("len".to_owned(), ItemFunc::define(
        "Impose last argument as a duration for others", 
        |ctx, mut args| {
            if args.len() <= 1 {
                ctx.errors.throw(SovaError::from(&*ctx).message("Too few arguments for 'len' ! Ignoring"));
            }
            let dur = match args.pop().unwrap() {
                Duration(d) => d,
                Number(f, _) => TimeSpan::Frames(f),
                _ => {
                    ctx.errors.throw(SovaError::from(&*ctx).message("Argument for 'len' is not a duration !"));
                    TimeSpan::default()
                }
            };
            WithDuration(Box::new(Simultaneous(args)), dur)
        }
    ));
    funcs.insert("at".to_owned(), ItemFunc::define(
        "Extract the n-th element of the arguments (or container), where n is the last argument", 
        |ctx, mut args| {
            if args.len() <= 1 {
                ctx.errors.throw(SovaError::from(ctx).message("Too few arguments for 'at' ! Ignoring"));
                return Mute;
            }
            let index = match args.pop().unwrap() {
                Note(i, _) => i as usize,
                Number(f, _) => f as usize,
                _ => {
                    ctx.errors.throw(SovaError::from(ctx).message("Argument for 'at' is not an index !"));
                    0
                }
            };
            let mut args = unpack_if_one(args);
            args.swap_remove(index % args.len())
        }
    ));
    funcs.insert(
        "ex".to_owned(),
        ItemFunc::define(
            "Explode a map such that each value is a primitive type",
            |ctx, mut args| {
                if args.len() > 1 {
                    ctx.errors.throw(
                        SovaError::from(&*ctx)
                            .message("Too many arguments for 'ex' function ! Taking last"),
                    );
                }
                match args.pop().unwrap() {
                    ArgMap(m) => explode_map(ctx, m),
                    item => item,
                }
            },
        ),
    );
    funcs.insert("alt".to_owned(), ItemFunc::define(
        "Alternate between arguments according to the number of times the frame has been triggered", 
        |ctx, mut args| {
            let len = args.len();
            let index = ctx.frame_triggers % len;
            args.swap_remove(index)
        }
    ));
    funcs.insert(
        "seq".to_owned(),
        ItemFunc::define(
            "Generates an empty sequence of N elements, where N is the argument",
            |ctx, mut args| {
                if args.len() > 1 {
                    ctx.errors.throw(
                        SovaError::from(&*ctx)
                            .message("Too many arguments for 'seq' function ! Taking last"),
                    )
                }
                let value = args.pop().unwrap();
                let size = VariableValue::from(value).yield_integer(ctx) as usize;
                Sequence(vec![Placeholder; size])
            },
        ),
    );
    funcs.insert(
        "lfo".to_owned(),
        ItemFunc::define(
            "Audio rate modulation oscillator for Doux (sine)",
            |ctx, args| Str(audio_rate_modulation_string(ctx, args, "", "~"), None),
        ),
    );
    funcs.insert(
        "tlfo".to_owned(),
        ItemFunc::define(
            "Audio rate modulation oscillator for Doux (triangle)",
            |ctx, args| Str(audio_rate_modulation_string(ctx, args, "t", "~"), None),
        ),
    );
    funcs.insert(
        "wlfo".to_owned(),
        ItemFunc::define(
            "Audio rate modulation oscillator for Doux (saw)",
            |ctx, args| Str(audio_rate_modulation_string(ctx, args, "w", "~"), None),
        ),
    );
    funcs.insert(
        "qlfo".to_owned(),
        ItemFunc::define(
            "Audio rate modulation oscillator for Doux (square)",
            |ctx, args| Str(audio_rate_modulation_string(ctx, args, "q", "~"), None),
        ),
    );
    funcs.insert(
        "slide".to_owned(),
        ItemFunc::define(
            "Audio rate modulation slide for Doux (linear)",
            |ctx, args| Str(audio_rate_modulation_string(ctx, args, "", ">"), None),
        ),
    );
    funcs.insert(
        "expslide".to_owned(),
        ItemFunc::define(
            "Audio rate modulation slide for Doux (exponential)",
            |ctx, args| Str(audio_rate_modulation_string(ctx, args, "e", ">"), None),
        ),
    );
    funcs.insert(
        "sslide".to_owned(),
        ItemFunc::define(
            "Audio rate modulation oscillator for Doux (smooth)",
            |ctx, args| Str(audio_rate_modulation_string(ctx, args, "s", ">"), None),
        ),
    );
    funcs.insert(
        "easein".to_owned(),
        ItemFunc::define(
            "Audio rate modulation slide for Doux (linear)",
            |ctx, args| Str(audio_rate_modulation_string(ctx, args, "i", ">"), None),
        ),
    );
    funcs.insert(
        "easeout".to_owned(),
        ItemFunc::define(
            "Audio rate modulation slide for Doux (exponential)",
            |ctx, args| Str(audio_rate_modulation_string(ctx, args, "o", ">"), None),
        ),
    );
    funcs.insert(
        "stair".to_owned(),
        ItemFunc::define(
            "Audio rate modulation oscillator for Doux (smooth)",
            |ctx, args| Str(audio_rate_modulation_string(ctx, args, "p", ">"), None),
        ),
    );
    funcs.insert(
        "jit".to_owned(),
        ItemFunc::define(
            "Audio rate modulation randomization for Doux (sample & hold)",
            |ctx, args| Str(audio_rate_modulation_string(ctx, args, "", "?"), None),
        ),
    );
    funcs.insert(
        "sjit".to_owned(),
        ItemFunc::define(
            "Audio rate modulation slide for Doux (smooth interpolation)",
            |ctx, args| Str(audio_rate_modulation_string(ctx, args, "s", "?"), None),
        ),
    );
    funcs.insert(
        "drunk".to_owned(),
        ItemFunc::define(
            "Audio rate modulation oscillator for Doux (Random walk)",
            |ctx, args| Str(audio_rate_modulation_string(ctx, args, "d", "?"), None),
        ),
    );
    funcs.insert(
        "euclid".to_owned(),
        ItemFunc::define("Euclidian rhythm (k,n,(r))", |ctx, args| {
            let mut args = unpack_if_one(args);
            if args.len() == 1 {
                ctx.errors.throw(
                    SovaError::from(&*ctx).message("Not enough arguments for 'euclid' ! Ignoring"),
                );
                return Mute;
            }
            if args.len() > 3 {
                ctx.errors.throw(
                    SovaError::from(&*ctx)
                        .message("Too many arguments for 'euclid', taking three last !"),
                );
            }

            let r = if args.len() == 3 {
                VariableValue::from(args.pop().unwrap()).yield_integer(ctx) as usize
            } else {
                0
            };

            let n = VariableValue::from(args.pop().unwrap()).yield_integer(ctx) as usize;
            let k = VariableValue::from(args.pop().unwrap()).yield_integer(ctx) as usize;
            let k = std::cmp::min(k, n);

            Sequence(euclid(k, n, r))
        }),
    );
    funcs.insert(
        "bitrhythm".to_owned(),
        ItemFunc::define("Bit rhythm (n)", |ctx, args| {
            let mut args = unpack_if_one(args);
            if args.len() > 1 {
                ctx.errors.throw(
                    SovaError::from(&*ctx)
                        .message("Too many arguments for 'bitrhythm', taking last !"),
                );
            }

            let i = VariableValue::from(args.pop().unwrap()).yield_integer(ctx) as u64;

            Sequence(bitrhythm(i))
        }),
    );
    funcs.insert(
        "cc".to_owned(),
        ItemFunc::define("Access the value of a midi input CC (cc, device?, channel?)", |ctx, args| {
            let mut args = unpack_if_one(args);
            if args.len() == 1 {
                ctx.errors.throw(
                    SovaError::from(&*ctx).message("Not enough arguments for 'cc' ! Ignoring"),
                );
                return Mute;
            }
            if args.len() > 3 {
                ctx.errors.throw(
                    SovaError::from(&*ctx)
                        .message("Too many arguments for 'cc', taking three last !"),
                );
            }

            let channel = if args.len() >= 3 {
                VariableValue::from(args.pop().unwrap()).yield_integer(ctx) as i8
            } else {
                DEFAULT_CHAN as i8
            };
            let device_id = if args.len() >= 2 {
                VariableValue::from(args.pop().unwrap()).yield_integer(ctx) as usize
            } else {
                DEFAULT_DEVICE as usize
            };
            let cc = VariableValue::from(args.pop().unwrap()).yield_integer(ctx) as i8;

            Note(
                ctx.device_map
                    .get_input_cc(device_id, cc, channel)
                    .unwrap_or_default(),
                None,
            )
        }),
    );
    funcs.insert(
        "osc".to_owned(),
        ItemFunc::define("Access the value of a OSC input (route, id?, index?)", |ctx, args| {
            let mut args = unpack_if_one(args);
            if args.len() > 3 {
                ctx.errors.throw(
                    SovaError::from(&*ctx)
                        .message("Too many arguments for 'osc', taking three last !"),
                );
            }

            let index = if args.len() == 3 {
                VariableValue::from(args.pop().unwrap()).yield_integer(ctx) as usize
            } else {
                0
            };

            let device_id = if args.len() >= 2 {
                VariableValue::from(args.pop().unwrap()).yield_integer(ctx) as usize
            } else {
                DEFAULT_DEVICE as usize
            };

            let route = VariableValue::from(args.pop().unwrap()).as_str(ctx);

            ctx.device_map
                .get_osc_input_values(device_id, &route)
                .and_then(|mut v| if v.len() > index {
                    Some(v.swap_remove(index))
                } else {
                    None
                })
                .unwrap_or_default()
                .into()
        }),
    );
    funcs
});

pub fn execute_boinx_function(
    ctx: &mut EvaluationContext,
    name: &str,
    args: Vec<BoinxItem>,
) -> BoinxItem {
    if let Some(func) = FUNCS.get(name) {
        func.evaluate(ctx, args)
    } else {
        ctx.errors.throw(
            SovaError::from(ctx).message(format!("Boinx function '{name}' does not exist !")),
        );
        BoinxItem::Mute
    }
}

pub fn add_funcs_doc(doc: &mut LanguageDocumentation) {
    for (key, value) in FUNCS.iter() {
        doc.reference.insert(
            LanguageElement::Word(key.clone()),
            ReferenceEntry::new(value.doc.clone()).with_category("Functions"),
        );
    }
}
