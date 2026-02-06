use std::{collections::BTreeSet, fmt::Display};

use crate::boinx::ast::{BoinxArithmeticOp, BoinxCompo, BoinxItem};
use sova_core::{
    clock::TimeSpan,
    log_eprintln,
    vm::{EvaluationContext, variable::Variable},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BoinxIdentQualif {
    #[default]
    LocalVar,
    GlobalVar,
    LineVar,
    FrameVar,
    EnvFunc,
}

impl Display for BoinxIdentQualif {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoinxIdentQualif::LocalVar => write!(f, ""),
            BoinxIdentQualif::GlobalVar => write!(f, "$"),
            BoinxIdentQualif::LineVar => write!(f, "$l_"),
            BoinxIdentQualif::FrameVar => write!(f, "$f_"),
            BoinxIdentQualif::EnvFunc => write!(f, "_"),
        }
    }
}

pub fn env_func(name: &str, ctx: &EvaluationContext) -> BoinxItem {
    use BoinxArithmeticOp::*;
    use BoinxItem::*;
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
        "arpmaj" => Sequence(vec![
            Placeholder,
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(4))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(7))),
        ]),
        "arpmin" => Sequence(vec![
            Placeholder,
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(3))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(7))),
        ]),
        "scalemaj" => Sequence(vec![
            Placeholder,
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(2))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(4))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(5))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(7))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(9))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(11))),
        ]),
        "scalemin" => Sequence(vec![
            Placeholder,
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(2))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(3))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(5))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(7))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(8))),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(10))),
        ]),
        "half" => Simultaneous(vec![WithDuration(
            Box::new(Placeholder),
            TimeSpan::Frames(0.5),
        )]),
        "stop" => Stop,
        "prev" => Previous,
        "beat" => Number(ctx.clock.beat()),
        "micros" => Duration(TimeSpan::Micros(ctx.logic_date)),
        "tempo" => Number(ctx.clock.tempo()),
        "quantum" => Number(ctx.clock.quantum()),
        "rand" => Number(rand::random()),
        "irand" => Note(rand::random()),
        _ if name.starts_with("seq") => {
            let value = &name[3..];
            if let Ok(n) = value.parse::<usize>() {
                return Sequence(vec![Placeholder; n]);
            }
            Mute
        }
        _ => {
            log_eprintln!("Boinx macro not found: {name}");
            Mute
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BoinxIdent(pub String, pub BoinxIdentQualif);

impl BoinxIdent {
    pub fn load_item(
        &self,
        ctx: &mut EvaluationContext,
        forbidden: &mut BTreeSet<BoinxIdent>,
    ) -> BoinxItem {
        use BoinxIdentQualif::*;
        if self.1 == EnvFunc {
            return env_func(&self.0, ctx);
        }
        if forbidden.contains(self) {
            return BoinxItem::default();
        }
        let var = self.get_var().unwrap();
        let obj = ctx.evaluate(&var);
        let compo = BoinxCompo::from(obj);
        forbidden.insert(self.clone());
        let res = compo.evaluate_vars(ctx, forbidden).flatten();
        forbidden.remove(self);
        res
    }

    pub fn get_var(&self) -> Option<Variable> {
        use BoinxIdentQualif::*;
        match &self.1 {
            LocalVar => Some(Variable::Instance(self.0.clone())),
            FrameVar => Some(Variable::Frame(self.0.clone())),
            LineVar => Some(Variable::Line(self.0.clone())),
            GlobalVar => Some(Variable::Global(self.0.clone())),
            EnvFunc => None,
        }
    }

    pub fn set(&self, ctx: &mut EvaluationContext, value: BoinxCompo) {
        let Some(var) = self.get_var() else {
            return;
        };
        ctx.redefine(&var, value);
    }
}

impl From<String> for BoinxIdent {
    fn from(value: String) -> Self {
        use BoinxIdentQualif::*;
        for qualif in [EnvFunc, FrameVar, LineVar, GlobalVar] {
            let start = qualif.to_string();
            if value.starts_with(&start) {
                return BoinxIdent(value.split_at(start.len()).1.to_owned(), qualif);
            }
        }
        BoinxIdent(value, BoinxIdentQualif::LocalVar)
    }
}

impl Display for BoinxIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.1, self.0)
    }
}
