use crate::App;
use crate::components::{Component, handle_common_keys, inner_area};
use crate::event::AppEvent;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    prelude::{Constraint, Direction, Layout, Rect},
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
        app: &mut App,
        key_event: KeyEvent,
    ) -> Result<bool, Box<dyn Error + 'static>> {
        // First try common key handlers
        if handle_common_keys(app, key_event)? {
            return Ok(true);
        }

        // Options-specific key handling
        match key_event.code {
            KeyCode::Tab => {
                app.events.send(AppEvent::SwitchToEditor);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        // Layout horizontal avec split 60%/40%
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // Boîte de logs (60% width)
        let log_area = main_chunks[0];
        let log_block = Block::default()
            .title("Log")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(log_block, log_area);

        // Contenu à la con
        let log_content = Paragraph::new(Text::from("System log entries will appear here..."))
            .style(Style::default())
            .block(Block::default());

        let log_text_area = inner_area(log_area);
        frame.render_widget(log_content, log_text_area);

        // Trois boites de taille égale (Devices, Peers, Options)
        let right_side = main_chunks[1];
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(right_side);

        // Devices
        let devices_block = Block::default()
            .title("Devices")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(devices_block, right_chunks[0]);

        let devices_content = if app.server.devices.is_empty() {
            String::from("No devices connected")
        } else {
            app.server.devices.join("\n")
        };

        let devices_text = Paragraph::new(Text::from(devices_content))
            .style(Style::default())
            .block(Block::default());

        let devices_text_area = inner_area(right_chunks[0]);
        frame.render_widget(devices_text, devices_text_area);

        // Peers
        let peers_block = Block::default()
            .title("Peers")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(peers_block, right_chunks[1]);

        let peers_content = if app.server.peers.is_empty() {
            String::from("No peers connected")
        } else {
            app.server.peers.join("\n")
        };

        let peers_text = Paragraph::new(Text::from(peers_content))
            .style(Style::default())
            .block(Block::default());

        let peers_text_area = inner_area(right_chunks[1]);
        frame.render_widget(peers_text, peers_text_area);

        // Options
        let options_block = Block::default()
            .title("Options")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(options_block, right_chunks[2]);

        let options_content = Paragraph::new(Text::from("IDK what to do :))))"))
            .style(Style::default())
            .block(Block::default());

        let options_text_area = inner_area(right_chunks[2]);
        frame.render_widget(options_content, options_text_area);
    }
}
