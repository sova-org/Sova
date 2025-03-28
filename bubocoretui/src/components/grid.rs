use crate::App;
use crate::components::{Component, handle_common_keys, inner_area};
use crate::event::AppEvent;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    prelude::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
};
use std::error::Error;

pub struct GridComponent;

impl GridComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for GridComponent {
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> Result<bool, Box<dyn Error + 'static>> {
        // First try common key handlers
        if handle_common_keys(app, key_event)? {
            return Ok(true);
        }

        // Grid-specific key handling
        match key_event.code {
            KeyCode::Tab => {
                app.events.send(AppEvent::SwitchToOptions);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn draw(&self, _app: &App, frame: &mut Frame, area: Rect) {
        // Création d'un bloc central
        let block = Block::default()
            .title("Grid")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(block, area);

        // On affiche n'importe quoi
        let grid_content = Paragraph::new(Text::from("Idk what to do :)))) "))
            .style(Style::default())
            .block(Block::default());

        let grid_area = inner_area(area);
        frame.render_widget(grid_content, grid_area);
    }
}
