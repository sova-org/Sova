//! Operator registry and implementations for the Bob compiler.
//!
//! This module defines all built-in operators with their arities and compilation functions.
//! Operators are looked up by name and arity using `find_operator()`.

use crate::bob::context::{CompileContext, LabeledInstr, resolve_labels};
use sova_core::vm::control_asm::ControlASM;
use sova_core::vm::variable::{Variable, VariableValue};
use sova_core::vm::{EnvironmentFunc, Instruction};

// ============================================================================
// Operator Registry
// ============================================================================

pub(crate) type SimpleOpFn = fn(&[Variable], &Variable, &mut CompileContext) -> Vec<Instruction>;

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
        arity: 2,
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
        compile: op_rand,
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
    OpDef {
        name: "CCIN",
        arity: 1,
        compile: op_ccin_context,
    },
    OpDef {
        name: "CCIN",
        arity: 3,
        compile: op_ccin_explicit,
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
        fn $name(
            args: &[Variable],
            dest: &Variable,
            _ctx: &mut CompileContext,
        ) -> Vec<Instruction> {
            vec![Instruction::Control(ControlASM::$variant(
                args[0].clone(),
                dest.clone(),
            ))]
        }
    };
}

macro_rules! binary_op {
    ($name:ident, $variant:ident) => {
        fn $name(
            args: &[Variable],
            dest: &Variable,
            _ctx: &mut CompileContext,
        ) -> Vec<Instruction> {
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
        fn $name(
            args: &[Variable],
            dest: &Variable,
            _ctx: &mut CompileContext,
        ) -> Vec<Instruction> {
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
unary_op!(op_mlen, Len);
unary_op!(op_len, Len);

fn op_bnot(args: &[Variable], dest: &Variable, _ctx: &mut CompileContext) -> Vec<Instruction> {
    vec![Instruction::Control(ControlASM::BitXor(
        args[0].clone(),
        Variable::Constant(VariableValue::Integer(-1)),
        dest.clone(),
    ))]
}

fn op_pick(args: &[Variable], dest: &Variable, ctx: &mut CompileContext) -> Vec<Instruction> {
    let vec = args[0].clone();
    let len_var = ctx.temp("_bob_pick_len");
    let idx_var = ctx.temp("_bob_pick_idx");

    vec![
        Instruction::Control(ControlASM::Len(vec.clone(), len_var.clone())),
        Instruction::Control(ControlASM::Mov(
            Variable::Environment(EnvironmentFunc::RandomFloat),
            idx_var.clone(),
        )),
        Instruction::Control(ControlASM::Mul(
            idx_var.clone(),
            len_var.clone(),
            idx_var.clone(),
        )),
        Instruction::Control(ControlASM::Index(vec, idx_var, dest.clone())),
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
binary_op!(op_shl, ShiftLeftL);
binary_op!(op_shr, ShiftRightL);
binary_op!(op_min, Min);
binary_op!(op_max, Max);
binary_op!(op_qt, Quantize);
binary_op!(op_get, Index);

ternary_op!(op_clamp, Clamp);

fn op_toss(_args: &[Variable], dest: &Variable, _ctx: &mut CompileContext) -> Vec<Instruction> {
    vec![Instruction::Control(ControlASM::Mov(
        Variable::Environment(EnvironmentFunc::RandomUInt(2)),
        dest.clone(),
    ))]
}

fn op_neg(args: &[Variable], dest: &Variable, _ctx: &mut CompileContext) -> Vec<Instruction> {
    vec![Instruction::Control(ControlASM::Sub(
        Variable::Constant(VariableValue::Integer(0)),
        args[0].clone(),
        dest.clone(),
    ))]
}

fn op_abs(args: &[Variable], dest: &Variable, ctx: &mut CompileContext) -> Vec<Instruction> {
    let zero = Variable::Constant(VariableValue::Integer(0));
    let cond = ctx.temp("_bob_abs_cond");
    let negate_label = ctx.new_label();
    let end_label = ctx.new_label();

    let labeled = vec![
        LabeledInstr::Instr(Instruction::Control(ControlASM::LowerThan(
            args[0].clone(),
            zero.clone(),
            cond.clone(),
        ))),
        LabeledInstr::JumpIf(cond, negate_label.clone()),
        LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            args[0].clone(),
            dest.clone(),
        ))),
        LabeledInstr::Jump(end_label.clone()),
        LabeledInstr::Mark(negate_label),
        LabeledInstr::Instr(Instruction::Control(ControlASM::Sub(
            zero,
            args[0].clone(),
            dest.clone(),
        ))),
        LabeledInstr::Mark(end_label),
    ];
    resolve_labels(labeled)
}

/// Random integer in [low, high] (inclusive).
/// Used by both RAND and RRAND (they are identical).
fn op_rand(args: &[Variable], dest: &Variable, ctx: &mut CompileContext) -> Vec<Instruction> {
    let rand_var = ctx.temp("_bob_rand");
    let range = ctx.temp("_bob_range");
    let one = Variable::Constant(VariableValue::Integer(1));
    vec![
        // range = high - low + 1
        Instruction::Control(ControlASM::Sub(
            args[1].clone(),
            args[0].clone(),
            range.clone(),
        )),
        Instruction::Control(ControlASM::Add(range.clone(), one, range.clone())),
        // rand_var = random float [0, 1)
        Instruction::Control(ControlASM::Mov(
            Variable::Environment(EnvironmentFunc::RandomFloat),
            rand_var.clone(),
        )),
        // rand_var = rand_var * range
        Instruction::Control(ControlASM::Mul(
            rand_var.clone(),
            range.clone(),
            rand_var.clone(),
        )),
        // dest = Integer(0) then dest = rand_var cast to integer (truncation via Mov)
        Instruction::Control(ControlASM::Redefine(0.into(), dest.clone())),
        Instruction::Control(ControlASM::Mov(rand_var, dest.clone())),
        // dest = dest % range (ensure within bounds)
        Instruction::Control(ControlASM::Mod(dest.clone(), range, dest.clone())),
        // dest = dest + low
        Instruction::Control(ControlASM::Add(args[0].clone(), dest.clone(), dest.clone())),
    ]
}

fn op_mmerge(args: &[Variable], dest: &Variable, _ctx: &mut CompileContext) -> Vec<Instruction> {
    vec![Instruction::Control(ControlASM::BitOr(
        args[1].clone(),
        args[0].clone(),
        dest.clone(),
    ))]
}

/// Brownian walk: dest = current + random(-step, +step)
fn op_drunk(args: &[Variable], dest: &Variable, ctx: &mut CompileContext) -> Vec<Instruction> {
    let rand_var = ctx.temp("_bob_drunk_rand");
    let range = ctx.temp("_bob_drunk_range");
    let two = Variable::Constant(VariableValue::Integer(2));
    let one = Variable::Constant(VariableValue::Integer(1));
    vec![
        // range = 2 * step + 1
        Instruction::Control(ControlASM::Mul(two, args[1].clone(), range.clone())),
        Instruction::Control(ControlASM::Add(range.clone(), one, range.clone())),
        // rand_var = random float [0, 1) * range
        Instruction::Control(ControlASM::Mov(
            Variable::Environment(EnvironmentFunc::RandomFloat),
            rand_var.clone(),
        )),
        Instruction::Control(ControlASM::Mul(rand_var.clone(), range, rand_var.clone())),
        // truncate to integer: dest = Integer(0), then Mov casts float to int
        Instruction::Control(ControlASM::Redefine(0.into(), dest.clone())),
        Instruction::Control(ControlASM::Mov(rand_var, dest.clone())),
        // dest = dest - step (center around 0)
        Instruction::Control(ControlASM::Sub(dest.clone(), args[1].clone(), dest.clone())),
        // dest = current + dest
        Instruction::Control(ControlASM::Add(args[0].clone(), dest.clone(), dest.clone())),
    ]
}

fn op_wrap(args: &[Variable], dest: &Variable, ctx: &mut CompileContext) -> Vec<Instruction> {
    let range = ctx.temp("_bob_wrap_range");
    let offset = ctx.temp("_bob_wrap_offset");
    let cond = ctx.temp("_bob_wrap_cond");
    let zero = Variable::Constant(VariableValue::Integer(0));
    let end_label = ctx.new_label();

    resolve_labels(vec![
        // range = max - min
        LabeledInstr::Instr(Instruction::Control(ControlASM::Sub(
            args[2].clone(),
            args[1].clone(),
            range.clone(),
        ))),
        // offset = (val - min) % range
        LabeledInstr::Instr(Instruction::Control(ControlASM::Sub(
            args[0].clone(),
            args[1].clone(),
            offset.clone(),
        ))),
        LabeledInstr::Instr(Instruction::Control(ControlASM::Mod(
            offset.clone(),
            range.clone(),
            offset.clone(),
        ))),
        // if offset < 0: offset += range
        LabeledInstr::Instr(Instruction::Control(ControlASM::LowerThan(
            offset.clone(),
            zero,
            cond.clone(),
        ))),
        LabeledInstr::JumpIfNot(cond, end_label.clone()),
        LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
            offset.clone(),
            range,
            offset.clone(),
        ))),
        // dest = min + offset
        LabeledInstr::Mark(end_label),
        LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
            args[1].clone(),
            offset,
            dest.clone(),
        ))),
    ])
}

fn op_scale(args: &[Variable], dest: &Variable, _ctx: &mut CompileContext) -> Vec<Instruction> {
    vec![Instruction::Control(ControlASM::Scale(
        args[0].clone(),
        args[1].clone(),
        args[2].clone(),
        args[3].clone(),
        args[4].clone(),
        dest.clone(),
    ))]
}

fn op_ccin_context(
    args: &[Variable],
    dest: &Variable,
    _ctx: &mut CompileContext,
) -> Vec<Instruction> {
    vec![Instruction::Control(ControlASM::GetMidiCC(
        Variable::Instance("_use_context_device".to_string()),
        Variable::Instance("_use_context_channel".to_string()),
        args[0].clone(),
        dest.clone(),
    ))]
}

fn op_ccin_explicit(
    args: &[Variable],
    dest: &Variable,
    _ctx: &mut CompileContext,
) -> Vec<Instruction> {
    vec![Instruction::Control(ControlASM::GetMidiCC(
        args[1].clone(),
        args[2].clone(),
        args[0].clone(),
        dest.clone(),
    ))]
}
