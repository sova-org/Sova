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
            // --- Column 1 Start ---
            Line::from(Span::styled("Navigation & Selection", Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))),
            Line::from(vec![
                Span::styled("  ↑↓←→       ", key_style),
                Span::styled(": Move Cursor", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Shift+↑↓←→ ", key_style),
                Span::styled(": Select Frames", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Esc        ", key_style),
                Span::styled(": Reset Selection", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  PgUp/PgDn  ", key_style),
                Span::styled(": Scroll Grid", desc_style),
            ]),
            Line::from(" "), // Spacer

            Line::from(Span::styled("Frame Editing", Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))),
             Line::from(vec![
                Span::styled("  Enter     ", key_style),
                Span::styled(": Edit Frame Script", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Space     ", key_style),
                Span::styled(": Enable Frame(s)", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  l         ", key_style),
                Span::styled(": Set Frame Length", desc_style),
            ]),
             Line::from(vec![
                Span::styled("  n         ", key_style),
                Span::styled(": Set Frame Name", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  B         ", key_style),
                Span::styled(": Set Loop", desc_style),
            ]),
            Line::from(" "), // Spacer

            // --- Column 1 End ---


            // --- Column 2 Start ---
             Line::from(Span::styled("Frame Manipulation", Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))),
            Line::from(vec![
                Span::styled("  i         ", key_style),
                Span::styled(": Insert Frame", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Del/Bksp  ", key_style),
                Span::styled(": Delete Frame(s)", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  a / d     ", key_style),
                Span::styled(": Duplicate Selection", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  c / p     ", key_style),
                Span::styled(": Copy / Paste Selected Frame(s)", desc_style),
            ]),
             Line::from(" "), // Spacer


            Line::from(Span::styled("Line Manipulation", Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))),
             Line::from(vec![
                 Span::styled("  Shift+A ", key_style),
                 Span::styled(": Add Line", desc_style),
             ]),
            Line::from(vec![
                Span::styled("  Shift+X   ", key_style),
                Span::styled(": Remove Current Line", desc_style),
            ]),
            Line::from(" "), // Spacer

             Line::from(Span::styled("General", Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))),
            Line::from(vec![
                Span::styled("  L         ", key_style), // Changed from Shift+L
                Span::styled(": Set Scene Length", desc_style),
            ]),
             Line::from(vec![
                Span::styled("  ?         ", key_style),
                Span::styled(": Toggle this Help", desc_style),
            ]),
            // --- Column 2 End ---
        ]
    }
}

impl Widget for GridHelpPopupWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // --- Create two-column layout (calculate combined lines first) ---
        let all_lines = Self::create_help_text();
        let mid = (all_lines.len() + 1) / 2;
        let left_col_lines = &all_lines[..mid];
        let right_col_lines = &all_lines[mid..];

        let num_rows = left_col_lines.len().max(right_col_lines.len());
        let mut combined_lines = Vec::with_capacity(num_rows);
        let spacer = Span::raw("    ");
        let desired_left_col_width: usize = 35;
        let mut max_line_width: usize = 0;

        for i in 0..num_rows {
            let left_line = left_col_lines.get(i).cloned().unwrap_or_else(|| Line::raw(""));
            let right_line = right_col_lines.get(i).cloned().unwrap_or_else(|| Line::raw(""));

            let left_width = left_line.width();
            let padding_width = desired_left_col_width.saturating_sub(left_width);
            let padding_span = Span::raw(" ".repeat(padding_width));

            let mut combined_spans = left_line.spans;
            combined_spans.push(padding_span);
            combined_spans.push(spacer.clone());
            combined_spans.extend(right_line.spans);

            let current_line = Line::from(combined_spans);
            max_line_width = max_line_width.max(current_line.width()); // Track max width
            combined_lines.push(current_line);
        }

        // --- Calculate popup size based on content ---
        let padding_and_border_width = 4; // 1 padding + 1 border on each side (left/right)
        let padding_and_border_height = 4; // 1 padding + 1 border on each side (top/bottom)

        let content_width = max_line_width;
        let content_height = num_rows;

        let popup_width = (content_width + padding_and_border_width)
            .min(area.width.into()); // Ensure it fits within the total area width
        let popup_height = (content_height + padding_and_border_height)
            .min(area.height.into()); // Ensure it fits within the total area height

        // --- Calculate centered rect using absolute dimensions ---
        let popup_area = {
            let vertical_margin = area.height.saturating_sub(popup_height as u16) / 2;
            let horizontal_margin = area.width.saturating_sub(popup_width as u16) / 2;
            Rect::new(
                area.x + horizontal_margin,
                area.y + vertical_margin,
                popup_width as u16, // Use calculated width
                popup_height as u16, // Use calculated height
            )
        };

        // --- Render the popup ---
        let popup_block = Block::default()
            .title(" Grid Help ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .style(Style::default().fg(Color::White))
            .padding(Padding::uniform(1));

        let help_paragraph = Paragraph::new(combined_lines)
            .block(popup_block)
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: false });

        Clear.render(popup_area, buf);
        help_paragraph.render(popup_area, buf);
    }
} 