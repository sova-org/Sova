use super::compile_and_run;
use sova_core::vm::event::ConcreteEvent;
use sova_core::vm::variable::VariableValue;

#[test]
fn if_else() {
    // Condition is false so should execute else branch
    // IF expression returns value directly
    let result = compile_and_run("SET G.X 0; SET G.Y IF G.X : 1 ELSE : 2 END");
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(2))
    );
}

#[test]
fn if_else_true_branch() {
    // Condition is true so should execute then branch
    let result = compile_and_run("SET G.X 1; SET G.Y IF G.X : 1 ELSE : 2 END");
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(1))
    );
}

#[test]
fn if_nested_for_elif() {
    // Nested IF expressions replace ELIF chains
    // G.X=2, so second condition matches
    let result = compile_and_run(
        "SET G.X 2; SET G.Y IF EQ G.X 1 : 10 ELSE : IF EQ G.X 2 : 20 ELSE : 30 END END",
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(20))
    );
}

#[test]
fn prob_compiles() {
    // PROB 100 body should always execute
    let result = compile_and_run("SET G.X 0; PROB 100 : SET G.X 1 ELSE : 0 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(1))
    );
}

#[test]
fn prob_zero() {
    // PROB 0 should never execute body, always go to else
    let result = compile_and_run("SET G.X 0; PROB 0 : SET G.X 1 ELSE : 0 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(0))
    );
}

#[test]
fn prob_else_always_body() {
    // PROB 100 with ELSE: always executes then branch
    let result = compile_and_run("SET G.X 0; PROB 100 : SET G.X 1 ELSE : SET G.X 2 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(1))
    );
}

#[test]
fn prob_else_always_else() {
    // PROB 0 with ELSE: always executes else branch
    let result = compile_and_run("SET G.X 0; PROB 0 : SET G.X 1 ELSE : SET G.X 2 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(2))
    );
}

#[test]
fn do_repeat() {
    // DO 5 : body END -> executes 5 times
    let result = compile_and_run("SET G.X 0; DO 5 : SET G.X ADD G.X 1 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(5))
    );
}

#[test]
fn do_zero_times() {
    // DO 0 : body never executes
    let result = compile_and_run("SET G.X 42; DO 0 : SET G.X 0 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn consecutive_do() {
    let result = compile_and_run(
        "SET G.X 0; SET G.Y 0; DO 3 : SET G.X ADD G.X 1 END; DO 4 : SET G.Y ADD G.Y 2 END",
    );
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3))
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(8))
    );
}

#[test]
fn break_exits_script() {
    // BREAK stops execution, G.Y should never be set
    let result = compile_and_run("SET G.X 1; BREAK; SET G.Y 2");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(1))
    );
    assert_eq!(result.global_vars.get("Y"), None);
}

#[test]
fn break_in_loop() {
    // BREAK inside conditional in loop exits entire script
    let result = compile_and_run(
        "SET G.X 0; RANGE 1 10 : SET G.X I; IF GT I 3 : BREAK ELSE : 0 END; I END; SET G.Y 99",
    );
    // G.X should be 4 (last value of I before BREAK)
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(4))
    );
    // G.Y never set because BREAK exits script
    assert_eq!(result.global_vars.get("Y"), None);
}

#[test]
fn switch_case_match() {
    let result = compile_and_run(
        "SET G.X 2; SET G.Y SWITCH G.X : CASE 1 : 10 CASE 2 : 20 CASE 3 : 30 DEFAULT : 0 END",
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(20))
    );
}

#[test]
fn switch_first_case() {
    let result =
        compile_and_run("SET G.X 1; SET G.Y SWITCH G.X : CASE 1 : 10 CASE 2 : 20 DEFAULT : 0 END");
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(10))
    );
}

#[test]
fn switch_default() {
    // No matching case, falls to DEFAULT
    let result = compile_and_run(
        "SET G.X 99; SET G.Y SWITCH G.X : CASE 1 : 10 CASE 2 : 20 DEFAULT : 100 END",
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(100))
    );
}

#[test]
fn switch_no_default_no_match() {
    // No matching case - falls to DEFAULT which returns 42
    let result = compile_and_run("SET G.Y SWITCH 99 : CASE 1 : 10 CASE 2 : 20 DEFAULT : 42 END");
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn switch_with_body_statements() {
    // Switch case with body statements before final expression
    let result = compile_and_run(
        "SET G.X 2; SET G.Z 0; SET G.Y SWITCH G.X : CASE 1 : 10 CASE 2 : SET G.Z 100; 20 DEFAULT : 0 END",
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(20))
    );
    assert_eq!(
        result.global_vars.get("Z"),
        Some(&VariableValue::Integer(100))
    );
}

