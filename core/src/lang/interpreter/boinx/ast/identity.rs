use std::{collections::BTreeSet, fmt::Display};

use crate::{clock::TimeSpan, lang::{evaluation_context::EvaluationContext, interpreter::boinx::ast::{BoinxArithmeticOp, BoinxCompo, BoinxItem}, variable::Variable}};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

pub fn env_func(name: &str, ctx: &EvaluationContext) -> BoinxItem {
    use BoinxItem::*;
    use BoinxArithmeticOp::*;
    match name {
        "maj" => Simultaneous(vec![
            Placeholder, 
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(4))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(7))),
        ]),
        "min" => Simultaneous(vec![
            Placeholder, 
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(3))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(7))),
        ]),
        "half" => Simultaneous(vec![
            WithDuration(Box::new(Placeholder), TimeSpan::Frames(0.5))
        ]),
        "stop" => Stop,
        "prev" => Previous,
        "beat" => Number(ctx.clock.beat()),
        "micros" => Duration(TimeSpan::Micros(ctx.clock.micros())),
        _ if name.starts_with("seq") => {
            let value = &name[3..];
            if let Ok(n) = value.parse::<usize>() {
                return Sequence(vec![Placeholder ; n]);
            }
            Mute
        }
        _ => Mute
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BoinxIdent(pub String, pub BoinxIdentQualif);

impl BoinxIdent {
    pub fn load_item(&self, ctx: &EvaluationContext, forbidden: &mut BTreeSet<BoinxIdent>) -> BoinxItem {
        if self.1 == BoinxIdentQualif::EnvFunc {
            return env_func(&self.0, ctx);
        }
        if forbidden.contains(self) {
            return BoinxItem::default();
        }
        let var = match &self.1 {
            BoinxIdentQualif::LocalVar => Variable::Instance(self.0.clone()),
            BoinxIdentQualif::SeqVar => Variable::Global(self.0.clone()),
            _ => unreachable!()
        };
        let obj = ctx.evaluate(&var);
        let compo = BoinxCompo::from(obj);
        forbidden.insert(self.clone());
        let res = compo.evaluate_vars(ctx, forbidden).flatten();
        forbidden.remove(self);
        res
    }

    pub fn set(&self, ctx: &mut EvaluationContext, value: BoinxCompo) {
        let var = match &self.1 {
            BoinxIdentQualif::LocalVar => Variable::Instance(self.0.clone()),
            BoinxIdentQualif::SeqVar => Variable::Global(self.0.clone()),
            BoinxIdentQualif::EnvFunc => return,
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