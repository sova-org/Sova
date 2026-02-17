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
    fn version(&self) -> (usize, usize, usize) {
        (1,0,0)
    }
    fn documentation(&self) -> sova_core::vm::language::LanguageDocumentation {
        use sova_core::vm::language::{LanguageDocumentation, LanguageElement::*, ReferenceEntry};
        let mut doc = LanguageDocumentation::default();
        doc.reference.insert(Word("play".into()), ReferenceEntry::new("Emit a note or event").with_example("play note: 60 vel: 100"));
        doc.reference.insert(Word("wait".into()), ReferenceEntry::new("Wait for a duration").with_example("play note: 60\nwait 1\nplay note: 64"));
        doc.reference.insert(Word("loop".into()), ReferenceEntry::new("Repeat a block").with_example("loop 4\n  play note: 60\n  wait 1\nend"));
        doc.reference.insert(Word("let".into()), ReferenceEntry::new("Bind a variable").with_example("let x = 60\nplay note: x"));
        doc.reference.insert(Word("fn".into()), ReferenceEntry::new("Define a function").with_example("fn hit n\n  play note: n\n  wait 1\nend"));
        doc.reference.insert(Word("if".into()), ReferenceEntry::new("Conditional expression"));
        doc
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
