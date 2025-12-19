use std::{
    cmp,
    collections::{BTreeSet, HashMap},
    iter,
};

use crate::{
    clock::{NEVER, SyncTime, TimeSpan},
    vm::{
        Program,
        EvaluationContext,
        variable::VariableValue,
    },
    lang::boinx::{
        BoinxPosition,
        ast::{
            BoinxArithmeticOp, BoinxCondition, BoinxConditionOp, BoinxIdent, BoinxProg,
            arithmetic_op, funcs::execute_boinx_function,
        },
    },
};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum BoinxItem {
    #[default]
    Mute,
    Placeholder,
    Stop,
    Previous,
    Note(i64),
    Number(f64),
    Sequence(Vec<BoinxItem>),
    Simultaneous(Vec<BoinxItem>),
    Duration(TimeSpan),
    Condition(BoinxCondition, Box<BoinxProg>, Box<BoinxProg>),
    Identity(BoinxIdent),
    SubProg(Box<BoinxProg>),
    Arithmetic(Box<BoinxItem>, BoinxArithmeticOp, Box<BoinxItem>),
    WithDuration(Box<BoinxItem>, TimeSpan),
    External(Program),
    Negative(Box<BoinxItem>),
    Str(String),
    ArgMap(HashMap<String, BoinxItem>),
    Escape(Box<BoinxItem>),
    Func(String, Vec<BoinxItem>),
}

impl BoinxItem {
    pub fn evaluate(&self, ctx: &mut EvaluationContext) -> BoinxItem {
        match self {
            Self::Identity(x) => x.load_item(ctx, &mut BTreeSet::new()).evaluate(ctx),
            Self::Placeholder => Self::Mute,
            Self::WithDuration(i, d) => {
                let sub_len = d.as_beats(ctx.clock, ctx.frame_len);
                let mut sub_ctx = ctx.with_len(sub_len);
                Self::WithDuration(Box::new(i.evaluate(&mut sub_ctx)), *d)
            }
            Self::Negative(i) => {
                let inner = i.evaluate(ctx);
                let mut value = VariableValue::from(inner);
                value = -value;
                BoinxItem::from(value)
            }
            Self::Escape(i) => i.evaluate(ctx),
            Self::Condition(c, p1, p2) => Self::SubProg(if c.evaluate(ctx).is_true(ctx) {
                p1.clone()
            } else {
                p2.clone()
            }),
            Self::Sequence(items) => {
                let slices = self.time_slices(ctx);
                let mut res = Vec::new();
                for (i, item) in items.iter().enumerate() {
                    let item = match item {
                        with if matches!(with, Self::WithDuration(_, _)) => with.evaluate(ctx),
                        other => {
                            let sub_len = ctx.clock.micros_to_beats(slices[i]);
                            let mut sub_ctx = ctx.with_len(sub_len);
                            other.evaluate(&mut sub_ctx)
                        }
                    };
                    res.push(item);
                }
                Self::Sequence(res)
            }
            Self::Simultaneous(items) => {
                Self::Simultaneous(items.iter().cloned().map(|i| i.evaluate(ctx)).collect())
            }
            Self::ArgMap(map) => {
                let mut evaluated = map.clone();
                for value in evaluated.values_mut() {
                    *value = value.evaluate(ctx);
                }
                Self::ArgMap(evaluated)
            }
            Self::Arithmetic(i1, op, i2) => {
                let i1 = i1.evaluate(ctx);
                let i2 = i2.evaluate(ctx);
                arithmetic_op(ctx, i1, *op, i2)
            }
            Self::Func(name, args) => {
                let args = args.iter().map(|i| i.evaluate(ctx)).collect();
                execute_boinx_function(ctx, &name, args)
            }
            _ => self.clone(),
        }
    }

