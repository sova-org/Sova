use super::compile_and_run;
use sova_core::vm::variable::VariableValue;

#[test]
fn choose_single() {
    // CHOOSE with single option always returns that option
    let result = compile_and_run("SET G.X CHOOSE: 42 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn choose_multiple() {
    // CHOOSE with multiple options returns one of them
    let result = compile_and_run("SET G.X CHOOSE: 1 2 3 4 END");
    let x = result.global_vars.get("X").expect("X should be set");
    match x {
        VariableValue::Integer(v) => {
            assert!(*v >= 1 && *v <= 4, "X should be 1, 2, 3, or 4, got {}", v);
        }
        other => panic!("Expected integer, got {:?}", other),
    }
}

#[test]
fn choose_with_expressions() {
    // CHOOSE can contain expressions
    let result = compile_and_run("SET G.X CHOOSE: ADD 1 2 MUL 3 4 END");
    let x = result.global_vars.get("X").expect("X should be set");
    match x {
        VariableValue::Integer(v) => {
            assert!(
                *v == 3 || *v == 12,
                "X should be 3 (1+2) or 12 (3*4), got {}",
                v
            );
        }
        other => panic!("Expected integer, got {:?}", other),
    }
}

#[test]
fn choose_in_expression() {
    // CHOOSE can be used inside other expressions
    let result = compile_and_run("SET G.X ADD 10 CHOOSE: 1 2 3 END");
    let x = result.global_vars.get("X").expect("X should be set");
    match x {
        VariableValue::Integer(v) => {
            assert!(*v >= 11 && *v <= 13, "X should be 11, 12, or 13, got {}", v);
        }
        other => panic!("Expected integer, got {:?}", other),
    }
}

#[test]
fn choose_in_loop_varies() {
    // Run CHOOSE multiple times, collect results - should see variation
    let result = compile_and_run("SET G.X 0; RANGE 0 99 : SET G.X ADD G.X CHOOSE: 0 1 END; I END");
    let x = result.global_vars.get("X").expect("X should be set");
    match x {
        VariableValue::Integer(v) => {
            // If CHOOSE was evaluated only once, X would be 0 or 100
            // With randomness, it should be somewhere in between
            assert!(
                *v > 0 && *v < 100,
                "X should vary between 0 and 100, got {}",
                v
            );
        }
        other => panic!("Expected integer, got {:?}", other),
    }
}

#[test]
fn alt_single() {
    // ALT with single option always returns that option
    let result = compile_and_run("SET G.X ALT: 42 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn alt_cycles_in_loop() {
    // ALT cycles through options: 1, 2, 1, 2 -> last assigned is 2
    let result = compile_and_run("RANGE 0 3 : SET G.X ALT: 1 2 END; I END");
    // Loop runs 4 times (0,1,2,3), ALT cycles: 1, 2, 1, 2
    // Last value assigned to X is 2
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(2))
    );
}

#[test]
fn alt_three_options() {
    // ALT with 3 options, loop 6 times: 10, 20, 30, 10, 20, 30
    let result = compile_and_run("SET G.X 0; RANGE 0 5 : SET G.X ADD G.X ALT: 10 20 30 END; I END");
    // Sum: 10 + 20 + 30 + 10 + 20 + 30 = 120
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(120))
    );
}

#[test]
fn alt_with_expressions() {
    // ALT can contain expressions
    let result = compile_and_run("SET G.X ALT: ADD 1 2 MUL 3 4 END");
    // First execution returns first option: ADD 1 2 = 3
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3))
    );
}

#[test]
fn alt_in_expression() {
    // ALT can be used inside other expressions
    let result = compile_and_run("SET G.X ADD 100 ALT: 1 2 3 END");
    // First execution: 100 + 1 = 101
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(101))
    );
}

// --- Tests for arrow with CHOOSE/ALT (Form A: >> CHOOSE: [...] END) ---

#[test]
fn arrow_choose() {
    // >> CHOOSE: randomly emits one of the map options
    let result = compile_and_run(">> CHOOSE: [note: 60] [note: 72] END");
    assert_eq!(result.events.len(), 1, "Should emit exactly one event");
}

#[test]
fn arrow_alt() {
    // >> ALT: cycles through options
    let result = compile_and_run(
        "
        >> ALT: [note: 60] [note: 72] END;
        >> ALT: [note: 60] [note: 72] END
    ",
    );
    assert_eq!(result.events.len(), 2, "Should emit two events");
}

#[test]
fn arrow_ternary() {
    // >> ? cond then else: emit based on condition
    let result = compile_and_run("SET G.X 1; >> ? G.X [note: 60] [note: 72]");
    assert_eq!(result.events.len(), 1, "Should emit exactly one event");
}

// --- Tests for emit expression (Form B: CHOOSE: >> [...] END) ---

#[test]
fn choose_with_emit_expr_lazy() {
    // CHOOSE with emit expressions - only ONE emit should happen (lazy evaluation)
    let result = compile_and_run("CHOOSE: >> [note: 60] >> [note: 72] END");
    assert_eq!(
        result.events.len(),
        1,
        "Lazy CHOOSE should emit only one event, got {}",
        result.events.len()
    );
}

#[test]
fn alt_with_emit_expr_lazy() {
    // ALT with emit expressions - only ONE emit per call
    let result = compile_and_run(
        "
        ALT: >> [note: 60] >> [note: 72] END;
        ALT: >> [note: 60] >> [note: 72] END
    ",
    );
    assert_eq!(
        result.events.len(),
        2,
        "Each ALT call should emit one event"
    );
}

#[test]
fn emit_expr_returns_value() {
    // => returns the map for assignment
    let result = compile_and_run("SET G.M >> [note: 60]; >> G.M");
    assert_eq!(
        result.events.len(),
        2,
        "Should emit twice: once from >> [note: 60] and once from >> G.M"
    );
}

#[test]
fn choose_lazy_many_options() {
    // Lazy CHOOSE with many emit options - still only one emit
    let result =
        compile_and_run("CHOOSE: >> [note: 60] >> [note: 62] >> [note: 64] >> [note: 65] END");
    assert_eq!(
        result.events.len(),
        1,
        "Lazy CHOOSE should emit only one event regardless of option count"
    );
}

#[test]
fn nested_choose_in_loop() {
    // CHOOSE inside loop - each iteration emits one event
    let result = compile_and_run("RANGE 0 2 : CHOOSE: >> [note: 60] >> [note: 72] END; I END");
    assert_eq!(
        result.events.len(),
        3,
        "Loop with 3 iterations should emit 3 events"
    );
}

#[test]
fn choose_options_with_function_calls() {
    // CHOOSE where options are function calls - tests lazy evaluation
    // Only the selected function should be called
    let result = compile_and_run(
        "SET G.A 0; SET G.B 0;
         FUNC FA : SET G.A 1; 10 END;
         FUNC FB : SET G.B 1; 20 END;
         SET G.X CHOOSE: (CALL FA) END",
    );
    // With single option, FA must be called
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(10))
    );
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(1))
    );
    // FB should never be called
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Integer(0))
    );
}
