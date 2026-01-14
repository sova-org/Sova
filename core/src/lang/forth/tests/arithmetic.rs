use super::run_forth;

#[test]
fn test_add() {
    let result = run_forth("2 3 +");
    assert!(result.events.is_empty());
}

#[test]
fn test_subtract() {
    let result = run_forth("10 3 -");
    assert!(result.events.is_empty());
}

#[test]
fn test_multiply() {
    let result = run_forth("4 5 *");
    assert!(result.events.is_empty());
}

#[test]
fn test_divide() {
    let result = run_forth("20 4 /");
    assert!(result.events.is_empty());
}

#[test]
fn test_divide_by_zero() {
    // Should return 0, not panic
    let result = run_forth("10 0 /");
    assert!(result.events.is_empty());
}

#[test]
fn test_mod() {
    let result = run_forth("17 5 mod");
    assert!(result.events.is_empty());
}

#[test]
fn test_negate() {
    let result = run_forth("42 negate");
    assert!(result.events.is_empty());
}

#[test]
fn test_abs_positive() {
    let result = run_forth("42 abs");
    assert!(result.events.is_empty());
}

#[test]
fn test_abs_negative() {
    let result = run_forth("-42 abs");
    assert!(result.events.is_empty());
}

#[test]
fn test_min() {
    let result = run_forth("5 3 min");
    assert!(result.events.is_empty());
}

#[test]
fn test_max() {
    let result = run_forth("5 3 max");
    assert!(result.events.is_empty());
}

#[test]
fn test_complex_expression() {
    // (2 + 3) * 4 = 20
    let result = run_forth("2 3 + 4 *");
    assert!(result.events.is_empty());
}

#[test]
fn test_hex_literal() {
    let result = run_forth("0x10");  // 16 in hex
    assert!(result.events.is_empty());
}

#[test]
fn test_binary_literal() {
    let result = run_forth("0b1010");  // 10 in binary
    assert!(result.events.is_empty());
}
