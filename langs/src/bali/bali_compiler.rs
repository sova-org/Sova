use sova_core::{
    compiler::{CompilationError, Compiler},
    vm::Language,
};
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
        (1, 0, 0)
    }
    fn syntax(&self) -> Option<sova_core::vm::language::LanguageSyntax> {
        use sova_core::vm::language::{LanguageSyntax, SyntaxRule, TokenCategory::*};
        Some(LanguageSyntax {
            rules: vec![
                SyntaxRule::new(Comment, r";[^\n]*"),
                SyntaxRule::new(String, r#""([^"\\]|\\.)*""#),
                SyntaxRule::new(Variable, r"\b(dev|ch|v|dur):"),
                SyntaxRule::new(Number, r"-?\d+\.\d+|-?\d+"),
                SyntaxRule::new(Special, r"\(//|\b\d+//\d+\b|:f\b"),
                SyntaxRule::new(
                    Keyword,
                    r"\(\b(loop|eucloop|binloop|spread|ramp|with|pick|alt|seq|for|if|fun)\b|\(\?|\(>>|\(<<|\(>|\(<",
                ),
                SyntaxRule::new(
                    Builtin,
                    r"\(\b(note|def|prog|control|at|chanpress|osc|dirt)\b",
                ),
                SyntaxRule::new(Operator, r":(neg|rev|step)\b|sh:"),
                SyntaxRule::new(Operator, r"\(\b(and|or|not|lt|leq|gt|geq|==|!=)\b"),
                SyntaxRule::new(
                    Operator,
                    r"\(\b(rand|scale|clamp|min|max|quantize|sine|saw|triangle|isaw|randstep|ccin)\b|\(\+|\(\*|\(-|\(/|\(%",
                ),
                SyntaxRule::new(Symbol, r":[a-zA-Z_][a-zA-Z0-9_]*"),
                SyntaxRule::new(Punctuation, r"[(){}\[\]<>!]"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use sova_core::vm::language::TokenCategory;

    fn categories_for(text: &str) -> Vec<(String, TokenCategory)> {
        let compiler = BaliCompiler;
        let syntax = compiler.syntax().expect("syntax() returned None");
        let mut parts = Vec::new();
        let mut cats = Vec::new();
        let mut names = Vec::new();
        for (i, rule) in syntax.rules.iter().enumerate() {
            let name = format!("g{i}");
            parts.push(format!("(?P<{name}>{})", rule.pattern));
            names.push(name);
            cats.push(rule.category);
        }
        let regex = regex::Regex::new(&parts.join("|")).expect("regex failed to compile");
        let mut result = Vec::new();
        for caps in regex.captures_iter(text) {
            for (i, cat) in cats.iter().enumerate() {
                if let Some(m) = caps.name(&names[i]) {
                    result.push((text[m.start()..m.end()].to_owned(), *cat));
                    break;
                }
            }
        }
        result
    }

    #[test]
    fn syntax_regex_compiles() {
        let _ = categories_for("");
    }

    #[test]
    fn syntax_highlights_sample() {
        use TokenCategory::*;
        let tokens = categories_for(
            "; a bali program\n(loop 4 (note 60 90 dev:1 ch:1) (+ 1 2) :kick \"hello\" 3//4 :f)",
        );
        let has = |cat: TokenCategory| tokens.iter().any(|(_, c)| *c == cat);
        assert!(has(Comment), "missing Comment");
        assert!(has(Keyword), "missing Keyword");
        assert!(has(Builtin), "missing Builtin");
        assert!(has(Number), "missing Number");
        assert!(has(Variable), "missing Variable");
        assert!(has(Operator), "missing Operator");
        assert!(has(Symbol), "missing Symbol");
        assert!(has(String), "missing String");
        assert!(has(Special), "missing Special");
        assert!(has(Punctuation), "missing Punctuation");
    }
}
