use std::collections::BTreeMap;

use sova_core::{
    compiler::{CompilationError, Compiler},
    vm::{Language, Program},
};

use crate::dummylang::dummygrammar;

#[derive(Debug)]
pub struct DummyCompiler;

impl Language for DummyCompiler {
    fn name(&self) -> &str {
        "dummy"
    }
    fn version(&self) -> (usize, usize, usize) {
        (1,0,0)
    }
}

impl Compiler for DummyCompiler {

    fn compile(
        &self,
        script: &str,
        _args: &BTreeMap<String, String>,
    ) -> Result<Program, CompilationError> {
        if let Ok(parsed) = dummygrammar::ProgParser::new().parse(script) {
            Ok(parsed.as_asm())
        } else {
            Err(CompilationError::default_error("dummylang".to_string()))
        }
    }
}
