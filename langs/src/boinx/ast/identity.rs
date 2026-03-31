use std::{cell::LazyCell, collections::{BTreeMap, BTreeSet}, fmt::Display};

use crate::boinx::ast::{BoinxArithmeticOp, BoinxCompo, BoinxItem, funcs::ItemFunc};
use sova_core::{
    clock::TimeSpan, error::SovaError, vm::{EvaluationContext, language::{LanguageDocumentation, LanguageElement, ReferenceEntry}, variable::Variable}
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

const MACROS : LazyCell<BTreeMap<String, ItemFunc>> = LazyCell::new(|| {
    use BoinxItem::*;
    use BoinxArithmeticOp::*;
    let mut funcs = BTreeMap::new();
    funcs.insert("maj".to_owned(), ItemFunc::define(
        "Composable major chord",
        |_, _| Simultaneous(vec![
            Placeholder,
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(4, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(7, None)), None),
        ])
    ));
    funcs.insert("min".to_owned(), ItemFunc::define(
        "Composable minor chord",
        |_, _| Simultaneous(vec![
            Placeholder,
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(3, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(7, None)), None),
        ])
    ));
    funcs.insert("arpmaj".to_owned(), ItemFunc::define(
        "Composable major chord arpeggio",
        |_, _| Sequence(vec![
            Placeholder,
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(4, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(7, None)), None),
        ])
    ));
    funcs.insert("arpmin".to_owned(), ItemFunc::define(
        "Composable minor chord arpeggio",
        |_, _| Sequence(vec![
            Placeholder,
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(3, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(7, None)), None),
        ])
    ));
    funcs.insert("scalemaj".to_owned(), ItemFunc::define(
        "Composable major scale sequence",
        |_, _| Sequence(vec![
            Placeholder,
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(2, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(4, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(5, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(7, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(9, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(11, None)), None),
        ])
    ));
    funcs.insert("scalemin".to_owned(), ItemFunc::define(
        "Composable minor scale sequence",
        |_, _| Sequence(vec![
            Placeholder,
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(2, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(3, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(5, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(7, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(8, None)), None),
            Arithmetic(Box::new(Placeholder), Add, Box::new(Note(10, None)), None),
        ])
    ));
    funcs.insert("half".to_owned(), ItemFunc::define(
        "Composable sequence to only use half of length",
        |_, _| Simultaneous(vec![WithDuration(
            Box::new(Placeholder),
            TimeSpan::Frames(0.5),
        )])
    ));
    funcs.insert("stop".to_owned(), ItemFunc::define(
        "Stops execution of the current line",
        |_, _| Stop
    ));
    funcs.insert("prev".to_owned(), ItemFunc::define(
        "Evaluates to the previous output value of the line",
        |_, _| Previous
    ));
    funcs.insert("beat".to_owned(), ItemFunc::define(
        "Evaluates to the current beat",
        |ctx, _| Number(ctx.clock.beat(), None)
    ));
    funcs.insert("micros".to_owned(), ItemFunc::define(
        "Evaluates to the current microseconds date",
        |ctx, _| Duration(TimeSpan::Micros(ctx.logic_date))
    ));
    funcs.insert("beat".to_owned(), ItemFunc::define(
        "Evaluates to the current microseconds date",
        |ctx, _| Duration(TimeSpan::Micros(ctx.logic_date))
    ));
    funcs.insert("tempo".to_owned(), ItemFunc::define(
        "Evaluates to the current tempo",
        |ctx, _| Number(ctx.clock.tempo(), None)
    ));
    funcs.insert("quantum".to_owned(), ItemFunc::define(
        "Evaluates to the current quantum",
        |ctx, _| Number(ctx.clock.quantum(), None)
    ));
    funcs.insert("rand".to_owned(), ItemFunc::define(
        "Evaluates to a random float between 0 and 1",
        |_, _| Number(rand::random(), None)
    ));
    funcs.insert("irand".to_owned(), ItemFunc::define(
        "Evaluates to a random integer",
        |_, _| Note(rand::random(), None)
    ));
    funcs
});

pub fn execute_boinx_macro(
    ctx: &mut EvaluationContext,
    name: &str,
) -> BoinxItem {
    if let Some(func) = MACROS.get(name) {
        func.evaluate(ctx, Vec::new())
    } else {
        ctx.errors.throw(SovaError::from(ctx).message(
            format!("Boinx macro '{name}' does not exist !")
        ));
        BoinxItem::Mute
    }
}

pub fn add_macros_doc(doc : &mut LanguageDocumentation) {
    for (key, value) in MACROS.iter() {
        doc.reference.insert(
            LanguageElement::Word(key.clone()), 
            ReferenceEntry::new(value.doc.clone())
                .with_category("Macros")
                .with_example(format!("_{key}"))
        );
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
            return execute_boinx_macro(ctx, &self.0);
        }
        let var = self.get_var().unwrap();
        if forbidden.contains(self) || !ctx.has_var(&var) {
            return BoinxItem::Str(self.0.clone(), None);
        }
        let obj = ctx.evaluate(&var);
        let mut compo = BoinxCompo::from(obj);
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
