use crate::alisp::{CONTEXT_REG, EXECUTE_ELEM_ADDR, words::Word};

use sova_core::{reg, sova_prog, vm::{EnvironmentFunc::*, control_asm::ControlASM::*, variable::Variable}};

pub const DEVICE_KEY : &str = "device";

pub const DEVICE : Word = Word {
    name: "device",
    description: "Sets the device to use and then executes the second item with that context.",
    example: None,
    prog: || sova_prog![
        PushList(StackBack),
        Insert(reg!(CONTEXT_REG), "device".into(), StackBack, reg!(CONTEXT_REG)),
        CallProcedure(EXECUTE_ELEM_ADDR),
        Return
    ]
};

pub const CONTEXT_WORDS : [Word ; 1] = [DEVICE]; 