#[test]
fn every_basic() {
    // EVERY 3 executes on iterations 0, 3, 6, 9 (4 times in 12 iterations)
    let result = compile_and_run("SET G.X 0; RANGE 0 11 : EVERY 3 : SET G.X ADD G.X 1 END; I END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(4))
    );
}

#[test]
fn every_one() {
    // EVERY 1 executes every time
    let result = compile_and_run("SET G.X 0; RANGE 0 4 : EVERY 1 : SET G.X ADD G.X 1 END; I END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(5))
    );
}

#[test]
fn every_two() {
    // EVERY 2 executes on 0, 2, 4 (3 times in 6 iterations)
    let result = compile_and_run("SET G.X 0; RANGE 0 5 : EVERY 2 : SET G.X ADD G.X 1 END; I END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3))
    );
}

#[test]
fn every_with_expression() {
    // EVERY with expression as period
    let result = compile_and_run(
        "SET G.N 4; SET G.X 0; RANGE 0 7 : EVERY G.N : SET G.X ADD G.X 1 END; I END",
    );
    // Executes on 0, 4 (2 times in 8 iterations)
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(2))
    );
}

#[test]
fn every_multiple_statements() {
    // EVERY body can have multiple statements
    let result = compile_and_run(
        "SET G.X 0; SET G.Y 0; RANGE 0 5 : EVERY 2 : SET G.X ADD G.X 1; SET G.Y ADD G.Y 10 END; I END",
    );
    // Executes on 0, 2, 4 (3 times)
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3))
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(30))
    );
}

#[test]
fn every_independent_counters() {
    // Two EVERY blocks have independent counters (different line scope)
    let result = compile_and_run(
        "SET G.X 0; SET G.Y 0; RANGE 0 5 : EVERY 2 : SET G.X ADD G.X 1 END; EVERY 3 : SET G.Y ADD G.Y 1 END; I END",
    );
    // First EVERY: executes on 0, 2, 4 (3 times)
    // Second EVERY: executes on 0, 3 (2 times)
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3))
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(2))
    );
}

#[test]
fn loop_with_step() {
    // RANGE 0 10 2 iterates: 0, 2, 4, 6, 8, 10 (6 iterations)
    let result = compile_and_run("SET G.X 0; RANGE 0 10 2 : SET G.X ADD G.X 1; I END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(6))
    );
}

#[test]
fn loop_with_step_sum() {
    // Sum of I values: 0 + 2 + 4 + 6 + 8 + 10 = 30
    let result = compile_and_run("SET G.X 0; RANGE 0 10 2 : SET G.X ADD G.X I; I END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(30))
    );
}

#[test]
fn loop_with_step_three() {
    // RANGE 0 10 3 iterates: 0, 3, 6, 9 (4 iterations)
    let result = compile_and_run("SET G.X 0; RANGE 0 10 3 : SET G.X ADD G.X 1; I END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(4))
    );
}

#[test]
fn loop_with_step_expression() {
    // Step can be an expression
    let result = compile_and_run("SET G.S 3; SET G.X 0; RANGE 0 9 G.S : SET G.X ADD G.X 1; I END");
    // Iterates: 0, 3, 6, 9 (4 iterations)
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(4))
    );
}

#[test]
fn loop_step_one_default() {
    // Without explicit step, should iterate by 1
    let result = compile_and_run("SET G.X 0; RANGE 0 4 : SET G.X ADD G.X 1; I END");
    // Iterates: 0, 1, 2, 3, 4 (5 iterations)
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(5))
    );
}

#[test]
fn nested_loops_share_iterator() {
    // Nested loops both using I - I is shared (not scoped)
    let result = compile_and_run(
        "SET G.X 0; SET G.O 0; RANGE 1 3 : SET G.O I; RANGE 10 12 : SET G.X ADD G.X I; I END; I END",
    );
    // Outer loop starts with I=1, inner runs (I becomes 10,11,12,then 13)
    // After inner loop, I=13 which is > 3, so outer loop exits
    // Inner loop: 10+11+12 = 33
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(33))
    );
    // G.O captured I=1 before inner loop ran
    assert_eq!(
        result.global_vars.get("O"),
        Some(&VariableValue::Integer(1))
    );
}

#[test]
fn while_basic() {
    // Sum 0+1+2+3+4 = 10
    let result = compile_and_run(
        "SET G.X 0; SET G.I 0; WHILE LT G.I 5 : SET G.X ADD G.X G.I; SET G.I ADD G.I 1 END",
    );
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(10))
    );
}