    pub fn evaluate_vars(
        &self,
        ctx: &EvaluationContext,
        forbidden: &mut BTreeSet<BoinxIdent>,
    ) -> BoinxItem {
        match self {
            Self::Identity(x) => x.load_item(ctx, forbidden),
            Self::WithDuration(i, d) => {
                Self::WithDuration(Box::new(i.evaluate_vars(ctx, forbidden)), *d)
            }
            Self::Negative(i) => Self::Negative(Box::new(i.evaluate_vars(ctx, forbidden))),
            Self::Escape(i) => Self::Escape(Box::new(i.evaluate_vars(ctx, forbidden))),
            Self::Condition(c, p1, p2) => {
                Self::Condition(c.evaluate_vars(ctx, forbidden), p1.clone(), p2.clone())
            }
            Self::Sequence(items) => Self::Sequence(
                items
                    .iter()
                    .cloned()
                    .map(|i| i.evaluate_vars(ctx, forbidden))
                    .collect(),
            ),
            Self::Simultaneous(items) => Self::Simultaneous(
                items
                    .iter()
                    .cloned()
                    .map(|i| i.evaluate_vars(ctx, forbidden))
                    .collect(),
            ),
            Self::ArgMap(map) => {
                let mut evaluated = map.clone();
                for value in evaluated.values_mut() {
                    *value = value.evaluate_vars(ctx, forbidden);
                }
                Self::ArgMap(evaluated)
            }
            Self::Arithmetic(i1, op, i2) => Self::Arithmetic(
                Box::new(i1.evaluate_vars(ctx, forbidden)),
                *op,
                Box::new(i2.evaluate_vars(ctx, forbidden)),
            ),
            Self::Func(name, args) => {
                let args = args
                    .iter()
                    .map(|i| i.evaluate_vars(ctx, forbidden))
                    .collect();
                if name.starts_with("_") {
                    execute_boinx_function(ctx, &name[1..], args)
                } else {
                    Self::Func(name.clone(), args)
                }
            }
            _ => self.clone(),
        }
    }

    pub fn has_vars(&self) -> bool {
        match self {
            Self::Identity(_) => true,
            Self::WithDuration(i, _) | Self::Negative(i) | Self::Escape(i) => i.has_vars(),
            Self::Condition(c, _, _) => c.has_vars(),
            Self::Sequence(items) | Self::Simultaneous(items) => {
                items.iter().any(BoinxItem::has_vars)
            }
            Self::ArgMap(map) => map.values().any(BoinxItem::has_vars),
            Self::Arithmetic(i1, _, i2) => i1.has_vars() || i2.has_vars(),
            Self::Func(name, items) => {
                name.starts_with("_") || items.iter().any(BoinxItem::has_vars)
            }
            _ => false,
        }
    }

    /// Assuming self has been evaluated
    pub fn duration(&self) -> Option<TimeSpan> {
        match self {
            BoinxItem::WithDuration(_, time_span) => Some(*time_span),
            _ => None,
        }
    }

    pub fn unescape(self) -> BoinxItem {
        match self {
            BoinxItem::Escape(i) => *i,
            item => item,
        }
    }

    pub fn position(
        &self,
        ctx: &mut EvaluationContext,
        mut date: SyncTime,
    ) -> (BoinxPosition, SyncTime) {
        if ctx.clock.beats_to_micros(ctx.frame_len) <= date {
            return (BoinxPosition::Undefined, NEVER);
        }
        match self {
            BoinxItem::WithDuration(i, t) => {
                let sub_len = t.as_beats(ctx.clock, ctx.frame_len);
                let mut sub_ctx = ctx.with_len(sub_len);
                i.position(&mut sub_ctx, date)
            }
            BoinxItem::Sequence(vec) => {
                let slices = self.time_slices(ctx);
                for (i, item) in vec.iter().enumerate() {
                    let dur = slices[i];
                    if dur > date {
                        let (sub_pos, mut sub_rem) = match item {
                            with if matches!(with, BoinxItem::WithDuration(_, _)) => {
                                item.position(ctx, date)
                            }
                            item => {
                                let sub_len = ctx.clock.micros_to_beats(dur);
                                let mut sub_ctx = ctx.with_len(sub_len);
                                item.position(&mut sub_ctx, date)
                            }
                        };
                        sub_rem = cmp::min(sub_rem, dur - date);
                        return (BoinxPosition::At(i, Box::new(sub_pos)), sub_rem);
                    }
                    date -= dur;
                }
                (BoinxPosition::Undefined, NEVER)
            }
            BoinxItem::Simultaneous(vec) => {
                let mut rem = NEVER;
                let mut pos = Vec::new();
                for item in vec.iter() {
                    let (in_pos, in_rem) = item.position(ctx, date);
                    rem = cmp::min(rem, in_rem);
                    pos.push(in_pos);
                }
                (BoinxPosition::Parallel(pos), rem)
            }
            _ => {
                let micros_len = ctx.clock.beats_to_micros(ctx.frame_len);
                let rem = micros_len.saturating_sub(date);
                (BoinxPosition::This, rem)
            }
        }
    }

