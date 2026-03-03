use rhai::{Engine, ParseError};
use sova_core::{
    compiler::{CompilationError, CompilationState},
    scene::script::Script,
    vm::{
        Language,
        interpreter::{Interpreter, InterpreterFactory},
        language::{
            LanguageDocumentation, LanguageElement::Word, LanguageSyntax, ReferenceEntry,
            SyntaxRule, TokenCategory::*,
        },
    },
};

use super::{interpreter::RhaiInterpreter, lowering::lower_ast, runtime::RhaiExecutor};

pub struct RhaiInterpreterFactory;

impl RhaiInterpreterFactory {
    fn build_executor(script: &str) -> Result<RhaiExecutor, CompilationError> {
        let engine = Engine::new();
        let ast = engine
            .compile(script)
            .map_err(parse_error_to_compilation_error)?;
        let lowered = lower_ast(&ast).map_err(|info| CompilationError {
            lang: "rhai".to_string(),
            info,
            from: 0,
            to: 0,
        })?;
        Ok(RhaiExecutor::new(lowered))
    }
}

impl Language for RhaiInterpreterFactory {
    fn name(&self) -> &str {
        "rhai"
    }

    fn version(&self) -> (usize, usize, usize) {
        (1, 0, 0)
    }

    fn documentation(&self) -> LanguageDocumentation {
        let mut doc = LanguageDocumentation::default();

        doc.articles.push((
            "Introduction".into(),
            include_str!("../../docs/rhai/intro.md").into(),
        ));
        doc.articles.push((
            "Language Reference".into(),
            include_str!("../../docs/rhai/reference.md").into(),
        ));

        doc.reference.insert(
            Word("EMIT".into()),
            ReferenceEntry::new("Emit a Generic event from Rhai.")
                .with_example("EMIT(#{ note: 60 })")
                .with_category("Event"),
        );
        doc.reference.insert(
            Word("DELAY".into()),
            ReferenceEntry::new("Yield a scheduler delay using a TimeSpan.")
                .with_example("DELAY(beats(0.25))")
                .with_category("Time"),
        );
        doc.reference.insert(
            Word("beats".into()),
            ReferenceEntry::new("Create a beats-based TimeSpan.")
                .with_example("beats(1.0)")
                .with_category("Time"),
        );
        doc.reference.insert(
            Word("frames".into()),
            ReferenceEntry::new("Create a frames-based TimeSpan.")
                .with_example("frames(0.5)")
                .with_category("Time"),
        );
        doc.reference.insert(
            Word("micros".into()),
            ReferenceEntry::new("Create a micros-based TimeSpan.")
                .with_example("micros(120000)")
                .with_category("Time"),
        );
        doc.reference.insert(
            Word("g_ / l_ / f_".into()),
            ReferenceEntry::new("Variable scope prefixes: global / line / frame.")
                .with_example("g_bpm = 128; l_step = 2; f_gate = 1")
                .with_category("Variables"),
        );

        doc
    }

    fn syntax(&self) -> Option<LanguageSyntax> {
        Some(LanguageSyntax {
            rules: vec![
                SyntaxRule::new(Comment, r"//[^\n]*"),
                SyntaxRule::new(String, r#""([^"\\]|\\.)*"|'([^'\\]|\\.)*'"#),
                SyntaxRule::new(Number, r"-?\d+\.\d+|-?\d+"),
                SyntaxRule::new(Builtin, r"\b(?:EMIT|DELAY|beats|frames|micros)\b"),
                SyntaxRule::new(
                    Keyword,
                    r"\b(?:if|else|while|loop|for|in|let|break|continue)\b",
                ),
                SyntaxRule::new(Variable, r"\b(?:g_|l_|f_)?[a-zA-Z_][a-zA-Z0-9_]*\b"),
                SyntaxRule::new(Operator, r"\+=|-=|\*=|/=|%=|\*\*=|\^=|<<=|>>=|\|=|&="),
                SyntaxRule::new(
                    Operator,
                    r"\*\*|==|!=|<=|>=|&&|\|\||<<|>>|\?\?|[+\-*/%<>&|^~=!]",
                ),
                SyntaxRule::new(Punctuation, r"[#{}\[\](),.:;]"),
            ],
        })
    }
}

impl InterpreterFactory for RhaiInterpreterFactory {
    fn make_instance(&self, script: &Script) -> Result<Box<dyn Interpreter>, String> {
        let executor = Self::build_executor(script.content()).map_err(|err| err.to_string())?;
        Ok(Box::new(RhaiInterpreter::new(executor)))
    }

    fn check(&self, script: &Script) -> CompilationState {
        match Self::build_executor(script.content()) {
            Ok(_) => CompilationState::Parsed(None),
            Err(err) => CompilationState::Error(err),
        }
    }
}

fn parse_error_to_compilation_error(err: ParseError) -> CompilationError {
    let pos = err.position();
    let info = if let Some(line) = pos.line() {
        let col = pos.position().unwrap_or(0);
        format!("line {line}, col {col}: {err}")
    } else {
        err.to_string()
    };

    CompilationError {
        lang: "rhai".to_string(),
        info,
        from: 0,
        to: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sova_core::vm::language::TokenCategory;

    fn categories_for(text: &str) -> Vec<(String, TokenCategory)> {
        let factory = RhaiInterpreterFactory;
        let syntax = factory.syntax().expect("syntax() returned None");
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
            "// rhai\nlet g_x = 1\nfor (x, i) in [1,2] { x += i; EMIT(#{note: x}); DELAY(beats(0.25)); }",
        );
        let has = |cat: TokenCategory| tokens.iter().any(|(_, c)| *c == cat);
        assert!(has(Comment), "missing Comment");
        assert!(has(Keyword), "missing Keyword");
        assert!(has(Builtin), "missing Builtin");
        assert!(has(Number), "missing Number");
        assert!(has(Variable), "missing Variable");
        assert!(has(Operator), "missing Operator");
        assert!(has(Punctuation), "missing Punctuation");
    }
}
