pub enum BoinxIdentQualif {
    LocalVar, SeqVar, EnvFunc
}

pub struct BoinxIdent(String, BoinxIdentQualif);

pub enum BoinxDuration {
    Relative(f64),
    Micros(f64),
    Semibeats(f64),
    Beats(f64)
}

pub struct BoinxCondition(Box<BoinxItem>, String, Box<BoinxItem>);

pub enum BoinxArithmeticOp {
    Add, Sub, Mul, Div, Rem, Shl, Shr, Pow
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
    Duration(BoinxDuration),
    Condition(BoinxCondition, Box<BoinxProg>, Box<BoinxProg>),
    Identity(BoinxIdent),
    SubProg(Box<BoinxProg>),
    Arithmetic(Box<BoinxItem>, BoinxArithmeticOp, Box<BoinxItem>),
}

impl BoinxItem {

    pub fn has_placeholders(&self) -> bool {
        match self {
            Self::Sequence(v) => v.iter().any(BoinxItem::has_placeholders),
            _ => false
        }
    }

}

pub enum BoinxCompoOp {
    Compose, Iterate, Each
}

pub struct BoinxCompo {
    pub item: BoinxItem,
    pub next: Option<(BoinxCompoOp, Box<BoinxCompo>)>
}

pub struct BoinxOutput {
    pub compo: BoinxCompo,
    pub device: Option<BoinxItem>,
    pub channel: Option<BoinxItem> 
}

pub enum BoinxStatement {
    Output(BoinxOutput),
    Assign(String, BoinxOutput),
}

pub type BoinxProg = Vec<BoinxStatement>;