    pub fn at(
        &self,
        ctx: &mut EvaluationContext,
        position: BoinxPosition,
    ) -> Vec<(BoinxItem, TimeSpan)> {
        use BoinxPosition::*;
        match (self, position) {
            (BoinxItem::WithDuration(item, t), pos) => {
                let sub_len = t.as_beats(ctx.clock, ctx.frame_len);
                let mut sub_ctx = ctx.with_len(sub_len);
                item.at(&mut sub_ctx, pos)
            }
            (BoinxItem::Sequence(vec), BoinxPosition::At(i, inner)) => {
                let slices = self.time_slices(ctx);
                vec.get(i)
                    .map(|item| match item {
                        with if matches!(with, BoinxItem::WithDuration(_, _)) => {
                            with.at(ctx, *inner)
                        }
                        item => {
                            let sub_len = slices[i];
                            let sub_len = ctx.clock.micros_to_beats(sub_len);
                            let mut sub_ctx = ctx.with_len(sub_len);
                            item.at(&mut sub_ctx, *inner)
                        }
                    })
                    .unwrap_or_default()
            }
            (BoinxItem::Simultaneous(vec), BoinxPosition::Parallel(positions)) => vec
                .iter()
                .zip(positions.into_iter())
                .map(|(item, pos)| item.at(ctx, pos))
                .flatten()
                .collect(),
            (_, This) => vec![(self.clone(), TimeSpan::Beats(ctx.frame_len))],
            (_, Undefined) => Vec::new(),
            _ => Vec::new(),
        }
    }

    pub fn untimed_at(&self, position: BoinxPosition) -> Vec<BoinxItem> {
        use BoinxPosition::*;
        match (self, position) {
            (BoinxItem::WithDuration(item, _), pos) => item.untimed_at(pos),
            (BoinxItem::Sequence(vec), BoinxPosition::At(i, inner)) => vec
                .get(i)
                .map(|item| item.untimed_at(*inner))
                .unwrap_or_default(),
            (BoinxItem::Simultaneous(vec), BoinxPosition::Parallel(positions)) => vec
                .iter()
                .zip(positions.into_iter())
                .map(|(item, pos)| item.untimed_at(pos))
                .flatten()
                .collect(),
            (_, This) => vec![self.clone()],
            (_, Undefined) => Vec::new(),
            _ => Vec::new(),
        }
    }

    pub fn time_slices(&self, ctx: &EvaluationContext) -> Vec<SyncTime> {
        match self {
            BoinxItem::WithDuration(_, t) => vec![t.as_micros(ctx.clock, ctx.frame_len)],
            BoinxItem::Sequence(vec) => {
                let mut items_no_duration: BTreeSet<usize> = BTreeSet::new();
                let mut forced_duration: SyncTime = 0;
                let mut durations: Vec<SyncTime> = vec![0; vec.len()];

                for (i, item) in vec.iter().enumerate() {
                    if let Some(d) = item.duration() {
                        let duration = d.as_micros(&ctx.clock, ctx.frame_len);
                        forced_duration += duration;
                        durations[i] = duration;
                    } else {
                        items_no_duration.insert(i);
                    }
                }
                let to_share = ctx.clock.beats_to_micros(ctx.frame_len);
                let to_share = to_share.saturating_sub(forced_duration);
                let (part, mut rem_share) = if items_no_duration.is_empty() {
                    (0, 0)
                } else {
                    (
                        to_share / (items_no_duration.len() as u64),
                        to_share % (items_no_duration.len() as u64),
                    )
                };
                let mut slices = Vec::with_capacity(vec.len());
                for i in 0..vec.len() {
                    let mut dur = if items_no_duration.contains(&i) {
                        part
                    } else {
                        durations[i]
                    };
                    if rem_share > 0 {
                        dur += 1;
                        rem_share -= 1;
                    }
                    slices.push(dur);
                }
                slices
            }
            _ => vec![ctx.clock.beats_to_micros(ctx.frame_len)],
        }
    }

