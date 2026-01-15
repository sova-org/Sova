use super::run_forth;

#[test]
fn test_word_definition() {
    let stack = run_forth(": double dup + ; 5 double");
    assert_eq!(stack, vec![10.0]);
}

#[test]
fn test_nested_word_definition() {
    let stack = run_forth(": double dup + ; : quadruple double double ; 3 quadruple");
    assert_eq!(stack, vec![12.0]);
}

#[test]
fn test_if_true() {
    let stack = run_forth(": test -1 if 42 then ; test");
    assert_eq!(stack, vec![42.0]);
}

#[test]
fn test_if_false() {
    let stack = run_forth(": test 0 if 42 then ; test");
    assert!(stack.is_empty());
}

#[test]
fn test_if_else_true() {
    let stack = run_forth(": test -1 if 42 else 0 then ; test");
    assert_eq!(stack, vec![42.0]);
}

#[test]
fn test_if_else_false() {
    let stack = run_forth(": test 0 if 42 else 99 then ; test");
    assert_eq!(stack, vec![99.0]);
}

#[test]
fn test_do_loop() {
    let stack = run_forth(": test 0 4 0 do i + loop ; test");
    assert_eq!(stack, vec![6.0]);
}

#[test]
fn test_begin_until() {
    let stack = run_forth(": test 5 begin 1 - dup 0= until ; test");
    assert_eq!(stack, vec![0.0]);
}

#[test]
fn test_comments_backslash() {
    let stack = run_forth("\\ this is a comment\n42");
    assert_eq!(stack, vec![42.0]);
}

#[test]
fn test_comments_paren() {
    let stack = run_forth("( this is a comment ) 42");
    assert_eq!(stack, vec![42.0]);
}

#[test]
fn test_case_insensitive_keywords() {
    let stack = run_forth(": test 0 IF 1 ELSE 2 THEN ; test");
    assert_eq!(stack, vec![2.0]);
}
