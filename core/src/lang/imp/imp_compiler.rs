use crate::compiler::{CompilationError, Compiler};
use crate::lang::imp::imp_ast::ImpProgram;
use crate::lang::imp::imp_grammar;
use crate::vm::Program;
use lalrpop_util::ParseError;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ImpCompiler;

impl Compiler for ImpCompiler {
    fn name(&self) -> &str {
        "imp"
    }

    fn compile(
        &self,
        script: &str,
        _args: &BTreeMap<String, String>,
    ) -> Result<Program, CompilationError> {
        match imp_grammar::ProgramParser::new().parse(script) {
            Ok(parsed) => Ok(imp_as_asm(parsed)),
            Err(parse_error) => {
                let (from, to) = match &parse_error {
                    ParseError::InvalidToken { location } => (*location, *location),
                    ParseError::UnrecognizedEof { location, .. } => (*location, *location),
                    ParseError::UnrecognizedToken {
                        token: (f, _, t), ..
                    } => (*f, *t),
                    ParseError::ExtraToken { token: (f, _, t) } => (*f, *t),
                    ParseError::User { .. } => (0, 0),
                };
                Err(CompilationError {
                    lang: "Imp".to_string(),
                    info: parse_error.to_string(),
                    from,
                    to,
                })
            }
        }
    }
}

fn imp_as_asm(_program: ImpProgram) -> Program {
    Vec::new()
}