    pub fn slots<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut BoinxItem> + 'a> {
        match self {
            Self::Sequence(v) | Self::Simultaneous(v) | Self::Func(_, v) => {
                Box::new(v.iter_mut().map(|i| i.slots()).flatten())
            }
            Self::Duration(_) | Self::Number(_) | Self::Placeholder | Self::Str(_) => {
                Box::new(iter::once(self))
            }
            Self::Condition(c, prog1, prog2) => {
                Box::new(c.slots().chain(prog1.slots()).chain(prog2.slots()))
            }
            Self::ArgMap(map) => Box::new(map.values_mut().map(|i| i.slots()).flatten()),
            Self::Identity(_) => Box::new(iter::empty()),
            Self::SubProg(p) => Box::new(p.slots()),
            Self::Arithmetic(a, _, b) => Box::new(a.slots().chain(b.slots())),
            Self::WithDuration(i, _) => Box::new(i.slots()),
            Self::Negative(i) => Box::new(i.slots()),
            _ => Box::new(iter::empty()),
        }
    }

    pub fn receive(&mut self, other: BoinxItem) {
        match self {
            BoinxItem::Placeholder => *self = other,
            BoinxItem::Number(f) => {
                *self = BoinxItem::WithDuration(Box::new(other), TimeSpan::Frames(*f))
            }
            BoinxItem::Duration(d) => *self = BoinxItem::WithDuration(Box::new(other), *d),
            BoinxItem::Str(s) => {
                let mut value_map = HashMap::new();
                value_map.insert(s.clone(), other);
                *self = BoinxItem::ArgMap(value_map);
            }
            _ => (),
        }
    }

    pub fn items<'a>(&'a self) -> Box<dyn Iterator<Item = &'a BoinxItem> + 'a> {
        match self {
            BoinxItem::Sequence(items) | BoinxItem::Simultaneous(items) => Box::new(items.iter()),
            _ => Box::new(iter::once(self)),
        }
    }

    pub fn items_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut BoinxItem> + 'a> {
        match self {
            BoinxItem::Sequence(items) | BoinxItem::Simultaneous(items) => {
                Box::new(items.iter_mut())
            }
            _ => Box::new(iter::once(self)),
        }
    }

    pub fn atomic_items_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut BoinxItem> + 'a> {
        match self {
            BoinxItem::Sequence(items) | BoinxItem::Simultaneous(items) => Box::new(
                items
                    .iter_mut()
                    .map(|item| item.atomic_items_mut())
                    .flatten(),
            ),
            _ => Box::new(iter::once(self)),
        }
    }

    pub fn type_id(&self) -> i64 {
        // Avoid using discriminant to be stable between enum redefinitions
        // in future updates, and avoid unsafe casting.
        match self {
            BoinxItem::Mute => 0,
            BoinxItem::Placeholder => 1,
            BoinxItem::Stop => 2,
            BoinxItem::Previous => 3,
            BoinxItem::Note(_) => 4,
            BoinxItem::Number(_) => 4,
            BoinxItem::Sequence(_) => 6,
            BoinxItem::Simultaneous(_) => 7,
            BoinxItem::Duration(_) => 8,
            BoinxItem::Condition(_, _, _) => 9,
            BoinxItem::Identity(_) => 10,
            BoinxItem::SubProg(_) => 11,
            BoinxItem::Arithmetic(_, _, _) => 12,
            BoinxItem::WithDuration(_, _) => 13,
            BoinxItem::External(_) => 14,
            BoinxItem::Negative(_) => 15,
            BoinxItem::Str(_) => 16,
            BoinxItem::ArgMap(_) => 17,
            BoinxItem::Escape(_) => 18,
            BoinxItem::Func(_, _) => 19,
        }
    }

    fn generate_map(&self) -> HashMap<String, VariableValue> {
        let mut map = HashMap::new();
        map.insert("_type_id".to_owned(), self.type_id().into());
        map
    }
}

