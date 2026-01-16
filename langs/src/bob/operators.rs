//! Operator registry and implementations for the Bob compiler.
//!
//! This module defines all built-in operators with their arities and compilation functions.
//! Operators are looked up by name and arity using `find_operator()`.

use sova_core::vm::control_asm::ControlASM;
use sova_core::vm::variable::{Variable, VariableValue};
use sova_core::vm::{EnvironmentFunc, Instruction};

// ============================================================================
// Operator Registry
// ============================================================================

pub(crate) type SimpleOpFn = fn(&[Variable], &Variable) -> Vec<Instruction>;

pub(crate) struct OpDef {
    pub name: &'static str,
    pub arity: usize,
    pub compile: SimpleOpFn,
}

pub(crate) const OPERATORS: &[OpDef] = &[
    OpDef {
        name: "TOSS",
        arity: 0,
        compile: op_toss,
    },
    OpDef {
        name: "NEG",
        arity: 1,
        compile: op_neg,
    },
    OpDef {
        name: "NOT",
        arity: 1,
        compile: op_not,
    },
    OpDef {
        name: "BNOT",
        arity: 1,
        compile: op_bnot,
    },
    OpDef {
        name: "ABS",
        arity: 1,
        compile: op_abs,
    },
    OpDef {
        name: "RAND",
        arity: 1,
        compile: op_rand,
    },
    OpDef {
        name: "ADD",
        arity: 2,
        compile: op_add,
    },
    OpDef {
        name: "SUB",
        arity: 2,
        compile: op_sub,
    },
    OpDef {
        name: "MUL",
        arity: 2,
        compile: op_mul,
    },
    OpDef {
        name: "DIV",
        arity: 2,
        compile: op_div,
    },
    OpDef {
        name: "MOD",
        arity: 2,
        compile: op_mod,
    },
    OpDef {
        name: "GT",
        arity: 2,
        compile: op_gt,
    },
    OpDef {
        name: "LT",
        arity: 2,
        compile: op_lt,
    },
    OpDef {
        name: "GTE",
        arity: 2,
        compile: op_gte,
    },
    OpDef {
        name: "LTE",
        arity: 2,
        compile: op_lte,
    },
    OpDef {
        name: "EQ",
        arity: 2,
        compile: op_eq,
    },
    OpDef {
        name: "NE",
        arity: 2,
        compile: op_ne,
    },
    OpDef {
        name: "AND",
        arity: 2,
        compile: op_and,
    },
    OpDef {
        name: "OR",
        arity: 2,
        compile: op_or,
    },
    OpDef {
        name: "XOR",
        arity: 2,
        compile: op_xor,
    },
    OpDef {
        name: "BAND",
        arity: 2,
        compile: op_band,
    },
    OpDef {
        name: "BOR",
        arity: 2,
        compile: op_bor,
    },
    OpDef {
        name: "MMERGE",
        arity: 2,
        compile: op_mmerge,
    },
    OpDef {
        name: "MLEN",
        arity: 1,
        compile: op_mlen,
    },
    OpDef {
        name: "BXOR",
        arity: 2,
        compile: op_bxor,
    },
    OpDef {
        name: "SHL",
        arity: 2,
        compile: op_shl,
    },
    OpDef {
        name: "SHR",
        arity: 2,
        compile: op_shr,
    },
    OpDef {
        name: "MIN",
        arity: 2,
        compile: op_min,
    },
    OpDef {
        name: "MAX",
        arity: 2,
        compile: op_max,
    },
    OpDef {
        name: "QT",
        arity: 2,
        compile: op_qt,
    },
    OpDef {
        name: "RRAND",
        arity: 2,
        compile: op_rrand,
    },
    OpDef {
        name: "DRUNK",
        arity: 2,
        compile: op_drunk,
    },
    OpDef {
        name: "CLAMP",
        arity: 3,
        compile: op_clamp,
    },
    OpDef {
        name: "WRAP",
        arity: 3,
        compile: op_wrap,
    },
    OpDef {
        name: "SCALE",
        arity: 5,
        compile: op_scale,
    },
    OpDef {
        name: "LEN",
        arity: 1,
        compile: op_len,
    },
    OpDef {
        name: "GET",
        arity: 2,
        compile: op_get,
    },
    OpDef {
        name: "PICK",
        arity: 1,
        compile: op_pick,
    },
];

pub(crate) fn find_operator(name: &str, arity: usize) -> Option<&'static OpDef> {
    OPERATORS
        .iter()
        .find(|op| op.name == name && op.arity == arity)
}

// ============================================================================
// Operator Macros
// ============================================================================

