use crate::alisp::{EXECUTE_ELEM_ADDR, INDEX_REG, LIST_LEN_REG, words::Word};

use sova_core::{sova_prog, vm::{EnvironmentFunc::*, control_asm::ControlASM::*, variable::Variable}};

pub const ALTERNATIVE : Word = Word {
    name: "alt",
    example: None,
    prog: || sova_prog![
        Mod(FrameTriggers.into(), Variable::reg(LIST_LEN_REG), Variable::reg(INDEX_REG)),
        Index(Variable::StackBack, Variable::reg(INDEX_REG), Variable::StackBack),
        CallProcedure(EXECUTE_ELEM_ADDR),
        Return
    ]
};

pub const CHOICE : Word = Word {
    name: "choice",
    example: None,
    prog: || sova_prog![
        Mod(RandomInt.into(), Variable::reg(LIST_LEN_REG), Variable::reg(INDEX_REG)),
        Index(Variable::StackBack, Variable::reg(INDEX_REG), Variable::StackBack),
        CallProcedure(EXECUTE_ELEM_ADDR),
        Return
    ]
};

pub const GENERATIVE_WORDS : [Word ; 2] = [CHOICE, ALTERNATIVE];