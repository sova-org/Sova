use super::*;

#[test]
fn test_add_decimal() {

    // simple addition, no simplification needed
    let (s, n, d) = add_decimal(1, 4, 3, 1, 7, 5);
    assert_eq!(s, 1);
    assert_eq!(n, 41);
    assert_eq!(d, 15);

    // negative numbers, no simplification needed
    let (s, n, d) = add_decimal(-1, 4, 3, -1, 7, 5);
    assert_eq!(s, -1);
    assert_eq!(n, 41);
    assert_eq!(d, 15);

    // largest is negative, no simplification needed
    let (s, n, d) = add_decimal(1, 4, 3, -1, 7, 5);
    assert_eq!(s, -1);
    assert_eq!(n, 1);
    assert_eq!(d, 15);

    // largest is positive, no simplification needed
    let (s, n, d) = add_decimal(-1, 4, 3, 1, 7, 5);
    assert_eq!(s, 1);
    assert_eq!(n, 1);
    assert_eq!(d, 15);

    // simplification needed
    let (s, n, d) = add_decimal(1, 3, 2, 1, 7, 2);
    assert_eq!(s, 1);
    assert_eq!(n, 5);
    assert_eq!(d, 1);

}

#[test]
fn test_sub_decimal() {

    // simple subtraction, no simplification needed
    let (s, n, d) = sub_decimal(1, 3, 5, 1, 4, 7);
    assert_eq!(s, 1);
    assert_eq!(n, 1);
    assert_eq!(d, 35);

    // negative numbers, no simplification needed
    let (s, n, d) = sub_decimal(-1, 3, 5, -1, 4, 7);
    assert_eq!(s, -1);
    assert_eq!(n, 1);
    assert_eq!(d, 35);

    // first is negative, no simplification needed
    let (s, n, d) = sub_decimal(-1, 3, 5, 1, 4, 7);
    assert_eq!(s, -1);
    assert_eq!(n, 41);
    assert_eq!(d, 35);

    // second is negative, no simplification needed
    let (s, n, d) = sub_decimal(1, 3, 5, -1, 4, 7);
    assert_eq!(s, 1);
    assert_eq!(n, 41);
    assert_eq!(d, 35);

    // simplification needed
    let (s, n, d) = sub_decimal(1, 3, 2, 1, 7, 2);
    assert_eq!(s, -1);
    assert_eq!(n, 2);
    assert_eq!(d, 1);

}

#[test]
fn test_mul_decimal() {

    // both positive, no simplification needed
    let (s, n, d) = mul_decimal(1, 2, 3, 1, 5, 7);
    assert_eq!(s, 1);
    assert_eq!(n, 10);
    assert_eq!(d, 21);

    // both negative, no simplification needed
    let (s, n, d) = mul_decimal(-1, 2, 3, -1, 5, 7);
    assert_eq!(s, 1);
    assert_eq!(n, 10);
    assert_eq!(d, 21);

    // first negative, no simplification needed
    let (s, n, d) = mul_decimal(-1, 2, 3, 1, 5, 7);
    assert_eq!(s, -1);
    assert_eq!(n, 10);
    assert_eq!(d, 21);

    // second negative, no simplification needed
    let (s, n, d) = mul_decimal(1, 2, 3, -1, 5, 7);
    assert_eq!(s, -1);
    assert_eq!(n, 10);
    assert_eq!(d, 21);

    // simplification needed
    let (s, n, d) = mul_decimal(1, 2, 3, 1, 3, 5);
    assert_eq!(s, 1);
    assert_eq!(n, 2);
    assert_eq!(d, 5);
}

#[test]
fn test_div_decimal() {

    // both positive, no simplification needed
    let (s, n, d) = div_decimal(1, 2, 3, 1, 5, 7);
    assert_eq!(s, 1);
    assert_eq!(n, 14);
    assert_eq!(d, 15);

    // both negative, no simplification needed
    let (s, n, d) = div_decimal(-1, 2, 3, -1, 5, 7);
    assert_eq!(s, 1);
    assert_eq!(n, 14);
    assert_eq!(d, 15);

    // first negative, no simplification needed
    let (s, n, d) = div_decimal(-1, 2, 3, 1, 5, 7);
    assert_eq!(s, -1);
    assert_eq!(n, 14);
    assert_eq!(d, 15);

    // second negative, no simplification needed
    let (s, n, d) = div_decimal(1, 2, 3, -1, 5, 7);
    assert_eq!(s, -1);
    assert_eq!(n, 14);
    assert_eq!(d, 15);

    // simplification needed
    let (s, n, d) = div_decimal(1, 2, 3, 1, 5, 3);
    assert_eq!(s, 1);
    assert_eq!(n, 2);
    assert_eq!(d, 5);
}

