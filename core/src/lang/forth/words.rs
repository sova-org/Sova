use std::collections::HashMap;

use crate::vm::event::ConcreteEvent;

use super::types::{BuiltinWord, ForthAction, ForthState, Word};

pub fn builtin_words() -> HashMap<String, Word> {
    let mut dict = HashMap::new();

    // Stack manipulation
    dict.insert("dup".into(), Word::Builtin(BuiltinWord(w_dup)));
    dict.insert("drop".into(), Word::Builtin(BuiltinWord(w_drop)));
    dict.insert("swap".into(), Word::Builtin(BuiltinWord(w_swap)));
    dict.insert("over".into(), Word::Builtin(BuiltinWord(w_over)));
    dict.insert("rot".into(), Word::Builtin(BuiltinWord(w_rot)));
    dict.insert("nip".into(), Word::Builtin(BuiltinWord(w_nip)));
    dict.insert("tuck".into(), Word::Builtin(BuiltinWord(w_tuck)));
    dict.insert("2dup".into(), Word::Builtin(BuiltinWord(w_2dup)));
    dict.insert("2drop".into(), Word::Builtin(BuiltinWord(w_2drop)));
    dict.insert("2swap".into(), Word::Builtin(BuiltinWord(w_2swap)));

    // Arithmetic
    dict.insert("+".into(), Word::Builtin(BuiltinWord(w_add)));
    dict.insert("-".into(), Word::Builtin(BuiltinWord(w_sub)));
    dict.insert("*".into(), Word::Builtin(BuiltinWord(w_mul)));
    dict.insert("/".into(), Word::Builtin(BuiltinWord(w_div)));
    dict.insert("mod".into(), Word::Builtin(BuiltinWord(w_mod)));
    dict.insert("negate".into(), Word::Builtin(BuiltinWord(w_negate)));
    dict.insert("abs".into(), Word::Builtin(BuiltinWord(w_abs)));
    dict.insert("min".into(), Word::Builtin(BuiltinWord(w_min)));
    dict.insert("max".into(), Word::Builtin(BuiltinWord(w_max)));

    // Comparison (Forth convention: -1 = true, 0 = false)
    dict.insert("<".into(), Word::Builtin(BuiltinWord(w_lt)));
    dict.insert(">".into(), Word::Builtin(BuiltinWord(w_gt)));
    dict.insert("=".into(), Word::Builtin(BuiltinWord(w_eq)));
    dict.insert("<>".into(), Word::Builtin(BuiltinWord(w_neq)));
    dict.insert("<=".into(), Word::Builtin(BuiltinWord(w_le)));
    dict.insert(">=".into(), Word::Builtin(BuiltinWord(w_ge)));
    dict.insert("0=".into(), Word::Builtin(BuiltinWord(w_zero_eq)));
    dict.insert("0<".into(), Word::Builtin(BuiltinWord(w_zero_lt)));
    dict.insert("0>".into(), Word::Builtin(BuiltinWord(w_zero_gt)));

    // Logic
    dict.insert("and".into(), Word::Builtin(BuiltinWord(w_and)));
    dict.insert("or".into(), Word::Builtin(BuiltinWord(w_or)));
    dict.insert("xor".into(), Word::Builtin(BuiltinWord(w_xor)));
    dict.insert("not".into(), Word::Builtin(BuiltinWord(w_not)));
    dict.insert("invert".into(), Word::Builtin(BuiltinWord(w_invert)));

    // Sova MIDI words
    dict.insert("note".into(), Word::Builtin(BuiltinWord(w_note)));
    dict.insert("cc".into(), Word::Builtin(BuiltinWord(w_cc)));
    dict.insert("pc".into(), Word::Builtin(BuiltinWord(w_pc)));
    dict.insert("wait".into(), Word::Builtin(BuiltinWord(w_wait)));

    // Context setters
    dict.insert("ch!".into(), Word::Builtin(BuiltinWord(w_set_channel)));
    dict.insert("dev!".into(), Word::Builtin(BuiltinWord(w_set_device)));
    dict.insert("vel!".into(), Word::Builtin(BuiltinWord(w_set_velocity)));
    dict.insert("dur!".into(), Word::Builtin(BuiltinWord(w_set_duration)));

    // Context getters
    dict.insert("ch@".into(), Word::Builtin(BuiltinWord(w_get_channel)));
    dict.insert("dev@".into(), Word::Builtin(BuiltinWord(w_get_device)));
    dict.insert("vel@".into(), Word::Builtin(BuiltinWord(w_get_velocity)));

    dict
}

// Stack manipulation
fn w_dup(state: &mut ForthState) -> Option<ForthAction> {
    let a = state.peek();
    state.push(a);
    None
}

fn w_drop(state: &mut ForthState) -> Option<ForthAction> {
    state.pop();
    None
}

fn w_swap(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    state.push(b);
    state.push(a);
    None
}

fn w_over(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.peek();
    state.push(b);
    state.push(a);
    None
}

fn w_rot(state: &mut ForthState) -> Option<ForthAction> {
    let c = state.pop();
    let b = state.pop();
    let a = state.pop();
    state.push(b);
    state.push(c);
    state.push(a);
    None
}

fn w_nip(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    state.pop();
    state.push(b);
    None
}

fn w_tuck(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    state.push(b);
    state.push(a);
    state.push(b);
    None
}

fn w_2dup(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.peek();
    state.push(b);
    state.push(a);
    state.push(b);
    None
}

fn w_2drop(state: &mut ForthState) -> Option<ForthAction> {
    state.pop();
    state.pop();
    None
}

