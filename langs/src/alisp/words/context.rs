use crate::alisp::words::Word;

use sova_core::vm::{EnvironmentFunc::*, control_asm::ControlASM::*, variable::Variable};

pub const DEVICE : Word = Word {
    name: "device",
    example: None,
    prog: || {
        vec![
            Return.into()
        ]
    }
};