#[test]
fn test_lt_decimal() {

    assert!(lt_decimal(1, 2, 1, 1, 3, 1));
    assert!(lt_decimal(1, 2, 1, 1, 5, 2));
    assert!(lt_decimal(1, 3, 5, 1, 3, 2));
    assert!(lt_decimal(-1, 3, 5, 1, 1, 5));
    assert!(lt_decimal(-1, 3, 5, 1, 3, 5));
    assert!(lt_decimal(-1, 3, 1, -1, 2, 1));
    assert!(lt_decimal(-1, 5, 2, -1, 2, 1));
    assert!(lt_decimal(-1, 3, 2, -1, 3, 5));
    assert!(lt_decimal(1, 5, 15, 1, 12, 24));

    assert!(!lt_decimal(1, 3, 1, 1, 2, 1));
    assert!(!lt_decimal(1, 5, 2, 1, 2, 1));
    assert!(!lt_decimal(1, 3, 2, 1, 3, 5));
    assert!(!lt_decimal(1, 1, 5, -1, 3, 5));
    assert!(!lt_decimal(1, 3, 5, -1, 3, 5));
    assert!(!lt_decimal(-1, 2, 1, -1, 3, 1));
    assert!(!lt_decimal(-1, 2, 1, -1, 5, 2));
    assert!(!lt_decimal(-1, 3, 5, -1, 3, 2));
    assert!(!lt_decimal(1, 12, 24, 1, 5, 15));

    assert!(!lt_decimal(1, 1, 1, 1, 1, 1));
    assert!(!lt_decimal(1, 12, 9, 1, 8, 6));
}

#[test]
fn test_leq_decimal() {

    assert!(leq_decimal(1, 2, 1, 1, 3, 1));
    assert!(leq_decimal(1, 2, 1, 1, 5, 2));
    assert!(leq_decimal(1, 3, 5, 1, 3, 2));
    assert!(leq_decimal(-1, 3, 5, 1, 1, 5));
    assert!(leq_decimal(-1, 3, 5, 1, 3, 5));
    assert!(leq_decimal(-1, 3, 1, -1, 2, 1));
    assert!(leq_decimal(-1, 5, 2, -1, 2, 1));
    assert!(leq_decimal(-1, 3, 2, -1, 3, 5));
    assert!(leq_decimal(1, 5, 15, 1, 12, 24));

    assert!(!leq_decimal(1, 3, 1, 1, 2, 1));
    assert!(!leq_decimal(1, 5, 2, 1, 2, 1));
    assert!(!leq_decimal(1, 3, 2, 1, 3, 5));
    assert!(!leq_decimal(1, 1, 5, -1, 3, 5));
    assert!(!leq_decimal(1, 3, 5, -1, 3, 5));
    assert!(!leq_decimal(-1, 2, 1, -1, 3, 1));
    assert!(!leq_decimal(-1, 2, 1, -1, 5, 2));
    assert!(!leq_decimal(-1, 3, 5, -1, 3, 2));
    assert!(!leq_decimal(1, 12, 24, 1, 5, 15));

    assert!(leq_decimal(1, 1, 1, 1, 1, 1));
    assert!(leq_decimal(1, 12, 9, 1, 8, 6));
}

#[test]
fn test_eq_decimal() {

    assert!(eq_decimal(1, 1, 1, 1, 1, 1));
    assert!(eq_decimal(1, 3, 5, 1, 3, 5));
    assert!(eq_decimal(1, 12, 9, 1, 8, 6));
    
    assert!(eq_decimal(-1, 1, 1, -1, 1, 1));
    assert!(eq_decimal(-1, 3, 5, -1, 3, 5));
    assert!(eq_decimal(-1, 12, 9, -1, 8, 6));

    assert!(!eq_decimal(1, 2, 1, 1, 3, 1));
    assert!(!eq_decimal(1, 2, 1, 1, 5, 2));
    assert!(!eq_decimal(1, 3, 5, 1, 3, 2));
    assert!(!eq_decimal(-1, 3, 5, 1, 1, 5));
    assert!(!eq_decimal(-1, 3, 5, 1, 3, 5));
    assert!(!eq_decimal(-1, 3, 1, -1, 2, 1));
    assert!(!eq_decimal(-1, 5, 2, -1, 2, 1));
    assert!(!eq_decimal(-1, 3, 2, -1, 3, 5));
    assert!(!eq_decimal(1, 5, 15, 1, 12, 24));

    assert!(!eq_decimal(1, 3, 1, 1, 2, 1));
    assert!(!eq_decimal(1, 5, 2, 1, 2, 1));
    assert!(!eq_decimal(1, 3, 2, 1, 3, 5));
    assert!(!eq_decimal(1, 1, 5, -1, 3, 5));
    assert!(!eq_decimal(1, 3, 5, -1, 3, 5));
    assert!(!eq_decimal(-1, 2, 1, -1, 3, 1));
    assert!(!eq_decimal(-1, 2, 1, -1, 5, 2));
    assert!(!eq_decimal(-1, 3, 5, -1, 3, 2));
    assert!(!eq_decimal(1, 12, 24, 1, 5, 15));
}

