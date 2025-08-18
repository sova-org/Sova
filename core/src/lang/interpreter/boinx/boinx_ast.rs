pub enum BoinxDuration {
    Relative(f64),
    Micros(f64),
    Semibeats(f64),
    Beats(f64)
}

pub struct BoinxCondition(Box<BoinxItem>, String, Box<BoinxItem>);

pub struct BoinxIfElse(BoinxCondition, Box<BoinxProg>, Box<BoinxProg>);

pub enum BoinxArithemicOp {
    Add, Sub, Mul, Div, Rem, Shl, Shr, Pow
}
pub struct BoinxArithmetic(Box<BoinxItem>, BoinxArithmeticOp, Box<BoinxItem>);

pub enum BoinxItem {
    Sequence(Vec<BoinxItem>),
    Simultaneous(Vec<BoinxItem>),
    Duration(BoinxDuration),
    Condition(BoinxIfElse),
    Identity(String),
    SubProg(Box<BoinxProg>),
    Arithmetic(BoinxArithmetic),
    Placeholder,
    Mute,
    Stop,
    Previous,
    Note(i64),
    Number(f64),
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

pub struct BoinxAssign {
    pub var: String,
    pub value: BoinxOutput
}

pub enum BoinxStatement {
    Output(BoinxCompo, Option<String>, Option<String>),
    Assign(String, BoinxOutput),
}

pub type BoinxProg = Vec<BoinxStatement>;
