use std::{
    collections::HashMap,
    fmt::Display,
    iter,
};

use crate::{
    clock::{SyncTime, TimeSpan},
    lang::{
        evaluation_context::EvaluationContext, interpreter::boinx::BoinxLine, variable::{Variable, VariableValue}, Program
    },
};

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
    pub fn evaluate(&self, ctx: &EvaluationContext) -> BoinxItem {
        let var = match &self.1 {
            BoinxIdentQualif::LocalVar => Variable::Instance(self.0.clone()),
            BoinxIdentQualif::SeqVar => Variable::Global(self.0.clone()),
            BoinxIdentQualif::EnvFunc => todo!(),
        };
        let obj = ctx.evaluate(&var);
        BoinxItem::from(obj)
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
}

impl BoinxItem {

    pub fn evaluate(&self, ctx: &EvaluationContext) -> BoinxItem {
        match self {
            Self::Identity(x) => x.evaluate(ctx),
            Self::Placeholder => Self::Mute,
            Self::Condition(c, p1, p2) => Self::SubProg(if c.is_true(ctx) {
                p1.clone()
            } else {
                p2.clone()
            }),
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

    /// Assuming self has been evaluated
    pub fn duration(&self) -> Option<TimeSpan> {
        match self {
            BoinxItem::WithDuration(_, time_span) => Some(*time_span),
            _ => None,
        }
    }

    pub fn at(
        &self,
        ctx: &EvaluationContext,
        len: f64,
        mut date: SyncTime,
    ) -> Vec<(BoinxItem, TimeSpan)> {
        match self {
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
                        return item.at(ctx, sub_len, dur - date);
                    }
                    date -= dur;
                }
                return vec![(BoinxItem::default(), TimeSpan::Micros(date))];
            }
            BoinxItem::Simultaneous(vec) => vec
                .iter()
                .cloned()
                .map(|i| i.at(ctx, len, date))
                .flatten()
                .collect(),
            BoinxItem::WithDuration(boinx_item, time_span) => {
                vec![(BoinxItem::clone(boinx_item), time_span.clone())]
            }
            _ => vec![(self.clone(), TimeSpan::Beats(len))],
        }
    }

    pub fn slots<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut BoinxItem> + 'a> {
        match self {
            Self::Sequence(v) | Self::Simultaneous(v) => {
                Box::new(v.iter_mut().map(|i| i.slots()).flatten())
            }
            Self::Duration(_) | Self::Placeholder => Box::new(iter::once(self)),
            Self::Condition(c, prog1, prog2) => {
                Box::new(c.slots().chain(prog1.slots()).chain(prog2.slots()))
            }
            Self::Identity(_) => Box::new(iter::empty()),
            Self::SubProg(p) => Box::new(p.slots()),
            Self::Arithmetic(a, _, b) => Box::new(a.slots().chain(b.slots())),
            Self::WithDuration(i, _) => Box::new(i.slots()),
            _ => Box::new(iter::empty()),
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
            BoinxItem::Identity(ident) => format!("{}", ident).into(),
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
            VariableValue::Str(s) => BoinxItem::Identity(s.into()),
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
    pub fn evaluate(&self, ctx: &EvaluationContext) -> BoinxCompo {
        let mut compo = BoinxCompo {
            item: self.item.evaluate(ctx),
            next: None,
        };
        if let Some((op, next)) = &self.next {
            compo.next = Some((*op, Box::new(next.evaluate(ctx))));
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
                    *slot = item.clone();
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
                    *slot = to_insert;
                }
            }
            BoinxCompoOp::Each => {
                for i in item.items_mut() {
                    let mut n = next.item.clone();
                    for slot in n.slots() {
                        *slot = i.clone();
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
    pub fn yield_item(&self, ctx: &EvaluationContext) -> BoinxItem {
        self.evaluate(ctx).flatten()
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
        if let (Some(VariableValue::Str(op)), Some(next)) = (map.remove("_op"), map.remove("_next"))
        {
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
