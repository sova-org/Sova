use super::compile_and_run;
use sova_core::vm::variable::VariableValue;

#[test]
fn symbolic_operators() {
    let result = compile_and_run(
        "SET G.A + 2 3; SET G.B - 10 3; SET G.C * 4 5; SET G.D / 20 4; SET G.E % 17 5",
    );
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(5))
    ); // 2 + 3
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Integer(7))
    ); // 10 - 3
    assert_eq!(
        result.global_vars.get("C"),
        Some(&VariableValue::Integer(20))
    ); // 4 * 5
    assert_eq!(
        result.global_vars.get("D"),
        Some(&VariableValue::Integer(5))
    ); // 20 / 4
    assert_eq!(
        result.global_vars.get("E"),
        Some(&VariableValue::Integer(2))
    ); // 17 % 5
}

#[test]
fn symbolic_comparison() {
    let result = compile_and_run(
        "SET G.A > 5 3; SET G.B < 2 8; SET G.C >= 5 5; SET G.D <= 3 3; SET G.E == 4 4; SET G.F != 1 2",
    );
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Bool(true))
    ); // 5 > 3
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Bool(true))
    ); // 2 < 8
    assert_eq!(
        result.global_vars.get("C"),
        Some(&VariableValue::Bool(true))
    ); // 5 >= 5
    assert_eq!(
        result.global_vars.get("D"),
        Some(&VariableValue::Bool(true))
    ); // 3 <= 3
    assert_eq!(
        result.global_vars.get("E"),
        Some(&VariableValue::Bool(true))
    ); // 4 == 4
    assert_eq!(
        result.global_vars.get("F"),
        Some(&VariableValue::Bool(true))
    ); // 1 != 2
}

#[test]
fn not_symbolic() {
    let result = compile_and_run("SET G.A ! 0; SET G.B ! 1");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Bool(true))
    ); // !0 = true
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Bool(false))
    ); // !1 = false
}

#[test]
fn bor_integers() {
    // BOR on integers: bitwise or
    let result = compile_and_run("SET G.A BOR 3 5");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(7))
    ); // 3 | 5 = 7
}

#[test]
fn bor_symbolic_integers() {
    // | on integers: bitwise or
    let result = compile_and_run("SET G.A | 3 5");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(7))
    ); // 3 | 5 = 7
}

#[test]
fn bor_maps() {
    // BOR on maps: union with first wins
    let result = compile_and_run("SET G.M BOR [x: 1] [x: 2 y: 3]");
    match result.global_vars.get("M") {
        Some(VariableValue::Map(m)) => {
            assert_eq!(m.get("x"), Some(&VariableValue::Integer(1))); // first wins
            assert_eq!(m.get("y"), Some(&VariableValue::Integer(3)));
        }
        other => panic!("Expected map, got {:?}", other),
    }
}

#[test]
fn bor_maps_via_variables() {
    // BOR on maps via variables
    let result = compile_and_run("SET G.A [x: 1]; SET G.B [x: 2 y: 3]; SET G.M BOR G.A G.B");
    match result.global_vars.get("M") {
        Some(VariableValue::Map(m)) => {
            assert_eq!(m.get("x"), Some(&VariableValue::Integer(1))); // first wins
            assert_eq!(m.get("y"), Some(&VariableValue::Integer(3)));
        }
        other => panic!("Expected map via variables, got {:?}", other),
    }
}

#[test]
fn add_maps() {
    // ADD on maps: merge with recursive add on shared keys
    let result = compile_and_run("SET G.M ADD [x: 1] [x: 2 y: 3]");
    match result.global_vars.get("M") {
        Some(VariableValue::Map(m)) => {
            assert_eq!(m.get("x"), Some(&VariableValue::Integer(3))); // 1 + 2
            assert_eq!(m.get("y"), Some(&VariableValue::Integer(3)));
        }
        other => panic!("Expected map, got {:?}", other),
    }
}

#[test]
fn add_maps_via_variables() {
    // ADD on maps via variables
    let result = compile_and_run("SET G.A [x: 1]; SET G.B [x: 2 y: 3]; SET G.M ADD G.A G.B");
    match result.global_vars.get("M") {
        Some(VariableValue::Map(m)) => {
            assert_eq!(m.get("x"), Some(&VariableValue::Integer(3))); // 1 + 2
            assert_eq!(m.get("y"), Some(&VariableValue::Integer(3)));
        }
        other => panic!("Expected map, got {:?}", other),
    }
}

#[test]
fn division_truncates_toward_zero() {
    // Integer division should truncate toward zero
    let result = compile_and_run("SET G.A / 7 3; SET G.B / NEG 7 3; SET G.C / 7 NEG 3");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(2))
    ); // 7 / 3 = 2
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Integer(-2))
    ); // -7 / 3 = -2
    assert_eq!(
        result.global_vars.get("C"),
        Some(&VariableValue::Integer(-2))
    ); // 7 / -3 = -2
}

