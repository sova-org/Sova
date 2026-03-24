use sova_core::vm::language::{LanguageSyntax, SyntaxRule, TokenCategory};

pub fn syntax() -> LanguageSyntax {
    use TokenCategory::*;
    LanguageSyntax {
        rules: vec![
            SyntaxRule::new(Comment, r"--[^\n]*"),
            SyntaxRule::new(String, r#""[^"]*""#),
            SyntaxRule::new(Symbol, r":[a-zA-Z_][a-zA-Z0-9_]*"),
            SyntaxRule::new(Variable, r"[GLF]\.\w+"),
            SyntaxRule::new(Number, r"\b\d+(\.\d+)?\b"),
            SyntaxRule::new(Keyword, r"\b(IF|ELSE|END|RANGE|DO|WHILE|EACH|EVERY|SWITCH|CASE|DEFAULT|PROB|EU|BIN|FORK|FUNC|FN|CALL|BREAK|CHOOSE|ALT|BYTES)\b"),
            SyntaxRule::new(Builtin, r"\b(PLAY|WAIT|DEV|PRINT|P|SET|MNEW|MGET|MSET|MHAS|MMERGE|MLEN|LEN|GET|PICK|CYCLE|MAP|FILTER|REDUCE)\b|>>|@"),
            SyntaxRule::new(Operator, r"\b(ADD|SUB|MUL|DIV|MOD|NEG|ABS|GT|LT|GTE|LTE|EQ|NE|AND|OR|XOR|NOT|BAND|BOR|BXOR|BNOT|SHL|SHR|MIN|MAX|CLAMP|WRAP|SCALE|QT|TOSS|RAND|RRAND|DRUNK)\b"),
            SyntaxRule::new(Special, r"\b[IETR]\b"),
            SyntaxRule::new(Punctuation, r"\?|'\[|\[|\]|:"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sova_core::vm::language::TokenCategory;

    fn categories_for(text: &str) -> Vec<(String, TokenCategory)> {
        let syn = syntax();
        let mut parts = Vec::new();
        let mut cats = Vec::new();
        let mut names = Vec::new();
        for (i, rule) in syn.rules.iter().enumerate() {
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
            "-- play a note\n60 90 1 PLAY\nIF G.x GT 10 END\n:kick \"hello\" 0.5 WAIT\nRAND I '[1 2 3]"
        );
        let has = |cat: TokenCategory| tokens.iter().any(|(_, c)| *c == cat);
        assert!(has(Comment), "missing Comment");
        assert!(has(Number), "missing Number");
        assert!(has(Builtin), "missing Builtin");
        assert!(has(Keyword), "missing Keyword");
        assert!(has(Operator), "missing Operator");
        assert!(has(Variable), "missing Variable");
        assert!(has(Symbol), "missing Symbol");
        assert!(has(String), "missing String");
        assert!(has(Special), "missing Special");
        assert!(has(Punctuation), "missing Punctuation");
    }
}
