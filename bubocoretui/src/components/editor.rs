use crate::App;
use crate::{
    components::Component,
    components::logs::LogLevel,
};
use color_eyre::Result as EyreResult;
use bubocorelib::server::client::ClientMessage;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    prelude::{Constraint, Direction, Layout, Rect, Modifier},
    style::{Color, Style},

    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, BorderType},
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
        // Handle Esc separately to leave the editor
        if key_event.code == KeyCode::Esc {
            // Send notification that we stopped editing this specific step
            app.send_client_message(ClientMessage::StoppedEditingStep(
                app.editor.active_sequence.sequence_index,
                app.editor.active_sequence.step_index
            ));
            // Switch back to grid mode
            app.events.sender.send(crate::event::Event::App(crate::event::AppEvent::SwitchToGrid))?;
            app.set_status_message("Exited editor (Esc).".to_string());
            return Ok(true);
        }

        // --- Handle specific Ctrl combinations first ---
        if key_event.modifiers == KeyModifiers::CONTROL {
            match key_event.code {
                // Send script with Ctrl+S
                KeyCode::Char('s') => {
                    app.add_log(LogLevel::Debug, "Ctrl+S detected, attempting to send script...".to_string());
                    app.send_client_message(ClientMessage::SetScript(
                        app.editor.active_sequence.sequence_index,
                        app.editor.active_sequence.step_index,
                        app.editor.textarea.lines().join("\n"))
                    );
                    app.set_status_message("Sent script content (Ctrl+S).".to_string());
                    app.flash_screen();
                    return Ok(true); // Handled
                }

                // Toggle step enabled/disabled with Ctrl+E
                KeyCode::Char('e') => {
                    if let Some(pattern) = &app.editor.pattern {
                        let seq_idx = app.editor.active_sequence.sequence_index;
                        let step_idx = app.editor.active_sequence.step_index;

                        if let Some(sequence) = pattern.sequences.get(seq_idx) {
                            if step_idx < sequence.steps.len() {
                                let current_enabled_status = sequence.is_step_enabled(step_idx);
                                let message = if current_enabled_status {
                                    ClientMessage::DisableSteps(seq_idx, vec![step_idx])
                                } else {
                                    ClientMessage::EnableSteps(seq_idx, vec![step_idx])
                                };
                                app.send_client_message(message);
                                app.set_status_message(format!(
                                    "Toggled Step {}/{} to {}",
                                    seq_idx, step_idx, if !current_enabled_status { "Enabled" } else { "Disabled" }
                                ));
                            } else {
                                app.set_status_message("Cannot toggle: Invalid step index.".to_string());
                            }
                        } else {
                            app.set_status_message("Cannot toggle: Invalid sequence index.".to_string());
                        }
                    } else {
                        app.set_status_message("Cannot toggle: Pattern not loaded.".to_string());
                    }
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
        let (status_str, length_str, is_enabled) = 
            if let Some(pattern) = &app.editor.pattern {
                if let Some(sequence) = pattern.sequences.get(seq_idx) {
                    if step_idx < sequence.steps.len() {
                        let enabled = sequence.is_step_enabled(step_idx);
                        let length = sequence.steps[step_idx];
                        ( if enabled { "Enabled" } else { "Disabled" },
                          format!("Len: {:.2}", length),
                          enabled
                        )
                    } else {
                        ("Invalid Step", "Len: N/A".to_string(), true) // Default to enabled appearance if invalid
                    }
                } else {
                    ("Invalid Seq", "Len: N/A".to_string(), true) // Default to enabled appearance if invalid
                }
            } else {
                ("No Pattern", "Len: N/A".to_string(), true) // Default to enabled appearance if no pattern
            };

        // Determine border color based on step status
        let border_color = if is_enabled { Color::White } else { Color::DarkGray };

        let editor_block = Block::default()
            .title(format!(
                " Editor (Seq: {}, Step: {} | {} | {}) ", 
                seq_idx,
                step_idx,
                status_str, // Show enabled/disabled status
                length_str  // Show length
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(border_color));

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
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        let help_spans = vec![
            Span::styled("Ctrl+S", key_style), Span::styled(": Send Script | ", help_style),
            Span::styled("Ctrl+E", key_style), Span::styled(": Toggle Step | ", help_style),
            Span::styled("Ctrl+Arrows", key_style), Span::styled(": Navigate | ", help_style),
            Span::styled("Standard Input", key_style), Span::styled(": Edit", help_style),
        ];
        let help = Paragraph::new(Line::from(help_spans))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help, editor_help_area);
    }
}
