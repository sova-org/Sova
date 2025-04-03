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
        match key_event.code {
            // Envoi du script lorsque la touche Ctrl+E est pressée
            KeyCode::Char('e') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.send_client_message(ClientMessage::SetScript(
                    app.editor.active_sequence.pattern as usize, 
                    app.editor.active_sequence.script as usize, 
                    app.editor.textarea.lines().join("\n"))
                );
                app.set_status_message("Sent script content.".to_string());
                app.flash_screen();
                Ok(true)
            }
            _ => {
                app.editor.textarea.input(key_event);
                Ok(true)
            }
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let editor_block = Block::default()
            .title(" Editor ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(editor_block.clone(), area);
        let inner_editor_area = editor_block.inner(area);

        let editor_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // Contenu
                Constraint::Length(1), // Aide
            ])
            .split(inner_editor_area);
        
        let editor_text_area = editor_chunks[0];
        let editor_help_area = editor_chunks[1];

        let mut text_area = app.editor.textarea.clone();
        text_area.set_line_number_style(Style::default().fg(Color::DarkGray));
        frame.render_widget(&text_area, editor_text_area);

        // Indication des touches
        let help_text = "Ctrl+E: Send Script | Standard Text Input";
        let help = Paragraph::new(Text::from(help_text))
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help, editor_help_area);
    }
}
