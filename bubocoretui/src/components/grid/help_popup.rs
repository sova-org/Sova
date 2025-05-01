use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Widget, Padding, BorderType},
};
use crate::components::grid::utils::centered_rect;

/// A widget that renders a help popup overlay for the grid component.
///
/// This widget displays keyboard shortcuts and their descriptions in a formatted overlay.
/// It is rendered when the user presses '?' to show help information about grid operations.
/// The help popup includes sections for navigation, selection, and frame editing commands.
pub struct GridHelpPopupWidget;

impl GridHelpPopupWidget {
    fn create_help_text() -> Vec<Line<'static>> {
        let key_style = Style::default()
            .fg(Color::Green);
        let desc_style = Style::default().fg(Color::White);

        vec![
            // Navigation & Selection
            Line::from(vec![
                Span::styled("  ↑↓←→      ", key_style),
                Span::styled(": Move Cursor", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Shift+↑↓←→", key_style),
                Span::styled(": Select Multiple Frames", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Esc       ", key_style),
                Span::styled(": Reset Selection to Cursor", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  PgUp/PgDn ", key_style),
                Span::styled(": Scroll Grid View", desc_style),
            ]),
            Line::from(" "), // Spacer
            // Frame Editing
            Line::from(vec![
                Span::styled("  Enter     ", key_style),
                Span::styled(": Edit Frame Script", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Space     ", key_style),
                Span::styled(": Enable/Disable Frame(s)", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  l         ", key_style),
                Span::styled(": Set Length (Enter Input Mode)", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  n         ", key_style),
                Span::styled(": Set Name (Enter Input Mode)", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  b / e     ", key_style),
                Span::styled(": Toggle Line Start/End Marker at Cursor", desc_style),
            ]),
            Line::from(" "), // Spacer
            // Frame Manipulation
            Line::from(vec![
                Span::styled("  i         ", key_style),
                Span::styled(": Insert Frame After (Enter Input Mode)", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Del/Bksp  ", key_style),
                Span::styled(": Delete Selected Frame(s)", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  a / d     ", key_style),
                Span::styled(
                    ": Duplicate Selection Before/After Cursor Column",
                    desc_style,
                ),
            ]),
            Line::from(vec![
                Span::styled("  c / p     ", key_style),
                Span::styled(": Copy / Paste Selected Frame(s)", desc_style),
            ]),
            Line::from(" "), // Spacer
            // Line Manipulation
            Line::from(vec![
                Span::styled("  Shift+A   ", key_style),
                Span::styled(": Add New Line", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Shift+D   ", key_style),
                Span::styled(": Remove Last Line", desc_style),
            ]),
            Line::from(" "), // Spacer
            // General
            Line::from(vec![
                Span::styled("  Shift+L   ", key_style),
                Span::styled(": Set Scene Length (Enter Input Mode)", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  ?         ", key_style),
                Span::styled(": Toggle this Help", desc_style),
            ]),
        ]
    }
}

impl Widget for GridHelpPopupWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = centered_rect(60, 60, area);

        let popup_block = Block::default()
            .title(" Grid Help ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .style(Style::default().fg(Color::White))
            .padding(Padding::uniform(1));

        let help_lines = Self::create_help_text();
        let help_paragraph = Paragraph::new(help_lines)
            .block(popup_block)
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        Clear.render(popup_area, buf);
        help_paragraph.render(popup_area, buf);
    }
} 