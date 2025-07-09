use crate::{
    compiler::{CompilationError, Compiler},
};
use std::borrow::Cow;


use crate::lang::{
    Program, 
    debug_print,
};

use crate::compiler::bali::{the_grammar_of_bali, bali_ast::{bali_as_asm, AltVariableGenerator, constants::DEBUG_INSTRUCTIONS}};

use lalrpop_util::ParseError;

#[derive(Debug)]
pub struct BaliCompiler;
impl Compiler for BaliCompiler {
    fn name(&self) -> String {
        "bali".to_string()
    }

    fn compile(&self, script : &str) -> Result<Program, CompilationError> {
        let mut alt_variables = AltVariableGenerator::new("_alt".to_string());
        match the_grammar_of_bali::ProgramParser::new().parse(&mut alt_variables, script) {
            Ok(parsed) => {
                let res = bali_as_asm(parsed);
                match res {
                    Ok(res) => {
                        // print program for debug
                        if DEBUG_INSTRUCTIONS {
                            debug_print(&res, "PROGRAM".to_string(), "".to_string());
                        }
                        Ok(res)
                    },
                    Err(info) => Err(CompilationError{
                        lang: "BaLi".to_string(),
                        info,
                        from: 0,
                        to: 0,
                    }),
                }
            },
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

    fn syntax(&self) -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed(include_str!(
            "../../static/syntaxes/bali.sublime-syntax"
        )))
    }
}
