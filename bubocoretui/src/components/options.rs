use crate::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Rect, Layout, Direction, Constraint},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
};

pub struct OptionsComponent;

impl OptionsComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for OptionsComponent {
    fn handle_key_event(
        &mut self,
        _app: &mut App,
        _key_event: KeyEvent,
    ) -> EyreResult<bool> {
        // No specific key handling for now
        Ok(false)
    }

    fn draw(&self, _app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Options ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        
        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(inner_area);

        let content_area = chunks[0];
        let help_area = chunks[1];

        let placeholder_text = "This is the Options view.\nConfiguration options will be available here.";
        let paragraph = Paragraph::new(Text::from(placeholder_text))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, content_area);

        let help_text = "Keybindings TBD";
        let help = Paragraph::new(Text::from(help_text))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
