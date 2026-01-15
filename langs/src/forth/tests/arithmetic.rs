use super::run_forth;

#[test]
fn test_add() {
    let stack = run_forth("2 3 +");
    assert_eq!(stack, vec![5.0]);
}

#[test]
fn test_subtract() {
    let stack = run_forth("10 3 -");
    assert_eq!(stack, vec![7.0]);
}

#[test]
fn test_multiply() {
    let stack = run_forth("4 5 *");
    assert_eq!(stack, vec![20.0]);
}

#[test]
fn test_divide() {
    let stack = run_forth("20 4 /");
    assert_eq!(stack, vec![5.0]);
}

#[test]
fn test_divide_by_zero() {
    let stack = run_forth("10 0 /");
    assert_eq!(stack, vec![0.0]);
}

#[test]
fn test_mod() {
    let stack = run_forth("17 5 mod");
    assert_eq!(stack, vec![2.0]);
}

#[test]
fn test_negate() {
    let stack = run_forth("42 negate");
    assert_eq!(stack, vec![-42.0]);
}

#[test]
fn test_abs_positive() {
    let stack = run_forth("42 abs");
    assert_eq!(stack, vec![42.0]);
}

#[test]
fn test_abs_negative() {
    let stack = run_forth("-42 abs");
    assert_eq!(stack, vec![42.0]);
}

#[test]
fn test_min() {
    let stack = run_forth("5 3 min");
    assert_eq!(stack, vec![3.0]);
}

#[test]
fn test_max() {
    let stack = run_forth("5 3 max");
    assert_eq!(stack, vec![5.0]);
}

#[test]
fn test_complex_expression() {
    let stack = run_forth("2 3 + 4 *");
    assert_eq!(stack, vec![20.0]);
}

#[test]
fn test_hex_literal() {
    let stack = run_forth("0x10");
    assert_eq!(stack, vec![16.0]);
}

#[test]
fn test_binary_literal() {
    let stack = run_forth("0b1010");
    assert_eq!(stack, vec![10.0]);
}
