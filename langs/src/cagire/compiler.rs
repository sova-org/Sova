use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use super::ops::Op;
use super::pattern;
use super::types::{CagireError, Span};
use super::words::compile_word;

pub(crate) type Dictionary = HashMap<String, Vec<Op>>;

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    span: Span,
}

#[derive(Clone, Debug)]
enum TokenKind {
    Int(i64),
    Float(f64),
    Str(String),
    Word(String),
}

fn err(message: impl Into<String>, span: Span) -> CagireError {
    CagireError::new(message, span)
}

pub(super) fn compile_script(
    input: &str,
    dict: &mut Dictionary,
) -> Result<(Vec<Op>, Vec<Span>), CagireError> {
    let tokens = tokenize(input);
    compile(&tokens, dict)
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some(&(pos, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }

        if c == '{' || c == '}' {
            chars.next();
            continue;
        }

        // String literals
        if c == '"' {
            let start = pos;
            chars.next();
            let mut s = String::new();
            let mut end = start + 1;
            while let Some(&(p, ch)) = chars.peek() {
                chars.next();
                end = p + ch.len_utf8();
                if ch == '"' {
                    break;
                }
                s.push(ch);
            }
            tokens.push(Token { kind: TokenKind::Str(s), span: Span { start, end } });
            continue;
        }

        // ;; line comments, or ; as a word
        if c == ';' {
            let start = pos;
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
            tokens.push(Token {
                kind: TokenKind::Word(";".to_string()),
                span: Span { start, end: start + 1 },
            });
            continue;
        }

        // Collect word
        let start = pos;
        let mut word = String::new();
        let mut end = pos;
        while let Some(&(p, ch)) = chars.peek() {
            if ch.is_whitespace() {
                break;
            }
            word.push(ch);
            end = p + ch.len_utf8();
            chars.next();
        }
        let span = Span { start, end };

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
            tokens.push(Token { kind: TokenKind::Int(i), span });
        } else if let Ok(f) = word_to_parse.parse::<f64>() {
            tokens.push(Token { kind: TokenKind::Float(f), span });
        } else {
            tokens.push(Token { kind: TokenKind::Word(word), span });
        }
    }

    tokens
}

fn push(ops: &mut Vec<Op>, spans: &mut Vec<Span>, op: Op, span: Span) {
    ops.push(op);
    spans.push(span);
}

fn extend(ops: &mut Vec<Op>, spans: &mut Vec<Span>, other_ops: Vec<Op>, other_spans: Vec<Span>) {
    ops.extend(other_ops);
    spans.extend(other_spans);
}