impl From<BoinxItem> for VariableValue {
    fn from(value: BoinxItem) -> Self {
        let mut map = value.generate_map();
        match value {
            BoinxItem::Mute | BoinxItem::Placeholder | BoinxItem::Stop | BoinxItem::Previous => {
                map.into()
            }
            BoinxItem::Note(i) => i.into(),
            BoinxItem::Number(f) => f.into(),
            BoinxItem::Str(s) => s.into(),
            BoinxItem::Sequence(items) => items
                .into_iter()
                .map(VariableValue::from)
                .collect::<Vec<VariableValue>>()
                .into(),
            BoinxItem::Simultaneous(items) => {
                let items: Vec<VariableValue> =
                    items.into_iter().map(VariableValue::from).collect();
                map.insert("_items".to_owned(), items.into());
                map.into()
            }
            BoinxItem::Duration(dur) => dur.into(),
            BoinxItem::Condition(cond, p1, p2) => {
                let BoinxCondition(i1, op, i2) = cond;
                map.insert("0".to_owned(), (*i1).into());
                map.insert("1".to_owned(), (*i2).into());
                map.insert("_op".to_owned(), op.to_string().into());
                map.insert("_if".to_owned(), (*p1).into());
                map.insert("_else".to_owned(), (*p2).into());
                map.into()
            }
            BoinxItem::Identity(ident) => {
                map.insert("_var".to_owned(), ident.to_string().into());
                map.into()
            }
            BoinxItem::SubProg(prog) => {
                map.insert("_prog".to_owned(), (*prog).into());
                map.into()
            }
            BoinxItem::Arithmetic(i1, op, i2) => {
                let op = op.to_string();
                map.insert("0".to_owned(), (*i1).into());
                map.insert("1".to_owned(), (*i2).into());
                map.insert("_op".to_owned(), op.into());
                map.into()
            }
            BoinxItem::WithDuration(boinx_item, dur) => {
                map.insert("_item".to_owned(), (*boinx_item).into());
                map.insert("_dur".to_owned(), dur.into());
                map.into()
            }
            BoinxItem::External(prog) => VariableValue::Func(prog),
            BoinxItem::Negative(i) | BoinxItem::Escape(i) => {
                map.insert("_item".to_owned(), (*i).into());
                map.into()
            }
            BoinxItem::ArgMap(map) => {
                let mut value_map = HashMap::new();
                for (key, value) in map {
                    value_map.insert(key, VariableValue::from(value));
                }
                value_map.into()
            }
            BoinxItem::Func(name, args) => {
                map.insert("_name".to_owned(), name.into());
                map.insert("_len".to_owned(), (args.len() as i64).into());
                for (i, item) in args.into_iter().enumerate() {
                    map.insert(i.to_string(), item.into());
                }
                map.into()
            }
        }
    }
}

