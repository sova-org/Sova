use std::cell::LazyCell;

use crate::alisp::{INDEX_REG, LIST_LEN_REG, words::Word};

use sova_core::vm::{EnvironmentFunc::*, control_asm::ControlASM::*, variable::Variable};

pub const ALTERNATIVE : Word = Word {
    name: "alt",
    example: None,
    prog: || vec![
        Mod(FrameTriggers.into(), Variable::reg(LIST_LEN_REG), Variable::reg(INDEX_REG)).into(),
        Index(Variable::StackBack, Variable::reg(INDEX_REG), Variable::StackBack).into(),
        Return.into()
    ]
};

pub const CHOICE : Word = Word {
    name: "choice",
    example: None,
    prog: || vec![
        Mod(RandomInt.into(), Variable::reg(LIST_LEN_REG), Variable::reg(INDEX_REG)).into(),
        Index(Variable::StackBack, Variable::reg(INDEX_REG), Variable::StackBack).into(),
        Return.into()
    ]
};