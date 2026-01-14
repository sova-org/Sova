use super::run_forth;

#[test]
fn test_push_number() {
    let result = run_forth("42");
    assert!(result.events.is_empty());
}

#[test]
fn test_dup() {
    // 5 dup + = 10
    let result = run_forth(": test 5 dup + ; test");
    assert!(result.events.is_empty());
}

#[test]
fn test_drop() {
    let result = run_forth("1 2 drop");
    assert!(result.events.is_empty());
}

#[test]
fn test_swap() {
    // 1 2 swap - = 2 - 1 = 1
    let result = run_forth(": test 1 2 swap - ; test");
    assert!(result.events.is_empty());
}

#[test]
fn test_over() {
    // 1 2 over = 1 2 1, then + + = 4
    let result = run_forth(": test 1 2 over + + ; test");
    assert!(result.events.is_empty());
}

#[test]
fn test_rot() {
    // 1 2 3 rot = 2 3 1
    let result = run_forth("1 2 3 rot");
    assert!(result.events.is_empty());
}

#[test]
fn test_nip() {
    // 1 2 nip = 2
    let result = run_forth("1 2 nip");
    assert!(result.events.is_empty());
}

#[test]
fn test_tuck() {
    // 1 2 tuck = 2 1 2
    let result = run_forth("1 2 tuck");
    assert!(result.events.is_empty());
}

#[test]
fn test_2dup() {
    // 1 2 2dup = 1 2 1 2
    let result = run_forth("1 2 2dup");
    assert!(result.events.is_empty());
}

#[test]
fn test_2drop() {
    // 1 2 3 4 2drop = 1 2
    let result = run_forth("1 2 3 4 2drop");
    assert!(result.events.is_empty());
}

#[test]
fn test_2swap() {
    // 1 2 3 4 2swap = 3 4 1 2
    let result = run_forth("1 2 3 4 2swap");
    assert!(result.events.is_empty());
}
