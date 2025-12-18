use std::{
    collections::{BTreeSet, HashMap},
    fmt::Display,
};

use crate::lang::{
    evaluation_context::EvaluationContext,
    interpreter::boinx::ast::{BoinxIdent, BoinxItem},
    variable::VariableValue,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoinxCompoOp {
    #[default]
    Compose,
    Iterate,
    Each,
    Zip,
    SuperEach,
}

impl BoinxCompoOp {
    pub fn parse(txt: &str) -> Self {
        match txt {
            "|" => Self::Compose,
            "°" => Self::Iterate,
            "~" => Self::Each,
            "!" => Self::Zip,
            "#" => Self::SuperEach,
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
            Self::Zip => write!(f, "!"),
            Self::SuperEach => write!(f, "#"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoinxCompo {
    pub item: BoinxItem,
    pub next: Option<(BoinxCompoOp, Box<BoinxCompo>)>,
}

impl BoinxCompo {
    pub fn slots<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut BoinxItem> + 'a> {
        self.item.slots()
    }

    /// Evaluates all identitifiers in the compo
    pub fn evaluate_vars(
        &self,
        ctx: &EvaluationContext,
        forbidden: &mut BTreeSet<BoinxIdent>,
    ) -> BoinxCompo {
        let mut compo = BoinxCompo {
            item: self.item.evaluate_vars(ctx, forbidden),
            next: None,
        };
        if let Some((op, next)) = &self.next {
            compo.next = Some((*op, Box::new(next.evaluate_vars(ctx, forbidden))));
        };
        compo
    }

    pub fn has_vars(&self) -> bool {
        self.item.has_vars()
            || self
                .next
                .as_ref()
                .map(|(_, next)| next.has_vars())
                .unwrap_or_default()
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
            BoinxCompoOp::SuperEach => {
                for i in item.atomic_items_mut() {
                    let mut n = next.item.clone();
                    for slot in n.slots() {
                        slot.receive(i.clone());
                    }
                    *i = n;
                }
                next.item = item;
            }
            BoinxCompoOp::Zip => {
                let mut items = item.items();
                for n_item in next.item.items_mut() {
                    let mut to_insert = &BoinxItem::default();
                    if let Some(i) = items.next() {
                        to_insert = i;
                    } else {
                        items = item.items();
                        if let Some(i) = items.next() {
                            to_insert = i;
                        };
                    }
                    for slot in n_item.slots() {
                        slot.receive(to_insert.clone());
                    }
                }
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
    pub fn yield_compiled(&self, ctx: &mut EvaluationContext) -> BoinxItem {
        let mut forbidden = BTreeSet::new();
        let flat = self.evaluate_vars(ctx, &mut forbidden).flatten();
        flat.evaluate(ctx)
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
            next: None,
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