macro_rules! unary_op {
    ($name:ident, $variant:ident) => {
        fn $name(args: &[Variable], dest: &Variable) -> Vec<Instruction> {
            vec![Instruction::Control(ControlASM::$variant(
                args[0].clone(),
                dest.clone(),
            ))]
        }
    };
}

macro_rules! binary_op {
    ($name:ident, $variant:ident) => {
        fn $name(args: &[Variable], dest: &Variable) -> Vec<Instruction> {
            vec![Instruction::Control(ControlASM::$variant(
                args[0].clone(),
                args[1].clone(),
                dest.clone(),
            ))]
        }
    };
}

macro_rules! ternary_op {
    ($name:ident, $variant:ident) => {
        fn $name(args: &[Variable], dest: &Variable) -> Vec<Instruction> {
            vec![Instruction::Control(ControlASM::$variant(
                args[0].clone(),
                args[1].clone(),
                args[2].clone(),
                dest.clone(),
            ))]
        }
    };
}

// ============================================================================
// Operator Implementations
// ============================================================================

unary_op!(op_not, Not);
unary_op!(op_bnot, BitNot);
unary_op!(op_mlen, MapLen);
unary_op!(op_len, VecLen);
fn op_pick(args: &[Variable], dest: &Variable) -> Vec<Instruction> {
    let vec = args[0].clone();
    let len_var = Variable::Instance("_bob_pick_len".to_string());
    let idx_var = Variable::Instance("_bob_pick_idx".to_string());

    vec![
        // Get vector length
        Instruction::Control(ControlASM::VecLen(vec.clone(), len_var.clone())),
        // Get random float 0-1
        Instruction::Control(ControlASM::Mov(
            Variable::Environment(EnvironmentFunc::RandomFloat),
            idx_var.clone(),
        )),
        // Multiply by length to get 0-len range
        Instruction::Control(ControlASM::Mul(
            idx_var.clone(),
            len_var.clone(),
            idx_var.clone(),
        )),
        // Get element at index (VecGet handles empty vec by returning 0)
        // No need to cast to integer, VecGet will do it
        Instruction::Control(ControlASM::VecGet(vec, idx_var, dest.clone())),
    ]
}

binary_op!(op_add, Add);
binary_op!(op_sub, Sub);
binary_op!(op_mul, Mul);
binary_op!(op_div, Div);
binary_op!(op_mod, Mod);
binary_op!(op_gt, GreaterThan);
binary_op!(op_lt, LowerThan);
binary_op!(op_gte, GreaterOrEqual);
binary_op!(op_lte, LowerOrEqual);
binary_op!(op_eq, Equal);
binary_op!(op_ne, Different);
binary_op!(op_and, And);
binary_op!(op_or, Or);
binary_op!(op_xor, Xor);
binary_op!(op_band, BitAnd);
binary_op!(op_bor, BitOr);
binary_op!(op_bxor, BitXor);
binary_op!(op_shl, ShiftLeft);
binary_op!(op_shr, ShiftRightA);
binary_op!(op_min, Min);
binary_op!(op_max, Max);
binary_op!(op_qt, Quantize);
binary_op!(op_get, VecGet);

ternary_op!(op_clamp, Clamp);

fn op_toss(_args: &[Variable], dest: &Variable) -> Vec<Instruction> {
    vec![Instruction::Control(ControlASM::Mov(
        Variable::Environment(EnvironmentFunc::RandomUInt(2)),
        dest.clone(),
    ))]
}

fn op_neg(args: &[Variable], dest: &Variable) -> Vec<Instruction> {
    vec![Instruction::Control(ControlASM::Sub(
        Variable::Constant(VariableValue::Integer(0)),
        args[0].clone(),
        dest.clone(),
    ))]
}

fn op_abs(args: &[Variable], dest: &Variable) -> Vec<Instruction> {
    let zero = Variable::Constant(VariableValue::Integer(0));
    let cond = Variable::Instance("_bob_abs_cond".to_string());
    vec![
        Instruction::Control(ControlASM::LowerThan(
            args[0].clone(),
            zero.clone(),
            cond.clone(),
        )),
        Instruction::Control(ControlASM::RelJumpIfNot(cond, 3)),
        Instruction::Control(ControlASM::Sub(zero, args[0].clone(), dest.clone())),
        Instruction::Control(ControlASM::RelJump(2)),
        Instruction::Control(ControlASM::Mov(args[0].clone(), dest.clone())),
    ]
}

