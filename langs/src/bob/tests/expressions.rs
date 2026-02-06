use super::compile_and_run;
use sova_core::vm::variable::VariableValue;

// Note symbol tests

#[test]
fn note_symbol_c3() {
    // :c3 should be MIDI 60
    let result = compile_and_run("SET G.X :c3");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(60))
    );
}

#[test]
fn note_symbol_default_octave() {
    // :c without octave defaults to c3 (MIDI 60)
    let result = compile_and_run("SET G.X :c");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(60))
    );
}

#[test]
fn note_symbol_sharp() {
    // :c#3 should be MIDI 61
    let result = compile_and_run("SET G.X :c#3");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(61))
    );
}

#[test]
fn note_symbol_flat() {
    // :db3 should be MIDI 61 (same as c#3)
    let result = compile_and_run("SET G.X :db3");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(61))
    );
}

#[test]
fn note_symbol_a3() {
    // :a3 should be MIDI 69 (concert A in this octave system where c3=60)
    let result = compile_and_run("SET G.X :a3");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(69))
    );
}

#[test]
fn note_symbol_in_expression() {
    // Notes can be used in expressions: ADD :c3 12 = 60 + 12 = 72
    let result = compile_and_run("SET G.X ADD :c3 12");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(72))
    );
}

#[test]
fn note_symbol_uppercase() {
    // Uppercase notes work too: :C#3 = 61
    let result = compile_and_run("SET G.X :C#3");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(61))
    );
}

#[test]
fn note_symbol_uppercase_no_octave() {
    // :C defaults to c3 = 60
    let result = compile_and_run("SET G.X :C");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(60))
    );
}

#[test]
fn non_note_symbol_stays_string() {
    // Non-note symbols stay as strings
    let result = compile_and_run("SET G.X :kick");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Str("kick".to_string()))
    );
}

// Ternary conditional tests

#[test]
fn ternary_true_condition() {
    // ? 1 10 20 should return 10 (1 is truthy)
    let result = compile_and_run("SET G.X ? 1 10 20");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(10))
    );
}

#[test]
fn ternary_false_condition() {
    // ? 0 10 20 should return 20 (0 is falsy)
    let result = compile_and_run("SET G.X ? 0 10 20");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(20))
    );
}

#[test]
fn ternary_with_comparison() {
    // G.X=5, ? GT G.X 3 100 0 should return 100 (5 > 3 is true)
    let result = compile_and_run("SET G.X 5; SET G.Y ? GT G.X 3 100 0");
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(100))
    );
}

#[test]
fn ternary_with_comparison_false() {
    // G.X=2, ? GT G.X 3 100 0 should return 0 (2 > 3 is false)
    let result = compile_and_run("SET G.X 2; SET G.Y ? GT G.X 3 100 0");
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(0))
    );
}

#[test]
fn ternary_nested() {
    // Nested: ? 1 ? 1 10 20 30 should return 10
    let result = compile_and_run("SET G.X ? 1 ? 1 10 20 30");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(10))
    );
}

#[test]
fn ternary_in_expression() {
    // Use ternary result in arithmetic: ADD ? 1 10 20 5 = 10 + 5 = 15
    let result = compile_and_run("SET G.X ADD ? 1 10 20 5");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(15))
    );
}

// IF expression tests

#[test]
fn if_expr_true_condition() {
    // IF with true condition returns then branch
    let result = compile_and_run("SET G.X IF 1 : 10 ELSE : 20 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(10))
    );
}

#[test]
fn if_expr_false_condition() {
    // IF with false condition returns else branch
    let result = compile_and_run("SET G.X IF 0 : 10 ELSE : 20 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(20))
    );
}

#[test]
fn if_expr_with_comparison() {
    // IF with comparison: GT 5 3 is true, returns 100
    let result = compile_and_run("SET G.A 5; SET G.X IF GT G.A 3 : 100 ELSE : 0 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(100))
    );
}

