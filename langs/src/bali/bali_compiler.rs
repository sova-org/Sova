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
    fn syntax(&self) -> Option<sova_core::vm::language::LanguageSyntax> {
        use sova_core::vm::language::{LanguageSyntax, SyntaxRule, TokenCategory::*};
        let rule = |cat, pat: &str| SyntaxRule { category: cat, pattern: pat.to_owned() };
        Some(LanguageSyntax {
            rules: vec![
                // Comments
                rule(Comment, r";[^\n]*"),
                // String literals
                rule(String, r#""([^"\\]|\\.)*""#),
                // Context prefixes
                rule(Variable, r"\b(dev|ch|v|dur):"),
                // Decimal and integer literals
                rule(Number, r"-?\d+\.\d+|-?\d+"),
                // Timing fractions
                rule(Special, r"\(//|\b\d+//\d+\b|:f\b"),
                // Keywords (statement-level forms)
                rule(Keyword, r"\(\b(loop|eucloop|binloop|spread|ramp|with|pick|alt|seq|for|if|fun)\b|\(\?|\(>>|\(<<|\(>|\(<"),
                // Effects (leaf-level forms)
                rule(Builtin, r"\(\b(note|def|prog|control|at|chanpress|osc|dirt)\b"),
                // Loop context modifiers
                rule(Operator, r":(neg|rev|step)\b|sh:"),
                // Boolean operators
                rule(Operator, r"\(\b(and|or|not|lt|leq|gt|geq|==|!=)\b"),
                // Expression operators
                rule(Operator, r"\(\b(rand|scale|clamp|min|max|quantize|sine|saw|triangle|isaw|randstep|ccin)\b|\(\+|\(\*|\(-|\(/|\(%"),
                // Dirt keywords
                rule(Symbol, r":[a-zA-Z_][a-zA-Z0-9_]*"),
                // Brackets
                rule(Punctuation, r"[(){}\[\]<>!]"),
            ],
        })
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
