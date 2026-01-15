use std::collections::HashMap;

use super::types::{BuiltinWord, ForthState, Word};

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

    dict
}

// Stack manipulation
fn w_dup(state: &mut ForthState) {
    let a = state.peek();
    state.push(a);
}

fn w_drop(state: &mut ForthState) {
    state.pop();
}

fn w_swap(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    state.push(b);
    state.push(a);
}

fn w_over(state: &mut ForthState) {
    let b = state.pop();
    let a = state.peek();
    state.push(b);
    state.push(a);
}

fn w_rot(state: &mut ForthState) {
    let c = state.pop();
    let b = state.pop();
    let a = state.pop();
    state.push(b);
    state.push(c);
    state.push(a);
}

fn w_nip(state: &mut ForthState) {
    let b = state.pop();
    state.pop();
    state.push(b);
}

fn w_tuck(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    state.push(b);
    state.push(a);
    state.push(b);
}

fn w_2dup(state: &mut ForthState) {
    let b = state.pop();
    let a = state.peek();
    state.push(b);
    state.push(a);
    state.push(b);
}

fn w_2drop(state: &mut ForthState) {
    state.pop();
    state.pop();
}

fn w_2swap(state: &mut ForthState) {
    let d = state.pop();
    let c = state.pop();
    let b = state.pop();
    let a = state.pop();
    state.push(c);
    state.push(d);
    state.push(a);
    state.push(b);
}

// Arithmetic
fn w_add(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    state.push(a + b);
}

fn w_sub(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    state.push(a - b);
}

fn w_mul(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    state.push(a * b);
}

fn w_div(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    if b == 0.0 {
        state.push(0.0);
    } else {
        state.push(a / b);
    }
}

fn w_mod(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    if b == 0.0 {
        state.push(0.0);
    } else {
        state.push(a % b);
    }
}

fn w_negate(state: &mut ForthState) {
    let a = state.pop();
    state.push(-a);
}

fn w_abs(state: &mut ForthState) {
    let a = state.pop();
    state.push(a.abs());
}

fn w_min(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    state.push(a.min(b));
}

fn w_max(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    state.push(a.max(b));
}

// Comparison (Forth convention: -1.0 = true, 0.0 = false)
fn w_lt(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    state.push(if a < b { -1.0 } else { 0.0 });
}

fn w_gt(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    state.push(if a > b { -1.0 } else { 0.0 });
}

fn w_eq(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    state.push(if a == b { -1.0 } else { 0.0 });
}

fn w_neq(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    state.push(if a != b { -1.0 } else { 0.0 });
}

fn w_le(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    state.push(if a <= b { -1.0 } else { 0.0 });
}

fn w_ge(state: &mut ForthState) {
    let b = state.pop();
    let a = state.pop();
    state.push(if a >= b { -1.0 } else { 0.0 });
}

fn w_zero_eq(state: &mut ForthState) {
    let a = state.pop();
    state.push(if a == 0.0 { -1.0 } else { 0.0 });
}

fn w_zero_lt(state: &mut ForthState) {
    let a = state.pop();
    state.push(if a < 0.0 { -1.0 } else { 0.0 });
}

fn w_zero_gt(state: &mut ForthState) {
    let a = state.pop();
    state.push(if a > 0.0 { -1.0 } else { 0.0 });
}

// Logic (cast to i64 for bitwise ops)
fn w_and(state: &mut ForthState) {
    let b = state.pop() as i64;
    let a = state.pop() as i64;
    state.push((a & b) as f64);
}

fn w_or(state: &mut ForthState) {
    let b = state.pop() as i64;
    let a = state.pop() as i64;
    state.push((a | b) as f64);
}

fn w_xor(state: &mut ForthState) {
    let b = state.pop() as i64;
    let a = state.pop() as i64;
    state.push((a ^ b) as f64);
}

fn w_not(state: &mut ForthState) {
    let a = state.pop();
    state.push(if a == 0.0 { -1.0 } else { 0.0 });
}

fn w_invert(state: &mut ForthState) {
    let a = state.pop() as i64;
    state.push(!a as f64);
}
