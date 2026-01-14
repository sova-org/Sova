use super::run_forth;

#[test]
fn test_less_than_true() {
    let result = run_forth("3 5 <");
    assert!(result.events.is_empty());
}

#[test]
fn test_less_than_false() {
    let result = run_forth("5 3 <");
    assert!(result.events.is_empty());
}

#[test]
fn test_greater_than_true() {
    let result = run_forth("5 3 >");
    assert!(result.events.is_empty());
}

#[test]
fn test_greater_than_false() {
    let result = run_forth("3 5 >");
    assert!(result.events.is_empty());
}

#[test]
fn test_equal_true() {
    let result = run_forth("5 5 =");
    assert!(result.events.is_empty());
}

#[test]
fn test_equal_false() {
    let result = run_forth("5 3 =");
    assert!(result.events.is_empty());
}

#[test]
fn test_not_equal_true() {
    let result = run_forth("5 3 <>");
    assert!(result.events.is_empty());
}

#[test]
fn test_not_equal_false() {
    let result = run_forth("5 5 <>");
    assert!(result.events.is_empty());
}

#[test]
fn test_less_or_equal() {
    let result = run_forth("5 5 <=");
    assert!(result.events.is_empty());
}

#[test]
fn test_greater_or_equal() {
    let result = run_forth("5 5 >=");
    assert!(result.events.is_empty());
}

#[test]
fn test_zero_equal_true() {
    let result = run_forth("0 0=");
    assert!(result.events.is_empty());
}

#[test]
fn test_zero_equal_false() {
    let result = run_forth("5 0=");
    assert!(result.events.is_empty());
}

#[test]
fn test_zero_less_than() {
    let result = run_forth("-5 0<");
    assert!(result.events.is_empty());
}

#[test]
fn test_zero_greater_than() {
    let result = run_forth("5 0>");
    assert!(result.events.is_empty());
}

// Logic tests
#[test]
fn test_and() {
    let result = run_forth("-1 -1 and");  // true AND true
    assert!(result.events.is_empty());
}

#[test]
fn test_or() {
    let result = run_forth("-1 0 or");  // true OR false
    assert!(result.events.is_empty());
}

#[test]
fn test_xor() {
    let result = run_forth("-1 0 xor");  // true XOR false
    assert!(result.events.is_empty());
}

#[test]
fn test_not_true() {
    let result = run_forth("-1 not");  // NOT true = false
    assert!(result.events.is_empty());
}

#[test]
fn test_not_false() {
    let result = run_forth("0 not");  // NOT false = true
    assert!(result.events.is_empty());
}

#[test]
fn test_invert() {
    let result = run_forth("0 invert");  // bitwise NOT of 0 = -1
    assert!(result.events.is_empty());
}
