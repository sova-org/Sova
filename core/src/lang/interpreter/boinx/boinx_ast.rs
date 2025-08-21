use std::fmt::Display;

use crate::{
    clock::TimeSpan,
    lang::{Program, evaluation_context::EvaluationContext, variable::VariableValue},
};

pub enum BoinxIdentQualif {
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

pub struct BoinxIdent(String, BoinxIdentQualif);

impl From<String> for BoinxIdent {
    fn from(value: String) -> Self {
        todo!()
    }
}

impl Display for BoinxIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.1, self.0)
    }
}

pub enum BoinxConditionOp {
    Less,
    LessEq,
    Equal,
    NotEqual,
    GreaterEq,
    Greater,
}

pub struct BoinxCondition(Box<BoinxItem>, BoinxConditionOp, Box<BoinxItem>);

impl BoinxCondition {
    pub fn has_slot(&self, ctx: &EvaluationContext) -> bool {
        self.0.has_slot(ctx) || self.2.has_slot(ctx)
    }
}

pub enum BoinxArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    Pow,
}

pub enum BoinxItem {
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
            Self::Identity(x) => {
                todo!()
            }
            Self::SubProg(p) => p.has_slot(ctx),
            Self::Arithmetic(a, _, b) => a.has_slot(ctx) || b.has_slot(ctx),
            Self::WithDuration(i, _) => i.has_slot(ctx),
            _ => false,
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
}

impl From<BoinxItem> for VariableValue {
    fn from(value: BoinxItem) -> Self {
        match value {
            BoinxItem::Mute => todo!(),
            BoinxItem::Placeholder => todo!(),
            BoinxItem::Stop => todo!(),
            BoinxItem::Previous => todo!(),
            BoinxItem::Note(_) => todo!(),
            BoinxItem::Number(_) => todo!(),
            BoinxItem::Sequence(boinx_items) => todo!(),
            BoinxItem::Simultaneous(boinx_items) => todo!(),
            BoinxItem::Duration(boinx_duration) => todo!(),
            BoinxItem::Condition(boinx_condition, boinx_prog, boinx_prog1) => todo!(),
            BoinxItem::Identity(boinx_ident) => todo!(),
            BoinxItem::SubProg(boinx_prog) => todo!(),
            BoinxItem::Arithmetic(boinx_item, boinx_arithmetic_op, boinx_item1) => todo!(),
            BoinxItem::WithDuration(boinx_item, boinx_duration) => todo!(),
            BoinxItem::External(instructions) => todo!(),
        }
    }
}

impl From<VariableValue> for BoinxItem {
    fn from(value: VariableValue) -> Self {
        match value {
            VariableValue::Integer(i) => BoinxItem::Note(i),
            VariableValue::Float(f) => BoinxItem::Number(f),
            VariableValue::Decimal(_, _, _) => todo!(),
            VariableValue::Bool(b) => BoinxItem::Note(b as i64),
            VariableValue::Str(s) => BoinxItem::Identity(s.into()),
            VariableValue::Dur(time_span) => BoinxItem::Duration(time_span),
            VariableValue::Func(instructions) => BoinxItem::External(instructions),
            VariableValue::Map(map) => {
                let Some(VariableValue::Integer(type_id)) = map.get("type_id") else {
                    return BoinxItem::Mute;
                };
                match type_id {
                    0 => BoinxItem::Mute,
                    1 => BoinxItem::Placeholder,
                    2 => BoinxItem::Stop,
                    3 => BoinxItem::Previous,
                    6 => {}
                    7 => {}
                    9 => {}
                    _ => BoinxItem::Mute,
                }
            }
        }
    }
}

pub enum BoinxCompoOp {
    Compose,
    Iterate,
    Each,
}

pub struct BoinxCompo {
    pub item: BoinxItem,
    pub next: Option<(BoinxCompoOp, Box<BoinxCompo>)>,
}

pub struct BoinxOutput {
    pub compo: BoinxCompo,
    pub device: Option<BoinxItem>,
    pub channel: Option<BoinxItem>,
}

pub enum BoinxStatement {
    Output(BoinxOutput),
    Assign(String, BoinxOutput),
}

pub struct BoinxProg(Vec<BoinxStatement>);

impl BoinxProg {
    pub fn has_slot(&self, ctx: &EvaluationContext) -> bool {
        todo!()
    }
}
