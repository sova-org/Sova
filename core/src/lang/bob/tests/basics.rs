use super::compile_and_run;
use crate::vm::variable::VariableValue;

#[test]
fn compile_and_run_basic() {
    let result = compile_and_run("SET G.X 42");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn bare_polish_notation() {
    let result = compile_and_run("SET G.X ADD 2 3");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(5))
    );
}

#[test]
fn nested_polish_notation() {
    // ADD 2 MUL 3 4 = 2 + (3 * 4) = 14
    let result = compile_and_run("SET G.X ADD 2 MUL 3 4");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(14))
    );
}

#[test]
fn consecutive_loops() {
    // First loop: I from 1 to 4, adds I to G.X each iteration (1+2+3+4 = 10)
    // Second loop: I from 1 to 6, adds I to G.Y each iteration (1+2+3+4+5+6 = 21)
    // New loop expression syntax: RANGE start end : body; collect_expr END
    let result = compile_and_run(
        "SET G.X 0; SET G.Y 0; RANGE 1 4 : SET G.X ADD G.X I; I END; RANGE 1 6 : SET G.Y ADD G.Y I; I END",
    );
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(10))
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(21))
    );
}

#[test]
fn toss() {
    let result = compile_and_run("SET G.X TOSS");
    let x = result.global_vars.get("X");
    assert!(x == Some(&VariableValue::Integer(0)) || x == Some(&VariableValue::Integer(1)));
}

#[test]
fn rand() {
    let result = compile_and_run("SET G.X RAND 10");
    if let Some(VariableValue::Integer(x)) = result.global_vars.get("X") {
        assert!(*x >= 0 && *x <= 10, "RAND 10 produced {}, expected 0-10", x);
    } else {
        panic!("X should be an integer");
    }
}

#[test]
fn rrand() {
    let result = compile_and_run("SET G.X RRAND 5 10");
    if let Some(VariableValue::Integer(x)) = result.global_vars.get("X") {
        assert!(*x >= 5 && *x <= 10);
    } else {
        panic!("X should be an integer");
    }
}

#[test]
fn clamp() {
    // CLAMP always returns Float (coerces inputs)
    let result = compile_and_run("SET G.X CLAMP 15 0 10");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Float(10.0))
    );
}

#[test]
fn wrap() {
    // WRAP 12 0 10 should return 2 (12 mod 10 = 2)
    let result = compile_and_run("SET G.X WRAP 12 0 10");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(2))
    );
}

#[test]
fn deeply_nested_polish_notation() {
    // Test deep nesting: ADD MUL SUB DIV 100 2 10 3 4 = ADD MUL (SUB (DIV 100 2) 10) 3 4
    // DIV 100 2 = 50, SUB 50 10 = 40, MUL 40 3 = 120, ADD 120 4 = 124
    let result = compile_and_run("SET G.X ADD MUL SUB DIV 100 2 10 3 4");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(124))
    );
}

#[test]
fn float_literal() {
    let result = compile_and_run("SET G.X 3.14");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Float(3.14))
    );
}

#[test]
fn negative_integer() {
    let result = compile_and_run("SET G.X -42");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(-42))
    );
}
