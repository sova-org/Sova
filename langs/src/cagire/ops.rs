use std::sync::Arc;

use super::types::Span;

#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    PushInt(i64),
    PushFloat(f64),
    PushStr(Arc<str>),
    PushSilence,
    Dup,
    Dupn,
    Drop,
    Swap,
    Over,
    Rot,
    Nip,
    Tuck,
    Dup2,
    Drop2,
    Swap2,
    Over2,
    Rev,
    Shuffle,
    Sort,
    RSort,
    Sum,
    Prod,
    Forget,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Concat,
    Neg,
    Abs,
    Floor,
    Ceil,
    Round,
    Min,
    Max,
    Pow,
    Sqrt,
    Sin,
    Cos,
    Log,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Not,
    Xor,
    Nand,
    Nor,
    IfElse,
    Pick,
    BranchIfZero(usize),
    Branch(usize),
    NewCmd,
    SetParam(&'static str),
    Emit,
    Get,
    Set,
    SetKeep,
    GetContext(&'static str),
    Rand(Option<Span>),
    ExpRand(Option<Span>),
    LogRand(Option<Span>),
    Seed,
    Cycle(Option<Span>),
    PCycle(Option<Span>),
    Choose(Option<Span>),
    Bounce(Option<Span>),
    WChoose(Option<Span>),
    ChanceExec(Option<Span>),
    ProbExec(Option<Span>),
    Coin(Option<Span>),
    Mtof,
    Ftom,
    Edo,
    BuildTuning,
    BuildScale,
    Mode,
    Deg,
    SetChord,
    PushScale(&'static [usize]),
    SetTempo,
    Every(Option<Span>),
    Bjork(Option<Span>),
    PBjork(Option<Span>),
    Quotation(Arc<[Op]>, Arc<[Span]>),
    When,
    Unless,
    Adsr,
    Ad,
    Apply,
    Ramp,
    Triangle,
    Range,
    Perlin,
    Ctl { shape: CtlShape, bipolar: bool },
    LinMap,
    ExpMap,
    Map,
    Loop,
    ClearCmd,
    SetSpeed,
    At,
    PatPush,
    PatRot,
    PatRev,
    PatInv,
    IntRange,
    StepRange,
    Generate,
    GeomRange,
    Euclid,
    EuclidRot,
    Subdivide,
    Swing,
    Times,
    ModLfo(u8),
    ModSlide(u8),
    ModSlew(u8),
    ModRnd(u8),
    ModEnv,
    ModEnvAd,
    ModEnvAdr,
    Lpg,
    GetMidiCC,
    GetOscIn,
    MidiClock,
    MidiStart,
    MidiStop,
    MidiContinue,
    PBounce(Option<Span>),
    Index(Option<Span>),
    Except(Option<Span>),
    EveryOffset(Option<Span>),
    ExceptOffset(Option<Span>),
    First(Option<Span>),
    After(Option<Span>),
    Once(Option<Span>),
    Mark,
    Count(Option<Span>),
    Rec,
    Overdub,
    Orec,
    Odub,
    Print,
    ExecuteFrame,
}

/// Control-rate LFO shape selector. Phase is `(freq * beat).fract()`.
/// `Ramp` consumes `(freq curve)` from the stack; the others consume `(freq)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtlShape {
    Sine,
    Triangle,
    Saw,
    Square,
    Ramp,
    Perlin,
    Noise,
    Sh,
}

impl Op {
    pub(crate) fn attach_span(&mut self, span: Span) {
        match self {
            Op::Rand(s)
            | Op::ExpRand(s)
            | Op::LogRand(s)
            | Op::Coin(s)
            | Op::Cycle(s)
            | Op::PCycle(s)
            | Op::Bounce(s)
            | Op::PBounce(s)
            | Op::Choose(s)
            | Op::WChoose(s)
            | Op::ChanceExec(s)
            | Op::ProbExec(s)
            | Op::Every(s)
            | Op::Except(s)
            | Op::EveryOffset(s)
            | Op::ExceptOffset(s)
            | Op::Bjork(s)
            | Op::PBjork(s)
            | Op::First(s)
            | Op::After(s)
            | Op::Once(s)
            | Op::Count(s)
            | Op::Index(s) => *s = Some(span),
            _ => {}
        }
    }
}
