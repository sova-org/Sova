use crate::{compiler::{CompilationError, Compiler}, lang::Program};

use crate::compiler::bali::{the_grammar_of_bali, bali_ast::bali_as_asm};

use lalrpop_util::ParseError;

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
                let mut from = 0;
                let mut to = 0;
                match parse_error {
                    ParseError::InvalidToken{location: loc} | ParseError::UnrecognizedEof{location: loc, expected: _} => {
                        from = loc;
                        to = loc;
                    },
                    ParseError::UnrecognizedToken{token: (f, _, t), expected: _} | ParseError::ExtraToken{token: (f, _, t)} => {
                        from = f;
                        to = t;
                    },
                    ParseError::User{error: _} => {},
                };
                Err(CompilationError{
                    lang: "BaLi".to_string(),
                    info: parse_error.to_string(),
                    from,
                    to,
                })
            },
        }
    }
}