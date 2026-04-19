use std::cell::LazyCell;

use sova_core::vm::{Program, variable::VariableValue};

pub mod generative;

pub struct Word {
    name: &'static str,
    example: Option<&'static str>,
    prog: fn() -> Program
}

impl Word {

    pub fn function(&self) -> VariableValue {
        VariableValue::Func((self.prog)())
    }

}