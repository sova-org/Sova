use super::compile_and_run;
use crate::vm::event::ConcreteEvent;
use crate::vm::variable::VariableValue;

#[test]
fn if_else_brace() {
    let result = compile_and_run("SET G.X 0; SET G.Y IF G.X { 1 } ELSE { 2 }");
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(2))
    );
}

#[test]
fn if_else_brace_true() {
    let result = compile_and_run("SET G.X 1; SET G.Y IF G.X { 1 } ELSE { 2 }");
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(1))
    );
}

#[test]
fn if_else_brace_no_spaces() {
    let result = compile_and_run("SET G.X 1; SET G.Y IF G.X{1}ELSE{2}");
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(1))
    );
}

#[test]
fn do_brace() {
    let result = compile_and_run("SET G.X 0; DO 5 { SET G.X ADD G.X 1 }");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(5))
    );
}

#[test]
fn while_brace() {
    let result = compile_and_run(
        "SET G.X 0; SET G.I 0; WHILE LT G.I 5 { SET G.X ADD G.X G.I; SET G.I ADD G.I 1 }",
    );
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(10))
    );
}

#[test]
fn loop_brace() {
    let result = compile_and_run("SET G.X 0; RANGE 1 5 { SET G.X ADD G.X I }");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(15))
    );
}

#[test]
fn loop_with_step_brace() {
    let result = compile_and_run("SET G.X 0; RANGE 0 10 2 { SET G.X ADD G.X 1 }");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(6))
    );
}

#[test]
fn each_brace() {
    let result = compile_and_run("SET G.X 0; EACH '[1 2 3] { SET G.X ADD G.X E }");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(6))
    );
}

#[test]
fn every_brace() {
    let result = compile_and_run("SET G.X 0; RANGE 0 5 { EVERY 2 { SET G.X ADD G.X 1 } }");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3))
    );
}

#[test]
fn prob_else_brace() {
    let result = compile_and_run("SET G.X PROB 100 { 42 } ELSE { 0 }");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn prob_else_brace_zero() {
    let result = compile_and_run("SET G.X PROB 0 { 42 } ELSE { 99 }");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(99))
    );
}

#[test]
fn switch_brace() {
    let result = compile_and_run(
        "SET G.X 2; SET G.Y SWITCH G.X { CASE 1 { 10 } CASE 2 { 20 } DEFAULT { 0 } }",
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(20))
    );
}

#[test]
fn switch_brace_default() {
    let result = compile_and_run(
        "SET G.X 99; SET G.Y SWITCH G.X { CASE 1 { 10 } CASE 2 { 20 } DEFAULT { 100 } }",
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(100))
    );
}

#[test]
fn func_brace() {
    let result = compile_and_run("FUNC DOUBLE A { * A 2 }; SET G.X (CALL DOUBLE 21)");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn fn_brace() {
    let result = compile_and_run("SET G.F FN A { * A 3 }; SET G.X (CALL G.F 14)");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn choose_brace() {
    // CHOOSE with single option always returns that option
    let result = compile_and_run("SET G.X CHOOSE { 42 }");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn alt_brace() {
    // ALT cycles through options - each ALT expression is independent
    let result = compile_and_run("SET G.X ALT { 1 2 3 }");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(1))
    );
}

#[test]
fn fork_brace() {
    let result = compile_and_run("FORK { DO 2 { 1 } }; SET G.X 99");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(99))
    );
}

#[test]
fn bytes_brace() {
    // BYTES with brace syntax inside sysex map
    let result = compile_and_run(">> [sysex: BYTES { 240 67 32 0 247 }]");
    // Just verify it compiles and runs without error
    assert!(result.events.len() > 0);
}

#[test]
fn nested_braces() {
    let result = compile_and_run("SET G.X 0; DO 3 { IF 1 { SET G.X ADD G.X 1 } ELSE { 0 } }");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3))
    );
}

#[test]
fn mixed_styles_different_constructs() {
    // Different constructs can use different styles
    let result = compile_and_run("SET G.X 0; DO 3 { RANGE 1 2 : SET G.X ADD G.X I END }");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(9))
    );
}

#[test]
fn multiline_brace_body() {
    let result = compile_and_run(
        "SET G.X 0
DO 3 {
    SET G.X ADD G.X 1
}",
    );
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3))
    );
}

#[test]
fn multiline_if_else_brace() {
    let result = compile_and_run(
        "SET G.X 1
SET G.Y IF G.X {
    42
} ELSE {
    0
}",
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(42))
    );
}

// ============================================================================
// Emit syntax alternatives: PLAY, =>, >>, @
// ============================================================================

#[test]
fn emit_play_keyword() {
    let result = compile_and_run("PLAY [note: 60]");
    assert_eq!(result.events.len(), 1);
    assert!(matches!(result.events[0].0, ConcreteEvent::MidiNote(..)));
}

#[test]
fn emit_double_arrow() {
    let result = compile_and_run(">> [note: 60]");
    assert_eq!(result.events.len(), 1);
    assert!(matches!(result.events[0].0, ConcreteEvent::MidiNote(..)));
}

#[test]
fn emit_at() {
    let result = compile_and_run("@ [note: 60]");
    assert_eq!(result.events.len(), 1);
    assert!(matches!(result.events[0].0, ConcreteEvent::MidiNote(..)));
}