#[test]
fn while_never_executes() {
    // Condition false from start - body never runs
    let result = compile_and_run("SET G.X 42; WHILE 0 : SET G.X 0 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn while_single_iteration() {
    // Condition becomes false after one iteration
    let result = compile_and_run("SET G.X 1; WHILE G.X : SET G.X 0 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(0))
    );
}

#[test]
fn consecutive_loops() {
    // Two consecutive loops
    let result = compile_and_run(
        "SET G.X 0; SET G.Y 0; RANGE 1 3 : SET G.X ADD G.X I; I END; RANGE 10 12 : SET G.Y ADD G.Y I; I END",
    );
    // First loop: 1+2+3 = 6
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(6))
    );
    // Second loop: 10+11+12 = 33
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(33))
    );
}

#[test]
fn loop_returns_list() {
    // Loop expression returns a list of collected values
    let result = compile_and_run("SET G.NOTES RANGE 1 3 : + 60 I END");
    assert_eq!(
        result.global_vars.get("NOTES"),
        Some(&VariableValue::Vec(vec![
            VariableValue::Integer(61),
            VariableValue::Integer(62),
            VariableValue::Integer(63),
        ]))
    );
}

#[test]
fn loop_returns_squares() {
    // Loop expression returns squares
    let result = compile_and_run("SET G.SQ RANGE 1 4 : * I I END");
    assert_eq!(
        result.global_vars.get("SQ"),
        Some(&VariableValue::Vec(vec![
            VariableValue::Integer(1),
            VariableValue::Integer(4),
            VariableValue::Integer(9),
            VariableValue::Integer(16),
        ]))
    );
}

// ============================================================================
// FORK tests - FORK spawns a single branch containing a sequence
// ============================================================================

#[test]
fn fork_basic_compiles() {
    // FORK with sequence should compile and emit 1 StartProgram event
    let result = compile_and_run("FORK: DO 2 : 1 END; DO 3 : 2 END END");
    // Should emit 1 StartProgram event (single branch with sequence)
    let start_program_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::StartProgram(_)))
        .count();
    assert_eq!(start_program_count, 1);
}

#[test]
fn fork_returns_zero() {
    // FORK expression returns 0
    let result = compile_and_run("SET G.X FORK: DO 1 : 42 END END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(0))
    );
}

#[test]
fn fork_continues_after_spawn() {
    // Code after FORK should execute in parent
    let result = compile_and_run("FORK: DO 1 : 1 END END; SET G.X 99");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(99))
    );
}

#[test]
fn fork_single_expression() {
    // FORK with single expression
    let result = compile_and_run("FORK: DO 1 : 1 END END");
    let start_program_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::StartProgram(_)))
        .count();
    assert_eq!(start_program_count, 1);
}

#[test]
fn fork_sequence() {
    // FORK with sequence of expressions (all in one branch)
    let result = compile_and_run("FORK: DO 1 : 1 END; DO 1 : 2 END; DO 1 : 3 END END");
    // All expressions are in ONE branch
    let start_program_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::StartProgram(_)))
        .count();
    assert_eq!(start_program_count, 1);
}

#[test]
fn fork_inherits_dev_context() {
    // DEV before FORK should be inherited by branch
    let result = compile_and_run("DEV 2; FORK: DO 1 : 1 END END");
    assert_eq!(
        result
            .events
            .iter()
            .filter(|(e, _)| matches!(e, ConcreteEvent::StartProgram(_)))
            .count(),
        1
    );
}

#[test]
fn fork_with_wait_before() {
    // WAIT before FORK - branch should spawn at the later time
    let result = compile_and_run("WAIT 1.0; FORK: DO 1 : 1 END END");
    let start_events: Vec<_> = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::StartProgram(_)))
        .collect();
    assert_eq!(start_events.len(), 1);
    // Time should be > 0 (after the WAIT)
    assert!(start_events[0].1 > 0);
}

#[test]
fn fork_nested() {
    // Nested FORK (FORK inside a branch) should compile
    let result = compile_and_run("FORK: FORK: DO 1 : 1 END END END");
    // Outer FORK emits 1 StartProgram (the inner FORK is inside that program)
    let start_program_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::StartProgram(_)))
        .count();
    assert_eq!(start_program_count, 1);
}

#[test]
fn fork_multiple_forks_for_parallel() {
    // To get multiple parallel branches, use multiple FORKs
    let result = compile_and_run("FORK: DO 2 : 1 END END; FORK: DO 3 : 2 END END");
    // Should emit 2 StartProgram events (one per FORK)
    let start_program_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::StartProgram(_)))
        .count();
    assert_eq!(start_program_count, 2);
}
