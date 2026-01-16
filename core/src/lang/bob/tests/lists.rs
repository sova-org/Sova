use super::{compile_and_run, compile_and_run_debug};
use crate::vm::variable::VariableValue;

#[test]
fn empty_list() {
    let result = compile_and_run("SET G.X LEN '[]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(0))
    );
}

#[test]
fn list_length() {
    let result = compile_and_run("SET G.X LEN '[1 2 3]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3))
    );
}

#[test]
fn list_get_basic() {
    let result = compile_and_run("SET G.X GET '[10 20 30] 0");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(10))
    );
}

#[test]
fn list_get_middle() {
    let result = compile_and_run("SET G.X GET '[10 20 30] 1");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(20))
    );
}

#[test]
fn list_get_last() {
    let result = compile_and_run("SET G.X GET '[10 20 30] 2");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(30))
    );
}

#[test]
fn list_get_wrap_positive() {
    // Index 5 on 3-element list: 5 % 3 = 2
    let result = compile_and_run("SET G.X GET '[10 20 30] 5");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(30))
    );
}

#[test]
fn list_get_wrap_negative() {
    // Index -1 on 3-element list should wrap to last element
    let result = compile_and_run("SET G.X GET '[10 20 30] -1");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(30))
    );
}

#[test]
fn list_pick_returns_element() {
    // PICK should return one of the elements
    let result = compile_and_run("SET G.X PICK '[42]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn list_pick_from_multiple() {
    // PICK should return one of the elements (10, 20, or 30)
    let result = compile_and_run("SET G.X PICK '[10 20 30]");
    let x = result.global_vars.get("X");
    assert!(
        x == Some(&VariableValue::Integer(10))
            || x == Some(&VariableValue::Integer(20))
            || x == Some(&VariableValue::Integer(30))
    );
}

#[test]
fn list_cycle_sequential() {
    // CYCLE should return elements in sequence
    // First call returns first element
    let result = compile_and_run("SET G.X CYCLE '[1 2 3]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(1))
    );
}

#[test]
fn list_with_expressions() {
    // List elements can be expressions
    let result = compile_and_run("SET G.X GET '[ADD 1 2 MUL 3 4 5] 0");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3)) // ADD 1 2 = 3
    );
}

#[test]
fn list_with_expressions_second() {
    let result = compile_and_run("SET G.X GET '[ADD 1 2 MUL 3 4 5] 1");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(12)) // MUL 3 4 = 12
    );
}

#[test]
fn nested_list() {
    // Nested lists
    let result = compile_and_run("SET G.X LEN GET '['[1 2] '[3 4 5]] 1");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3)) // Inner list '[3 4 5] has length 3
    );
}

#[test]
fn list_stored_in_variable() {
    // Use global variable (G.M) to store list
    let result = compile_and_run("SET G.M '[10 20 30]; SET G.X GET G.M 1");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(20))
    );
}

// =============================================================================
// ForEach (EACH) tests
// =============================================================================

#[test]
fn foreach_basic() {
    // First test: manual iteration using L loop - THIS WORKS
    let result =
        compile_and_run("SET G.M '[10 20 30]; SET G.X 0; RANGE 0 2 : SET G.X ADD G.X GET G.M I; I END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(60)) // 10 + 20 + 30 = 60
    );
}

#[test]
fn foreach_with_each() {
    // Sum elements using EACH loop
    let result = compile_and_run_debug("SET G.X 0; EACH '[10 20 30] : SET G.X ADD G.X E END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(60)) // 10 + 20 + 30 = 60
    );
}

#[test]
fn foreach_with_index() {
    // Use both I (index) and E (element)
    // Sum of (index * element): 0*10 + 1*20 + 2*30 = 0 + 20 + 60 = 80
    let result = compile_and_run("SET G.X 0; EACH '[10 20 30] : SET G.X ADD G.X MUL I E END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(80))
    );
}

#[test]
fn foreach_empty_list() {
    // Empty list should not execute body
    let result = compile_and_run("SET G.X 42; EACH '[] : SET G.X 0 END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42)) // unchanged
    );
}

#[test]
fn foreach_with_variable_list() {
    let result = compile_and_run("SET G.M '[5 10 15]; SET G.X 0; EACH G.M : SET G.X ADD G.X E END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(30)) // 5 + 10 + 15 = 30
    );
}

// =============================================================================
// MAP tests
// =============================================================================

