use crate::app::App;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    prelude::*,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};

pub fn render_editor_help_popup(app: &App, frame: &mut Frame, area: Rect) {
    if !app.editor.is_help_popup_active {
        return;
    }

    // Apply bold white style to titles
    let title_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let key_style = Style::default().fg(Color::Green);
    let desc_style = Style::default().fg(Color::White);

    let help_lines = vec![
        // --- Column 1 Start ---
        Line::from(Span::styled("General:", title_style)),
        Line::from(vec![
            Span::styled("  Ctrl+H/ ? ", key_style),
            Span::styled(": Open Help", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Esc / ?   ", key_style),
            Span::styled(": Close Help", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Esc       ", key_style),
            Span::styled(": Exit Editor", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+S    ", key_style),
            Span::styled(": Send Script", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+G    ", key_style),
            Span::styled(": Search", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+E    ", key_style),
            Span::styled(": Enable/Disable Frame", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+L    ", key_style),
            Span::styled(": Change Language", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+↑/↓/←/→", key_style),
            Span::styled(": Navigate Frames", desc_style),
        ]),
        Line::from(""),
        // --- Column 1 End ---

        // --- Column 2 Start ---
        Line::from(Span::styled("Search Mode (Ctrl+G)", title_style)),
        Line::from(vec![
            Span::styled("  Enter       ", key_style),
            Span::styled(": Find next & Close", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Esc         ", key_style),
            Span::styled(": Cancel Search", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+N / ↓  ", key_style),
            Span::styled(": Find Next Match", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+P / ↑  ", key_style),
            Span::styled(": Find Previous Match", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("Language Popup (Ctrl+L):", title_style)),
        Line::from(vec![
            Span::styled("  ↑ / ↓       ", key_style),
            Span::styled(": Select Language", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Enter       ", key_style),
            Span::styled(": Confirm Selection", desc_style),
        ]),
        // --- Column 2 End ---
    ];

    // --- Create two-column layout ---
    let mid = 10; // Index 10 starts the right column
    let left_col_lines = &help_lines[..mid];
    let right_col_lines = &help_lines[mid..];

    let num_rows = left_col_lines.len().max(right_col_lines.len());
    let mut combined_lines = Vec::with_capacity(num_rows);
    let spacer = Span::raw("    "); // Adjust spacer width if needed
    let desired_left_col_width: usize = 35; // Adjust based on expected left col content width
    let mut max_line_width: usize = 0;

    for i in 0..num_rows {
        let left_line = left_col_lines
            .get(i)
            .cloned()
            .unwrap_or_else(|| Line::raw(""));
        let right_line = right_col_lines
            .get(i)
            .cloned()
            .unwrap_or_else(|| Line::raw(""));

        let left_width = left_line.width();
        let padding_width = desired_left_col_width.saturating_sub(left_width);
        let padding_span = Span::raw(" ".repeat(padding_width));

        let mut combined_spans = left_line.spans;
        combined_spans.push(padding_span);
        combined_spans.push(spacer.clone());
        combined_spans.extend(right_line.spans);

        let current_line = Line::from(combined_spans);
        max_line_width = max_line_width.max(current_line.width());
        combined_lines.push(current_line);
    }

    // --- Calculate popup size based on content ---
    let content_width = max_line_width;
    let content_height = num_rows; // Height is now based on combined rows

    let padding_and_border_width = 4; // 1 padding + 1 border each side
    let padding_and_border_height = 2; // 1 padding + 1 border top/bottom

    let popup_width = (content_width + padding_and_border_width).min(area.width.into());
    let popup_height = (content_height + padding_and_border_height).min(area.height.into());

    // --- Calculate centered rect using absolute dimensions ---
    let popup_area = {
        let vertical_margin = area.height.saturating_sub(popup_height as u16) / 2;
        let horizontal_margin = area.width.saturating_sub(popup_width as u16) / 2;
        Rect::new(
            area.x + horizontal_margin,
            area.y + vertical_margin,
            popup_width as u16,
            popup_height as u16,
        )
    };

    // --- Render the popup ---
    frame.render_widget(Clear, popup_area);

    let help_title = " Editor Help ";
    let help_block = Block::default()
        .title(help_title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().fg(Color::White))
        .padding(Padding::uniform(1));

    let help_paragraph = Paragraph::new(combined_lines) // Use combined lines
        .block(help_block)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: false }); // Disable wrap trimming for columns

    frame.render_widget(help_paragraph, popup_area);
}

/// Handles keyboard input events for the help popup component.
///
/// This function processes keyboard events when the help popup is active:
/// - Esc or '?' key closes the help popup and clears any status message
/// - All other keys are ignored
///
/// # Arguments
/// * `app` - The application state
/// * `key_event` - The keyboard event to process
///
/// # Returns
/// * `EyreResult<bool>` - Ok(true) if the event was handled, Ok(false) if not
///
/// # Examples
/// ```
/// // Close help popup with Esc
/// let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
/// handle_help_popup_input(&mut app, key_event)?;
/// ```
pub fn handle_help_popup_input(app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
    if !app.editor.is_help_popup_active {
        return Ok(false);
    }

    match key_event.code {
        KeyCode::Esc | KeyCode::Char('?') => {
            app.editor.is_help_popup_active = false;
            app.set_status_message("".to_string()); // Use set_status_message to clear
            Ok(true) // Input was handled
        }
        _ => Ok(false), // Input not handled by the popup
    }
}
