use super::run_forth;

#[test]
fn test_push_number() {
    let stack = run_forth("42");
    assert_eq!(stack, vec![42.0]);
}

#[test]
fn test_dup() {
    let stack = run_forth("5 dup");
    assert_eq!(stack, vec![5.0, 5.0]);
}

#[test]
fn test_drop() {
    let stack = run_forth("1 2 drop");
    assert_eq!(stack, vec![1.0]);
}

#[test]
fn test_swap() {
    let stack = run_forth("1 2 swap");
    assert_eq!(stack, vec![2.0, 1.0]);
}

#[test]
fn test_over() {
    let stack = run_forth("1 2 over");
    assert_eq!(stack, vec![1.0, 2.0, 1.0]);
}

#[test]
fn test_rot() {
    let stack = run_forth("1 2 3 rot");
    assert_eq!(stack, vec![2.0, 3.0, 1.0]);
}

#[test]
fn test_nip() {
    let stack = run_forth("1 2 nip");
    assert_eq!(stack, vec![2.0]);
}

#[test]
fn test_tuck() {
    let stack = run_forth("1 2 tuck");
    assert_eq!(stack, vec![2.0, 1.0, 2.0]);
}

#[test]
fn test_2dup() {
    let stack = run_forth("1 2 2dup");
    assert_eq!(stack, vec![1.0, 2.0, 1.0, 2.0]);
}

#[test]
fn test_2drop() {
    let stack = run_forth("1 2 3 4 2drop");
    assert_eq!(stack, vec![1.0, 2.0]);
}

#[test]
fn test_2swap() {
    let stack = run_forth("1 2 3 4 2swap");
    assert_eq!(stack, vec![3.0, 4.0, 1.0, 2.0]);
}
