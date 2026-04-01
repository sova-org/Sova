use eframe::egui;
use egui::text::LayoutJob;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SpanStyle {
    strong: bool,
    emphasis: bool,
    strike: bool,
    code: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Span {
    text: String,
    style: SpanStyle,
}

pub fn append_inline_markdown(
    job: &mut LayoutJob,
    text: &str,
    base_format: &egui::TextFormat,
    strong_color: egui::Color32,
    code_bg: egui::Color32,
) {
    for span in parse_inline_markdown(text) {
        let mut format = base_format.clone();
        if span.style.code {
            format.font_id = egui::FontId::monospace(base_format.font_id.size);
            format.background = code_bg;
        } else {
            if span.style.strong {
                format.color = strong_color;
            }
            if span.style.emphasis {
                format.italics = true;
            }
            if span.style.strike {
                format.strikethrough = egui::Stroke::new(1.0, format.color);
            }
        }
        job.append(&span.text, 0.0, format);
    }
}

fn parse_inline_markdown(text: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut style = SpanStyle::default();
    let mut buffer = String::new();
    let mut i = 0;

    while i < text.len() {
        if !style.code
            && let Some((escaped, consumed)) = parse_escape(text, i)
        {
            buffer.push(escaped);
            i += consumed;
            continue;
        }

        if text[i..].starts_with('`') {
            if style.code {
                flush_span(&mut spans, &mut buffer, style);
                style.code = false;
                i += 1;
                continue;
            }
            if has_closing_backtick(text, i + 1) {
                flush_span(&mut spans, &mut buffer, style);
                style.code = true;
                i += 1;
                continue;
            }
        }

        if !style.code && text[i..].starts_with("~~") {
            if style.strike {
                if can_close_generic(text, i, 2) {
                    flush_span(&mut spans, &mut buffer, style);
                    style.strike = false;
                    i += 2;
                    continue;
                }
            } else if can_open_generic(text, i, 2) && has_closing_generic(text, i + 2, "~~", 2) {
                flush_span(&mut spans, &mut buffer, style);
                style.strike = true;
                i += 2;
                continue;
            }
        }

        if !style.code && text[i..].starts_with("**") {
            if style.strong {
                if can_close_emphasis(text, i, 2, '*') {
                    flush_span(&mut spans, &mut buffer, style);
                    style.strong = false;
                    i += 2;
                    continue;
                }
            } else if can_open_emphasis(text, i, 2, '*')
                && has_closing_emphasis(text, i + 2, "**", 2, '*')
            {
                flush_span(&mut spans, &mut buffer, style);
                style.strong = true;
                i += 2;
                continue;
            }
        }

        if !style.code && text[i..].starts_with("__") {
            if style.strong {
                if can_close_emphasis(text, i, 2, '_') {
                    flush_span(&mut spans, &mut buffer, style);
                    style.strong = false;
                    i += 2;
                    continue;
                }
            } else if can_open_emphasis(text, i, 2, '_')
                && has_closing_emphasis(text, i + 2, "__", 2, '_')
            {
                flush_span(&mut spans, &mut buffer, style);
                style.strong = true;
                i += 2;
                continue;
            }
        }

        if !style.code && text[i..].starts_with('*') {
            if style.emphasis {
                if can_close_emphasis(text, i, 1, '*') {
                    flush_span(&mut spans, &mut buffer, style);
                    style.emphasis = false;
                    i += 1;
                    continue;
                }
            } else if can_open_emphasis(text, i, 1, '*')
                && has_closing_emphasis(text, i + 1, "*", 1, '*')
            {
                flush_span(&mut spans, &mut buffer, style);
                style.emphasis = true;
                i += 1;
                continue;
            }
        }

        if !style.code && text[i..].starts_with('_') {
            if style.emphasis {
                if can_close_emphasis(text, i, 1, '_') {
                    flush_span(&mut spans, &mut buffer, style);
                    style.emphasis = false;
                    i += 1;
                    continue;
                }
            } else if can_open_emphasis(text, i, 1, '_')
                && has_closing_emphasis(text, i + 1, "_", 1, '_')
            {
                flush_span(&mut spans, &mut buffer, style);
                style.emphasis = true;
                i += 1;
                continue;
            }
        }

        let ch = text[i..].chars().next().unwrap();
        buffer.push(ch);
        i += ch.len_utf8();
    }

    flush_span(&mut spans, &mut buffer, style);
    spans
}

fn flush_span(spans: &mut Vec<Span>, buffer: &mut String, style: SpanStyle) {
    if buffer.is_empty() {
        return;
    }
    spans.push(Span {
        text: std::mem::take(buffer),
        style,
    });
}

fn parse_escape(text: &str, index: usize) -> Option<(char, usize)> {
    if !text[index..].starts_with('\\') {
        return None;
    }
    let (escaped, len) = text[index + 1..]
        .chars()
        .next()
        .map(|ch| (ch, ch.len_utf8()))?;
    if matches!(escaped, '\\' | '*' | '_' | '`' | '~') {
        return Some((escaped, 1 + len));
    }
    None
}

fn has_closing_backtick(text: &str, from: usize) -> bool {
    find_unescaped_token(text, from, "`", |_| true).is_some()
}

fn has_closing_generic(text: &str, from: usize, token: &str, len: usize) -> bool {
    find_unescaped_token(text, from, token, |index| {
        can_close_generic(text, index, len)
    })
    .is_some()
}

fn has_closing_emphasis(text: &str, from: usize, token: &str, len: usize, delim: char) -> bool {
    find_unescaped_token(text, from, token, |index| {
        can_close_emphasis(text, index, len, delim)
    })
    .is_some()
}

fn find_unescaped_token(
    text: &str,
    from: usize,
    token: &str,
    predicate: impl Fn(usize) -> bool,
) -> Option<usize> {
    let mut index = from;
    while index < text.len() {
        if text[index..].starts_with(token) && !is_escaped(text, index) && predicate(index) {
            return Some(index);
        }
        let ch = text[index..].chars().next().unwrap();
        index += ch.len_utf8();
    }
    None
}

fn is_escaped(text: &str, index: usize) -> bool {
    let mut slash_count = 0;
    for b in text.as_bytes()[..index].iter().rev() {
        if *b == b'\\' {
            slash_count += 1;
        } else {
            break;
        }
    }
    slash_count % 2 == 1
}

fn can_open_generic(text: &str, index: usize, len: usize) -> bool {
    next_char(text, index + len).is_some_and(|ch| !ch.is_whitespace())
}

fn can_close_generic(text: &str, index: usize, _len: usize) -> bool {
    prev_char(text, index).is_some_and(|ch| !ch.is_whitespace())
}

fn can_open_emphasis(text: &str, index: usize, len: usize, delim: char) -> bool {
    let next = match next_char(text, index + len) {
        Some(ch) if !ch.is_whitespace() => ch,
        _ => return false,
    };
    if delim == '_' && prev_char(text, index).is_some_and(|prev| prev.is_alphanumeric()) {
        return !next.is_alphanumeric();
    }
    true
}

fn can_close_emphasis(text: &str, index: usize, len: usize, delim: char) -> bool {
    let prev = match prev_char(text, index) {
        Some(ch) if !ch.is_whitespace() => ch,
        _ => return false,
    };
    if delim == '_' && next_char(text, index + len).is_some_and(|next| next.is_alphanumeric()) {
        return !prev.is_alphanumeric();
    }
    true
}

fn prev_char(text: &str, index: usize) -> Option<char> {
    text[..index].chars().next_back()
}

fn next_char(text: &str, index: usize) -> Option<char> {
    text[index..].chars().next()
}

#[cfg(test)]
mod tests {
    use super::{Span, SpanStyle, parse_inline_markdown};

    #[test]
    fn parses_basic_styles() {
        assert_eq!(
            parse_inline_markdown("plain **bold** *italics* ~~gone~~"),
            vec![
                Span {
                    text: "plain ".into(),
                    style: SpanStyle::default(),
                },
                Span {
                    text: "bold".into(),
                    style: SpanStyle {
                        strong: true,
                        ..Default::default()
                    },
                },
                Span {
                    text: " ".into(),
                    style: SpanStyle::default(),
                },
                Span {
                    text: "italics".into(),
                    style: SpanStyle {
                        emphasis: true,
                        ..Default::default()
                    },
                },
                Span {
                    text: " ".into(),
                    style: SpanStyle::default(),
                },
                Span {
                    text: "gone".into(),
                    style: SpanStyle {
                        strike: true,
                        ..Default::default()
                    },
                },
            ]
        );
    }

    #[test]
    fn keeps_underscores_inside_words_literal() {
        assert_eq!(
            parse_inline_markdown("snake_case and __bold__"),
            vec![
                Span {
                    text: "snake_case and ".into(),
                    style: SpanStyle::default(),
                },
                Span {
                    text: "bold".into(),
                    style: SpanStyle {
                        strong: true,
                        ..Default::default()
                    },
                },
            ]
        );
    }

    #[test]
    fn supports_nested_and_escaped_markup() {
        assert_eq!(
            parse_inline_markdown("***both*** and \\*literal\\*"),
            vec![
                Span {
                    text: "both".into(),
                    style: SpanStyle {
                        strong: true,
                        emphasis: true,
                        ..Default::default()
                    },
                },
                Span {
                    text: " and *literal*".into(),
                    style: SpanStyle::default(),
                },
            ]
        );
    }

    #[test]
    fn treats_code_as_literal() {
        assert_eq!(
            parse_inline_markdown("before `**code**` after"),
            vec![
                Span {
                    text: "before ".into(),
                    style: SpanStyle::default(),
                },
                Span {
                    text: "**code**".into(),
                    style: SpanStyle {
                        code: true,
                        ..Default::default()
                    },
                },
                Span {
                    text: " after".into(),
                    style: SpanStyle::default(),
                },
            ]
        );
    }
}