// =============================================================================
// Bitwise operators
// =============================================================================

#[test]
fn band_integers() {
    // BAND 12 10 → 8 (1100 & 1010 = 1000)
    let result = compile_and_run("SET G.A BAND 12 10; SET G.B & 12 10");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(8))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Integer(8))
    );
}

#[test]
fn bxor_integers() {
    // BXOR 12 10 → 6 (1100 ^ 1010 = 0110)
    let result = compile_and_run("SET G.A BXOR 12 10; SET G.B ^ 12 10");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(6))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Integer(6))
    );
}

#[test]
fn bnot_integer() {
    // BNOT 0 → -1 (all bits flipped)
    let result = compile_and_run("SET G.A BNOT 0; SET G.B ~ 0");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(-1))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Integer(-1))
    );
}

#[test]
fn shl_integer() {
    // SHL 1 4 → 16
    let result = compile_and_run("SET G.A SHL 1 4; SET G.B << 1 4");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(16))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Integer(16))
    );
}

#[test]
fn shr_integer() {
    // SHR 16 2 → 4 (>> is now used for emit, use SHR keyword only)
    let result = compile_and_run("SET G.A SHR 16 2");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(4))
    );
}

// =============================================================================
// Logical operators
// =============================================================================

#[test]
fn and_logical() {
    let result = compile_and_run("SET G.A AND 1 1; SET G.B AND 1 0; SET G.C AND 0 0");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Bool(true))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Bool(false))
    );
    assert_eq!(
        result.global_vars.get("C"),
        Some(&VariableValue::Bool(false))
    );
}

#[test]
fn or_logical() {
    let result = compile_and_run("SET G.A OR 1 1; SET G.B OR 1 0; SET G.C OR 0 0");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Bool(true))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Bool(true))
    );
    assert_eq!(
        result.global_vars.get("C"),
        Some(&VariableValue::Bool(false))
    );
}

#[test]
fn xor_logical() {
    let result = compile_and_run("SET G.A XOR 1 0; SET G.B XOR 1 1; SET G.C XOR 0 0");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Bool(true))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Bool(false))
    );
    assert_eq!(
        result.global_vars.get("C"),
        Some(&VariableValue::Bool(false))
    );
}

// =============================================================================
// Utility operators
// =============================================================================

#[test]
fn neg_operator() {
    let result = compile_and_run("SET G.A NEG 5; SET G.B NEG NEG 5");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(-5))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Integer(5))
    );
}

#[test]
fn abs_operator() {
    let result = compile_and_run("SET G.A ABS 5; SET G.B ABS NEG 5");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Integer(5))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Integer(5))
    );
}

#[test]
fn min_operator() {
    // MIN returns Float (coerces inputs)
    let result = compile_and_run("SET G.A MIN 3 7; SET G.B MIN 10 2");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Float(3.0))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Float(2.0))
    );
}

#[test]
fn max_operator() {
    // MAX returns Float (coerces inputs)
    let result = compile_and_run("SET G.A MAX 3 7; SET G.B MAX 10 2");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Float(7.0))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Float(10.0))
    );
}

#[test]
fn scale_remap() {
    // SCALE 5 0 10 0 100 → 50 (remap 5 from [0,10] to [0,100])
    let result = compile_and_run("SET G.X SCALE 5 0 10 0 100");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Float(50.0))
    );
}

#[test]
fn scale_remap_offset() {
    // SCALE 0 0 10 100 200 → 100 (remap 0 from [0,10] to [100,200])
    let result = compile_and_run("SET G.X SCALE 0 0 10 100 200");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Float(100.0))
    );
}

#[test]
fn qt_quantize() {
    // QT 7 4 → 8 (quantize 7 to nearest multiple of 4)
    let result = compile_and_run("SET G.A QT 7 4; SET G.B QT 5 4; SET G.C QT 2 4");
    assert_eq!(
        result.global_vars.get("A"),
        Some(&VariableValue::Float(8.0))
    );
    assert_eq!(
        result.global_vars.get("B"),
        Some(&VariableValue::Float(4.0))
    );
    assert_eq!(
        result.global_vars.get("C"),
        Some(&VariableValue::Float(4.0))
    );
}

#[test]
fn drunk_bounded() {
    // DRUNK X step returns X ± random(0..step)
    // Test that result is within expected bounds
    let result = compile_and_run("SET G.X DRUNK 50 5");
    if let Some(VariableValue::Integer(x)) = result.global_vars.get("X") {
        assert!(
            *x >= 45 && *x <= 55,
            "DRUNK 50 5 produced {}, expected 45-55",
            x
        );
    } else {
        panic!("X should be an integer");
    }
}
