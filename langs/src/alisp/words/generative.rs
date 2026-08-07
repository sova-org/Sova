use crate::alisp::{EXECUTE_ELEM_ADDR, INDEX_REG, LIST_LEN_REG, words::Word};

use sova_core::{reg, sova_prog, vm::{EnvironmentFunc::*, control_asm::ControlASM::*, variable::Variable}};

pub const ALTERNATIVE : Word = Word {
    name: "alt",
    description: "Alternates between the items using the number of times the frame has been triggered.",
    example: None,
    prog: || sova_prog![
        Mod(FrameTriggers.into(), reg!(LIST_LEN_REG), reg!(INDEX_REG)),
        Index(StackBack, reg!(INDEX_REG), Variable::StackBack),
        CallProcedure(EXECUTE_ELEM_ADDR),
        Return
    ]
};

pub const CHOICE : Word = Word {
    name: "choice",
    description: "Randomly (uniform distribution) chooses an item.",
    example: None,
    prog: || sova_prog![
        Mod(RandomInt.into(), reg!(LIST_LEN_REG), reg!(INDEX_REG)),
        Index(Variable::StackBack, reg!(INDEX_REG), Variable::StackBack),
        CallProcedure(EXECUTE_ELEM_ADDR),
        Return
    ]
};

pub const GENERATIVE_WORDS : [Word ; 2] = [CHOICE, ALTERNATIVE];