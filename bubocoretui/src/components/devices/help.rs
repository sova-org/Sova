use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
    layout::Alignment,
};

/// A widget that displays context-sensitive help text at the bottom of the devices screen.
/// 
/// The help text changes based on the current user interaction state:
/// - When naming a virtual MIDI port
/// - When assigning a slot number to a device
/// - When creating a new OSC device
/// - When in the default navigation state
/// 
/// # Fields
/// 
/// * `is_naming_virtual` - Whether the user is currently entering a name for a new virtual MIDI port
/// * `is_assigning_slot` - Whether the user is currently assigning a slot number to a device
/// * `is_creating_osc` - Whether the user is in the multi-step OSC device creation process
pub struct HelpTextWidget {
    pub is_naming_virtual: bool,
    pub is_assigning_slot: bool,
    pub is_creating_osc: bool,
}

impl Widget for HelpTextWidget {

    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let key_style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);
        let text_style = Style::default().fg(Color::DarkGray);
        let help_spans1: Vec<Span>;
        let help_spans2: Vec<Span>;

        if self.is_naming_virtual {
            help_spans1 = vec![
                Span::styled("Enter", key_style),
                Span::styled(": Confirm | ", text_style),
                Span::styled("Esc", key_style),
                Span::styled(": Cancel", text_style),
            ];
            help_spans2 = vec![
                Span::styled("↑↓", key_style),
                Span::styled(": Browse through history", text_style),
            ];
        } else if self.is_assigning_slot {
            help_spans1 = vec![
                Span::styled("Enter", key_style),
                Span::styled(": Confirm | ", text_style),
                Span::styled("Esc", key_style),
                Span::styled(": Cancel | ", text_style),
                Span::styled("0-9", key_style),
                Span::styled(": Enter Slot Number", text_style),
            ];
            help_spans2 = vec![Span::raw("")];
        } else if self.is_creating_osc {
            help_spans1 = vec![
                Span::styled("Enter", key_style),
                Span::styled(": Next/Confirm | ", text_style),
                Span::styled("Esc", key_style),
                Span::styled(": Back/Cancel", text_style),
            ];
            help_spans2 = vec![Span::raw("")];
        } else {
            help_spans1 = vec![
                Span::styled("↑↓", key_style),
                Span::styled(": Navigate | ", text_style),
                Span::styled("M", key_style),
                Span::styled("/", text_style),
                Span::styled("O", key_style),
                Span::styled(": MIDI/OSC | ", text_style),
                Span::styled("s", key_style),
                Span::styled(": Assign Slot", text_style),
            ];
            help_spans2 = vec![
                Span::styled("Enter", key_style),
                Span::styled(": Connect | ", text_style),
                Span::styled("Bksp/Del", key_style),
                Span::styled(": Disconnect | ", text_style),
                Span::styled("Ctrl+N", key_style),
                Span::styled(": New ", text_style),
            ];
        }

        let help_text = vec![Line::from(help_spans1), Line::from(help_spans2)];
        let help_paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
        ratatui::widgets::Widget::render(help_paragraph, area, buf);
    }
}

