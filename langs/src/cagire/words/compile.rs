use std::sync::Arc;

use crate::cagire::ops::Op;
use crate::cagire::theory;

use super::{lookup_word, WordCompile::*};

pub(super) fn simple_op(name: &str) -> Option<Op> {
    Some(match name {
        "dup" => Op::Dup,
        "dupn" => Op::Dupn,
        "drop" => Op::Drop,
        "swap" => Op::Swap,
        "over" => Op::Over,
        "rot" => Op::Rot,
        "nip" => Op::Nip,
        "tuck" => Op::Tuck,
        "2dup" => Op::Dup2,
        "2drop" => Op::Drop2,
        "2swap" => Op::Swap2,
        "2over" => Op::Over2,
        "rev" => Op::Rev,
        "shuffle" => Op::Shuffle,
        "sort" => Op::Sort,
        "rsort" => Op::RSort,
        "sum" => Op::Sum,
        "prod" => Op::Prod,
        "+" => Op::Add,
        "-" => Op::Sub,
        "*" => Op::Mul,
        "/" => Op::Div,
        "mod" => Op::Mod,
        "neg" => Op::Neg,
        "abs" => Op::Abs,
        "floor" => Op::Floor,
        "ceil" => Op::Ceil,
        "round" => Op::Round,
        "min" => Op::Min,
        "max" => Op::Max,
        "pow" => Op::Pow,
        "sqrt" => Op::Sqrt,
        "sin" => Op::Sin,
        "cos" => Op::Cos,
        "log" => Op::Log,
        "=" => Op::Eq,
        "!=" => Op::Ne,
        "lt" => Op::Lt,
        "gt" => Op::Gt,
        "<=" => Op::Le,
        ">=" => Op::Ge,
        "and" => Op::And,
        "or" => Op::Or,
        "not" => Op::Not,
        "xor" => Op::Xor,
        "nand" => Op::Nand,
        "nor" => Op::Nor,
        "ifelse" => Op::IfElse,
        "pick" => Op::Pick,
        "sound" => Op::NewCmd,
        "." => Op::Emit,
        "rand" => Op::Rand,
        "exprand" => Op::ExpRand,
        "logrand" => Op::LogRand,
        "seed" => Op::Seed,
        "cycle" => Op::Cycle,
        "pcycle" => Op::PCycle,
        "choose" => Op::Choose,
        "bounce" => Op::Bounce,
        "pbounce" => Op::PBounce,
        "index" => Op::Index,
        "wchoose" => Op::WChoose,
        "every" => Op::Every,
        "except" => Op::Except,
        "every+" => Op::EveryOffset,
        "except+" => Op::ExceptOffset,
        "bjork" => Op::Bjork,
        "pbjork" => Op::PBjork,
        "chance" => Op::ChanceExec,
        "prob" => Op::ProbExec,
        "coin" => Op::Coin,
        "mtof" => Op::Mtof,
        "ftom" => Op::Ftom,
        "inv" => Op::Invert,
        "dinv" => Op::DownInvert,
        "drop2" => Op::VoiceDrop2,
        "drop3" => Op::VoiceDrop3,
        "tp" => Op::Transpose,
        "key!" => Op::SetKey,
        "all" => Op::EmitAll,
        "noall" => Op::ClearGlobal,
        "rec" => Op::Rec,
        "overdub" | "dub" => Op::Overdub,
        "orec" => Op::Orec,
        "odub" => Op::Odub,
        "?" => Op::When,
        "!?" => Op::Unless,
        "tempo!" => Op::SetTempo,
        "speed!" => Op::SetSpeed,
        "at" => Op::At,
        "arp" => Op::Arp,
        "adsr" => Op::Adsr,
        "ad" => Op::Ad,
        "apply" => Op::Apply,
        "ramp" => Op::Ramp,
        "triangle" => Op::Triangle,
        "range" => Op::Range,
        "perlin" => Op::Perlin,
        "chain" => Op::Chain,
        "loop" => Op::Loop,
        "oct" => Op::Oct,
        "clear" => Op::ClearCmd,
        ".." => Op::IntRange,
        ".," => Op::StepRange,
        "gen" => Op::Generate,
        "geom.." => Op::GeomRange,
        "euclid" => Op::Euclid,
        "euclidrot" => Op::EuclidRot,
        "times" => Op::Times,
        "ccval" => Op::GetMidiCC,
        "mclock" => Op::MidiClock,
        "mstart" => Op::MidiStart,
        "mstop" => Op::MidiStop,
        "mcont" => Op::MidiContinue,
        "forget" => Op::Forget,
        "print" => Op::Print,
        "lfo" => Op::ModLfo(0),
        "tlfo" => Op::ModLfo(1),
        "wlfo" => Op::ModLfo(2),
        "qlfo" => Op::ModLfo(3),
        "slide" => Op::ModSlide(0),
        "expslide" => Op::ModSlide(1),
        "sslide" => Op::ModSlide(2),
        "jit" => Op::ModRnd(0),
        "sjit" => Op::ModRnd(1),
        "drunk" => Op::ModRnd(2),
        "env" => Op::ModEnv,
        _ => return None,
    })
}

