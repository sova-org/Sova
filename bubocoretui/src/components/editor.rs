use crate::App;
use crate::{
    components::Component,
    components::logs::LogLevel,
};
use color_eyre::Result as EyreResult;
use bubocorelib::server::client::ClientMessage;
use bubocorelib::schedule::ActionTiming;
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
            // Send notification that we stopped editing this specific frame
            app.send_client_message(ClientMessage::StoppedEditingFrame(
                app.editor.active_line.line_index,
                app.editor.active_line.frame_index
            ));
            // Clear any compilation error when exiting
            app.editor.compilation_error = None;
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
                        app.editor.active_line.line_index,
                        app.editor.active_line.frame_index,
                        app.editor.textarea.lines().join("\n"),
                        ActionTiming::Immediate
                    ));
                    // Clear error on successful send attempt
                    app.editor.compilation_error = None;
                    app.set_status_message("Sent script content (Ctrl+S).".to_string());
                    app.flash_screen();
                    return Ok(true); // Handled
                }

                // Toggle frame enabled/disabled with Ctrl+E
                KeyCode::Char('e') => {
                    if let Some(scene) = &app.editor.scene {
                        let line_idx = app.editor.active_line.line_index;
                        let frame_idx = app.editor.active_line.frame_index;

                        if let Some(line) = scene.lines.get(line_idx) {
                            if frame_idx < line.frames.len() {
                                let current_enabled_status = line.is_frame_enabled(frame_idx);
                                let message = if current_enabled_status {
                                    ClientMessage::DisableFrames(
                                        line_idx, vec![frame_idx], ActionTiming::Immediate
                                    )
                                } else {
                                    ClientMessage::EnableFrames(
                                        line_idx, vec![frame_idx], ActionTiming::Immediate
                                    )
                                };
                                app.send_client_message(message);
                                app.set_status_message(format!(
                                    "Toggled Frame {}/{} to {}",
                                    line_idx, frame_idx, if !current_enabled_status { "Enabled" } else { "Disabled" }
                                ));
                            } else {
                                app.set_status_message("Cannot toggle: Invalid frame index.".to_string());
                            }
                        } else {
                            app.set_status_message("Cannot toggle: Invalid line index.".to_string());
                        }
                    } else {
                        app.set_status_message("Cannot toggle: scene not loaded.".to_string());
                    }
                    return Ok(true); // Handled
                }

                // Ctrl + Arrow navigation
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                    if let Some(scene) = &app.editor.scene {
                        let current_line_idx = app.editor.active_line.line_index;
                        let current_frame_idx = app.editor.active_line.frame_index;
                        let num_lines = scene.lines.len();

                        if num_lines == 0 {
                            app.set_status_message("No lines to navigate.".to_string());
                            return Ok(true); // Handled (no-op)
                        }

                        match key_event.code {
                            KeyCode::Up => {
                                if current_frame_idx == 0 {
                                    app.set_status_message("Already at first frame.".to_string());
                                    return Ok(true);
                                }
                                let target_line_idx = current_line_idx;
                                let target_frame_idx = current_frame_idx - 1;
                                // Validity check is implicit as we are moving within the same line and checked current_frame_idx > 0
                                // Clear error when requesting new script
                                app.editor.compilation_error = None;
                                app.send_client_message(ClientMessage::GetScript(target_line_idx, target_frame_idx));
                                app.set_status_message(format!("Requested script Line {}, Frame {}", target_line_idx, target_frame_idx));
                            }
                            KeyCode::Down => {
                                if let Some(line) = scene.lines.get(current_line_idx) {
                                    if current_frame_idx + 1 >= line.frames.len() {
                                        app.set_status_message("Already at last frame.".to_string());
                                        return Ok(true);
                                    }
                                    let target_line_idx = current_line_idx;
                                    let target_frame_idx = current_frame_idx + 1;
                                    // Clear error when requesting new script
                                    app.editor.compilation_error = None;
                                    app.send_client_message(ClientMessage::GetScript(target_line_idx, target_frame_idx));
                                    app.set_status_message(format!("Requested script Line {}, Frame {}", target_line_idx, target_frame_idx));
                                } else { return Ok(true); /* Should not happen */ }
                            }
                            KeyCode::Left => {
                                if current_line_idx == 0 {
                                    app.set_status_message("Already at first line.".to_string());
                                    return Ok(true);
                                }
                                let target_line_idx = current_line_idx - 1;
                                let target_line_len = scene.lines[target_line_idx].frames.len();
                                if target_line_len == 0 {
                                    app.set_status_message(format!("Line {} is empty.", target_line_idx));
                                    return Ok(true);
                                }
                                let target_frame_idx = min(current_frame_idx, target_line_len - 1);
                                // Clear error when requesting new script
                                app.editor.compilation_error = None;
                                app.send_client_message(ClientMessage::GetScript(target_line_idx, target_frame_idx));
                                app.set_status_message(format!("Requested script Line {}, Frame {}", target_line_idx, target_frame_idx));
                            }
                            KeyCode::Right => {
                                if current_line_idx + 1 >= num_lines {
                                    app.set_status_message("Already at last line.".to_string());
                                    return Ok(true);
                                }
                                let target_line_idx = current_line_idx + 1;
                                let target_line_len = scene.lines[target_line_idx].frames.len();
                                if target_line_len == 0 {
                                    app.set_status_message(format!("Line {} is empty.", target_line_idx));
                                    return Ok(true);
                                }
                                let target_frame_idx = min(current_frame_idx, target_line_len - 1);
                                // Clear error when requesting new script
                                app.editor.compilation_error = None;
                                app.send_client_message(ClientMessage::GetScript(target_line_idx, target_frame_idx));
                                app.set_status_message(format!("Requested script Line {}, Frame {}", target_line_idx, target_frame_idx));
                            }
                            _ => unreachable!(),
                        }
                        return Ok(true); // Navigation handled (or attempted)
                    } else {
                        app.set_status_message("scene not loaded, cannot navigate.".to_string());
                        return Ok(true); // Handled (no-op)
                    }
                } // End Ctrl + Arrow case

                // Let other Ctrl combinations fall through to the default handler
                _ => { /* Do nothing here, let it fall through */ }
            }
        }

        // --- Handle all other key events (including non-Ctrl) by passing to textarea ---
        let handled_by_textarea = app.editor.textarea.input(key_event);
        Ok(handled_by_textarea)
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let line_idx = app.editor.active_line.line_index;
        let frame_idx = app.editor.active_line.frame_index;

        // Get frame status and length, with default values if not found
        let (status_str, length_str, is_enabled) = 
            if let Some(scene) = &app.editor.scene {
                if let Some(line) = scene.lines.get(line_idx) {
                    if frame_idx < line.frames.len() {
                        let enabled = line.is_frame_enabled(frame_idx);
                        let length = line.frames[frame_idx];
                        ( if enabled { "Enabled" } else { "Disabled" },
                          format!("Len: {:.2}", length),
                          enabled
                        )
                    } else {
                        ("Invalid Frame", "Len: N/A".to_string(), true) // Default to enabled appearance if invalid
                    }
                } else {
                    ("Invalid Line", "Len: N/A".to_string(), true) // Default to enabled appearance if invalid
                }
            } else {
                ("No scene", "Len: N/A".to_string(), true) // Default to enabled appearance if no scene
            };

        // Determine border color based on frame status
        let border_color = if is_enabled { Color::White } else { Color::DarkGray };

        let editor_block = Block::default()
            .title(format!(
                " Editor (Line: {}, Frame: {} | {} | {}) ", 
                line_idx,
                frame_idx,
                status_str, // Show enabled/disabled status
                length_str  // Show length
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(border_color));

        frame.render_widget(editor_block.clone(), area);
        let inner_editor_area = editor_block.inner(area);

        let editor_text_area: Rect;
        let editor_help_area: Rect;
        let error_area: Option<Rect> = None; // Initialize error_area

        // Conditionally create layout based on compilation error
        if let Some(error_msg) = &app.editor.compilation_error {
            let editor_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),       // Editor Content
                    Constraint::Length(5),    // Error Panel (adjustable)
                    Constraint::Length(1),    // Help Text
                ])
                .split(inner_editor_area);
            editor_text_area = editor_chunks[0];
            let error_panel_area = editor_chunks[1]; // Assign error_area inside the if block
            editor_help_area = editor_chunks[2];

            // Render error panel
            let error_block = Block::default()
                .title(" Compilation Error ")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .style(Style::default().fg(Color::Red));
            let error_paragraph = Paragraph::new(error_msg.as_str())
                .wrap(ratatui::widgets::Wrap { trim: true })
                .block(error_block.clone());
            frame.render_widget(error_paragraph, error_panel_area);
            // render border separately to ensure it's drawn over content
            frame.render_widget(error_block, error_panel_area); 

        } else {
            // Layout without error panel
            let editor_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0), // Editor Content
                    Constraint::Length(1), // Help Text
                ])
                .split(inner_editor_area);
            editor_text_area = editor_chunks[0];
            editor_help_area = editor_chunks[1];
            // error_area remains None
        }

        let mut text_area = app.editor.textarea.clone();
        text_area.set_line_number_style(Style::default().fg(Color::DarkGray));
        frame.render_widget(&text_area, editor_text_area);

        // Indication des touches
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        let help_spans = vec![
            Span::styled("Ctrl+S", key_style), Span::styled(": Send Script | ", help_style),
            Span::styled("Ctrl+E", key_style), Span::styled(": Toggle Frame | ", help_style),
            Span::styled("Ctrl+Arrows", key_style), Span::styled(": Navigate | ", help_style),
            Span::styled("Standard Input", key_style), Span::styled(": Edit", help_style),
        ];
        let help = Paragraph::new(Line::from(help_spans))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help, editor_help_area);
    }
}