#[test]
fn map_double() {
    // Double each element
    let result = compile_and_run("SET G.X LEN MAP FN A : MUL A 2 END '[1 2 3]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3)) // Result has 3 elements
    );
}

#[test]
fn map_double_values() {
    // Check actual doubled values
    let result = compile_and_run(
        "SET G.M MAP FN A : MUL A 2 END '[1 2 3]; SET G.X GET G.M 0; SET G.Y GET G.M 1; SET G.Z GET G.M 2",
    );
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(2))
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(4))
    );
    assert_eq!(
        result.global_vars.get("Z"),
        Some(&VariableValue::Integer(6))
    );
}

#[test]
fn map_add_offset() {
    // Add 10 to each element
    let result = compile_and_run("SET G.M MAP FN A : ADD A 10 END '[1 2 3]; SET G.X GET G.M 1");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(12)) // 2 + 10 = 12
    );
}

#[test]
fn map_empty_list() {
    // Mapping over empty list returns empty list
    let result = compile_and_run("SET G.X LEN MAP FN A : MUL A 2 END '[]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(0))
    );
}

// =============================================================================
// FILTER tests
// =============================================================================

#[test]
fn filter_positive() {
    // Keep only positive numbers
    let result = compile_and_run("SET G.X LEN FILTER FN A : GT A 0 END '[-1 2 -3 4 5]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3)) // 2, 4, 5
    );
}

#[test]
fn filter_positive_values() {
    // Check actual filtered values
    let result = compile_and_run(
        "SET G.M FILTER FN A : GT A 0 END '[-1 2 -3 4]; SET G.X GET G.M 0; SET G.Y GET G.M 1",
    );
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(2))
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(4))
    );
}

#[test]
fn filter_greater_than_threshold() {
    // Keep elements > 50
    let result = compile_and_run("SET G.X LEN FILTER FN A : GT A 50 END '[10 60 30 80 40]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(2)) // 60, 80
    );
}

#[test]
fn filter_all_pass() {
    // All elements pass the filter
    let result = compile_and_run("SET G.X LEN FILTER FN A : GT A 0 END '[1 2 3]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3))
    );
}

#[test]
fn filter_none_pass() {
    // No elements pass the filter
    let result = compile_and_run("SET G.X LEN FILTER FN A : GT A 100 END '[1 2 3]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(0))
    );
}

#[test]
fn filter_empty_list() {
    // Filtering empty list returns empty list
    let result = compile_and_run("SET G.X LEN FILTER FN A : GT A 0 END '[]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(0))
    );
}

#[test]
fn cycle_empty_list() {
    // CYCLE on empty list should return 0, not crash
    let result = compile_and_run("SET G.X CYCLE '[]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(0))
    );
}

// =============================================================================
// REDUCE tests
// =============================================================================

#[test]
fn reduce_sum() {
    // Sum: 1 + 2 + 3 = 6
    let result = compile_and_run("SET G.X REDUCE FN A B : ADD A B END 0 '[1 2 3]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(6))
    );
}

#[test]
fn reduce_product() {
    // Product: 1 * 2 * 3 * 4 = 24
    let result = compile_and_run("SET G.X REDUCE FN A B : MUL A B END 1 '[1 2 3 4]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(24))
    );
}

#[test]
fn reduce_max() {
    // Find max: max of [10, 30, 20] = 30
    let result = compile_and_run("SET G.X REDUCE FN A B : MAX A B END 0 '[10 30 20]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Float(30.0))
    );
}

#[test]
fn reduce_empty_list() {
    // Empty list returns init value
    let result = compile_and_run("SET G.X REDUCE FN A B : ADD A B END 42 '[]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn map_filter_reduce_chain() {
    // Chain: double each element, filter > 5, sum remaining
    // '[1 2 3 4] -> '[2 4 6 8] -> '[6 8] -> 14
    let result = compile_and_run(
        "SET G.A '[1 2 3 4]; SET G.B MAP FN X : MUL X 2 END G.A; SET G.C FILTER FN X : GT X 5 END G.B; SET G.X REDUCE FN Y Z : ADD Y Z END 0 G.C",
    );
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(14))
    );
}

// =============================================================================
// Edge case tests
// =============================================================================

#[test]
fn get_empty_list() {
    // GET on empty list returns 0 (default)
    let result = compile_and_run("SET G.X GET '[] 0");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(0))
    );
}

#[test]
fn pick_empty_list() {
    // PICK on empty list returns 0 (default)
    let result = compile_and_run("SET G.X PICK '[]");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(0))
    );
}
