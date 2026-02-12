use sova_core::{compiler::{CompilationError, Compiler}, vm::Language};
use std::collections::BTreeMap;

use sova_core::vm::{Program, debug_print};

use crate::bali::{
    bali_ast::{AltVariableGenerator, bali_as_asm, constants::DEBUG_INSTRUCTIONS},
    bali_grammar,
};

use lalrpop_util::ParseError;

#[derive(Debug)]
pub struct BaliCompiler;

impl Language for BaliCompiler {
    fn name(&self) -> &str {
        "bali"
    }
}

impl Compiler for BaliCompiler {

    fn compile(
        &self,
        script: &str,
        _args: &BTreeMap<String, String>,
    ) -> Result<Program, CompilationError> {
        let mut alt_variables = AltVariableGenerator::new("_alt".to_string());
        match bali_grammar::ProgramParser::new().parse(&mut alt_variables, script) {
            Ok(parsed) => {
                let res = bali_as_asm(parsed);
                match res {
                    Ok(res) => {
                        // print program for debug
                        if DEBUG_INSTRUCTIONS {
                            debug_print(&res, "PROGRAM".to_string(), "".to_string());
                        }
                        Ok(res)
                    }
                    Err(info) => Err(CompilationError {
                        lang: "BaLi".to_string(),
                        info,
                        from: 0,
                        to: 0,
                    }),
                }
            }
            Err(parse_error) => {
                let mut from = 0;
                let mut to = 0;
                match parse_error {
                    ParseError::InvalidToken { location: loc }
                    | ParseError::UnrecognizedEof {
                        location: loc,
                        expected: _,
                    } => {
                        from = loc;
                        to = loc;
                    }
                    ParseError::UnrecognizedToken {
                        token: (f, _, t),
                        expected: _,
                    }
                    | ParseError::ExtraToken { token: (f, _, t) } => {
                        from = f;
                        to = t;
                    }
                    ParseError::User { error: _ } => {}
                };
                Err(CompilationError {
                    lang: "BaLi".to_string(),
                    info: parse_error.to_string(),
                    from,
                    to,
                })
            }
        }
    }
}
