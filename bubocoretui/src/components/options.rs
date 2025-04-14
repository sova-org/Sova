use crate::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Rect, Layout, Direction, Constraint},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem, BorderType},
};

pub struct OptionsComponent {
    /// Index de l'option actuellement sélectionnée
    selected_option_index: usize,
}

impl OptionsComponent {
    pub fn new() -> Self {
        Self { selected_option_index: 0 }
    }
}

impl Component for OptionsComponent {

    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        match key_event.code {
             // Enable/disable the phase bar (only option for now)
            KeyCode::Enter => {
                if self.selected_option_index == 0 {
                    app.settings.show_phase_bar = !app.settings.show_phase_bar;
                    app.set_status_message(format!(
                        "Phase bar visibility set to {}",
                        if app.settings.show_phase_bar { "On" } else { "Off" }
                    ));
                    return Ok(true);
                }
            }
            // TODO: add up/down navigation
            _ => {}
        }
        Ok(false)
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Options ")
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::White));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // Liste des options
                Constraint::Length(1), // Aide
            ])
            .split(inner_area);

        let options_area = chunks[0];
        let help_area = chunks[1];

        // List of available options
        let options = vec![
            ListItem::new(Line::from(vec![
                Span::raw("Show Phase Bar: "),
                Span::styled(
                    if app.settings.show_phase_bar { "On" } else { "Off" },
                    Style::default().fg(if app.settings.show_phase_bar { Color::Green } else { Color::Red }),
                ),
            ])),
        ];

        // Creating the options list
        let options_list = List::new(options)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray));
        frame.render_widget(options_list, options_area);

        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        let help_spans = vec![
            Span::styled("Enter", key_style), Span::styled(": Toggle Option", help_style),
            // Span::styled(" | ", help_style),
            // Span::styled("↑↓", key_style), Span::styled(": Navigate", help_style),
        ];
        let help = Paragraph::new(Line::from(help_spans))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
