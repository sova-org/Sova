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
use std::cmp::min;

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
        // --- Handle specific Ctrl combinations first ---
        if key_event.modifiers == KeyModifiers::CONTROL {
            match key_event.code {
                // Send script with Ctrl+S
                KeyCode::Char('s') => {
                    app.add_log(crate::app::LogLevel::Debug, "Ctrl+S detected, attempting to send script...".to_string());
                    app.send_client_message(ClientMessage::SetScript(
                        app.editor.active_sequence.sequence_index,
                        app.editor.active_sequence.step_index,
                        app.editor.textarea.lines().join("\n"))
                    );
                    app.set_status_message("Sent script content (Ctrl+S).".to_string());
                    app.flash_screen();
                    return Ok(true); // Handled
                }

                // Ctrl + Arrow navigation
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                    if let Some(pattern) = &app.editor.pattern {
                        let current_seq_idx = app.editor.active_sequence.sequence_index;
                        let current_step_idx = app.editor.active_sequence.step_index;
                        let num_sequences = pattern.sequences.len();

                        if num_sequences == 0 {
                            app.set_status_message("No sequences to navigate.".to_string());
                            return Ok(true); // Handled (no-op)
                        }

                        match key_event.code {
                            KeyCode::Up => {
                                if current_step_idx == 0 {
                                    app.set_status_message("Already at first step.".to_string());
                                    return Ok(true);
                                }
                                let target_seq_idx = current_seq_idx;
                                let target_step_idx = current_step_idx - 1;
                                // Validity check is implicit as we are moving within the same sequence and checked current_step_idx > 0
                                app.send_client_message(ClientMessage::GetScript(target_seq_idx, target_step_idx));
                                app.set_status_message(format!("Requested script Seq {}, Step {}", target_seq_idx, target_step_idx));
                            }
                            KeyCode::Down => {
                                if let Some(seq) = pattern.sequences.get(current_seq_idx) {
                                    if current_step_idx + 1 >= seq.steps.len() {
                                        app.set_status_message("Already at last step.".to_string());
                                        return Ok(true);
                                    }
                                    let target_seq_idx = current_seq_idx;
                                    let target_step_idx = current_step_idx + 1;
                                    app.send_client_message(ClientMessage::GetScript(target_seq_idx, target_step_idx));
                                    app.set_status_message(format!("Requested script Seq {}, Step {}", target_seq_idx, target_step_idx));
                                } else { return Ok(true); /* Should not happen */ }
                            }
                            KeyCode::Left => {
                                if current_seq_idx == 0 {
                                    app.set_status_message("Already at first sequence.".to_string());
                                    return Ok(true);
                                }
                                let target_seq_idx = current_seq_idx - 1;
                                let target_seq_len = pattern.sequences[target_seq_idx].steps.len();
                                if target_seq_len == 0 {
                                    app.set_status_message(format!("Sequence {} is empty.", target_seq_idx));
                                    return Ok(true);
                                }
                                let target_step_idx = min(current_step_idx, target_seq_len - 1);
                                app.send_client_message(ClientMessage::GetScript(target_seq_idx, target_step_idx));
                                app.set_status_message(format!("Requested script Seq {}, Step {}", target_seq_idx, target_step_idx));
                            }
                            KeyCode::Right => {
                                if current_seq_idx + 1 >= num_sequences {
                                    app.set_status_message("Already at last sequence.".to_string());
                                    return Ok(true);
                                }
                                let target_seq_idx = current_seq_idx + 1;
                                let target_seq_len = pattern.sequences[target_seq_idx].steps.len();
                                if target_seq_len == 0 {
                                    app.set_status_message(format!("Sequence {} is empty.", target_seq_idx));
                                    return Ok(true);
                                }
                                let target_step_idx = min(current_step_idx, target_seq_len - 1);
                                app.send_client_message(ClientMessage::GetScript(target_seq_idx, target_step_idx));
                                app.set_status_message(format!("Requested script Seq {}, Step {}", target_seq_idx, target_step_idx));
                            }
                            _ => unreachable!(),
                        }
                        return Ok(true); // Navigation handled (or attempted)

                    } else {
                        app.set_status_message("Pattern not loaded, cannot navigate.".to_string());
                        return Ok(true); // Handled (no-op)
                    }
                } // End Ctrl + Arrow case

                // Other Ctrl combinations are not handled by the editor text area
                _ => return Ok(false),
            }
        }

        // --- Handle all other key events (including non-Ctrl) by passing to textarea ---
        let handled_by_textarea = app.editor.textarea.input(key_event);
        Ok(handled_by_textarea)
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
        let help_text = "Ctrl+S: Send Script | Ctrl+Arrows: Navigate | Standard Text Input";
        let help = Paragraph::new(Text::from(help_text))
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help, editor_help_area);
    }
}
