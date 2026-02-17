use sova_core::compiler::CompilationState;
use sova_core::scene::script::Script;
use sova_core::vm::Language;
use sova_core::vm::interpreter::{Interpreter, InterpreterFactory};

use super::interpreter::ForthInterpreter;

pub struct ForthInterpreterFactory;

impl Language for ForthInterpreterFactory {
    fn name(&self) -> &str {
        "forth"
    }
    fn version(&self) -> (usize, usize, usize) {
        (1,0,0)
    }
    fn syntax(&self) -> Option<sova_core::vm::language::LanguageSyntax> {
        use sova_core::vm::language::{LanguageSyntax, SyntaxRule, TokenCategory::*};
        let rule = |cat, pat: &str| SyntaxRule { category: cat, pattern: pat.to_owned() };
        Some(LanguageSyntax {
            rules: vec![
                // Comments (backslash to end of line, and parenthetical)
                rule(Comment, r"\\[^\n]*|\([ \t][^)]*\)"),
                // Colon definitions
                rule(Keyword, r"\b:\b|;\b"),
                // Control flow
                rule(Keyword, r"(?i)\b(if|else|then|do|loop|begin|until|i)\b"),
                // Stack manipulation
                rule(Builtin, r"(?i)\b(dup|drop|swap|over|rot|nip|tuck|2dup|2drop|2swap)\b"),
                // Arithmetic builtins
                rule(Builtin, r"(?i)\b(mod|negate|abs|min|max)\b"),
                // Logic builtins
                rule(Builtin, r"(?i)\b(and|or|xor|not|invert)\b"),
                // Comparison (word-form)
                rule(Operator, r"\b(0=|0<|0>)\b"),
                // Numeric literals (hex, binary, decimal)
                rule(Number, r"\b0x[0-9a-fA-F]+\b|\b0b[01]+\b|-?\b\d+(\.\d+)?\b"),
                // Arithmetic/comparison operators
                rule(Operator, r"[+\-*/]|<>|<=|>=|[<>=]"),
            ],
        })
    }
}

impl InterpreterFactory for ForthInterpreterFactory {
    
    fn make_instance(&self, script: &Script) -> Result<Box<dyn Interpreter>, String> {
        Ok(Box::new(ForthInterpreter::new(script.content())))
    }

    fn check(&self, _script: &Script) -> CompilationState {
        // Parsed(None) indicates "checked and valid" without caching anything
        CompilationState::Parsed(None)
    }
}
