use crate::App;
use crate::components::Component;
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
        // Create the main horizontal layout directly on the input `area`
        let inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(area); // Use area directly

        let editor_area = inner_chunks[0];
        let info_area = inner_chunks[1];

        // Draw editor block first
        let editor_block = Block::default()
            .title(" Editor ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(editor_block.clone(), editor_area);
        // Get inner area of the editor block
        let inner_editor_area = editor_block.inner(editor_area);

        // Split the inner editor area for text and help
        let editor_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // Text area
                Constraint::Length(1), // Help line
            ])
            .split(inner_editor_area);
        
        let editor_text_area = editor_chunks[0];
        let editor_help_area = editor_chunks[1];

        // Render the text area
        let mut text_area = app.editor.textarea.clone();
        text_area.set_line_number_style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text_area.widget(), editor_text_area); // Render in top part

        // Render editor help text inside the editor block
        let help_text = "Ctrl+E: Send Script | Standard Text Input"; // Updated help
        let help = Paragraph::new(Text::from(help_text))
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help, editor_help_area); // Render in bottom part

        // Info panel (right side - 20%) - No changes needed here
        let info_block = Block::default()
            .title(" Info ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(info_block.clone(), info_area);
        let info_text_area = info_block.inner(info_area);

        let (pattern, script) = (
            app.editor.active_sequence.pattern,
            app.editor.active_sequence.script,
        );

        let info_content = Paragraph::new(Text::from(format!(
            "Pattern: {} \nScript: {}",
            pattern, script
        )))
        .style(Style::default().fg(Color::White));

        frame.render_widget(info_content, info_text_area);
    }
}
