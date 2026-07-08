use sova_core::{reg, sova_prog};

use crate::alisp::{LIST_REG, words::Word};

pub const IF : Word = Word {
    name: "if",
    example: None,
    prog: || sova_prog![
        Pop(reg!(LIST_REG)),
        Return
    ],
};