#[test]
fn test_neq_decimal() {

    assert!(!neq_decimal(1, 1, 1, 1, 1, 1));
    assert!(!neq_decimal(1, 3, 5, 1, 3, 5));
    assert!(!neq_decimal(1, 12, 9, 1, 8, 6));
    
    assert!(!neq_decimal(-1, 1, 1, -1, 1, 1));
    assert!(!neq_decimal(-1, 3, 5, -1, 3, 5));
    assert!(!neq_decimal(-1, 12, 9, -1, 8, 6));

    assert!(neq_decimal(1, 2, 1, 1, 3, 1));
    assert!(neq_decimal(1, 2, 1, 1, 5, 2));
    assert!(neq_decimal(1, 3, 5, 1, 3, 2));
    assert!(neq_decimal(-1, 3, 5, 1, 1, 5));
    assert!(neq_decimal(-1, 3, 5, 1, 3, 5));
    assert!(neq_decimal(-1, 3, 1, -1, 2, 1));
    assert!(neq_decimal(-1, 5, 2, -1, 2, 1));
    assert!(neq_decimal(-1, 3, 2, -1, 3, 5));
    assert!(neq_decimal(1, 5, 15, 1, 12, 24));

    assert!(neq_decimal(1, 3, 1, 1, 2, 1));
    assert!(neq_decimal(1, 5, 2, 1, 2, 1));
    assert!(neq_decimal(1, 3, 2, 1, 3, 5));
    assert!(neq_decimal(1, 1, 5, -1, 3, 5));
    assert!(neq_decimal(1, 3, 5, -1, 3, 5));
    assert!(neq_decimal(-1, 2, 1, -1, 3, 1));
    assert!(neq_decimal(-1, 2, 1, -1, 5, 2));
    assert!(neq_decimal(-1, 3, 5, -1, 3, 2));
    assert!(neq_decimal(1, 12, 24, 1, 5, 15));
}


#[test]
fn test_simplify_decimal() {

    // no simplification needed
    let (s, n, d) = simplify_decimal(1, 1, 2);
    assert_eq!(s, 1);
    assert_eq!(n, 1);
    assert_eq!(d, 2); 

    // simplification needed
    let (s, n, d) = simplify_decimal(1, 32, 24);
    assert_eq!(s, 1);
    assert_eq!(n, 4);
    assert_eq!(d, 3); 

    // zero
    let (s, n, d) = simplify_decimal(-1, 0, 24);
    assert_eq!(s, 1);
    assert_eq!(n, 0);
    assert_eq!(d, 1); 

}

#[test]
fn test_decimal_from_float() {

    let (s, n, d) = decimal_from_float64(0.5);
    assert_eq!(s, 1);
    assert_eq!(n, 1);
    assert_eq!(d, 2);

    let (s, n, d) = decimal_from_float64(0.25);
    assert_eq!(s, 1);
    assert_eq!(n, 1);
    assert_eq!(d, 4);

    let (s, n, d) = decimal_from_float64(2.25);
    assert_eq!(s, 1);
    assert_eq!(n, 9);
    assert_eq!(d, 4);

    let (s, n, d) = decimal_from_float64(-2.25);
    assert_eq!(s, -1);
    assert_eq!(n, 9);
    assert_eq!(d, 4);
}

#[test]
fn test_rem_decimal() {

    // test only for integers
    let (s, n, d) = rem_decimal(1, 5, 1, 1, 3, 1);
    assert_eq!(s, 1);
    assert_eq!(n, 2);
    assert_eq!(d, 1);

    let (s, n, d) = rem_decimal(1, 5, 1, 1, 2, 1);
    assert_eq!(s, 1);
    assert_eq!(n, 1);
    assert_eq!(d, 1);

    let (s, n, d) = rem_decimal(1, 123, 1, 1, 47, 1);
    assert_eq!(s, 1);
    assert_eq!(n, 29);
    assert_eq!(d, 1);
}