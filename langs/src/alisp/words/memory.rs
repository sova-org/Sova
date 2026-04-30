use crate::alisp::{EXECUTE_ELEM_ADDR, INDEX_REG, LIST_LEN_REG, words::Word};

use sova_core::{sova_prog, vm::{EnvironmentFunc::*, control_asm::ControlASM::*, variable::Variable}};

pub const LET : Word = Word {
    name: "let",
    example: None,
    prog: || sova_prog![
        
        Index(Variable::StackBack, Variable::reg(INDEX_REG), Variable::StackBack),
        CallProcedure(EXECUTE_ELEM_ADDR),
        Return
    ]
};

pub const MEMORY_WORDS : [Word ; 1] = [LET];