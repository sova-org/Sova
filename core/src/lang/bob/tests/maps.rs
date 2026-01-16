use super::compile_and_run;
use crate::vm::variable::VariableValue;

#[test]
fn map_new() {
    let result = compile_and_run("SET G.M MNEW");
    match result.global_vars.get("M") {
        Some(VariableValue::Map(m)) => assert!(m.is_empty()),
        other => panic!("Expected empty map, got {:?}", other),
    }
}

#[test]
fn map_literal() {
    let result = compile_and_run("SET G.M [note: 60 vel: 100]");
    match result.global_vars.get("M") {
        Some(VariableValue::Map(m)) => {
            assert_eq!(m.get("note"), Some(&VariableValue::Integer(60)));
            assert_eq!(m.get("vel"), Some(&VariableValue::Integer(100)));
        }
        other => panic!("Expected map, got {:?}", other),
    }
}

#[test]
fn map_literal_with_expressions() {
    let result = compile_and_run("SET G.X 20; SET G.M [a: ADD G.X 5 b: MUL G.X 2]");
    match result.global_vars.get("M") {
        Some(VariableValue::Map(m)) => {
            assert_eq!(m.get("a"), Some(&VariableValue::Integer(25)));
            assert_eq!(m.get("b"), Some(&VariableValue::Integer(40)));
        }
        other => panic!("Expected map, got {:?}", other),
    }
}

#[test]
fn map_get() {
    let result = compile_and_run("SET G.M [x: 42 y: 99]; SET G.V MGET G.M \"x\"");
    assert_eq!(
        result.global_vars.get("V"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn map_has() {
    let result = compile_and_run("SET G.M [x: 42]; SET G.A MHAS G.M \"x\"; SET G.B MHAS G.M \"z\"");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Bool(true))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Bool(false))
    );
}

#[test]
fn map_set() {
    // MSET returns a new map, so chain assignments
    let result = compile_and_run(
        "SET G.M MNEW; SET G.M MSET G.M \"note\" 60; SET G.M MSET G.M \"vel\" 100; SET G.V MGET G.M \"note\"",
    );
    assert_eq!(
        result.global_vars.get("V"),
        Some(&VariableValue::Integer(60))
    );
}

#[test]
fn emit_map() {
    // >> [key: val] emits using the map
    let result = compile_and_run(">> [cutoff: 60 resonance: ADD 20 5]");
    assert!(!result.events.is_empty());
}

#[test]
fn emit_map_variable() {
    // >> G.M emits from a map variable
    let result = compile_and_run("SET G.M [note: 60 vel: 100]; >> G.M");
    assert!(!result.events.is_empty());
}

// ============================================================================
// MMERGE Tests
// ============================================================================

#[test]
fn mmerge_second_wins() {
    // MMERGE a b -> b's values win on conflict
    let result = compile_and_run("SET G.M MMERGE [note: 60 vel: 100] [vel: 50]");
    match result.global_vars.get("M") {
        Some(VariableValue::Map(m)) => {
            assert_eq!(m.get("note"), Some(&VariableValue::Integer(60)));
            assert_eq!(m.get("vel"), Some(&VariableValue::Integer(50))); // second wins
        }
        other => panic!("Expected map, got {:?}", other),
    }
}

#[test]
fn mmerge_disjoint_keys() {
    // MMERGE with no conflicts - all keys preserved
    let result = compile_and_run("SET G.M MMERGE [a: 1 b: 2] [c: 3 d: 4]");
    match result.global_vars.get("M") {
        Some(VariableValue::Map(m)) => {
            assert_eq!(m.get("a"), Some(&VariableValue::Integer(1)));
            assert_eq!(m.get("b"), Some(&VariableValue::Integer(2)));
            assert_eq!(m.get("c"), Some(&VariableValue::Integer(3)));
            assert_eq!(m.get("d"), Some(&VariableValue::Integer(4)));
        }
        other => panic!("Expected map, got {:?}", other),
    }
}

#[test]
fn mmerge_defaults_pattern() {
    // Common use case: defaults + overrides
    let result =
        compile_and_run("SET G.D [vel: 100 chan: 0 dur: 0.5]; SET G.M MMERGE G.D [note: 60]");
    match result.global_vars.get("M") {
        Some(VariableValue::Map(m)) => {
            assert_eq!(m.get("note"), Some(&VariableValue::Integer(60)));
            assert_eq!(m.get("vel"), Some(&VariableValue::Integer(100)));
            assert_eq!(m.get("chan"), Some(&VariableValue::Integer(0)));
        }
        other => panic!("Expected map, got {:?}", other),
    }
}

#[test]
fn mmerge_vs_bor() {
    // BOR: first wins, MMERGE: second wins
    let result = compile_and_run("SET G.A BOR [x: 1] [x: 2]; SET G.B MMERGE [x: 1] [x: 2]");
    match result.global_vars.get("A") {
        Some(VariableValue::Map(m)) => {
            assert_eq!(m.get("x"), Some(&VariableValue::Integer(1))); // first wins
        }
        other => panic!("Expected map for A, got {:?}", other),
    }
    match result.global_vars.get("B") {
        Some(VariableValue::Map(m)) => {
            assert_eq!(m.get("x"), Some(&VariableValue::Integer(2))); // second wins
        }
        other => panic!("Expected map for B, got {:?}", other),
    }
}

#[test]
fn mlen_returns_key_count() {
    let result = compile_and_run("SET G.A MLEN [a: 1 b: 2 c: 3]; SET G.B MLEN MNEW");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(3))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Integer(0))
    );
}

// ============================================================================
// Emit List-of-Maps (Chord) Tests
// ============================================================================

#[test]
fn emit_list_of_maps_chord() {
    // Emitting a list of maps should produce multiple events (chord)
    let result = compile_and_run("SET G.C '[[note: 60] [note: 64] [note: 67]]; >> G.C");
    // Should emit 3 events (one per map in the list)
    assert_eq!(
        result.events.len(),
        3,
        "Expected 3 events for chord, got {}",
        result.events.len()
    );
}

#[test]
fn emit_single_map_still_works() {
    // Emitting a single map should still produce one event
    let result = compile_and_run("SET G.M [note: 60 vel: 100]; >> G.M");
    assert_eq!(
        result.events.len(),
        1,
        "Expected 1 event for single map, got {}",
        result.events.len()
    );
}

#[test]
fn emit_inline_list_of_maps() {
    // List literal via variable (grammar requires variable for list emit)
    let result = compile_and_run("SET G.X '[[note: 60] [note: 64]]; >> G.X");
    assert_eq!(
        result.events.len(),
        2,
        "Expected 2 events, got {}",
        result.events.len()
    );
}

#[test]
fn map_with_nested_map_value() {
    // Map containing another map as a value
    let result = compile_and_run(
        "SET G.M [outer: [inner: 42]];
         SET G.N MGET G.M \"outer\";
         SET G.X MGET G.N \"inner\"",
    );
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn mget_nonexistent_key() {
    // MGET on non-existent key returns 0 (default)
    let result = compile_and_run("SET G.M [a: 1]; SET G.X MGET G.M \"z\"");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(0))
    );
}
