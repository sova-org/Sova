use std::{collections::HashMap, fmt::Display, iter};

use crate::{
    clock::TimeSpan,
    lang::{Program, evaluation_context::EvaluationContext, variable::VariableValue},
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
pub struct BoinxIdent(String, BoinxIdentQualif);

impl BoinxIdent {
    pub fn evaluate(&self, ctx : &EvaluationContext) -> BoinxItem {
        let var = match &self.1 {
            BoinxIdentQualif::LocalVar => Variable::Instance(self.0.clone()),
            BoinxIdentQualif::SeqVar => Variable::Global(self.0.clone()),
            BoinxIdentQualif::EnvFunc => todo!(),
        };
        let obj = ctx.evaluate(&var);
        BoinxItem::from(obj)
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

#[derive(Debug, Clone, Default)]
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
pub struct BoinxCondition(Box<BoinxItem>, BoinxConditionOp, Box<BoinxItem>);

impl BoinxCondition {
    pub fn has_slot(&self, ctx: &EvaluationContext) -> bool {
        self.0.has_slot(ctx) || self.2.has_slot(ctx)
    }
}

#[derive(Debug, Clone, Default)]
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
}

impl BoinxItem {
    pub fn has_slot(&self, ctx: &EvaluationContext) -> bool {
        match self {
            Self::Sequence(v) | Self::Simultaneous(v) => v.iter().any(|i| i.has_slot(ctx)),
            Self::Duration(_) | Self::Placeholder => true,
            Self::Condition(c, prog1, prog2) => {
                c.has_slot(ctx) || prog1.has_slot(ctx) || prog2.has_slot(ctx)
            }
            Self::Identity(x) => x.evaluate(ctx).has_slot(ctx),
            Self::SubProg(p) => p.has_slot(ctx),
            Self::Arithmetic(a, _, b) => a.has_slot(ctx) || b.has_slot(ctx),
            Self::WithDuration(i, _) => i.has_slot(ctx),
            _ => false,
        }
    }

    pub fn evaluate(&self, ctx : &EvaluationContext) -> BoinxItem {
        match self {
            Self::Identity(x) => x.evaluate(ctx),
            _ => self
        }
    }

    pub fn slots(&mut self) -> Box<dyn Iterator<Item = &mut BoinxItem>> {
        Box::new(match self {
            Self::Sequence(v) | Self::Simultaneous(v) => 
                v.iter().map(|i| i.slots()).flatten(),
            Self::Duration(_) | Self::Placeholder => iter::once(self),
            Self::Condition(c, prog1, prog2) => {
                c.slots().chain(prog1.slots()).chain(prog2.slots())
            }
            Self::Identity(x) => iter::empty(),
            Self::SubProg(p) => p.slots(),
            Self::Arithmetic(a, _, b) => 
                a.slots().chain(b.slots()),
            Self::WithDuration(i, _) => i.slots(),
            _ => iter::empty(),
        })
    }

    pub fn items(&self) -> Box<dyn Iterator<Item = &BoinxItem>> {
        match self {
            BoinxItem::Sequence(items) | BoinxItem::Simultaneous(items) 
                => items.iter(),
            _ => iter::once(self),
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
                    _ => BoinxItem::Mute,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
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
    pub fn has_slot(&self, ctx: &EvaluationContext) -> bool {
        self.item.has_slot(ctx)
    }

    pub fn slots(&mut self) -> Box<dyn Iterator<Item = &mut BoinxItem>> {
        self.item.slots()
    }

    pub fn flatten(&self, ctx: &EvaluationContext) -> BoinxItem {
        let Some((op, next)) = self.next else {
            return self.item.clone();
        };
        let item = self.item;
        let mut next = next.clone();
        match op {
            BoinxCompoOp::Compose => {
                for slot in next.slots() {
                    *slot = self.item.clone();
                }
            },
            BoinxCompoOp::Iterate => {
                let items = item.items().cycle();
                for slot in next.slots() {
                    if let Some(i) = items.next() {
                        *slot = i.clone();
                    };
                }
            },
            BoinxCompoOp::Each => todo!(),
        }
        next.flatten(ctx)
    }
}

impl From<VariableValue> for BoinxCompo {
    fn from(value: VariableValue) -> Self {
        let Map(mut map) = value else {
            return Self::default();
        };
        let Some(item) = map.remove("_item") else {
            return Self::default();
        };
        let item = BoinxItem::from(item);
        let mut compo = BoinxCompo { item, next: None };
        if let (Some(VariableValue::Str(op)), Some(next)) = (map.remove("_op"), map.remove("_next")) {
            let op = BoinxCompoOp::parse(op);
            let next = BoinxCompo::from(next);
            compo.next = Some((op, next));
        };
        compo
    }
}

impl From<BoinxCompo> for VariableValue {
    fn from(value: BoinxCompo) -> Self {
        let mut map : HashMap<String, VariableValue> = HashMap::new();
        let BoinxCompo { item, next } = value;
        map.insert("_item".to_owned(), item);
        if let Some((op, compo)) = next {
            map.insert("_op".to_owned(), op.to_string());
            map.insert("_next".to_owned(), (*compo).into());
        };
        map.into()
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

#[derive(Debug, Clone)]
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
            device, channel
        }
    }
}

impl From<BoinxOutput> for VariableValue {
    fn from(value: BoinxOutput) -> Self {
        let mut map : HashMap<String, VariableValue> = HashMap::new();
        map.insert("_out".to_owned(), self.compo.into());
        if let Some(item) = self.device {
            map.insert("_dev".to_owned(), item.into());
        };
        if let Some(item) = self.channel {
            map.insert("_chan".to_owned(), item.into());
        };
        map.into()
    }
}

#[derive(Debug, Clone)]
pub enum BoinxStatement {
    Output(BoinxOutput),
    Assign(String, BoinxOutput),
}

impl BoinxStatement {
    pub fn compo(&self) -> &BoinxCompo {
        match self {
            Output(out) => &out.compo,
            Assign(name, out) => &out.compo
        }
    }

    pub fn compo_mut(&mut self) -> &mut BoinxCompo {
        match self {
            Output(out) => &mut out.compo,
            Assign(name, out) => &mut out.compo
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
            Self::Assign(target, output.into())
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
                map.insert("_target".to_owned(), name.into());
            }
        };
        map.into()
    }
}

#[derive(Debug, Clone, Default)]
pub struct BoinxProg(Vec<BoinxStatement>);

impl BoinxProg {
    pub fn has_slot(&self, ctx: &EvaluationContext) -> bool {
        self.0.iter().any(|s| s.compo().has_slot(ctx))
    }
    
    pub fn slots(&mut self) -> Box<dyn Iterator<Item = &mut BoinxItem>> {
        Box::new(self.0.iter_mut().map(|s| {
            s.compo().slots()
        }).flatten())
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
