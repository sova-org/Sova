use std::{
    cmp, collections::HashMap, fmt::Display, iter, mem
};

use crate::{
    clock::{NEVER, SyncTime, TimeSpan},
    lang::{
        Program, evaluation_context::EvaluationContext, interpreter::boinx::BoinxLine, variable::{Variable, VariableValue}
    }, log_println,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum BoinxPosition {
    #[default]
    Undefined,
    This,
    At(usize, Box<BoinxPosition>),
    Parallel(Vec<BoinxPosition>),
}

impl BoinxPosition {

    pub fn diff(self, other: &BoinxPosition) -> BoinxPosition {
        if self == *other {
            return BoinxPosition::Undefined;
        }
        if mem::discriminant(&self) != mem::discriminant(other) {
            return other.clone();
        } 
        match (self, other) {
            (BoinxPosition::This, BoinxPosition::This) |
            (BoinxPosition::Undefined, BoinxPosition::Undefined) 
                => BoinxPosition::Undefined,
            (BoinxPosition::At(i, p1), BoinxPosition::At(j, p2)) => {
                if i != *j {
                    return other.clone();
                }
                let pos = p1.diff(p2);
                if matches!(pos, BoinxPosition::Undefined) {
                    BoinxPosition::Undefined
                } else {
                    BoinxPosition::At(i, Box::new(pos))
                }
            }
            (BoinxPosition::Parallel(p1), BoinxPosition::Parallel(p2)) => {
                let mut seen_defined = false;
                let mut pos = Vec::new();
                for (i, inner1) in p1.into_iter().enumerate() {
                    let d = inner1.diff(&p2[i]);
                    seen_defined |= !matches!(d, BoinxPosition::Undefined);
                    pos.push(d);
                }
                if pos.len() < p2.len() {
                    pos.extend_from_slice(&p2[pos.len()..]);
                    seen_defined = true;
                }
                if !seen_defined {
                    BoinxPosition::Undefined
                } else {
                    BoinxPosition::Parallel(pos)
                }
            },
            _ => unreachable!()
        }
    }

}

#[derive(Debug, Clone, Default)]
pub enum BoinxIdentQualif {
    #[default]
    LocalVar,
    SeqVar,
    EnvFunc,
}

impl Display for BoinxIdentQualif {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoinxIdentQualif::LocalVar => write!(f, ""),
            BoinxIdentQualif::SeqVar => write!(f, "§"),
            BoinxIdentQualif::EnvFunc => write!(f, "_"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoinxIdent(pub String, pub BoinxIdentQualif);

impl BoinxIdent {
    pub fn load_item(&self, ctx: &EvaluationContext) -> BoinxItem {
        let var = match &self.1 {
            BoinxIdentQualif::LocalVar => Variable::Instance(self.0.clone()),
            BoinxIdentQualif::SeqVar => Variable::Global(self.0.clone()),
            BoinxIdentQualif::EnvFunc => todo!(),
        };
        let obj = ctx.evaluate(&var);
        let compo = BoinxCompo::from(obj);
        compo.yield_compiled(ctx)
    }

    pub fn set(&self, ctx: &mut EvaluationContext, value: BoinxCompo) {
        let var = match &self.1 {
            BoinxIdentQualif::LocalVar => Variable::Instance(self.0.clone()),
            BoinxIdentQualif::SeqVar => Variable::Global(self.0.clone()),
            BoinxIdentQualif::EnvFunc => todo!(),
        };
        ctx.set_var(&var, value.into());
    }
}

impl From<String> for BoinxIdent {
    fn from(value: String) -> Self {
        if value.starts_with("_") {
            BoinxIdent(value.split_at(1).1.to_owned(), BoinxIdentQualif::EnvFunc)
        } else if value.starts_with("§") {
            BoinxIdent(value.split_at(1).1.to_owned(), BoinxIdentQualif::SeqVar)
        } else {
            BoinxIdent(value, BoinxIdentQualif::LocalVar)
        }
    }
}

impl Display for BoinxIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.1, self.0)
    }
}

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

