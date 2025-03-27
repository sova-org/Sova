use crate::{compiler::{CompilationError, Compiler}, lang::Program};

use crate::compiler::dummylang::dummygrammar;

pub struct DummyCompiler;
impl Compiler for DummyCompiler {
    fn compile(&self, script : &str) -> Result<Program, CompilationError> {
        if let Ok(parsed) = dummygrammar::ProgParser::new().parse(script) {
            Ok(parsed.as_asm())
        } else {
            Err(CompilationError)
        }
    }
}