fn compile(
    tokens: &[Token],
    dict: &mut Dictionary,
) -> Result<(Vec<Op>, Vec<Span>), CagireError> {
    let mut ops = Vec::new();
    let mut spans = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let tok = &tokens[i];
        let sp = tok.span;
        match &tok.kind {
            TokenKind::Int(n) => push(&mut ops, &mut spans, Op::PushInt(*n), sp),
            TokenKind::Float(f) => push(&mut ops, &mut spans, Op::PushFloat(*f), sp),
            TokenKind::Str(s) => push(&mut ops, &mut spans, Op::PushStr(Arc::from(s.as_str())), sp),
            TokenKind::Word(w) => {
                let word = w.as_str();
                if word == "(" {
                    let (quote_ops, quote_spans, consumed) =
                        compile_quotation(&tokens[i + 1..], sp, dict)?;
                    i += consumed;
                    let close_span = tokens.get(i).map_or(sp, |t| t.span);
                    let full_span = Span { start: sp.start, end: close_span.end };
                    push(&mut ops, &mut spans, Op::Quotation(Arc::from(quote_ops), Arc::from(quote_spans)), full_span);
                } else if word == ")" {
                    return Err(err("unexpected ')'", sp));
                } else if word == "[" {
                    let (bracket_ops, bracket_spans, consumed) =
                        compile_bracket(&tokens[i + 1..], sp, dict)?;
                    i += consumed;
                    push(&mut ops, &mut spans, Op::Mark, sp);
                    extend(&mut ops, &mut spans, bracket_ops, bracket_spans);
                    push(&mut ops, &mut spans, Op::Count(Some(sp)), sp);
                } else if word == "]" {
                    return Err(err("unexpected ']'", sp));
                } else if word == ":" {
                    let (consumed, name, body) = compile_colon_def(&tokens[i + 1..], sp, dict)?;
                    i += consumed;
                    dict.insert(name, body);
                } else if word == ";" {
                    return Err(err("unexpected ';'", sp));
                } else if word == "if" {
                    let (then_ops, then_spans, else_ops, else_spans, consumed) =
                        compile_if(&tokens[i + 1..], sp, dict)?;
                    i += consumed;
                    if else_ops.is_empty() {
                        push(&mut ops, &mut spans, Op::BranchIfZero(then_ops.len()), sp);
                        extend(&mut ops, &mut spans, then_ops, then_spans);
                    } else {
                        push(&mut ops, &mut spans, Op::BranchIfZero(then_ops.len() + 1), sp);
                        extend(&mut ops, &mut spans, then_ops, then_spans);
                        push(&mut ops, &mut spans, Op::Branch(else_ops.len()), sp);
                        extend(&mut ops, &mut spans, else_ops, else_spans);
                    }
                } else if word == "at" {
                    if let Some((body_ops, body_spans, consumed)) =
                        compile_at(&tokens[i + 1..], dict)?
                    {
                        i += consumed;
                        push(
                            &mut ops,
                            &mut spans,
                            Op::AtLoop(Arc::from(body_ops), Arc::from(body_spans)),
                            sp,
                        );
                    } else {
                        compile_word(word, sp, &mut ops, &mut spans, dict);
                    }
                } else if word == "pat" {
                    if let Some(Op::PushStr(s)) = ops.last() {
                        pattern::parse_pattern(s)
                            .map_err(|e| err(format!("pat: {e}"), sp))?;
                    }
                    push(&mut ops, &mut spans, Op::PatPush, sp);
                } else if word == "case" {
                    let (case_ops, case_spans, consumed) =
                        compile_case(&tokens[i + 1..], sp, dict)?;
                    i += consumed;
                    extend(&mut ops, &mut spans, case_ops, case_spans);
                } else if word == "of" || word == "endof" || word == "endcase" {
                    return Err(err(format!("unexpected '{word}'"), sp));
                } else {
                    compile_word(word, sp, &mut ops, &mut spans, dict);
                }
            }
        }
        i += 1;
    }

    debug_assert_eq!(ops.len(), spans.len());
    Ok((ops, spans))
}

