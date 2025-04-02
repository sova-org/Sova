use crate::App;
use crate::components::{Component, inner_area};
use color_eyre::Result as EyreResult;
use bubocorelib::server::client::ClientMessage;
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
    ) -> EyreResult<bool> {
        // Handle editor-specific keys (e.g., passing to textarea)
        match key_event.code {
            // Example: Send script on Ctrl+E
            KeyCode::Char('e') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.send_client_message(ClientMessage::SetScript(
                    app.editor.active_sequence.pattern as usize, 
                    app.editor.active_sequence.script as usize, 
                    app.editor.textarea.lines().join("\n"))
                );
                app.set_status_message("Sent script content.".to_string());
                Ok(true)
            }
            // Pass other keys to the textarea
            _ => {
                app.editor.textarea.input(key_event);
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
            .title(" Editor ")
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
