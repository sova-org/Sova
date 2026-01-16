//! Preprocessor for Bob language - Automatic Semicolon Insertion (ASI)
//!
//! Inserts semicolons at newlines when an expression is complete.
//! This allows writing Bob code without explicit semicolons.

/// Token types for preprocessing
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Newline,
    Whitespace(String),
    Operator(String, usize), // (name, arity)
    Value(String),           // literals, vars
    BlockStart(String),      // IF, DO, FUNC, etc.
    End,                     // END keyword
    Else,                    // ELSE keyword
    Colon,
    OpenBracket,  // [
    CloseBracket, // ]
    QuoteBracket, // '[
    OpenParen,    // (
    CloseParen,   // )
    OpenBrace,    // {
    CloseBrace,   // }
    Semicolon,    // explicit ;
    Other(String),
}

/// Preprocess Bob source code, inserting semicolons at newlines
/// when expressions are complete.
pub fn preprocess(input: &str) -> String {
    let tokens = tokenize(input);
    insert_semicolons(tokens)
}

/// Tokenize input, preserving newlines
fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Handle comments (# to end of line)
        if c == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Handle strings - pass through as value
        if c == '"' {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < chars.len() {
                i += 1; // closing quote
            }
            tokens.push(Token::Value(chars[start..i].iter().collect()));
            continue;
        }

        // Newlines
        if c == '\n' {
            tokens.push(Token::Newline);
            i += 1;
            continue;
        }

        // Other whitespace
        if c.is_whitespace() {
            let start = i;
            while i < chars.len() && chars[i].is_whitespace() && chars[i] != '\n' {
                i += 1;
            }
            tokens.push(Token::Whitespace(chars[start..i].iter().collect()));
            continue;
        }

        // Semicolon
        if c == ';' {
            tokens.push(Token::Semicolon);
            i += 1;
            continue;
        }

        // Colon (standalone, not part of symbol)
        if c == ':'
            && (i + 1 >= chars.len()
                || !chars[i + 1].is_alphabetic()
                || (i > 0 && !chars[i - 1].is_whitespace() && chars[i - 1] != '['))
        {
            // Check if it's a symbol like :note or map key syntax [key:]
            // If next char is alphabetic and we're at start or after whitespace/[, it's a symbol
            if i + 1 < chars.len() && chars[i + 1].is_alphabetic() {
                // It's a symbol :name - read as value
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '#') {
                    i += 1;
                }
                tokens.push(Token::Value(chars[start..i].iter().collect()));
                continue;
            }
            tokens.push(Token::Colon);
            i += 1;
            continue;
        }

        // Symbol :name (when preceded by whitespace or at start)
        if c == ':' && i + 1 < chars.len() && chars[i + 1].is_alphabetic() {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '#') {
                i += 1;
            }
            tokens.push(Token::Value(chars[start..i].iter().collect()));
            continue;
        }

        // Brackets
        if c == '[' {
            tokens.push(Token::OpenBracket);
            i += 1;
            continue;
        }
        if c == ']' {
            tokens.push(Token::CloseBracket);
            i += 1;
            continue;
        }

        // Quote-bracket '[
        if c == '\'' && i + 1 < chars.len() && chars[i + 1] == '[' {
            tokens.push(Token::QuoteBracket);
            i += 2;
            continue;
        }

        // Parens
        if c == '(' {
            tokens.push(Token::OpenParen);
            i += 1;
            continue;
        }
        if c == ')' {
            tokens.push(Token::CloseParen);
            i += 1;
            continue;
        }

        // Braces
        if c == '{' {
            tokens.push(Token::OpenBrace);
            i += 1;
            continue;
        }
        if c == '}' {
            tokens.push(Token::CloseBrace);
            i += 1;
            continue;
        }

        // Multi-char operators
        if i + 1 < chars.len() {
            let two: String = chars[i..i + 2].iter().collect();
            if let Some(arity) = get_symbol_arity(&two) {
                tokens.push(Token::Operator(two, arity));
                i += 2;
                continue;
            }
        }

        // Single-char operators
        if let Some(arity) = get_symbol_arity(&c.to_string()) {
            tokens.push(Token::Operator(c.to_string(), arity));
            i += 1;
            continue;
        }

        // Numbers
        if c.is_ascii_digit() || (c == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
        {
            let start = i;
            if c == '-' {
                i += 1;
            }
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            tokens.push(Token::Value(chars[start..i].iter().collect()));
            continue;
        }

        // Identifiers
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            // Check for G., F., L. prefix
            if i < chars.len() && chars[i] == '.' && (i - start == 1) {
                let prefix = chars[start];
                if prefix == 'G' || prefix == 'F' || prefix == 'L' {
                    i += 1; // consume the dot
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    tokens.push(Token::Value(chars[start..i].iter().collect()));
                    continue;
                }
            }
            let word: String = chars[start..i].iter().collect();
            if word == "END" {
                tokens.push(Token::End);
            } else if word == "ELSE" {
                tokens.push(Token::Else);
            } else if is_block_keyword(&word) {
                tokens.push(Token::BlockStart(word));
            } else if let Some(arity) = get_word_arity(&word) {
                tokens.push(Token::Operator(word, arity));
            } else {
                // Single uppercase letter or lowercase identifier = value
                tokens.push(Token::Value(word));
            }
            continue;
        }

        // Unknown - pass through
        tokens.push(Token::Other(c.to_string()));
        i += 1;
    }

    tokens
}

