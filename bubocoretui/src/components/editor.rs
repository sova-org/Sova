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
                    app.editor.active_sequence.sequence_index,
                    app.editor.active_sequence.step_index,
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
        let seq_idx = app.editor.active_sequence.sequence_index;
        let step_idx = app.editor.active_sequence.step_index;

        // Get step status and length, with default values if not found
        let (status_str, length_str) = 
            if let Some(pattern) = &app.editor.pattern {
                if let Some(sequence) = pattern.sequences.get(seq_idx) {
                    if step_idx < sequence.steps.len() {
                        let is_enabled = sequence.is_step_enabled(step_idx);
                        let length = sequence.steps[step_idx];
                        ( if is_enabled { "Enabled" } else { "Disabled" },
                          format!("Len: {:.2}", length)
                        )
                    } else {
                        ("Invalid Step", "Len: N/A".to_string())
                    }
                } else {
                    ("Invalid Seq", "Len: N/A".to_string())
                }
            } else {
                ("No Pattern", "Len: N/A".to_string())
            };

        let editor_block = Block::default()
            .title(format!(
                " Editor (Seq: {}, Step: {} | {} | {}) ", 
                seq_idx,
                step_idx,
                status_str, // Show enabled/disabled status
                length_str  // Show length
            ))
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
