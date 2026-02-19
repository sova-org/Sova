use sova_core::compiler::{CompilationError, CompilationState};
use sova_core::scene::script::Script;
use sova_core::vm::Language;
use sova_core::vm::interpreter::{Interpreter, InterpreterFactory};
use sova_core::vm::language::{LanguageDocumentation, LanguageElement::*, ReferenceEntry};

use super::interpreter::CagireInterpreter;
use super::words::WORDS;

pub struct CagireInterpreterFactory;

impl Language for CagireInterpreterFactory {
    fn name(&self) -> &str {
        "cagire"
    }

    fn version(&self) -> (usize, usize, usize) {
        (1, 0, 0)
    }

    fn documentation(&self) -> LanguageDocumentation {
        let mut doc = LanguageDocumentation::default();

        doc.articles.push(("Introduction".into(), include_str!("../../docs/cagire/intro.md").into()));
        doc.articles.push(("Language Reference".into(), include_str!("../../docs/cagire/reference.md").into()));

        for word in WORDS.iter() {
            let desc = if word.example.is_empty() {
                format!("{} {}", word.stack, word.desc)
            } else {
                format!("{} {} — {}", word.stack, word.desc, word.example)
            };
            let mut entry = ReferenceEntry::new(&desc)
                .with_category(word.category);
            if !word.example.is_empty() {
                entry = entry.with_example(word.example);
            }
            if !word.aliases.is_empty() {
                entry = entry.with_aliases(word.aliases);
            }
            doc.reference.insert(Word(word.name.into()), entry);
        }

        doc
    }

