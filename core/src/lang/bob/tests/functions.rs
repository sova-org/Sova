use super::compile_and_run;
use crate::vm::variable::VariableValue;

#[test]
fn function_simple() {
    let result = compile_and_run(
        "FUNC ADDTWO X Y : ADD X Y END;
         SET G.Z (CALL ADDTWO 3 4)",
    );
    assert_eq!(
        result.global_vars.get("Z"),
        Some(&VariableValue::Integer(7))
    );
}

#[test]
fn function_factorial() {
    // Recursive factorial using IF expression
    let result = compile_and_run(
        "FUNC FACT N :
            IF LTE N 1 : 1 ELSE : MUL N (CALL FACT SUB N 1) END
         END;
         SET G.X (CALL FACT 5)",
    );
    // 5! = 120
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(120))
    );
}

#[test]
fn function_fibonacci() {
    // Recursive Fibonacci using IF expression
    let result = compile_and_run(
        "FUNC FIB N :
            IF LTE N 1 : N ELSE : ADD (CALL FIB SUB N 1) (CALL FIB SUB N 2) END
         END;
         SET G.X (CALL FIB 10)",
    );
    // FIB(10) = 55
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(55))
    );
}

#[test]
fn function_gcd() {
    // Euclidean GCD using IF expression - clean implicit return
    let result = compile_and_run(
        "FUNC GCD A B :
            IF EQ B 0 : A ELSE : (CALL GCD B MOD A B) END
         END;
         SET G.X (CALL GCD 48 18)",
    );
    // GCD(48, 18) = 6
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(6))
    );
}

#[test]
fn function_power() {
    // Recursive power using IF expression
    let result = compile_and_run(
        "FUNC POW X N :
            IF EQ N 0 : 1 ELSE : MUL X (CALL POW X SUB N 1) END
         END;
         SET G.Y (CALL POW 2 10)",
    );
    // 2^10 = 1024
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(1024))
    );
}

#[test]
fn function_sum_1_to_n() {
    // Recursive sum using IF expression
    let result = compile_and_run(
        "FUNC SUM N :
            IF LTE N 0 : 0 ELSE : ADD N (CALL SUM SUB N 1) END
         END;
         SET G.X (CALL SUM 10)",
    );
    // 1+2+3+4+5+6+7+8+9+10 = 55
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(55))
    );
}

#[test]
fn function_ackermann() {
    // Ackermann function using nested IF expressions
    let result = compile_and_run(
        "FUNC ACK M N :
            IF EQ M 0 :
                ADD N 1
            ELSE :
                IF EQ N 0 :
                    (CALL ACK SUB M 1 1)
                ELSE :
                    (CALL ACK SUB M 1 (CALL ACK M SUB N 1))
                END
            END
         END;
         SET G.X (CALL ACK 2 3)",
    );
    // ACK(2,3) = 9
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(9))
    );
}

#[test]
fn function_max2_simple() {
    // Test MAX2 directly
    let result = compile_and_run(
        "FUNC MAX2 A B :
            IF GT A B : A ELSE : B END
         END;
         SET G.X (CALL MAX2 5 9)",
    );
    // 5 > 9 is false, should return B=9
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(9))
    );
}

#[test]
fn function_max2_nested() {
    // Test nested MAX2 calls from top-level
    let result = compile_and_run(
        "FUNC MAX2 A B :
            IF GT A B : A ELSE : B END
         END;
         SET G.X (CALL MAX2 5 (CALL MAX2 9 3))",
    );
    // Inner: MAX2 9 3 → 9 > 3 → true → 9
    // Outer: MAX2 5 9 → 5 > 9 → false → 9
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(9))
    );
}

#[test]
fn function_if_returns_param() {
    // Test IF expression returning a function parameter
    let result = compile_and_run(
        "FUNC TEST A :
            IF EQ A 0 : 99 ELSE : A END
         END;
         SET G.X (CALL TEST 5)",
    );
    // TEST 5 → A != 0 → return A → 5
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(5))
    );
}