fn w_2swap(state: &mut ForthState) -> Option<ForthAction> {
    let d = state.pop();
    let c = state.pop();
    let b = state.pop();
    let a = state.pop();
    state.push(c);
    state.push(d);
    state.push(a);
    state.push(b);
    None
}

// Arithmetic
fn w_add(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    state.push(a + b);
    None
}

fn w_sub(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    state.push(a - b);
    None
}

fn w_mul(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    state.push(a * b);
    None
}

fn w_div(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    if b == 0.0 {
        state.push(0.0);
    } else {
        state.push(a / b);
    }
    None
}

fn w_mod(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    if b == 0.0 {
        state.push(0.0);
    } else {
        state.push(a % b);
    }
    None
}

fn w_negate(state: &mut ForthState) -> Option<ForthAction> {
    let a = state.pop();
    state.push(-a);
    None
}

fn w_abs(state: &mut ForthState) -> Option<ForthAction> {
    let a = state.pop();
    state.push(a.abs());
    None
}

fn w_min(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    state.push(a.min(b));
    None
}

fn w_max(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    state.push(a.max(b));
    None
}

// Comparison (Forth convention: -1.0 = true, 0.0 = false)
fn w_lt(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    state.push(if a < b { -1.0 } else { 0.0 });
    None
}

fn w_gt(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    state.push(if a > b { -1.0 } else { 0.0 });
    None
}

fn w_eq(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    state.push(if a == b { -1.0 } else { 0.0 });
    None
}

fn w_neq(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    state.push(if a != b { -1.0 } else { 0.0 });
    None
}

fn w_le(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    state.push(if a <= b { -1.0 } else { 0.0 });
    None
}

fn w_ge(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop();
    let a = state.pop();
    state.push(if a >= b { -1.0 } else { 0.0 });
    None
}

fn w_zero_eq(state: &mut ForthState) -> Option<ForthAction> {
    let a = state.pop();
    state.push(if a == 0.0 { -1.0 } else { 0.0 });
    None
}

fn w_zero_lt(state: &mut ForthState) -> Option<ForthAction> {
    let a = state.pop();
    state.push(if a < 0.0 { -1.0 } else { 0.0 });
    None
}

fn w_zero_gt(state: &mut ForthState) -> Option<ForthAction> {
    let a = state.pop();
    state.push(if a > 0.0 { -1.0 } else { 0.0 });
    None
}

// Logic (cast to i64 for bitwise ops)
fn w_and(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop() as i64;
    let a = state.pop() as i64;
    state.push((a & b) as f64);
    None
}

fn w_or(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop() as i64;
    let a = state.pop() as i64;
    state.push((a | b) as f64);
    None
}

fn w_xor(state: &mut ForthState) -> Option<ForthAction> {
    let b = state.pop() as i64;
    let a = state.pop() as i64;
    state.push((a ^ b) as f64);
    None
}

fn w_not(state: &mut ForthState) -> Option<ForthAction> {
    let a = state.pop();
    state.push(if a == 0.0 { -1.0 } else { 0.0 });
    None
}

fn w_invert(state: &mut ForthState) -> Option<ForthAction> {
    let a = state.pop() as i64;
    state.push(!a as f64);
    None
}

// Sova MIDI words
fn w_note(state: &mut ForthState) -> Option<ForthAction> {
    let note = state.pop() as u64;
    let vel = state.velocity;
    let ch = state.channel;
    let dur_micros = (state.duration_beats * 1_000_000.0 * 60.0 / 120.0) as u64; // Assume 120 BPM for now
    let dev = state.device;
    Some(ForthAction::Emit(ConcreteEvent::MidiNote(note, vel, ch, dur_micros, dev)))
}

fn w_cc(state: &mut ForthState) -> Option<ForthAction> {
    let value = state.pop() as u64;
    let controller = state.pop() as u64;
    let ch = state.channel;
    let dev = state.device;
    Some(ForthAction::Emit(ConcreteEvent::MidiControl(controller, value, ch, dev)))
}

fn w_pc(state: &mut ForthState) -> Option<ForthAction> {
    let program = state.pop() as u64;
    let ch = state.channel;
    let dev = state.device;
    Some(ForthAction::Emit(ConcreteEvent::MidiProgram(program, ch, dev)))
}

fn w_wait(state: &mut ForthState) -> Option<ForthAction> {
    let beats = state.pop();
    let micros = (beats * 1_000_000.0 * 60.0 / 120.0) as u64; // Assume 120 BPM
    Some(ForthAction::Wait(micros))
}

// Context setters
fn w_set_channel(state: &mut ForthState) -> Option<ForthAction> {
    state.channel = state.pop() as u64;
    None
}

fn w_set_device(state: &mut ForthState) -> Option<ForthAction> {
    state.device = state.pop() as usize;
    None
}

fn w_set_velocity(state: &mut ForthState) -> Option<ForthAction> {
    state.velocity = state.pop() as u64;
    None
}

fn w_set_duration(state: &mut ForthState) -> Option<ForthAction> {
    state.duration_beats = state.pop() / 100.0; // Input in centbeats
    None
}

// Context getters
fn w_get_channel(state: &mut ForthState) -> Option<ForthAction> {
    state.push(state.channel as f64);
    None
}

fn w_get_device(state: &mut ForthState) -> Option<ForthAction> {
    state.push(state.device as f64);
    None
}

fn w_get_velocity(state: &mut ForthState) -> Option<ForthAction> {
    state.push(state.velocity as f64);
    None
}