impl From<VariableValue> for BoinxItem {
    fn from(value: VariableValue) -> Self {
        match value {
            VariableValue::Integer(i) => BoinxItem::Note(i),
            VariableValue::Float(f) => BoinxItem::Number(f),
            VariableValue::Decimal(s, p, q) => {
                BoinxItem::Number((s as f64) * (p as f64) / (q as f64))
            }
            VariableValue::Bool(b) => BoinxItem::Note(b as i64),
            VariableValue::Str(s) => BoinxItem::Str(s),
            VariableValue::Dur(time_span) => BoinxItem::Duration(time_span),
            VariableValue::Func(instructions) => BoinxItem::External(instructions),
            VariableValue::Vec(items) => {
                let items: Vec<BoinxItem> = items.into_iter().map(BoinxItem::from).collect();
                BoinxItem::Sequence(items)
            }
            VariableValue::Map(mut map) => {
                let Some(VariableValue::Integer(type_id)) = map.remove("_type_id") else {
                    let mut value_map = HashMap::new();
                    for (key, value) in map {
                        value_map.insert(key, BoinxItem::from(value));
                    }
                    return BoinxItem::ArgMap(value_map);
                };
                match type_id {
                    0 => BoinxItem::Mute,
                    1 => BoinxItem::Placeholder,
                    2 => BoinxItem::Stop,
                    3 => BoinxItem::Previous,
                    7 => {
                        let Some(VariableValue::Vec(items)) = map.remove("_items") else {
                            return BoinxItem::Mute;
                        };
                        let items: Vec<BoinxItem> =
                            items.into_iter().map(BoinxItem::from).collect();
                        BoinxItem::Simultaneous(items)
                    }
                    9 => {
                        let (Some(i1), Some(i2)) = (map.remove("0"), map.remove("1")) else {
                            return BoinxItem::Mute;
                        };
                        let (Some(p1), Some(p2)) = (map.remove("_if"), map.remove("_else")) else {
                            return BoinxItem::Mute;
                        };
                        let Some(VariableValue::Str(op)) = map.get("_op") else {
                            return BoinxItem::Mute;
                        };
                        let condition = BoinxCondition(
                            Box::new(i1.into()),
                            BoinxConditionOp::parse(op),
                            Box::new(i2.into()),
                        );
                        BoinxItem::Condition(condition, Box::new(p1.into()), Box::new(p2.into()))
                    }
                    10 => {
                        let Some(VariableValue::Str(s)) = map.remove("_var") else {
                            return BoinxItem::Mute;
                        };
                        BoinxItem::Identity(s.into())
                    }
                    11 => {
                        let Some(prog) = map.remove("_prog") else {
                            return BoinxItem::Mute;
                        };
                        BoinxItem::SubProg(Box::new(prog.into()))
                    }
                    12 => {
                        let (Some(i1), Some(i2)) = (map.remove("0"), map.remove("1")) else {
                            return BoinxItem::Mute;
                        };
                        let Some(VariableValue::Str(op)) = map.get("_op") else {
                            return BoinxItem::Mute;
                        };
                        BoinxItem::Arithmetic(
                            Box::new(i1.into()),
                            BoinxArithmeticOp::parse(op),
                            Box::new(i2.into()),
                        )
                    }
                    13 => {
                        let Some(item) = map.remove("_item") else {
                            return BoinxItem::Mute;
                        };
                        let Some(VariableValue::Dur(time_span)) = map.remove("_dur") else {
                            return BoinxItem::Mute;
                        };
                        BoinxItem::WithDuration(Box::new(item.into()), time_span)
                    }
                    15 => {
                        let Some(item) = map.remove("_item") else {
                            return BoinxItem::Mute;
                        };
                        BoinxItem::Negative(Box::new(item.into()))
                    }
                    18 => {
                        let Some(item) = map.remove("_item") else {
                            return BoinxItem::Mute;
                        };
                        BoinxItem::Escape(Box::new(item.into()))
                    }
                    19 => {
                        let Some(VariableValue::Str(name)) = map.remove("_name") else {
                            return BoinxItem::Mute;
                        };
                        let Some(VariableValue::Integer(len)) = map.remove("_len") else {
                            return BoinxItem::Mute;
                        };
                        let mut vec: Vec<BoinxItem> = Vec::with_capacity(len as usize);
                        for i in 0..len {
                            let index = i.to_string();
                            let Some(item) = map.remove(&index) else {
                                return BoinxItem::Mute;
                            };
                            vec.push(item.into());
                        }
                        BoinxItem::Func(name, vec)
                    }
                    _ => BoinxItem::Mute,
                }
            }
            _ => Self::default(),
        }
    }
}
