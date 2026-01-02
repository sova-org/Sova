use std::collections::BTreeMap;

use crate::{
    compiler::{CompilationError, Compiler},
    vm::Program,
};

use crate::compiler::dummyvm::dummygrammar;

#[derive(Debug)]
pub struct DummyCompiler;
impl Compiler for DummyCompiler {
    fn name(&self) -> &str {
        "dummy"
    }

    fn compile(&self, script: &str, _args: &BTreeMap<String, String>) -> Result<Program, CompilationError> {
        if let Ok(parsed) = dummygrammar::ProgParser::new().parse(script) {
            Ok(parsed.as_asm())
        } else {
            Err(CompilationError::default_error("dummylang".to_string()))
        }
    }
}