fn parse_note_name(name: &str) -> Option<i64> {
    let name = name.to_lowercase();
    let bytes = name.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let base = match bytes[0] {
        b'c' => 0,
        b'd' => 2,
        b'e' => 4,
        b'f' => 5,
        b'g' => 7,
        b'a' => 9,
        b'b' => 11,
        _ => return None,
    };
    let (modifier, octave_start) = match bytes[1] {
        b'#' | b's' => (1, 2),
        b'b' if bytes.len() > 2 && bytes[2].is_ascii_digit() => (-1, 2),
        b'0'..=b'9' => (0, 1),
        _ => return None,
    };
    let octave: i64 = name[octave_start..].parse().ok()?;
    if !(-1..=9).contains(&octave) {
        return None;
    }
    Some((octave + 1) * 12 + base + modifier)
}

fn parse_interval(name: &str) -> Option<i64> {
    Some(match name {
        "P1" | "unison" => 0,
        "m2" => 1, "M2" => 2,
        "m3" => 3, "M3" => 4,
        "P4" => 5,
        "aug4" | "dim5" | "tritone" => 6,
        "P5" => 7,
        "m6" => 8, "M6" => 9,
        "m7" => 10, "M7" => 11,
        "P8" => 12,
        "m9" => 13, "M9" => 14,
        "m10" => 15, "M10" => 16,
        "P11" => 17, "aug11" => 18,
        "P12" => 19,
        "m13" => 20, "M13" => 21,
        "m14" => 22, "M14" => 23,
        "P15" => 24,
        _ => return None,
    })
}

type Dictionary = std::collections::HashMap<String, Vec<Op>>;

pub(crate) fn compile_word(
    name: &str,
    ops: &mut Vec<Op>,
    dict: &Dictionary,
) -> bool {
    match name {
        "linramp" => {
            ops.push(Op::PushFloat(1.0));
            ops.push(Op::Ramp);
            return true;
        }
        "expramp" => {
            ops.push(Op::PushFloat(3.0));
            ops.push(Op::Ramp);
            return true;
        }
        "logramp" => {
            ops.push(Op::PushFloat(0.3));
            ops.push(Op::Ramp);
            return true;
        }
        _ => {}
    }

    if (name == "triad" || name == "seventh")
        && let Some(Op::Degree(pattern)) = ops.last()
    {
        let pattern = *pattern;
        ops.pop();
        ops.push(if name == "triad" {
            Op::DiatonicTriad(pattern)
        } else {
            Op::DiatonicSeventh(pattern)
        });
        return true;
    }

    if let Some(pattern) = theory::lookup(name) {
        ops.push(Op::Degree(pattern));
        return true;
    }

    if let Some(intervals) = theory::chords::lookup(name) {
        ops.push(Op::Chord(intervals));
        return true;
    }

    if let Some(word) = lookup_word(name) {
        match &word.compile {
            Simple => {
                if let Some(op) = simple_op(word.name) {
                    ops.push(op);
                }
            }
            Context(ctx) => ops.push(Op::GetContext(ctx)),
            Param => ops.push(Op::SetParam(word.name)),
            Probability(p) => {
                ops.push(Op::PushFloat(*p));
                ops.push(Op::ChanceExec);
            }
        }
        return true;
    }

    if let Some(var_name) = name.strip_prefix('@').filter(|s| !s.is_empty()) {
        ops.push(Op::PushStr(Arc::from(var_name)));
        ops.push(Op::Get);
        return true;
    }

    if let Some(var_name) = name.strip_prefix('!').filter(|s| !s.is_empty()) {
        ops.push(Op::PushStr(Arc::from(var_name)));
        ops.push(Op::Set);
        return true;
    }

    if let Some(var_name) = name.strip_prefix(',').filter(|s| !s.is_empty()) {
        ops.push(Op::PushStr(Arc::from(var_name)));
        ops.push(Op::SetKeep);
        return true;
    }

    if let Some(midi) = parse_note_name(name) {
        ops.push(Op::PushInt(midi));
        return true;
    }

    if let Some(semitones) = parse_interval(name) {
        ops.push(Op::Dup);
        ops.push(Op::PushInt(semitones));
        ops.push(Op::Add);
        return true;
    }

    if let Some(op) = simple_op(name) {
        ops.push(op);
        return true;
    }

    if let Some(body) = dict.get(name) {
        ops.extend(body.iter().cloned());
        return true;
    }

    // Unknown words become strings
    ops.push(Op::PushStr(Arc::from(name)));
    true
}