    fn syntax(&self) -> Option<sova_core::vm::language::LanguageSyntax> {
        use sova_core::vm::language::{LanguageSyntax, SyntaxRule, TokenCategory::*};

        // Bucket words by highlight category
        let mut builtin_words = Vec::new();
        let mut symbol_words = Vec::new();
        let mut special_words = Vec::new();

        for word in WORDS.iter() {
            if !word.name.chars().next().is_some_and(|c| c.is_alphanumeric()) {
                continue;
            }
            if word.name.contains('<') {
                continue;
            }

            let bucket = match word.category {
                "Stack" | "Arithmetic" | "Comparison" | "Logic" | "Control" | "Definitions" =>
                    &mut builtin_words,
                "Sample" | "Oscillator" | "Wavetable" | "FM" | "Modulation"
                | "Envelope" | "Filter" | "Reverb" | "Delay" | "Lo-fi" | "Stereo"
                | "Mod FX" | "MIDI" | "Context" =>
                    &mut symbol_words,
                "Sound" | "Probability" | "Time" | "Generator" | "Music" | "Chord"
                | "LFO" | "Audio Modulation" | "Debug" =>
                    &mut special_words,
                _ => &mut builtin_words,
            };
            bucket.push(word.name);
            for alias in word.aliases {
                if alias.chars().next().is_some_and(|c| c.is_alphanumeric()) {
                    bucket.push(alias);
                }
            }
        }

        let word_pattern = |words: &[&str]| -> std::string::String {
            let mut sorted = words.to_vec();
            sorted.sort_by_key(|w| std::cmp::Reverse(w.len()));
            format!(r"\b(?:{})\b", sorted.join("|"))
        };

        let mut rules = vec![
            SyntaxRule::new(Comment, r";;[^\n]*|\([^)]*\)"),
            SyntaxRule::new(Keyword, r"\b:\b|;\b"),
            SyntaxRule::new(Keyword, r"\b(?:if|else|then|case|of|endof|endcase|times)\b"),
            SyntaxRule::new(Keyword, r"\{|\}"),
            SyntaxRule::new(Special, r"\bgeom\.\.|\.\.|\.,"),
            SyntaxRule::new(Special, r"\b(?:tempo|speed)!"),
        ];

        if !special_words.is_empty() {
            rules.push(SyntaxRule::new(Special, &word_pattern(&special_words)));
        }
        if !symbol_words.is_empty() {
            rules.push(SyntaxRule::new(Symbol, &word_pattern(&symbol_words)));
        }
        if !builtin_words.is_empty() {
            rules.push(SyntaxRule::new(Builtin, &word_pattern(&builtin_words)));
        }

        rules.extend([
            SyntaxRule::new(Variable, r"[@!,](?:[GLF]\.)?[a-zA-Z_][a-zA-Z0-9_]*"),
            SyntaxRule::new(String, r#""[^"]*""#),
            SyntaxRule::new(String, r"\b[a-gA-G][s#b]?[0-9]\b"),
            SyntaxRule::new(Number, r"-?\.?\d+(?:\.\d+)?"),
            SyntaxRule::new(Special, r"\."),
            SyntaxRule::new(Operator, r"[+\-*/]|<>|<=|>=|[<>=]|!="),
            SyntaxRule::new(Operator, r"\?\B|\?\b|!\?\b"),
        ]);

        Some(LanguageSyntax { rules })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sova_core::vm::language::TokenCategory;

    fn categories_for(text: &str) -> Vec<(std::string::String, TokenCategory)> {
        let factory = CagireInterpreterFactory;
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
    fn syntax_highlights_all_categories() {
        use TokenCategory::*;
        let tokens = categories_for(
            ";; comment\n\"kick\" sound 0.8 gain 2000 lpf .\nc4 maj note 100 velocity\n{ 2 distort } sometimes .\nstep 4 mod 0 = if 60 else 72 then\n@counter 1 + ,counter"
        );
        let has = |cat: TokenCategory| tokens.iter().any(|(_, c)| *c == cat);
        assert!(has(Comment), "missing Comment");
        assert!(has(Keyword), "missing Keyword");
        assert!(has(Special), "missing Special");
        assert!(has(Symbol), "missing Symbol");
        assert!(has(Builtin), "missing Builtin");
        assert!(has(Variable), "missing Variable");
        assert!(has(String), "missing String");
        assert!(has(Number), "missing Number");
        assert!(has(Operator), "missing Operator");
    }

    #[test]
    fn syntax_word_buckets() {
        use TokenCategory::*;
        let tokens = categories_for("gain lpf chan step sound dup rand");
        let find = |w: &str| tokens.iter().find(|(t, _)| t == w).map(|(_, c)| *c);
        assert_eq!(find("gain"), Some(Symbol), "gain should be Symbol (param)");
        assert_eq!(find("lpf"), Some(Symbol), "lpf should be Symbol (param)");
        assert_eq!(find("chan"), Some(Symbol), "chan should be Symbol (MIDI)");
        assert_eq!(find("step"), Some(Symbol), "step should be Symbol (context)");
        assert_eq!(find("sound"), Some(Special), "sound should be Special");
        assert_eq!(find("dup"), Some(Builtin), "dup should be Builtin (stack)");
        assert_eq!(find("rand"), Some(Special), "rand should be Special (probability)");
    }

    #[test]
    fn syntax_emit_dot() {
        use TokenCategory::*;
        let tokens = categories_for("\"kick\" s . 0.5 gain");
        let dots: Vec<_> = tokens.iter().filter(|(t, _)| t == ".").collect();
        assert_eq!(dots.len(), 1);
        assert_eq!(dots[0].1, Special, "emit dot should be Special");
    }

    #[test]
    fn syntax_notes_are_strings() {
        use TokenCategory::*;
        let tokens = categories_for("c4 fs4 a3");
        for (tok, cat) in &tokens {
            assert_eq!(*cat, String, "note {tok} should be String (green)");
        }
    }
}

impl InterpreterFactory for CagireInterpreterFactory {
    fn make_instance(&self, script: &Script) -> Result<Box<dyn Interpreter>, String> {
        Ok(Box::new(CagireInterpreter::new(script.content())))
    }

    fn check(&self, script: &Script) -> CompilationState {
        let mut dict = std::collections::HashMap::new();
        match super::compiler::compile_script(script.content(), &mut dict) {
            Ok(_) => CompilationState::Parsed(None),
            Err(e) => CompilationState::Error(CompilationError { lang: "cagire".into(), info: e, from: 0, to: 0 }),
        }
    }
}
