use crate::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
};
use std::error::Error;

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
            .style(Style::default().bg(Color::Black).fg(Color::Cyan));
        
        let placeholder_text = "This is the Options view.\nConfiguration options will be available here.";
        let paragraph = Paragraph::new(Text::from(placeholder_text))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(block); // Put the text inside the block

        frame.render_widget(paragraph, area);
    }
}
