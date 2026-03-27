use std::sync::Arc;

use crate::cagire::ops::Op;
use crate::cagire::theory;
use crate::cagire::types::Span;

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
        "select" => Op::Pick,
        "sound" => Op::NewCmd,
        "." => Op::Emit,
        "rand" => Op::Rand(None),
        "exprand" => Op::ExpRand(None),
        "logrand" => Op::LogRand(None),
        "seed" => Op::Seed,
        "cycle" => Op::Cycle(None),
        "pcycle" => Op::PCycle(None),
        "choose" => Op::Choose(None),
        "bounce" => Op::Bounce(None),
        "pbounce" => Op::PBounce(None),
        "index" => Op::Index(None),
        "wchoose" => Op::WChoose(None),
        "every" => Op::Every(None),
        "except" => Op::Except(None),
        "every+" => Op::EveryOffset(None),
        "except+" => Op::ExceptOffset(None),
        "bjork" => Op::Bjork(None),
        "pbjork" => Op::PBjork(None),
        "chance" => Op::ChanceExec(None),
        "prob" => Op::ProbExec(None),
        "coin" => Op::Coin(None),
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
        "adsr" => Op::Adsr,
        "ad" => Op::Ad,
        "apply" => Op::Apply,
        "ramp" => Op::Ramp,
        "triangle" => Op::Triangle,
        "range" => Op::Range,
        "perlin" => Op::Perlin,
        "linmap" => Op::LinMap,
        "expmap" => Op::ExpMap,
        "map" => Op::Map,
        "loop" => Op::Loop,
        "oct" => Op::Oct,
        "clear" => Op::ClearCmd,
        ".." => Op::IntRange,
        ".," => Op::StepRange,
        "gen" => Op::Generate,
        "geom.." => Op::GeomRange,
        "euclid" => Op::Euclid,
        "euclidrot" => Op::EuclidRot,
        "div" => Op::Subdivide,
        "swing" => Op::Swing,
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
        "islide" => Op::ModSlide(3),
        "oslide" => Op::ModSlide(4),
        "pslide" => Op::ModSlide(5),
        "jit" => Op::ModRnd(0),
        "sjit" => Op::ModRnd(1),
        "drunk" => Op::ModRnd(2),
        "eadsr" | "env" => Op::ModEnv,
        "ead" => Op::ModEnvAd,
        "eadr" => Op::ModEnvAdr,
        "lpg" => Op::Lpg,
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

fn parse_french_note_name(name: &str) -> Option<i64> {
    // Normalize: lowercase and replace é with e
    let name = name.to_lowercase().replace('é', "e");
    let (base, rest) = if let Some(r) = name.strip_prefix("sol") {
        (7, r)
    } else if let Some(r) = name.strip_prefix("do") {
        (0, r)
    } else if let Some(r) = name.strip_prefix("re") {
        (2, r)
    } else if let Some(r) = name.strip_prefix("mi") {
        (4, r)
    } else if let Some(r) = name.strip_prefix("fa") {
        (5, r)
    } else if let Some(r) = name.strip_prefix("la") {
        (9, r)
    } else if let Some(r) = name.strip_prefix("si") {
        (11, r)
    } else if let Some(r) = name.strip_prefix("ti") {
        (11, r)
    } else if let Some(r) = name.strip_prefix("ut") {
        (0, r)
    } else {
        return None;
    };
    let bytes = rest.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let (modifier, octave_start) = match bytes[0] {
        b'#' => (1, 1),
        b'b' if bytes.len() > 1 && bytes[1].is_ascii_digit() => (-1, 1),
        b'0'..=b'9' => (0, 0),
        _ => return None,
    };
    let octave: i64 = rest[octave_start..].parse().ok()?;
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

fn push(ops: &mut Vec<Op>, spans: &mut Vec<Span>, op: Op, span: Span) {
    ops.push(op);
    spans.push(span);
}

pub(crate) fn compile_word(
    name: &str,
    span: Span,
    ops: &mut Vec<Op>,
    spans: &mut Vec<Span>,
    dict: &Dictionary,
) {
    match name {
        "linramp" => {
            push(ops, spans, Op::PushFloat(1.0), span);
            push(ops, spans, Op::Ramp, span);
            return;
        }
        "expramp" => {
            push(ops, spans, Op::PushFloat(3.0), span);
            push(ops, spans, Op::Ramp, span);
            return;
        }
        "logramp" => {
            push(ops, spans, Op::PushFloat(0.3), span);
            push(ops, spans, Op::Ramp, span);
            return;
        }
        _ => {}
    }

    if (name == "triad" || name == "seventh")
        && let Some(Op::Degree(pattern)) = ops.last()
    {
        let pattern = *pattern;
        ops.pop();
        spans.pop();
        push(ops, spans, if name == "triad" {
            Op::DiatonicTriad(pattern)
        } else {
            Op::DiatonicSeventh(pattern)
        }, span);
        return;
    }

    if let Some(pattern) = theory::lookup(name) {
        push(ops, spans, Op::Degree(pattern), span);
        return;
    }

    if let Some(intervals) = theory::chords::lookup(name) {
        push(ops, spans, Op::Chord(intervals), span);
        return;
    }

    if let Some(word) = lookup_word(name) {
        match &word.compile {
            Simple => {
                if let Some(mut op) = simple_op(word.name) {
                    op.attach_span(span);
                    push(ops, spans, op, span);
                }
            }
            Context(ctx) => push(ops, spans, Op::GetContext(ctx), span),
            Param => push(ops, spans, Op::SetParam(word.name), span),
            Probability(p) => {
                push(ops, spans, Op::PushFloat(*p), span);
                push(ops, spans, Op::ChanceExec(Some(span)), span);
            }
        }
        return;
    }

    if let Some(var_name) = name.strip_prefix('@').filter(|s| !s.is_empty()) {
        push(ops, spans, Op::PushStr(Arc::from(var_name)), span);
        push(ops, spans, Op::Get, span);
        return;
    }

    if let Some(var_name) = name.strip_prefix('!').filter(|s| !s.is_empty()) {
        push(ops, spans, Op::PushStr(Arc::from(var_name)), span);
        push(ops, spans, Op::Set, span);
        return;
    }

    if let Some(var_name) = name.strip_prefix(',').filter(|s| !s.is_empty()) {
        push(ops, spans, Op::PushStr(Arc::from(var_name)), span);
        push(ops, spans, Op::SetKeep, span);
        return;
    }

    if let Some(midi) = parse_note_name(name).or_else(|| parse_french_note_name(name)) {
        push(ops, spans, Op::PushInt(midi), span);
        return;
    }

    if let Some(semitones) = parse_interval(name) {
        push(ops, spans, Op::Dup, span);
        push(ops, spans, Op::PushInt(semitones), span);
        push(ops, spans, Op::Add, span);
        return;
    }

    if let Some(mut op) = simple_op(name) {
        op.attach_span(span);
        push(ops, spans, op, span);
        return;
    }

    if let Some(body) = dict.get(name) {
        for op in body.iter().cloned() {
            push(ops, spans, op, span);
        }
        return;
    }

    // Unknown words become strings (intentional language feature)
    push(ops, spans, Op::PushStr(Arc::from(name)), span);
}
