use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use super::ops::Op;
use super::words::compile_word;

pub(super) type Dictionary = HashMap<String, Vec<Op>>;

#[derive(Clone, Debug)]
enum Token {
    Int(i64),
    Float(f64),
    Str(String),
    Word(String),
}

pub(super) fn compile_script(input: &str, dict: &mut Dictionary) -> Result<Vec<Op>, String> {
    let tokens = tokenize(input);
    compile(&tokens, dict)
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some(&(_, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }

        // Curly braces silently ignored (legacy syntax)
        if c == '{' || c == '}' {
            chars.next();
            continue;
        }

        // String literals
        if c == '"' {
            chars.next();
            let mut s = String::new();
            while let Some(&(_, ch)) = chars.peek() {
                chars.next();
                if ch == '"' {
                    break;
                }
                s.push(ch);
            }
            tokens.push(Token::Str(s));
            continue;
        }

        // ;; line comments, or ; as a word
        if c == ';' {
            chars.next();
            if let Some(&(_, ';')) = chars.peek() {
                chars.next();
                while let Some(&(_, ch)) = chars.peek() {
                    if ch == '\n' {
                        break;
                    }
                    chars.next();
                }
                continue;
            }
            tokens.push(Token::Word(";".to_string()));
            continue;
        }

        // Collect word
        let mut word = String::new();
        while let Some(&(_, ch)) = chars.peek() {
            if ch.is_whitespace() {
                break;
            }
            word.push(ch);
            chars.next();
        }

        // Normalize shorthand float syntax: .25 -> 0.25, -.5 -> -0.5
        let word_to_parse: Cow<str> = if word.starts_with('.')
            && word.len() > 1
            && word.as_bytes()[1].is_ascii_digit()
        {
            Cow::Owned(format!("0{word}"))
        } else if word.starts_with("-.")
            && word.len() > 2
            && word.as_bytes()[2].is_ascii_digit()
        {
            Cow::Owned(format!("-0{}", &word[1..]))
        } else {
            Cow::Borrowed(&word)
        };

        if let Ok(i) = word_to_parse.parse::<i64>() {
            tokens.push(Token::Int(i));
        } else if let Ok(f) = word_to_parse.parse::<f64>() {
            tokens.push(Token::Float(f));
        } else {
            tokens.push(Token::Word(word));
        }
    }

    tokens
}

fn compile(tokens: &[Token], dict: &mut Dictionary) -> Result<Vec<Op>, String> {
    let mut ops = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Int(n) => ops.push(Op::PushInt(*n)),
            Token::Float(f) => ops.push(Op::PushFloat(*f)),
            Token::Str(s) => ops.push(Op::PushStr(Arc::from(s.as_str()))),
            Token::Word(w) => {
                let word = w.as_str();
                if word == "(" {
                    let (quote_ops, consumed) = compile_quotation(&tokens[i + 1..], dict)?;
                    i += consumed;
                    ops.push(Op::Quotation(Arc::from(quote_ops)));
                } else if word == ")" {
                    return Err("unexpected )".into());
                } else if word == "[" {
                    let (bracket_ops, consumed) = compile_bracket(&tokens[i + 1..], dict)?;
                    i += consumed;
                    ops.push(Op::Mark);
                    ops.extend(bracket_ops);
                    ops.push(Op::Count);
                } else if word == "]" {
                    return Err("unexpected ]".into());
                } else if word == ":" {
                    let (consumed, name, body) = compile_colon_def(&tokens[i + 1..], dict)?;
                    i += consumed;
                    dict.insert(name, body);
                } else if word == ";" {
                    return Err("unexpected ;".into());
                } else if word == "if" {
                    let (then_ops, else_ops, consumed) = compile_if(&tokens[i + 1..], dict)?;
                    i += consumed;
                    if else_ops.is_empty() {
                        ops.push(Op::BranchIfZero(then_ops.len()));
                        ops.extend(then_ops);
                    } else {
                        ops.push(Op::BranchIfZero(then_ops.len() + 1));
                        ops.extend(then_ops);
                        ops.push(Op::Branch(else_ops.len()));
                        ops.extend(else_ops);
                    }
                } else if word == "case" {
                    let (case_ops, consumed) = compile_case(&tokens[i + 1..], dict)?;
                    i += consumed;
                    ops.extend(case_ops);
                } else if word == "of" || word == "endof" || word == "endcase" {
                    return Err(format!("unexpected '{word}'"));
                } else if !compile_word(word, &mut ops, dict) {
                    return Err(format!("unknown word: {word}"));
                }
            }
        }
        i += 1;
    }

    Ok(ops)
}

