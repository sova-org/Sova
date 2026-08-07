use sova_core::{reg, sova_prog};

use crate::alisp::{COMPUTE_REG, EXECUTE_ELEM_ADDR, LIST_LEN_REG, LIST_REG, words::Word};

pub const IF : Word = Word {
    name: "if",
    description: "According to the truth value of the first item, executes either the second or the third.",
    example: None,
    prog: || sova_prog![
        Pop(reg!(LIST_REG)),
        Len(reg!(LIST_REG), reg!(LIST_LEN_REG)),
        JumpIfLess(reg!(LIST_LEN_REG), 2.into(), 8),
        PushList(reg!(LIST_REG)),
        CallProcedure(EXECUTE_ELEM_ADDR),
        Delete(reg!(COMPUTE_REG)),
        Pop(reg!(COMPUTE_REG)),
        RelJumpIfNot(reg!(COMPUTE_REG), 3),
        CallProcedure(EXECUTE_ELEM_ADDR),
        Return,
        JumpIfLess(reg!(LIST_LEN_REG), 3.into(), 3),
        FreeStack(1.into()),
        CallProcedure(EXECUTE_ELEM_ADDR),
        Return,
        Error(vec!["Not enough arguments for 'if' !".into()]),
        Return
    ],
};

pub const QUOTE : Word = Word {
    name: "'",
    description: "Quotes the next item, returning it as without executing.",
    example: None,
    prog: || sova_prog![
        Return
    ]
};

pub const CONTROL_WORDS : [Word ; 2] = [IF, QUOTE];