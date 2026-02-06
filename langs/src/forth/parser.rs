pub fn tokenize(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            // Skip whitespace
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            // Comment: skip to end of line
            '\\' => {
                chars.next();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '\n' {
                        break;
                    }
                }
            }
            // Parenthesized comment
            '(' => {
                chars.next();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == ')' {
                        break;
                    }
                }
            }
            // Regular token
            _ => {
                let mut token = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() {
                        break;
                    }
                    token.push(c);
                    chars.next();
                }
                if !token.is_empty() {
                    tokens.push(token);
                }
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenize() {
        let tokens = tokenize("1 2 + .");
        assert_eq!(tokens, vec!["1", "2", "+", "."]);
    }

    #[test]
    fn test_comment() {
        let tokens = tokenize("1 2 \\ this is a comment\n+ .");
        assert_eq!(tokens, vec!["1", "2", "+", "."]);
    }

    #[test]
    fn test_paren_comment() {
        let tokens = tokenize("1 2 ( stack: n1 n2 -- sum ) + .");
        assert_eq!(tokens, vec!["1", "2", "+", "."]);
    }

    #[test]
    fn test_word_definition() {
        let tokens = tokenize(": double dup + ;");
        assert_eq!(tokens, vec![":", "double", "dup", "+", ";"]);
    }
}