#[derive(Debug, Clone)]
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

    pub fn evaluate(&self, ctx: &EvaluationContext) -> BoinxCondition {
        BoinxCondition(
            Box::new(self.0.evaluate(ctx)),
            self.1,
            Box::new(self.2.evaluate(ctx))
        )
    }

    pub fn evaluate_vars(&self, ctx: &EvaluationContext) -> BoinxCondition {
        BoinxCondition(
            Box::new(self.0.evaluate_vars(ctx)),
            self.1,
            Box::new(self.2.evaluate_vars(ctx))
        )
    }
}

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

#[derive(Debug, Clone, Default)]
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
    Str(String)
}

impl BoinxItem {

    pub fn evaluate(&self, ctx: &EvaluationContext) -> BoinxItem {
        match self {
            Self::Identity(x) => x.load_item(ctx).evaluate(ctx),
            Self::Placeholder => Self::Mute,
            Self::WithDuration(i, d) => {
                Self::WithDuration(Box::new(i.evaluate(ctx)), *d)
            }
            Self::Negative(i) => {
                let inner = i.evaluate(ctx);
                let mut value = VariableValue::from(inner);
                value = -value;
                BoinxItem::from(value)
            }
            Self::Condition(c, p1, p2) => Self::SubProg(
                if c.evaluate(ctx).is_true(ctx) {
                    p1.clone()
                } else {
                    p2.clone()
                }
            ),
            Self::Sequence(items) => {
                Self::Sequence(items.iter().cloned().map(|i| i.evaluate(ctx)).collect())
            }
            Self::Simultaneous(items) => {
                Self::Simultaneous(items.iter().cloned().map(|i| i.evaluate(ctx)).collect())
            }
            Self::Arithmetic(i1, op, i2) => {
                let mut i1 = VariableValue::from(i1.evaluate(ctx));
                let mut i2 = VariableValue::from(i2.evaluate(ctx));
                i1.compatible_cast(&mut i2, ctx);
                let res = match op {
                    BoinxArithmeticOp::Add => i1.add(i2, ctx),
                    BoinxArithmeticOp::Sub => i1.sub(i2, ctx),
                    BoinxArithmeticOp::Mul => i1.mul(i2, ctx),
                    BoinxArithmeticOp::Div => i1.div(i2, ctx),
                    BoinxArithmeticOp::Rem => i1.rem(i2, ctx),
                    BoinxArithmeticOp::Shl => {
                        i1 = i1.cast_as_integer(&ctx.clock, ctx.frame_len);
                        i2 = i2.cast_as_integer(&ctx.clock, ctx.frame_len);
                        i1 << i2
                    }
                    BoinxArithmeticOp::Shr => {
                        i1 = i1.cast_as_integer(&ctx.clock, ctx.frame_len);
                        i2 = i2.cast_as_integer(&ctx.clock, ctx.frame_len);
                        i1 >> i2
                    }
                    BoinxArithmeticOp::Pow => i1.pow(i2, ctx),
                };
                BoinxItem::from(res)
            }
            _ => self.clone(),
        }
    }

    pub fn evaluate_vars(&self, ctx: &EvaluationContext) -> BoinxItem {
        match self {
            Self::Identity(x) => x.load_item(ctx).evaluate_vars(ctx),
            Self::WithDuration(i, d) => {
                Self::WithDuration(Box::new(i.evaluate_vars(ctx)), *d)
            }
            Self::Negative(i) => {
                Self::Negative(Box::new(i.evaluate_vars(ctx)))
            }
            Self::Condition(c, p1, p2) => {
                Self::Condition(c.evaluate_vars(ctx), p1.clone(), p2.clone())
            },
            Self::Sequence(items) => {
                Self::Sequence(items.iter().cloned().map(|i| i.evaluate_vars(ctx)).collect())
            }
            Self::Simultaneous(items) => {
                Self::Simultaneous(items.iter().cloned().map(|i| i.evaluate_vars(ctx)).collect())
            }
            Self::Arithmetic(i1, op, i2) => {
                Self::Arithmetic(
                    Box::new(i1.evaluate_vars(ctx)), 
                    *op, 
                    Box::new(i2.evaluate_vars(ctx))
                )
            }
            _ => self.clone(),
        }
    }