fn op_rand(args: &[Variable], dest: &Variable) -> Vec<Instruction> {
    let rand_var = Variable::Instance("_bob_rand".to_string());
    let one = Variable::Constant(VariableValue::Integer(1));
    let range = Variable::Instance("_bob_range".to_string());
    vec![
        Instruction::Control(ControlASM::Add(args[0].clone(), one, range.clone())),
        Instruction::Control(ControlASM::Mov(
            Variable::Environment(EnvironmentFunc::RandomFloat),
            rand_var.clone(),
        )),
        Instruction::Control(ControlASM::Mul(
            rand_var.clone(),
            range.clone(),
            rand_var.clone(),
        )),
        Instruction::Control(ControlASM::Redefine(0.into(), dest.clone())),
        Instruction::Control(ControlASM::Mov(rand_var, dest.clone())),
        Instruction::Control(ControlASM::Mod(dest.clone(), range, dest.clone())),
    ]
}

fn op_mmerge(args: &[Variable], dest: &Variable) -> Vec<Instruction> {
    vec![Instruction::Control(ControlASM::BitOr(
        args[1].clone(),
        args[0].clone(),
        dest.clone(),
    ))]
}

fn op_rrand(args: &[Variable], dest: &Variable) -> Vec<Instruction> {
    let rand_var = Variable::Instance("_bob_rand".to_string());
    let range = Variable::Instance("_bob_range".to_string());
    let one = Variable::Constant(VariableValue::Integer(1));
    vec![
        Instruction::Control(ControlASM::Sub(
            args[1].clone(),
            args[0].clone(),
            range.clone(),
        )),
        Instruction::Control(ControlASM::Add(range.clone(), one, range.clone())),
        Instruction::Control(ControlASM::Mov(
            Variable::Environment(EnvironmentFunc::RandomFloat),
            rand_var.clone(),
        )),
        Instruction::Control(ControlASM::Mul(
            rand_var.clone(),
            range.clone(),
            rand_var.clone(),
        )),
        Instruction::Control(ControlASM::Redefine(0.into(), rand_var.clone())),
        Instruction::Control(ControlASM::Mov(rand_var.clone(), rand_var.clone())),
        Instruction::Control(ControlASM::Mod(rand_var.clone(), range, rand_var.clone())),
        Instruction::Control(ControlASM::Add(args[0].clone(), rand_var, dest.clone())),
    ]
}

fn op_drunk(args: &[Variable], dest: &Variable) -> Vec<Instruction> {
    let rand_var = Variable::Instance("_bob_rand".to_string());
    let range = Variable::Instance("_bob_range".to_string());
    let two = Variable::Constant(VariableValue::Integer(2));
    let one = Variable::Constant(VariableValue::Integer(1));
    vec![
        Instruction::Control(ControlASM::Mul(two, args[1].clone(), range.clone())),
        Instruction::Control(ControlASM::Add(range.clone(), one, range.clone())),
        Instruction::Control(ControlASM::Mov(
            Variable::Environment(EnvironmentFunc::RandomFloat),
            rand_var.clone(),
        )),
        Instruction::Control(ControlASM::Mul(rand_var.clone(), range, rand_var.clone())),
        Instruction::Control(ControlASM::Redefine(0.into(), rand_var.clone())),
        Instruction::Control(ControlASM::Mov(rand_var.clone(), rand_var.clone())),
        Instruction::Control(ControlASM::Sub(
            rand_var.clone(),
            args[1].clone(),
            rand_var.clone(),
        )),
        Instruction::Control(ControlASM::Add(args[0].clone(), rand_var, dest.clone())),
    ]
}

fn op_wrap(args: &[Variable], dest: &Variable) -> Vec<Instruction> {
    let range = Variable::Instance("_bob_wrap_range".to_string());
    let offset = Variable::Instance("_bob_wrap_offset".to_string());
    let cond = Variable::Instance("_bob_wrap_cond".to_string());
    let zero = Variable::Constant(VariableValue::Integer(0));
    vec![
        Instruction::Control(ControlASM::Sub(
            args[2].clone(),
            args[1].clone(),
            range.clone(),
        )),
        Instruction::Control(ControlASM::Sub(
            args[0].clone(),
            args[1].clone(),
            offset.clone(),
        )),
        Instruction::Control(ControlASM::Mod(
            offset.clone(),
            range.clone(),
            offset.clone(),
        )),
        Instruction::Control(ControlASM::LowerThan(offset.clone(), zero, cond.clone())),
        Instruction::Control(ControlASM::RelJumpIfNot(cond, 2)),
        Instruction::Control(ControlASM::Add(offset.clone(), range, offset.clone())),
        Instruction::Control(ControlASM::Add(args[1].clone(), offset, dest.clone())),
    ]
}

fn op_scale(args: &[Variable], dest: &Variable) -> Vec<Instruction> {
    vec![Instruction::Control(ControlASM::Scale(
        args[0].clone(),
        args[1].clone(),
        args[2].clone(),
        args[3].clone(),
        args[4].clone(),
        dest.clone(),
    ))]
}
