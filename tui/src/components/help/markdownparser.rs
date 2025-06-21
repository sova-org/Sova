use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

pub fn parse_markdown<'a>(markdown_input: &'a str) -> Text<'a> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown_input, options);

    let mut lines: Vec<Line<'a>> = Vec::new();
    let mut current_spans: Vec<Span<'a>> = Vec::new();
    let base_style = Style::default().fg(Color::White);
    let mut style_stack: Vec<Style> = vec![base_style];
    let mut list_level: usize = 0;
    let mut current_heading_level: Option<HeadingLevel> = None;

    for event in parser {
        match event {
            Event::Start(tag) => {
                let current_style = *style_stack.last().unwrap_or(&base_style);

                if !matches!(
                    tag,
                    Tag::Item
                        | Tag::Emphasis
                        | Tag::Strong
                        | Tag::Strikethrough
                        | Tag::Link { .. }
                        | Tag::Image { .. }
                        | Tag::CodeBlock(_)
                ) && !current_spans.is_empty()
                {
                    lines.push(Line::from(std::mem::take(&mut current_spans)));
                }

                match tag {
                    Tag::Paragraph => {
                        style_stack.push(current_style);
                    }
                    Tag::Heading { level, .. } => {
                        current_heading_level = Some(level);
                        if !lines.is_empty() && lines.last().is_some_and(|l| !l.spans.is_empty()) {
                            lines.push(Line::raw(""));
                        }
                        let heading_text_style = match level {
                            HeadingLevel::H1 => Style::default()
                                .fg(Color::Black)
                                .add_modifier(Modifier::BOLD),
                            _ => Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        };
                        style_stack.push(heading_text_style);
                    }
                    Tag::List(_) => {
                        list_level += 1;
                        style_stack.push(current_style);
                    }
                    Tag::Item => {
                        if !current_spans.is_empty() {
                            lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                        let indent = "  ".repeat(list_level.saturating_sub(1));
                        let item_marker_style = *style_stack.last().unwrap_or(&base_style);
                        current_spans
                            .push(Span::styled(format!("{}* ", indent), item_marker_style));
                        style_stack.push(item_marker_style);
                    }
                    Tag::Emphasis => {
                        style_stack.push(current_style.add_modifier(Modifier::ITALIC));
                    }
                    Tag::Strong => {
                        style_stack.push(current_style.add_modifier(Modifier::BOLD));
                    }
                    Tag::Strikethrough => {
                        style_stack.push(current_style.add_modifier(Modifier::CROSSED_OUT));
                    }
                    Tag::CodeBlock(_) => {
                        style_stack.push(Style::default().fg(Color::Cyan));
                        if !lines.is_empty() && lines.last().is_some_and(|l| !l.spans.is_empty()) {
                            lines.push(Line::raw(""));
                        }
                    }
                    _ => {
                        style_stack.push(current_style);
                    }
                }
            }
            Event::End(tag_end) => {
                if !matches!(tag_end, TagEnd::Item | TagEnd::Heading(_)) && style_stack.len() > 1 {
                    style_stack.pop();
                }

                match tag_end {
                    TagEnd::Paragraph => {
                        if !current_spans.is_empty() {
                            lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                    }
                    TagEnd::CodeBlock => {
                        if !current_spans.is_empty() {
                            lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                        lines.push(Line::raw(""));
                        if style_stack.len() > 1 {
                            style_stack.pop();
                        }
                    }
                    TagEnd::Heading(_) => {
                        let level = current_heading_level.take();
                        if !current_spans.is_empty() {
                            let mut heading_line = Line::from(std::mem::take(&mut current_spans));
                            if level == Some(HeadingLevel::H1) {
                                heading_line.style = Style::default().bg(Color::White);
                            }
                            lines.push(heading_line);
                        }
                        lines.push(Line::raw(""));
                        if style_stack.len() > 1 {
                            style_stack.pop();
                        }
                    }
                    TagEnd::List(_) => {
                        list_level = list_level.saturating_sub(1);
                        if list_level == 0
                            && !lines.is_empty()
                            && lines.last().is_some_and(|l| !l.spans.is_empty())
                            && lines
                                .last()
                                .is_some_and(|l| !l.spans.is_empty() || l.style != Style::default())
                        {
                            lines.push(Line::raw(""));
                        }
                    }
                    TagEnd::Item => {
                        if !current_spans.is_empty() {
                            lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                        if style_stack.len() > 1 {
                            style_stack.pop();
                        }
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                let style = *style_stack.last().unwrap_or(&base_style);
                let is_in_code_block = style_stack
                    .last()
                    .is_some_and(|s| s.fg == Some(Color::Cyan));

                for (i, part) in text.split('\n').enumerate() {
                    if i > 0 {
                        lines.push(Line::from(std::mem::take(&mut current_spans)));
                        if list_level > 0 && !is_in_code_block {
                            let indent = "  ".repeat(list_level.saturating_sub(1));
                            current_spans.push(Span::raw(format!("{}  ", indent)));
                        } else if is_in_code_block {
                            // Code blocks typically preserve leading whitespace, but here we split lines
                            // Could potentially add logic to preserve original indentation if needed
                        }
                    }
                    if !part.is_empty() {
                        current_spans.push(Span::styled(part.to_string(), style));
                    } else if i > 0 && is_in_code_block {
                        lines.push(Line::raw(""));
                    }
                }
            }
            Event::Code(text) => {
                let style = Style::default().fg(Color::Cyan);
                current_spans.push(Span::styled(text.to_string(), style));
            }
            Event::HardBreak => {
                lines.push(Line::from(std::mem::take(&mut current_spans)));
                if list_level > 0 {
                    let indent = "  ".repeat(list_level.saturating_sub(1));
                    current_spans.push(Span::raw(format!("{}  ", indent)));
                }
            }
            Event::SoftBreak => {
                current_spans.push(Span::raw(" "));
            }
            Event::Rule => {
                if !current_spans.is_empty() {
                    lines.push(Line::from(std::mem::take(&mut current_spans)));
                }
                lines.push(Line::from("─".repeat(50)).style(Style::default().fg(Color::DarkGray)));
                lines.push(Line::raw(""));
            }
            _ => {}
        }
    }

    if !current_spans.is_empty() {
        lines.push(Line::from(current_spans));
    }

    while lines
        .last()
        .is_some_and(|l| l.spans.is_empty() && l.style == Style::default())
    {
        lines.pop();
    }

    Text::from(lines)
}