    /// Assuming self has been evaluated
    pub fn duration(&self) -> Option<TimeSpan> {
        match self {
            BoinxItem::WithDuration(_, time_span) => Some(*time_span),
            _ => None,
        }
    }

    pub fn position(&self, ctx: &EvaluationContext, len: f64, mut date: SyncTime) 
        -> (BoinxPosition, SyncTime)
    {
        match self {
            BoinxItem::WithDuration(i, t) => {
                let sub_len = t.as_beats(ctx.clock, len);
                i.position(ctx, sub_len, date)
            }
            BoinxItem::Sequence(vec) => {
                let mut items_no_duration: Vec<usize> = Vec::new();
                let mut forced_duration: SyncTime = 0;
                let mut durations: Vec<SyncTime> = vec![0; vec.len()];

                for (i, item) in vec.iter().enumerate() {
                    if let Some(d) = item.duration() {
                        let duration = d.as_micros(&ctx.clock, len);
                        forced_duration += duration;
                        durations[i] = duration;
                    } else {
                        items_no_duration.push(i);
                    }
                }
                let to_share = ctx.clock.beats_to_micros(len);
                let to_share = to_share.saturating_sub(forced_duration);
                let part = to_share / (items_no_duration.len() as u64);
                for (i, item) in vec.iter().enumerate() {
                    let dur = if items_no_duration.contains(&i) {
                        part
                    } else {
                        durations[i]
                    };
                    if dur > date {
                        let sub_len = ctx.clock.micros_to_beats(dur);
                        let (sub_pos, sub_rem) = item.position(ctx, sub_len, date);
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
                    let (in_pos, in_rem) = item.position(ctx, len, date);
                    rem = cmp::min(rem, in_rem);
                    pos.push(in_pos);
                }
                (BoinxPosition::Parallel(pos), rem)
            }
            _ => {
                let micros_len = ctx.clock.beats_to_micros(len);
                let rem = micros_len.saturating_sub(date);
                (BoinxPosition::This, rem)
            }
        }
    }

    pub fn at(
        &self,
        ctx: &mut EvaluationContext,
        position: BoinxPosition,
        len: f64
    ) -> Vec<(BoinxItem, TimeSpan)> {
        use BoinxPosition::*;
        match (self, position) {
            (BoinxItem::WithDuration(item, t), pos) => {
                item.at(ctx, pos, t.as_beats(ctx.clock, len))
            }
            (BoinxItem::Sequence(vec), BoinxPosition::At(i, inner)) => {
                let mut items_no_duration: Vec<usize> = Vec::new();
                let mut forced_duration: SyncTime = 0;
                let mut durations: Vec<SyncTime> = vec![0; vec.len()];

                for (j, item) in vec.iter().enumerate() {
                    if let Some(d) = item.duration() {
                        let duration = d.as_micros(ctx.clock, len);
                        forced_duration += duration;
                        durations[j] = duration;
                    } else {
                        items_no_duration.push(j);
                    }
                }
                let to_share = ctx.clock.beats_to_micros(len);
                let to_share = to_share.saturating_sub(forced_duration);
                let part = to_share / (items_no_duration.len() as u64);
                let sub_len = if items_no_duration.contains(&i) {
                    part
                } else {
                    durations[i]
                };
                let sub_len = ctx.clock.micros_to_beats(sub_len);

                vec.get(i)
                    .map(|item| item.at(ctx, *inner, sub_len))
                    .unwrap_or_default()
            }
            (BoinxItem::Simultaneous(vec), BoinxPosition::Parallel(positions)) => {
                vec.iter().zip(positions.into_iter())
                    .map(|(item, pos)| item.at(ctx, pos, len))
                    .flatten().collect()
            }
            (_, This) => vec![(self.clone(), TimeSpan::Beats(len))],
            (_, Undefined) => Vec::new(),
            _ => Vec::new()
        }
    }

    pub fn slots<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut BoinxItem> + 'a> {
        match self {
            Self::Sequence(v) | Self::Simultaneous(v) => {
                Box::new(v.iter_mut().map(|i| i.slots()).flatten())
            }
            Self::Duration(_) | Self::Number(_) | Self::Placeholder 
                => Box::new(iter::once(self)),
            Self::Condition(c, prog1, prog2) => {
                Box::new(c.slots().chain(prog1.slots()).chain(prog2.slots()))
            }
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
            BoinxItem::Number(f) => 
                *self = BoinxItem::WithDuration(Box::new(other), TimeSpan::Frames(*f)),
            BoinxItem::Duration(d) => 
                *self = BoinxItem::WithDuration(Box::new(other), *d),
            _ => ()
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
            BoinxItem::Str(_) => 16
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
            BoinxItem::Sequence(items) | BoinxItem::Simultaneous(items) => {
                map.insert("_len".to_owned(), (items.len() as i64).into());
                for (i, item) in items.into_iter().enumerate() {
                    map.insert(format!("{}", i), item.into());
                }
                map.into()
            }
            BoinxItem::Duration(dur) => dur.into(),
            BoinxItem::Condition(cond, p1, p2) => {
                let BoinxCondition(i1, op, i2) = cond;
                map.insert("0".to_owned(), (*i1).into());
                map.insert("1".to_owned(), (*i2).into());
                map.insert("_op".to_owned(), format!("{}", op).into());
                map.insert("_if".to_owned(), (*p1).into());
                map.insert("_else".to_owned(), (*p2).into());
                map.into()
            }
            BoinxItem::Identity(ident) => {
                map.insert("_var".to_owned(), format!("{}", ident).into());
                map.into()
            }
            BoinxItem::SubProg(prog) => {
                map.insert("_prog".to_owned(), (*prog).into());
                map.into()
            }
            BoinxItem::Arithmetic(i1, op, i2) => {
                let op = format!("{}", op);
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
            BoinxItem::Negative(i) => {
                map.insert("0".to_owned(), (*i).into());
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
            VariableValue::Map(mut map) => {
                let Some(VariableValue::Integer(type_id)) = map.remove("_type_id") else {
                    return BoinxItem::Mute;
                };
                match type_id {
                    0 => BoinxItem::Mute,
                    1 => BoinxItem::Placeholder,
                    2 => BoinxItem::Stop,
                    3 => BoinxItem::Previous,
                    6 | 7 => {
                        let Some(VariableValue::Integer(len)) = map.remove("_len") else {
                            return BoinxItem::Mute;
                        };
                        let mut vec: Vec<BoinxItem> = Vec::new();
                        for i in 0..len {
                            let index = format!("{}", i);
                            let Some(item) = map.remove(&index) else {
                                return BoinxItem::Mute;
                            };
                            vec.push(item.into());
                        }
                        if type_id == 6 {
                            return BoinxItem::Sequence(vec);
                        } else {
                            return BoinxItem::Simultaneous(vec);
                        }
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
                        let Some(item) = map.remove("0") else {
                            return BoinxItem::Mute;
                        };
                        BoinxItem::Negative(Box::new(item.into()))
                    }
                    _ => BoinxItem::Mute,
                }
            },
            VariableValue::Blob(_) => Self::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoinxCompoOp {
    #[default]
    Compose,
    Iterate,
    Each,
}

impl BoinxCompoOp {
    pub fn parse(txt: &str) -> Self {
        match txt {
            "|" => Self::Compose,
            "°" => Self::Iterate,
            "~" => Self::Each,
            _ => Self::Compose,
        }
    }
}

impl Display for BoinxCompoOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compose => write!(f, "|"),
            Self::Iterate => write!(f, "°"),
            Self::Each => write!(f, "~"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoinxCompo {
    pub item: BoinxItem,
    pub next: Option<(BoinxCompoOp, Box<BoinxCompo>)>,
}

impl BoinxCompo {

    pub fn slots<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut BoinxItem> + 'a> {
        self.item.slots()
    }

    /// Evaluates all arithmetic expressions and identities in the compo
    pub fn evaluate_vars(&self, ctx: &EvaluationContext) -> BoinxCompo {
        let mut compo = BoinxCompo {
            item: self.item.evaluate_vars(ctx),
            next: None,
        };
        if let Some((op, next)) = &self.next {
            compo.next = Some((*op, Box::new(next.evaluate_vars(ctx))));
        };
        compo
    }

    /// Apply all composition operators to produce a single item
    /// ASSUMES the composition is pre-evaluated !
    pub fn flatten(self) -> BoinxItem {
        let mut item = self.item;
        let Some((op, mut next)) = self.next else {
            return item;
        };
        match op {
            BoinxCompoOp::Compose => {
                for slot in next.slots() {
                    slot.receive(item.clone());
                }
            }
            BoinxCompoOp::Iterate => {
                let mut items = item.items();
                for slot in next.slots() {
                    let mut to_insert = BoinxItem::default();
                    if let Some(i) = items.next() {
                        to_insert = i.clone();
                    } else {
                        items = item.items();
                        if let Some(i) = items.next() {
                            to_insert = i.clone();
                        };
                    }
                    slot.receive(to_insert);
                }
            }
            BoinxCompoOp::Each => {
                for i in item.items_mut() {
                    let mut n = next.item.clone();
                    for slot in n.slots() {
                        slot.receive(i.clone());
                    }
                    *i = n;
                }
                next.item = item;
            }
        }
        next.flatten()
    }

    pub fn chain(mut self, op: BoinxCompoOp, other: BoinxCompo) -> BoinxCompo {
        let mut placeholder = &mut self.next;
        while placeholder.is_some() {
            placeholder = &mut placeholder.as_mut().unwrap().1.next;
        }
        *placeholder = Some((op, Box::new(other)));
        self
    }

    /// Evaluates the composition, then flattens it into a single item
    pub fn yield_compiled(&self, ctx: &EvaluationContext) -> BoinxItem {
        self.evaluate_vars(ctx).flatten().evaluate(ctx)
    }

    pub fn extract(self) -> BoinxItem {
        self.item
    }

}

impl From<VariableValue> for BoinxCompo {
    fn from(value: VariableValue) -> Self {
        let VariableValue::Map(mut map) = value else {
            return Self::default();
        };
        let Some(item) = map.remove("_item") else {
            return Self::default();
        };
        let item = BoinxItem::from(item);
        let mut compo = BoinxCompo { item, next: None };
        if let (Some(VariableValue::Str(op)), Some(next)) = (map.remove("_op"), map.remove("_next")) {
            let op = BoinxCompoOp::parse(&op);
            let next = BoinxCompo::from(next);
            compo.next = Some((op, Box::new(next)));
        };
        compo
    }
}

impl From<BoinxCompo> for VariableValue {
    fn from(value: BoinxCompo) -> Self {
        let mut map: HashMap<String, VariableValue> = HashMap::new();
        let BoinxCompo { item, next } = value;
        map.insert("_item".to_owned(), item.into());
        if let Some((op, compo)) = next {
            map.insert("_op".to_owned(), op.to_string().into());
            map.insert("_next".to_owned(), (*compo).into());
        };
        map.into()
    }
}

impl From<BoinxItem> for BoinxCompo {
    fn from(value: BoinxItem) -> Self {
        BoinxCompo {
            item: value,
            next: None
        }
    }
}

impl Default for BoinxCompo {
    fn default() -> Self {
        BoinxCompo {
            item: BoinxItem::Mute,
            next: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BoinxOutput {
    pub compo: BoinxCompo,
    pub device: Option<BoinxItem>,
    pub channel: Option<BoinxItem>,
}

impl From<VariableValue> for BoinxOutput {
    fn from(value: VariableValue) -> Self {
        let VariableValue::Map(mut map) = value else {
            return Self::default();
        };
        let Some(output) = map.remove("_out") else {
            return Self::default();
        };
        let device = map.remove("_dev").map(|d| BoinxItem::from(d));
        let channel = map.remove("_chan").map(|c| BoinxItem::from(c));
        BoinxOutput {
            compo: output.into(),
            device,
            channel,
        }
    }
}

impl From<BoinxOutput> for VariableValue {
    fn from(value: BoinxOutput) -> Self {
        let mut map: HashMap<String, VariableValue> = HashMap::new();
        map.insert("_out".to_owned(), value.compo.into());
        if let Some(item) = value.device {
            map.insert("_dev".to_owned(), item.into());
        };
        if let Some(item) = value.channel {
            map.insert("_chan".to_owned(), item.into());
        };
        map.into()
    }
}

#[derive(Debug, Clone)]
pub enum BoinxStatement {
    Output(BoinxOutput),
    Assign(BoinxIdent, BoinxCompo),
}

impl BoinxStatement {
    pub fn compo(&self) -> &BoinxCompo {
        match self {
            Self::Output(out) => &out.compo,
            Self::Assign(_, out) => out,
        }
    }

    pub fn compo_mut(&mut self) -> &mut BoinxCompo {
        match self {
            Self::Output(out) => &mut out.compo,
            Self::Assign(_, out) => out,
        }
    }
}

impl Default for BoinxStatement {
    fn default() -> Self {
        let output = BoinxOutput {
            compo: BoinxCompo::default(),
            device: None,
            channel: None,
        };
        Self::Output(output)
    }
}

impl From<VariableValue> for BoinxStatement {
    fn from(value: VariableValue) -> Self {
        let VariableValue::Map(mut map) = value else {
            return Self::default();
        };
        let Some(output) = map.remove("_out") else {
            return Self::default();
        };
        if let Some(VariableValue::Str(target)) = map.remove("_target") {
            Self::Assign(target.into(), output.into())
        } else {
            Self::Output(output.into())
        }
    }
}

impl From<BoinxStatement> for VariableValue {
    fn from(value: BoinxStatement) -> Self {
        let mut map: HashMap<String, VariableValue> = HashMap::new();
        match value {
            BoinxStatement::Output(output) => {
                map.insert("_out".to_owned(), output.into());
            }
            BoinxStatement::Assign(name, output) => {
                map.insert("_out".to_owned(), output.into());
                map.insert("_target".to_owned(), name.to_string().into());
            }
        };
        map.into()
    }
}

#[derive(Debug, Clone, Default)]
pub struct BoinxProg(pub Vec<BoinxStatement>);

impl BoinxProg {

    pub fn slots<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut BoinxItem> + 'a> {
        Box::new(self.0.iter_mut().map(|s| s.compo_mut().slots()).flatten())
    }

    pub fn start(&self, at: SyncTime, time_span: TimeSpan, ctx: &mut EvaluationContext) -> Vec<BoinxLine> {
        let mut lines = Vec::new();
        for statement in self.0.iter() {
            match statement {
                BoinxStatement::Output(output) => {
                    lines.push(BoinxLine::new(at, time_span, output.clone()));
                },
                BoinxStatement::Assign(var, compo) => {
                    var.set(ctx, compo.clone());
                },
            }
        }
        lines
    }
 
}

impl From<VariableValue> for BoinxProg {
    fn from(value: VariableValue) -> Self {
        let VariableValue::Map(mut map) = value else {
            return Self::default();
        };
        let Some(VariableValue::Integer(len)) = map.remove("_len") else {
            return Self::default();
        };
        let mut prog: Vec<BoinxStatement> = Vec::new();
        for i in 0..len {
            let index = format!("{}", i);
            let Some(item) = map.remove(&index) else {
                return Self::default();
            };
            prog.push(item.into());
        }
        BoinxProg(prog)
    }
}

impl From<BoinxProg> for VariableValue {
    fn from(value: BoinxProg) -> Self {
        let mut map: HashMap<String, VariableValue> = HashMap::new();
        map.insert("_len".to_owned(), (value.0.len() as i64).into());
        for (i, item) in value.0.into_iter().enumerate() {
            map.insert(format!("{}", i), item.into());
        }
        map.into()
    }
}
