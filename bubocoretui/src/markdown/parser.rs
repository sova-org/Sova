use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
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
    let mut style_stack: Vec<Style> = vec![Style::default().fg(Color::White)]; // Base style
    let mut list_level: usize = 0;

    for event in parser {
        match event {
            Event::Start(tag) => {
                let current_style = *style_stack.last().unwrap_or(&Style::default());

                if !matches!(tag, Tag::Item | Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link {..} | Tag::Image {..} ) && !current_spans.is_empty() {
                     lines.push(Line::from(std::mem::take(&mut current_spans)));
                }

                match tag {
                    Tag::Paragraph => {
                        style_stack.push(current_style);
                    }
                    Tag::Heading { level: _, .. } => { // Changed level to _ as it's not used here
                        // Add spacing *before* the heading if needed
                        if !lines.is_empty() && lines.last().map_or(false, |l| !l.spans.is_empty()) {
                             lines.push(Line::raw(""));
                        }
                        let heading_style = Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD);
                        style_stack.push(heading_style);
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
                        let item_marker_style = *style_stack.last().unwrap_or(&Style::default());
                        current_spans.push(Span::styled(format!("{}* ", indent), item_marker_style));
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
                    }
                    _ => { style_stack.push(current_style); }
                }
            }
            Event::End(tag_end) => {
                // Pop style associated with the *ending* tag, EXCEPT for Item.
                // Item's style is popped specially *after* finalizing its line.
                 if !matches!(tag_end, TagEnd::Item) {
                     // Avoid popping the base style
                    if style_stack.len() > 1 {
                         style_stack.pop();
                    }
                }

                match tag_end {
                    TagEnd::Paragraph | TagEnd::CodeBlock => {
                        // Finalize the line for these block elements
                        if !current_spans.is_empty() {
                            lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                        // Add consistent spacing after paragraphs and code blocks
                        lines.push(Line::raw(""));
                    }
                    TagEnd::Heading { .. } => {
                        // Finalize heading line
                        if !current_spans.is_empty() {
                            lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                        // Add a blank line *after* headings for spacing
                        lines.push(Line::raw(""));
                    }
                    TagEnd::List(_) => {
                        // List level is decremented *after* its items are processed
                        if list_level > 0 {
                             list_level -= 1;
                        }
                    }
                     TagEnd::Item => {
                        // Finalize the item line which includes marker + content
                        if !current_spans.is_empty() {
                           lines.push(Line::from(std::mem::take(&mut current_spans)));
                        }
                        // Now pop the style that was pushed specifically for this item's content in Start(Item)
                        if style_stack.len() > 1 {
                            style_stack.pop();
                        }
                     }
                     // Inline styles (Emphasis, Strong, etc.) popping is handled above the match.
                     _ => {}
                }
            }
            Event::Text(text) => {
                let style = *style_stack.last().unwrap_or(&Style::default());
                // Handle potential newlines within the text chunk
                for (i, part) in text.split('\n').enumerate() {
                    if i > 0 { // If we split, finalize the previous line part
                        lines.push(Line::from(std::mem::take(&mut current_spans)));
                        // If inside list, add indentation for wrapped lines
                        if list_level > 0 {
                            let indent = "  ".repeat(list_level.saturating_sub(1));
                            // Add indentation matching the list item marker's level, plus space for marker itself (`* `)
                            current_spans.push(Span::raw(format!("{}  ", indent)));
                        }
                    }
                    if !part.is_empty() {
                         current_spans.push(Span::styled(part.to_string(), style));
                    }
                }
            }
            Event::Code(text) => {
                // Inline code style - different from code block
                let style = Style::default().fg(Color::Cyan); // Removed background color
                current_spans.push(Span::styled(text.to_string(), style));
            }
            Event::HardBreak => {
                // Treat hard break as a definitive line break
                lines.push(Line::from(std::mem::take(&mut current_spans)));
                // If inside list, add indentation for the next line start after hard break
                 if list_level > 0 {
                    let indent = "  ".repeat(list_level.saturating_sub(1));
                    current_spans.push(Span::raw(format!("{}  ", indent)));
                 }
            }
            Event::SoftBreak => {
                 // In TUI, soft breaks often become spaces or are ignored if wrapping handles it.
                 // Adding a space is a reasonable default.
                 current_spans.push(Span::raw(" "));
            }
            Event::Rule => {
                 // Finalize any pending spans before the rule
                 if !current_spans.is_empty() {
                    lines.push(Line::from(std::mem::take(&mut current_spans)));
                 }
                 // Add the rule itself
                 lines.push(Line::from("──────").style(Style::default().fg(Color::DarkGray)));
                 // Add a blank line after the rule for spacing
                 lines.push(Line::raw(""));
            }
            // Ignore TaskListMarker, Html, FootnoteReference, etc.
            _ => {}
        }
    }

    // Add any remaining spans as the last line
    if !current_spans.is_empty() {
        lines.push(Line::from(current_spans));
    }

    Text::from(lines)
}