/// Insert semicolons based on token stream
fn insert_semicolons(tokens: Vec<Token>) -> String {
    let mut output = String::new();
    let mut depth: i32 = 0; // Expression depth
    let mut bracket_depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    let mut has_content = false; // Have we seen content since last ; or start?
    let mut after_colon = false; // Just saw : (block body start)
    let mut after_open_brace = false; // Just saw { (block body start)

    let _len = tokens.len();
    for (idx, token) in tokens.iter().enumerate() {
        match token {
            Token::Newline => {
                // Peek at next non-whitespace token
                let next_significant = tokens[idx + 1..]
                    .iter()
                    .find(|t| !matches!(t, Token::Whitespace(_) | Token::Newline));

                // Should we insert ;?
                // Don't insert before END, ELSE, CloseBrace, or at EOF
                let should_insert = bracket_depth == 0
                    && paren_depth == 0
                    && brace_depth == 0
                    && depth == 0
                    && has_content
                    && !after_colon
                    && !after_open_brace
                    && !matches!(
                        next_significant,
                        Some(Token::End) | Some(Token::Else) | Some(Token::CloseBrace) | None
                    );

                if should_insert {
                    output.push(';');
                    has_content = false;
                }
                after_colon = false;
                after_open_brace = false;
                output.push(' ');
            }

            Token::Whitespace(ws) => {
                output.push_str(ws);
            }

            Token::Semicolon => {
                output.push(';');
                has_content = false;
                after_colon = false;
            }

            Token::Colon => {
                after_colon = true;
                after_open_brace = false;
                output.push(':');
                has_content = true;
            }

            Token::OpenBracket => {
                bracket_depth += 1;
                output.push('[');
                has_content = true;
                after_colon = false;
                after_open_brace = false;
            }

            Token::CloseBracket => {
                bracket_depth -= 1;
                if depth > 0 {
                    depth -= 1;
                }
                output.push(']');
                has_content = true;
                after_colon = false;
                after_open_brace = false;
            }

            Token::QuoteBracket => {
                bracket_depth += 1;
                output.push_str("'[");
                has_content = true;
                after_colon = false;
                after_open_brace = false;
            }

            Token::OpenParen => {
                paren_depth += 1;
                output.push('(');
                has_content = true;
                after_colon = false;
                after_open_brace = false;
            }

            Token::CloseParen => {
                paren_depth -= 1;
                if depth > 0 {
                    depth -= 1;
                }
                output.push(')');
                has_content = true;
                after_colon = false;
                after_open_brace = false;
            }

            Token::OpenBrace => {
                brace_depth += 1;
                after_open_brace = true;
                output.push('{');
                has_content = true;
                after_colon = false;
            }

            Token::CloseBrace => {
                brace_depth -= 1;
                if depth > 0 {
                    depth -= 1;
                }
                output.push('}');
                has_content = true;
                after_colon = false;
                after_open_brace = false;
            }

            Token::BlockStart(name) => {
                // Block constructs are treated as single values when complete
                output.push_str(name);
                has_content = true;
                after_colon = false;
                after_open_brace = false;
                // Don't change depth - block consumes args until END
            }

            Token::End => {
                // END closes a block - the block becomes one value
                if depth > 0 {
                    depth -= 1;
                }
                output.push_str("END");
                has_content = true;
                after_colon = false;
                after_open_brace = false;
            }

            Token::Else => {
                // ELSE is part of IF construct
                output.push_str("ELSE");
                has_content = true;
                after_colon = false;
                after_open_brace = false;
            }

            Token::Operator(name, arity) => {
                if depth > 0 {
                    depth -= 1;
                }
                depth += *arity as i32;
                output.push_str(name);
                has_content = true;
                after_colon = false;
                after_open_brace = false;
            }

            Token::Value(val) => {
                if depth > 0 {
                    depth -= 1;
                }
                output.push_str(val);
                has_content = true;
                after_colon = false;
                after_open_brace = false;
            }

            Token::Other(s) => {
                output.push_str(s);
                if !s.trim().is_empty() {
                    has_content = true;
                    after_colon = false;
                    after_open_brace = false;
                }
            }
        }
    }

    output
}

