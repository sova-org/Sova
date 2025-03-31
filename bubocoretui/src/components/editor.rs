use crate::App;
use crate::components::{Component, handle_common_keys, inner_area};
use crate::event::AppEvent;
use bubocorelib::server::client::ClientMessage;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    prelude::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
};
use std::error::Error;

pub struct EditorComponent;

impl EditorComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for EditorComponent {
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> Result<bool, Box<dyn Error + 'static>> {
        if handle_common_keys(app, key_event)? {
            return Ok(true);
        }

        match key_event.code {
            // Envoie le contenu du script édité au serveur et flashe l'écran
            KeyCode::Char('e') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.send_client_message(ClientMessage::SetScript(
                    app.editor.active_sequence.pattern as usize, 
                    app.editor.active_sequence.script as usize, 
                    app.editor.textarea.lines().join("\n"))
                );
                Ok(true)
            }
            KeyCode::Tab => {
                app.events.send(AppEvent::SwitchToGrid);
                Ok(true)
            }
            _ => {
                // Handle text input
                app.editor.textarea.input(key_event);
                app.set_content(app.editor.textarea.lines().join("\n"));
                Ok(true)
            }
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        // Create the main horizontal layout with 80%/20% split
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(area);

        // Editor area (left side - 80%)
        let editor_area = chunks[0];
        let editor = Block::default()
            .title("Editor")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(editor.clone(), editor_area);

        let editor_text_area = inner_area(editor_area);
        // TODO: should we really clone here?
        let mut text_area = app.editor.textarea.clone();
        text_area.set_line_number_style(Style::default().fg(Color::DarkGray));
        frame.render_widget(&text_area, editor_text_area);

        // Info panel (right side - 20%)
        let info_area = chunks[1];
        let info_panel = Block::default()
            .title("Info")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(info_panel.clone(), info_area);

        let (pattern, script) = (
            app.editor.active_sequence.pattern,
            app.editor.active_sequence.script,
        );

        let info_content = Paragraph::new(Text::from(format!(
            "Pattern: {} \nScript: {}",
            pattern, script
        )))
        .style(Style::default());

        let info_text_area = inner_area(info_area);
        frame.render_widget(info_content, info_text_area);
    }
}
