use super::run_forth;

#[test]
fn test_less_than_true() {
    let stack = run_forth("3 5 <");
    assert_eq!(stack, vec![-1.0]);
}

#[test]
fn test_less_than_false() {
    let stack = run_forth("5 3 <");
    assert_eq!(stack, vec![0.0]);
}

#[test]
fn test_greater_than_true() {
    let stack = run_forth("5 3 >");
    assert_eq!(stack, vec![-1.0]);
}

#[test]
fn test_greater_than_false() {
    let stack = run_forth("3 5 >");
    assert_eq!(stack, vec![0.0]);
}

#[test]
fn test_equal_true() {
    let stack = run_forth("5 5 =");
    assert_eq!(stack, vec![-1.0]);
}

#[test]
fn test_equal_false() {
    let stack = run_forth("5 3 =");
    assert_eq!(stack, vec![0.0]);
}

#[test]
fn test_not_equal_true() {
    let stack = run_forth("5 3 <>");
    assert_eq!(stack, vec![-1.0]);
}

#[test]
fn test_not_equal_false() {
    let stack = run_forth("5 5 <>");
    assert_eq!(stack, vec![0.0]);
}

#[test]
fn test_less_or_equal() {
    let stack = run_forth("5 5 <=");
    assert_eq!(stack, vec![-1.0]);
}

#[test]
fn test_greater_or_equal() {
    let stack = run_forth("5 5 >=");
    assert_eq!(stack, vec![-1.0]);
}

#[test]
fn test_zero_equal_true() {
    let stack = run_forth("0 0=");
    assert_eq!(stack, vec![-1.0]);
}

#[test]
fn test_zero_equal_false() {
    let stack = run_forth("5 0=");
    assert_eq!(stack, vec![0.0]);
}

#[test]
fn test_zero_less_than() {
    let stack = run_forth("-5 0<");
    assert_eq!(stack, vec![-1.0]);
}

#[test]
fn test_zero_greater_than() {
    let stack = run_forth("5 0>");
    assert_eq!(stack, vec![-1.0]);
}

#[test]
fn test_and() {
    let stack = run_forth("-1 -1 and");
    assert_eq!(stack, vec![-1.0]);
}

#[test]
fn test_or() {
    let stack = run_forth("-1 0 or");
    assert_eq!(stack, vec![-1.0]);
}

#[test]
fn test_xor() {
    let stack = run_forth("-1 0 xor");
    assert_eq!(stack, vec![-1.0]);
}

#[test]
fn test_not_true() {
    let stack = run_forth("-1 not");
    assert_eq!(stack, vec![0.0]);
}

#[test]
fn test_not_false() {
    let stack = run_forth("0 not");
    assert_eq!(stack, vec![-1.0]);
}

#[test]
fn test_invert() {
    let stack = run_forth("0 invert");
    assert_eq!(stack, vec![-1.0]);
}
