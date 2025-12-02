use std::collections::HashMap;

use crate::{
    clock::{SyncTime, TimeSpan},
    lang::{
        evaluation_context::EvaluationContext, interpreter::boinx::BoinxLine, variable::VariableValue
    }
};

mod compo;
mod identity;
mod boinx_item;
mod condition;
mod arithmetic;

pub mod funcs;

pub use compo::*;
pub use identity::*;
pub use boinx_item::*;
pub use condition::*;
pub use arithmetic::*;

#[derive(Debug, Clone, Default, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, Default, PartialEq)]
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
        let mut prog: Vec<BoinxStatement> = Vec::with_capacity(len as usize);
        for i in 0..len {
            let index = i.to_string();
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
            map.insert(i.to_string(), item.into());
        }
        map.into()
    }
}