#[test]
fn if_expr_with_body_statements() {
    // IF with statements in body: side effects happen, then expr returned
    let result = compile_and_run("SET G.A 0; SET G.X IF 1 : SET G.A 5; ADD G.A 10 ELSE : 99 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(15))
    );
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(5))
    );
}

#[test]
fn if_expr_else_body_statements() {
    // IF with statements in else body
    let result = compile_and_run("SET G.A 0; SET G.X IF 0 : 99 ELSE : SET G.A 7; MUL G.A 2 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(14))
    );
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(7))
    );
}

#[test]
fn if_expr_in_arithmetic() {
    // Use IF result in arithmetic expression
    let result = compile_and_run("SET G.X ADD IF 1 : 10 ELSE : 20 END 5");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(15))
    );
}

#[test]
fn if_expr_nested() {
    // Nested IF expressions
    let result = compile_and_run("SET G.X IF 1 : IF 1 : 10 ELSE : 20 END ELSE : 30 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(10))
    );
}

#[test]
fn if_expr_no_else() {
    // IF without ELSE returns 0 when condition is false
    let result = compile_and_run("SET G.X IF 0 : 42 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(0))
    );
}

#[test]
fn if_expr_no_else_true() {
    // IF without ELSE returns value when condition is true
    let result = compile_and_run("SET G.X IF 1 : 42 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

// Lambda tests

#[test]
fn lambda_simple() {
    let result = compile_and_run("SET G.D FN X : MUL X 2 END; SET G.Y (CALL G.D 5)");
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(10))
    );
}

#[test]
fn lambda_multiple_args() {
    let result = compile_and_run("SET G.H FN A B : ADD A B END; SET G.Z (CALL G.H 3 7)");
    assert_eq!(
        result.global_vars.get("Z"),
        Some(&VariableValue::Integer(10))
    );
}

#[test]
fn lambda_three_args() {
    let result = compile_and_run("SET G.H FN A B C : ADD A ADD B C END; SET G.Z (CALL G.H 1 2 3)");
    assert_eq!(
        result.global_vars.get("Z"),
        Some(&VariableValue::Integer(6))
    );
}

#[test]
fn lambda_with_body() {
    // Lambda with body statements before return
    let result =
        compile_and_run("SET G.H FN X : SET G.Z MUL X X; ADD G.Z 1 END; SET G.W (CALL G.H 3)");
    // 3*3 = 9, 9+1 = 10
    assert_eq!(
        result.global_vars.get("W"),
        Some(&VariableValue::Integer(10))
    );
}

#[test]
fn lambda_passed_to_func() {
    // Pass lambda to a declared function
    let result = compile_and_run(
        r#"
        FUNC APPLY H X : (CALL H X) END;
        SET G.F FN N : MUL N 3 END;
        SET G.Z (CALL APPLY G.F 4)
        "#,
    );
    assert_eq!(
        result.global_vars.get("Z"),
        Some(&VariableValue::Integer(12))
    );
}

#[test]
fn lambda_no_args() {
    let result = compile_and_run("SET G.H FN : 42 END; SET G.Z (CALL G.H)");
    assert_eq!(
        result.global_vars.get("Z"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn lambda_with_ternary() {
    let result = compile_and_run(
        "SET G.H FN X : ? GT X 5 100 0 END; SET G.A (CALL G.H 10); SET G.B (CALL G.H 3)",
    );
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(100))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Integer(0))
    );
}

#[test]
fn if_expr_both_branches_have_side_effects() {
    // Both branches modify state, but only one should execute
    // Tests that the non-taken branch doesn't run
    let result = compile_and_run(
        "SET G.A 0; SET G.B 0;
         SET G.X IF 1 : SET G.A 10; 100 ELSE : SET G.B 20; 200 END",
    );
    // Condition is truthy (1), so then-branch runs: G.A=10, returns 100
    // Else-branch should NOT run, so G.B stays 0
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(100))
    );
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(10))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Integer(0))
    );
}
