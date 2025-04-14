use crate::{compiler::{CompilationError, Compiler}, lang::Program};

use crate::compiler::bali::{the_grammar_of_bali, bali_ast::bali_as_asm};

#[derive(Debug)]
pub struct BaliCompiler;
impl Compiler for BaliCompiler {

    fn name(&self) -> String {
        "bali".to_string()
    }

    fn compile(&self, script : &str) -> Result<Program, CompilationError> {
        match the_grammar_of_bali::ProgramParser::new().parse(script) {
            Ok(parsed) => Ok(bali_as_asm(parsed)),
            Err(parse_error) => {
                Err(CompilationError{
                    lang: "BaLi".to_string(),
                    info: parse_error.to_string()
                })
            },
        }
    }
}