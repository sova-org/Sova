use crate::app::App;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::*,
    style::{Color, Style, Stylize},
    widgets::*,
};

pub fn render_editor_help_popup(app: &App, frame: &mut Frame, area: Rect) {
    if !app.editor.is_help_popup_active {
        return;
    }

    // Define popup area (e.g., 60% width, 70% height)
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15), // Top margin
            Constraint::Percentage(70), // Popup height
            Constraint::Percentage(15), // Bottom margin
        ])
        .split(area);

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Left margin
            Constraint::Percentage(60), // Popup width
            Constraint::Percentage(20), // Right margin
        ])
        .split(popup_layout[1])[1];

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    let help_title = " Editor Help ";
    let help_block = Block::default()
        .title(help_title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().fg(Color::White))
        .padding(Padding::left(1));

    let help_text = vec![
        Line::from(vec![Span::styled("General:", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))]),
        Line::from(vec![
            Span::styled("  Esc/?     ", Style::default().fg(Color::Green)),
            Span::styled(": Close Help", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Esc         ", Style::default().fg(Color::Green)),
            Span::styled(": Exit Editor", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+S      ", Style::default().fg(Color::Green)),
            Span::styled(": Send Script", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+G      ", Style::default().fg(Color::Green)),
            Span::styled(": Search", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+E      ", Style::default().fg(Color::Green)),
            Span::styled(": Enable/Disable Frame", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+L      ", Style::default().fg(Color::Green)),
            Span::styled(": Change Language", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+↑/↓/←/→", Style::default().fg(Color::Green)),
            Span::styled(": Navigate Frames", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("Normal Mode:", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))]),
        Line::from("  See Emacs keybindings documentation."),
        Line::from(vec![Span::styled("Vim Mode:", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))]),
        Line::from("  See Vim keybindings documentation.".fg(Color::White)),
        // Add more Vim commands as needed
        Line::from(""),
        Line::from(vec![Span::styled("Search Mode (Ctrl+G):", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))]),
        Line::from("  Type query...".fg(Color::White)),
        Line::from(vec![
            Span::styled("  Enter       ", Style::default().fg(Color::Green)),
            Span::styled(": Find next & Close", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Esc         ", Style::default().fg(Color::Green)),
            Span::styled(": Cancel Search", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+N / ↓  ", Style::default().fg(Color::Green)),
            Span::styled(": Find Next Match", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+P / ↑  ", Style::default().fg(Color::Green)),
            Span::styled(": Find Previous Match", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("Language Popup (Ctrl+L):", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))]),
        Line::from(vec![
            Span::styled("  ↑ / ↓       ", Style::default().fg(Color::Green)),
            Span::styled(": Select Language", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Enter       ", Style::default().fg(Color::Green)),
            Span::styled(": Confirm Selection", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Esc         ", Style::default().fg(Color::Green)),
            Span::styled(": Cancel", Style::default().fg(Color::White)),
        ]),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(help_block.clone())
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);

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