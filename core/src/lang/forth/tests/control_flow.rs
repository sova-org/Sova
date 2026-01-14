use super::run_forth;

#[test]
fn test_word_definition() {
    let result = run_forth(": double dup + ; 5 double");
    assert!(result.events.is_empty());
}

#[test]
fn test_nested_word_definition() {
    let result = run_forth(": double dup + ; : quadruple double double ; 3 quadruple");
    assert!(result.events.is_empty());
}

#[test]
fn test_if_true() {
    // -1 is true in Forth
    let result = run_forth(": test -1 if 42 then ; test");
    assert!(result.events.is_empty());
}

#[test]
fn test_if_false() {
    // 0 is false in Forth
    let result = run_forth(": test 0 if 42 then ; test");
    assert!(result.events.is_empty());
}

#[test]
fn test_if_else_true() {
    let result = run_forth(": test -1 if 42 else 0 then ; test");
    assert!(result.events.is_empty());
}

#[test]
fn test_if_else_false() {
    let result = run_forth(": test 0 if 42 else 99 then ; test");
    assert!(result.events.is_empty());
}

#[test]
fn test_do_loop() {
    // Sum numbers 0 to 3 (loop runs with I = 0, 1, 2, 3)
    let result = run_forth(": test 0 4 0 do i + loop ; test");
    assert!(result.events.is_empty());
}

#[test]
fn test_begin_until() {
    // Count down from 5 to 0
    let result = run_forth(": test 5 begin 1 - dup 0= until ; test");
    assert!(result.events.is_empty());
}

#[test]
fn test_comments_backslash() {
    let result = run_forth("\\ this is a comment\n42");
    assert!(result.events.is_empty());
}

#[test]
fn test_comments_paren() {
    let result = run_forth("( this is a comment ) 42");
    assert!(result.events.is_empty());
}

#[test]
fn test_case_insensitive_keywords() {
    let result = run_forth(": test 0 IF 1 ELSE 2 THEN ; test");
    assert!(result.events.is_empty());
}