fn compile_quotation(
    tokens: &[Token],
    open_span: Span,
    dict: &mut Dictionary,
) -> Result<(Vec<Op>, Vec<Span>, usize), CagireError> {
    let mut depth = 1;
    let mut end_idx = None;

    for (i, tok) in tokens.iter().enumerate() {
        if let TokenKind::Word(w) = &tok.kind {
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

    let end_idx = end_idx.ok_or_else(|| err("missing ')'", open_span))?;
    let (quote_ops, quote_spans) = compile(&tokens[..end_idx], dict)?;
    Ok((quote_ops, quote_spans, end_idx + 1))
}

fn compile_bracket(
    tokens: &[Token],
    open_span: Span,
    dict: &mut Dictionary,
) -> Result<(Vec<Op>, Vec<Span>, usize), CagireError> {
    let mut depth = 1;
    let mut end_idx = None;

    for (i, tok) in tokens.iter().enumerate() {
        if let TokenKind::Word(w) = &tok.kind {
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

    let end_idx = end_idx.ok_or_else(|| err("missing ']'", open_span))?;
    let (bracket_ops, bracket_spans) = compile(&tokens[..end_idx], dict)?;
    Ok((bracket_ops, bracket_spans, end_idx + 1))
}

fn compile_colon_def(
    tokens: &[Token],
    colon_span: Span,
    dict: &mut Dictionary,
) -> Result<(usize, String, Vec<Op>), CagireError> {
    if tokens.is_empty() {
        return Err(err("expected word name after ':'", colon_span));
    }
    let name = match &tokens[0].kind {
        TokenKind::Word(w) => w.clone(),
        TokenKind::Int(n) => n.to_string(),
        TokenKind::Float(f) => f.to_string(),
        TokenKind::Str(s) => s.clone(),
    };
    let mut semi_pos = None;
    for (i, tok) in tokens[1..].iter().enumerate() {
        if let TokenKind::Word(w) = &tok.kind {
            if w == ";" {
                semi_pos = Some(i + 1);
                break;
            }
        }
    }
    let semi_pos = semi_pos.ok_or_else(|| err("missing ';' in word definition", colon_span))?;
    let body_tokens = &tokens[1..semi_pos];
    let (body_ops, _body_spans) = compile(body_tokens, dict)?;
    Ok((semi_pos + 1, name, body_ops))
}

fn compile_if(
    tokens: &[Token],
    if_span: Span,
    dict: &mut Dictionary,
) -> Result<(Vec<Op>, Vec<Span>, Vec<Op>, Vec<Span>, usize), CagireError> {
    let mut depth = 1;
    let mut else_pos = None;
    let mut then_pos = None;

    for (i, tok) in tokens.iter().enumerate() {
        if let TokenKind::Word(w) = &tok.kind {
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

    let then_pos = then_pos.ok_or_else(|| err("missing 'then'", if_span))?;

    let (then_ops, then_spans, else_ops, else_spans) = if let Some(ep) = else_pos {
        let (to, ts) = compile(&tokens[..ep], dict)?;
        let (eo, es) = compile(&tokens[ep + 1..then_pos], dict)?;
        (to, ts, eo, es)
    } else {
        let (to, ts) = compile(&tokens[..then_pos], dict)?;
        (to, ts, Vec::new(), Vec::new())
    };

    Ok((then_ops, then_spans, else_ops, else_spans, then_pos + 1))
}

fn compile_case(
    tokens: &[Token],
    case_span: Span,
    dict: &mut Dictionary,
) -> Result<(Vec<Op>, Vec<Span>, usize), CagireError> {
    let mut depth = 1;
    let mut endcase_pos = None;
    let mut clauses: Vec<(usize, usize)> = Vec::new();
    let mut last_of = None;

    for (i, tok) in tokens.iter().enumerate() {
        if let TokenKind::Word(w) = &tok.kind {
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
                    let of_pos = last_of.ok_or_else(|| err("'endof' without matching 'of'", tok.span))?;
                    clauses.push((of_pos, i));
                    last_of = None;
                }
                _ => {}
            }
        }
    }

    let endcase_pos = endcase_pos.ok_or_else(|| err("missing 'endcase'", case_span))?;

    let mut ops = Vec::new();
    let mut op_spans = Vec::new();
    let mut branch_fixups: Vec<usize> = Vec::new();
    let mut clause_start = 0;

    for &(of_pos, endof_pos) in &clauses {
        let (test_ops, test_spans) = compile(&tokens[clause_start..of_pos], dict)?;
        let (body_ops, body_spans) = compile(&tokens[of_pos + 1..endof_pos], dict)?;
        let of_span = tokens[of_pos].span;

        extend(&mut ops, &mut op_spans, test_ops, test_spans);
        push(&mut ops, &mut op_spans, Op::Over, of_span);
        push(&mut ops, &mut op_spans, Op::Eq, of_span);
        push(&mut ops, &mut op_spans, Op::BranchIfZero(body_ops.len() + 2), of_span);
        push(&mut ops, &mut op_spans, Op::Drop, of_span);
        extend(&mut ops, &mut op_spans, body_ops, body_spans);
        branch_fixups.push(ops.len());
        push(&mut ops, &mut op_spans, Op::Branch(0), of_span);

        clause_start = endof_pos + 1;
    }

    let default_tokens = &tokens[clause_start..endcase_pos];
    if !default_tokens.is_empty() {
        let (default_ops, default_spans) = compile(default_tokens, dict)?;
        extend(&mut ops, &mut op_spans, default_ops, default_spans);
    }

    push(&mut ops, &mut op_spans, Op::Drop, case_span);

    let end = ops.len();
    for pos in branch_fixups {
        ops[pos] = Op::Branch(end - pos - 1);
    }

    Ok((ops, op_spans, endcase_pos + 1))
}

fn compile_at(
    tokens: &[Token],
    dict: &mut Dictionary,
) -> Result<Option<(Vec<Op>, Vec<Span>, usize)>, CagireError> {
    let mut depth = 1;
    let mut paren_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;

    enum AtCloser { Dot, Done }
    let mut found: Option<(usize, AtCloser)> = None;

    for (i, tok) in tokens.iter().enumerate() {
        if let TokenKind::Word(w) = &tok.kind {
            match w.as_str() {
                "(" => paren_depth += 1,
                ")" => paren_depth -= 1,
                "[" => bracket_depth += 1,
                "]" => bracket_depth -= 1,
                "at" if paren_depth == 0 && bracket_depth == 0 => depth += 1,
                "." if depth == 1 && paren_depth == 0 && bracket_depth == 0 => {
                    found = Some((i, AtCloser::Dot)); break;
                }
                "done" if depth == 1 && paren_depth == 0 && bracket_depth == 0 => {
                    found = Some((i, AtCloser::Done)); break;
                }
                "." | "done" if paren_depth == 0 && bracket_depth == 0 => depth -= 1,
                _ => {}
            }
        }
    }

    let Some((pos, closer)) = found else {
        return Ok(None);
    };
    let (mut body_ops, mut body_spans) = compile(&tokens[..pos], dict)?;
    if matches!(closer, AtCloser::Dot) {
        let dot_span = tokens[pos].span;
        push(&mut body_ops, &mut body_spans, Op::Emit, dot_span);
    }
    Ok(Some((body_ops, body_spans, pos + 1)))
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
    fn test_tokenize_spans() {
        let tokens = tokenize("3 4 +");
        assert_eq!(tokens[0].span, Span { start: 0, end: 1 });
        assert_eq!(tokens[1].span, Span { start: 2, end: 3 });
        assert_eq!(tokens[2].span, Span { start: 4, end: 5 });
    }

    #[test]
    fn test_tokenize_string_span() {
        let tokens = tokenize("\"kick\" sound");
        assert_eq!(tokens[0].span, Span { start: 0, end: 6 });
        assert_eq!(tokens[1].span, Span { start: 7, end: 12 });
    }

    #[test]
    fn test_compile_basic() {
        let mut dict = Dictionary::new();
        let (ops, spans) = compile_script("3 4 +", &mut dict).unwrap();
        assert_eq!(ops.len(), 3);
        assert_eq!(spans.len(), 3);
    }

    #[test]
    fn test_string_literal() {
        let mut dict = Dictionary::new();
        let (ops, _) = compile_script("\"kick\" sound", &mut dict).unwrap();
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn test_line_comment() {
        let mut dict = Dictionary::new();
        let (ops, _) = compile_script("3 ;; this is a comment\n4 +", &mut dict).unwrap();
        assert_eq!(ops.len(), 3);
    }

    #[test]
    fn test_shorthand_float() {
        let tokens = tokenize(".25 -.5");
        assert!(matches!(tokens[0].kind, TokenKind::Float(f) if (f - 0.25).abs() < f64::EPSILON));
        assert!(matches!(tokens[1].kind, TokenKind::Float(f) if (f + 0.5).abs() < f64::EPSILON));
    }

    #[test]
    fn test_curly_braces_ignored() {
        let tokens = tokenize("{hello world} 5");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[2].kind, TokenKind::Int(5)));
    }

    #[test]
    fn test_paren_quotation() {
        let mut dict = Dictionary::new();
        let (ops, _) = compile_script("( 2 3 + ) apply", &mut dict).unwrap();
        assert!(matches!(ops[0], Op::Quotation(..)));
    }

    #[test]
    fn test_sound_emit_pipeline() {
        let mut dict = Dictionary::new();
        let (ops, _) = compile_script("\"kick\" sound .", &mut dict).unwrap();
        assert_eq!(ops.len(), 3);
        assert!(matches!(ops[1], Op::NewCmd));
        assert!(matches!(ops[2], Op::Emit));
    }

    #[test]
    fn test_param_emit() {
        let mut dict = Dictionary::new();
        let (ops, _) = compile_script("60 note .", &mut dict).unwrap();
        assert_eq!(ops.len(), 3);
        assert!(matches!(ops[0], Op::PushInt(60)));
        assert!(matches!(ops[1], Op::SetParam("note")));
        assert!(matches!(ops[2], Op::Emit));
    }

    #[test]
    fn test_variables() {
        let mut dict = Dictionary::new();
        let (ops, _) = compile_script("42 !x @x", &mut dict).unwrap();
        assert_eq!(ops.len(), 5);
        assert!(matches!(ops[1], Op::PushStr(_)));
        assert!(matches!(ops[2], Op::Set));
        assert!(matches!(ops[3], Op::PushStr(_)));
        assert!(matches!(ops[4], Op::Get));
    }

    #[test]
    fn test_note_names() {
        let mut dict = Dictionary::new();
        let (ops, _) = compile_script("c4 a3", &mut dict).unwrap();
        assert_eq!(ops.len(), 2);
        assert!(matches!(ops[0], Op::PushInt(60)));
        assert!(matches!(ops[1], Op::PushInt(57)));
    }

    #[test]
    fn test_french_note_names() {
        let mut dict = Dictionary::new();
        let (ops, _) = compile_script("do4 mib4 sol#3 ré4 re4 si4 ti4 ut4 la3 fa5", &mut dict).unwrap();
        assert_eq!(ops.len(), 10);
        assert!(matches!(ops[0], Op::PushInt(60)));  // do4 = C4
        assert!(matches!(ops[1], Op::PushInt(63)));  // mib4 = Eb4
        assert!(matches!(ops[2], Op::PushInt(56)));  // sol#3 = G#3
        assert!(matches!(ops[3], Op::PushInt(62)));  // ré4 = D4
        assert!(matches!(ops[4], Op::PushInt(62)));  // re4 = D4
        assert!(matches!(ops[5], Op::PushInt(71)));  // si4 = B4
        assert!(matches!(ops[6], Op::PushInt(71)));  // ti4 = B4
        assert!(matches!(ops[7], Op::PushInt(60)));  // ut4 = C4
        assert!(matches!(ops[8], Op::PushInt(57)));  // la3 = A3
        assert!(matches!(ops[9], Op::PushInt(77)));  // fa5 = F5
    }

    #[test]
    fn test_colon_definition() {
        let mut dict = Dictionary::new();
        let (ops, _) = compile_script(": double dup + ; 3 double", &mut dict).unwrap();
        assert!(dict.contains_key("double"));
        assert_eq!(ops.len(), 3);
        assert!(matches!(ops[0], Op::PushInt(3)));
        assert!(matches!(ops[1], Op::Dup));
        assert!(matches!(ops[2], Op::Add));
    }

    #[test]
    fn test_if_then() {
        let mut dict = Dictionary::new();
        let (ops, _) = compile_script("1 if 42 then", &mut dict).unwrap();
        assert!(!ops.is_empty());
    }

    #[test]
    fn test_if_else_then() {
        let mut dict = Dictionary::new();
        let (ops, _) = compile_script("0 if 10 else 20 then", &mut dict).unwrap();
        assert!(!ops.is_empty());
    }

    #[test]
    fn test_error_has_span() {
        let mut dict = Dictionary::new();
        let e = compile_script("3 )", &mut dict).unwrap_err();
        assert_eq!(e.span, Span { start: 2, end: 3 });
        assert!(e.message.contains(")"));
    }

    #[test]
    fn test_missing_paren_points_at_opener() {
        let mut dict = Dictionary::new();
        let e = compile_script("( 2 3", &mut dict).unwrap_err();
        assert_eq!(e.span, Span { start: 0, end: 1 });
        assert!(e.message.contains(")"));
    }

    #[test]
    fn test_at_with_quoted_emit() {
        let mut dict = Dictionary::new();
        // ( . ) inside an at block must not be mistaken for the at closer
        let result = compile_script("0 0.5 at ( . ) 50 prob .", &mut dict);
        assert!(result.is_ok(), "quoted emit inside at block should compile: {result:?}");
    }
}
