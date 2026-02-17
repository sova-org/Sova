use sova_core::vm::language::{LanguageSyntax, SyntaxRule, TokenCategory};

pub fn syntax() -> LanguageSyntax {
    use TokenCategory::*;
    let rule = |cat, pat: &str| SyntaxRule { category: cat, pattern: pat.to_owned() };
    LanguageSyntax {
        rules: vec![
            // Comments first — must beat operator tokens like SUB
            rule(Comment, r"--[^\n]*"),
            // String literals
            rule(String, r#""[^"]*""#),
            // Symbols (:name, including note names like :c3, :fs4)
            rule(Symbol, r":[a-zA-Z_][a-zA-Z0-9_]*"),
            // Scoped variables: G.x, F.count, L.phase
            rule(Variable, r"[GLF]\.\w+"),
            // Numeric literals
            rule(Number, r"\b\d+(\.\d+)?\b"),
            // Keywords
            rule(Keyword, r"\b(IF|ELSE|END|RANGE|DO|WHILE|EACH|EVERY|SWITCH|CASE|DEFAULT|PROB|EU|BIN|FORK|FUNC|FN|CALL|BREAK|CHOOSE|ALT|BYTES)\b"),
            // Builtins
            rule(Builtin, r"\b(PLAY|WAIT|DEV|PRINT|P|SET|MNEW|MGET|MSET|MHAS|MMERGE|MLEN|LEN|GET|PICK|CYCLE|MAP|FILTER|REDUCE)\b|>>|@"),
            // Operators (word-form + symbolic)
            rule(Operator, r"\b(ADD|SUB|MUL|DIV|MOD|NEG|ABS|GT|LT|GTE|LTE|EQ|NE|AND|OR|XOR|NOT|BAND|BOR|BXOR|BNOT|SHL|SHR|MIN|MAX|CLAMP|WRAP|SCALE|QT|TOSS|RAND|RRAND|DRUNK)\b"),
            // Special read-only variables
            rule(Special, r"\b[IETR]\b"),
            // Punctuation
            rule(Punctuation, r"\?|'\[|\[|\]|\{|\}|:"),
        ],
    }
}