fn compile_quotation(
    tokens: &[Token],
    dict: &mut Dictionary,
) -> Result<(Vec<Op>, usize), String> {
    let mut depth = 1;
    let mut end_idx = None;

    for (i, tok) in tokens.iter().enumerate() {
        if let Token::Word(w) = tok {
            match w.as_str() {
                "(" => depth += 1,
                ")" => {
                    depth -= 1;
                    if depth == 0 {
                        end_idx = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    let end_idx = end_idx.ok_or("missing )")?;
    let quote_ops = compile(&tokens[..end_idx], dict)?;
    Ok((quote_ops, end_idx + 1))
}

fn compile_bracket(
    tokens: &[Token],
    dict: &mut Dictionary,
) -> Result<(Vec<Op>, usize), String> {
    let mut depth = 1;
    let mut end_idx = None;

    for (i, tok) in tokens.iter().enumerate() {
        if let Token::Word(w) = tok {
            match w.as_str() {
                "[" => depth += 1,
                "]" => {
                    depth -= 1;
                    if depth == 0 {
                        end_idx = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    let end_idx = end_idx.ok_or("missing ]")?;
    let bracket_ops = compile(&tokens[..end_idx], dict)?;
    Ok((bracket_ops, end_idx + 1))
}

fn compile_colon_def(
    tokens: &[Token],
    dict: &mut Dictionary,
) -> Result<(usize, String, Vec<Op>), String> {
    if tokens.is_empty() {
        return Err("expected word name after ':'".into());
    }
    let name = match &tokens[0] {
        Token::Word(w) => w.clone(),
        Token::Int(n) => n.to_string(),
        Token::Float(f) => f.to_string(),
        Token::Str(s) => s.clone(),
    };
    let mut semi_pos = None;
    for (i, tok) in tokens[1..].iter().enumerate() {
        if let Token::Word(w) = tok && w == ";" {
            semi_pos = Some(i + 1);
            break;
        }
    }
    let semi_pos = semi_pos.ok_or("missing ';' in word definition")?;
    let body_tokens = &tokens[1..semi_pos];
    let body_ops = compile(body_tokens, dict)?;
    Ok((semi_pos + 1, name, body_ops))
}

fn compile_if(
    tokens: &[Token],
    dict: &mut Dictionary,
) -> Result<(Vec<Op>, Vec<Op>, usize), String> {
    let mut depth = 1;
    let mut else_pos = None;
    let mut then_pos = None;

    for (i, tok) in tokens.iter().enumerate() {
        if let Token::Word(w) = tok {
            match w.as_str() {
                "if" => depth += 1,
                "else" if depth == 1 => else_pos = Some(i),
                "then" => {
                    depth -= 1;
                    if depth == 0 {
                        then_pos = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    let then_pos = then_pos.ok_or("missing 'then'")?;

    let (then_ops, else_ops) = if let Some(ep) = else_pos {
        let then_ops = compile(&tokens[..ep], dict)?;
        let else_ops = compile(&tokens[ep + 1..then_pos], dict)?;
        (then_ops, else_ops)
    } else {
        let then_ops = compile(&tokens[..then_pos], dict)?;
        (then_ops, Vec::new())
    };

    Ok((then_ops, else_ops, then_pos + 1))
}

fn compile_case(tokens: &[Token], dict: &mut Dictionary) -> Result<(Vec<Op>, usize), String> {
    let mut depth = 1;
    let mut endcase_pos = None;
    let mut clauses: Vec<(usize, usize)> = Vec::new();
    let mut last_of = None;

    for (i, tok) in tokens.iter().enumerate() {
        if let Token::Word(w) = tok {
            match w.as_str() {
                "case" => depth += 1,
                "endcase" => {
                    depth -= 1;
                    if depth == 0 {
                        endcase_pos = Some(i);
                        break;
                    }
                }
                "of" if depth == 1 => last_of = Some(i),
                "endof" if depth == 1 => {
                    let of_pos = last_of.ok_or("'endof' without matching 'of'")?;
                    clauses.push((of_pos, i));
                    last_of = None;
                }
                _ => {}
            }
        }
    }

    let endcase_pos = endcase_pos.ok_or("missing 'endcase'")?;

    let mut ops = Vec::new();
    let mut branch_fixups: Vec<usize> = Vec::new();
    let mut clause_start = 0;

    for &(of_pos, endof_pos) in &clauses {
        let test_ops = compile(&tokens[clause_start..of_pos], dict)?;
        let body_ops = compile(&tokens[of_pos + 1..endof_pos], dict)?;

        ops.extend(test_ops);
        ops.push(Op::Over);
        ops.push(Op::Eq);
        ops.push(Op::BranchIfZero(body_ops.len() + 2));
        ops.push(Op::Drop);
        ops.extend(body_ops);
        branch_fixups.push(ops.len());
        ops.push(Op::Branch(0));

        clause_start = endof_pos + 1;
    }

    let default_tokens = &tokens[clause_start..endcase_pos];
    if !default_tokens.is_empty() {
        let default_ops = compile(default_tokens, dict)?;
        ops.extend(default_ops);
    }

    ops.push(Op::Drop);

    let end = ops.len();
    for pos in branch_fixups {
        ops[pos] = Op::Branch(end - pos - 1);
    }

    Ok((ops, endcase_pos + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let tokens = tokenize("3 4 +");
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn test_compile_basic() {
        let mut dict = Dictionary::new();
        let ops = compile_script("3 4 +", &mut dict).unwrap();
        assert_eq!(ops.len(), 3);
    }

    #[test]
    fn test_string_literal() {
        let mut dict = Dictionary::new();
        let ops = compile_script("\"kick\" sound", &mut dict).unwrap();
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn test_line_comment() {
        let mut dict = Dictionary::new();
        let ops = compile_script("3 ;; this is a comment\n4 +", &mut dict).unwrap();
        assert_eq!(ops.len(), 3);
    }

    #[test]
    fn test_shorthand_float() {
        let tokens = tokenize(".25 -.5");
        assert!(matches!(tokens[0], Token::Float(f) if (f - 0.25).abs() < f64::EPSILON));
        assert!(matches!(tokens[1], Token::Float(f) if (f + 0.5).abs() < f64::EPSILON));
    }

    #[test]
    fn test_curly_braces_ignored() {
        let tokens = tokenize("{hello world} 5");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[2], Token::Int(5)));
    }

    #[test]
    fn test_paren_quotation() {
        let mut dict = Dictionary::new();
        let ops = compile_script("( 2 3 + ) apply", &mut dict).unwrap();
        assert!(matches!(ops[0], Op::Quotation(_)));
    }

    #[test]
    fn test_sound_emit_pipeline() {
        let mut dict = Dictionary::new();
        let ops = compile_script("\"kick\" sound .", &mut dict).unwrap();
        assert_eq!(ops.len(), 3);
        assert!(matches!(ops[1], Op::NewCmd));
        assert!(matches!(ops[2], Op::Emit));
    }

    #[test]
    fn test_param_emit() {
        let mut dict = Dictionary::new();
        let ops = compile_script("60 note .", &mut dict).unwrap();
        assert_eq!(ops.len(), 3);
        assert!(matches!(ops[0], Op::PushInt(60)));
        assert!(matches!(ops[1], Op::SetParam("note")));
        assert!(matches!(ops[2], Op::Emit));
    }

    #[test]
    fn test_variables() {
        let mut dict = Dictionary::new();
        let ops = compile_script("42 !x @x", &mut dict).unwrap();
        assert_eq!(ops.len(), 5);
        assert!(matches!(ops[1], Op::PushStr(_)));
        assert!(matches!(ops[2], Op::Set));
        assert!(matches!(ops[3], Op::PushStr(_)));
        assert!(matches!(ops[4], Op::Get));
    }

    #[test]
    fn test_note_names() {
        let mut dict = Dictionary::new();
        let ops = compile_script("c4 a3", &mut dict).unwrap();
        assert_eq!(ops.len(), 2);
        assert!(matches!(ops[0], Op::PushInt(60)));
        assert!(matches!(ops[1], Op::PushInt(57)));
    }

    #[test]
    fn test_colon_definition() {
        let mut dict = Dictionary::new();
        let ops = compile_script(": double dup + ; 3 double", &mut dict).unwrap();
        assert!(dict.contains_key("double"));
        assert_eq!(ops.len(), 3);
        assert!(matches!(ops[0], Op::PushInt(3)));
        assert!(matches!(ops[1], Op::Dup));
        assert!(matches!(ops[2], Op::Add));
    }

    #[test]
    fn test_if_then() {
        let mut dict = Dictionary::new();
        let ops = compile_script("1 if 42 then", &mut dict).unwrap();
        assert!(!ops.is_empty());
    }

    #[test]
    fn test_if_else_then() {
        let mut dict = Dictionary::new();
        let ops = compile_script("0 if 10 else 20 then", &mut dict).unwrap();
        assert!(!ops.is_empty());
    }
}