#[test]
fn function_if_returns_second_param() {
    // Test IF expression returning second param (like MAX2 does)
    let result = compile_and_run(
        "FUNC TEST A B :
            IF GT A B : A ELSE : B END
         END;
         SET G.X (CALL TEST 5 9)",
    );
    // 5 > 9 is false → return B → 9
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(9))
    );
}

#[test]
fn function_call_from_function_simple() {
    // Simplest case: one function calling another
    let result = compile_and_run(
        "FUNC INNER X : X END;
         FUNC OUTER Y : (CALL INNER Y) END;
         SET G.X (CALL OUTER 42)",
    );
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
}

#[test]
fn function_max_of_three() {
    // MAX2 and MAX3 - clean implicit return, no workarounds needed
    let result = compile_and_run(
        "FUNC MAX2 A B :
            IF GT A B : A ELSE : B END
         END;
         FUNC MAX3 A B C :
            (CALL MAX2 A (CALL MAX2 B C))
         END;
         SET G.X (CALL MAX3 5 9 3)",
    );
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(9))
    );
}

#[test]
fn function_absolute_value() {
    let result = compile_and_run(
        "FUNC MYABS N :
            IF LT N 0 : NEG N ELSE : N END
         END;
         SET G.X (CALL MYABS NEG 42);
         SET G.Y (CALL MYABS 17)",
    );
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(42))
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(17))
    );
}

#[test]
fn function_min_of_array() {
    // Chain of MIN2 calls using IF expression
    let result = compile_and_run(
        "FUNC MIN2 A B :
            IF LT A B : A ELSE : B END
         END;
         SET G.X (CALL MIN2 (CALL MIN2 (CALL MIN2 8 3) 7) 5)",
    );
    // min(8, 3, 7, 5) = 3
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3))
    );
}

#[test]
fn function_collatz_steps() {
    // Count steps to reach 1 in Collatz sequence using nested IF expressions
    let result = compile_and_run(
        "FUNC COLLATZ N :
            IF EQ N 1 :
                0
            ELSE :
                IF EQ MOD N 2 0 :
                    ADD 1 (CALL COLLATZ DIV N 2)
                ELSE :
                    ADD 1 (CALL COLLATZ ADD MUL 3 N 1)
                END
            END
         END;
         SET G.X (CALL COLLATZ 7)",
    );
    // Collatz sequence for 7: 7→22→11→34→17→52→26→13→40→20→10→5→16→8→4→2→1 = 16 steps
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(16))
    );
}

#[test]
fn function_digit_sum() {
    // Recursive digit sum using IF expression
    let result = compile_and_run(
        "FUNC DIGSUM N :
            IF LT N 10 : N ELSE : ADD MOD N 10 (CALL DIGSUM DIV N 10) END
         END;
         SET G.X (CALL DIGSUM 12345)",
    );
    // 1+2+3+4+5 = 15
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(15))
    );
}

#[test]
fn func_implicit_return() {
    // Function returns the last expression in its body
    let result = compile_and_run(
        "FUNC DOUBLE X : * X 2 END;
         SET G.Y (CALL DOUBLE 7)",
    );
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(14))
    );
}

#[test]
fn func_implicit_return_with_body() {
    // Function with body statements and implicit return
    let result = compile_and_run(
        "FUNC COMPUTE X :
             SET G.A * X 2;
             SET G.B + G.A 10;
             G.B
         END;
         SET G.Y (CALL COMPUTE 5)",
    );
    // X=5, A=10, B=20
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(20))
    );
}

#[test]
fn function_triple_nested_calls_same_args() {
    // Three levels of function calls, all using same arg name
    // Tests that arg save/restore works correctly at each level
    let result = compile_and_run(
        "FUNC F1 X : ADD X 1 END;
         FUNC F2 X : ADD (CALL F1 X) 10 END;
         FUNC F3 X : ADD (CALL F2 X) 100 END;
         SET G.Y (CALL F3 5)",
    );
    // F3(5) -> F2(5) + 100 -> (F1(5) + 10) + 100 -> ((5+1) + 10) + 100 = 116
    assert_eq!(
        result.global_vars.get("Y"),
        Some(&VariableValue::Integer(116))
    );
}