fn is_block_keyword(word: &str) -> bool {
    matches!(
        word,
        "IF" | "WHILE"
            | "DO"
            | "EACH"
            | "EVERY"
            | "L"
            | "PROB"
            | "SWITCH"
            | "FUNC"
            | "FN"
            | "CHOOSE"
            | "ALT"
            | "FORK"
            | "BYTES"
    )
}

fn get_word_arity(word: &str) -> Option<usize> {
    match word {
        // Nullary
        "TOSS" | "MNEW" | "BREAK" => Some(0),
        // Unary
        "NEG" | "NOT" | "BNOT" | "ABS" | "RAND" | "LEN" | "PICK" | "CYCLE" | "WAIT" | "DEV" => {
            Some(1)
        }
        // Binary
        "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "GT" | "LT" | "GTE" | "LTE" | "EQ" | "NE"
        | "AND" | "OR" | "XOR" | "BAND" | "BOR" | "BXOR" | "SHL" | "SHR" | "MIN" | "MAX" | "QT"
        | "RRAND" | "DRUNK" | "GET" | "MGET" | "MHAS" | "MAP" | "FILTER" | "MMERGE" => Some(2),
        // Unary map/list operations
        "MLEN" => Some(1),
        // Ternary
        "CLAMP" | "WRAP" | "MSET" | "REDUCE" => Some(3),
        // Quinary
        "SCALE" => Some(5),
        // Play/Emit
        "PLAY" => Some(1),
        // SET
        "SET" => Some(2),
        // CALL, CASE, DEFAULT, ELSE are not operators
        _ => None,
    }
}

fn get_symbol_arity(sym: &str) -> Option<usize> {
    match sym {
        // Unary
        "!" | "~" => Some(1),
        // Binary
        "+" | "-" | "*" | "/" | "%" | ">" | "<" | ">=" | "<=" | "==" | "!=" | "&&" | "||" | "&"
        | "|" | "^" | "<<" => Some(2),
        // Emit
        ">>" | "@" => Some(1),
        // Ternary
        "?" => Some(3),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_semicolon_at_start() {
        let input = "SET G.X 1";
        let output = preprocess(input);
        assert!(
            !output.starts_with(';'),
            "Should not start with ;: {}",
            output
        );
    }

    #[test]
    fn test_no_semicolon_before_end() {
        let input = "DO 4:\n1\nEND";
        let output = preprocess(input);
        assert!(
            !output.contains("; END"),
            "Should not have ; before END: {}",
            output
        );
    }

    #[test]
    fn test_basic_asi() {
        let input = "=> [note: 60]\nWAIT 0.5";
        let output = preprocess(input);
        assert!(
            output.contains(';'),
            "Should insert semicolon between expressions: {}",
            output
        );
    }

    #[test]
    fn test_no_asi_inside_brackets() {
        let input = "[note: 60\nvel: 100]";
        let output = preprocess(input);
        let semicolons: usize = output.chars().filter(|&c| c == ';').count();
        assert_eq!(
            semicolons, 0,
            "Should not insert ; inside brackets: {}",
            output
        );
    }

    #[test]
    fn test_no_asi_after_colon() {
        let input = "DO 4:\n=> [note: 60]\nEND";
        let output = preprocess(input);
        assert!(
            !output.contains(": ;"),
            "Should not insert ; after colon: {}",
            output
        );
    }

    #[test]
    fn test_func_definition() {
        let input = "FUNC DOUBLE A:\n+ A A\nEND";
        let output = preprocess(input);
        // Should not have ; before END
        assert!(
            !output.contains("; END"),
            "Should not have ; before END in FUNC: {}",
            output
        );
    }

    #[test]
    fn test_multiline_expression() {
        let input = "ADD 1\n2";
        let output = preprocess(input);
        // ADD needs 2 args, so newline after 1 should NOT insert ;
        // The expression is complete only after 2
        let semicolons: usize = output.chars().filter(|&c| c == ';').count();
        assert!(
            semicolons <= 1,
            "Should not insert ; mid-expression: {}",
            output
        );
    }

    #[test]
    fn test_explicit_semicolons_preserved() {
        let input = "SET G.X 1; SET G.Y 2";
        let output = preprocess(input);
        assert!(
            output.contains(';'),
            "Should preserve explicit semicolons: {}",
            output
        );
    }
}
