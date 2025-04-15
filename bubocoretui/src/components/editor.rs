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
use tui_textarea::{TextArea, Input, Key, CursorMove};

// Define the state for the search functionality
#[derive(Clone)] // Clone might be needed if App::editor gets cloned, adjust if not
pub struct SearchState {
    pub is_active: bool,
    pub query_textarea: TextArea<'static>,
    pub error_message: Option<String>,
}

impl SearchState {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search Query (Esc: Cancel, Enter: Find, ^N/↓: Next, ^P/↑: Prev) ")
        );
        // Ensure it doesn't allow multi-line input by default
        // Note: tui-textarea doesn't have a strict single-line mode, but
        // we prevent Enter from inserting newlines in the handler.
        Self {
            is_active: false,
            query_textarea: textarea,
            error_message: None,
        }
    }
}

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
        // --- Handle Search Mode --- 
        if app.editor.search_state.is_active {
            let search_state = &mut app.editor.search_state;
            let main_textarea = &mut app.editor.textarea;
            match key_event.into() { // Convert KeyEvent to tui_textarea::Input
                Input { key: Key::Esc, .. } => {
                    search_state.is_active = false;
                    search_state.error_message = None;
                    tui_textarea::TextArea::set_search_pattern(main_textarea, "").expect("Empty pattern should be valid"); // Clear search highlighting
                    // Clear the query input for next time
                    search_state.query_textarea.move_cursor(CursorMove::End);
                    search_state.query_textarea.delete_line_by_head();
                    app.set_status_message("Search cancelled.".to_string());
                    return Ok(true);
                }
                Input { key: Key::Enter, .. } => {
                    // Use the current pattern set in main_textarea
                    // Try to find the first match from the current cursor position
                    if !tui_textarea::TextArea::search_forward(main_textarea, true) {
                        search_state.error_message = Some("Pattern not found".to_string());
                    } else {
                        search_state.error_message = None; // Found
                    }
                    search_state.is_active = false;
                    // Keep the pattern for highlighting, but clear error/input
                    search_state.query_textarea.move_cursor(CursorMove::End);
                    search_state.query_textarea.delete_line_by_head(); 
                    app.set_status_message("Search closed.".to_string());
                    return Ok(true);
                }
                Input { key: Key::Char('n'), ctrl: true, .. } | Input { key: Key::Down, .. } => {
                    // Find next match, move cursor from current position
                    if !tui_textarea::TextArea::search_forward(main_textarea, false) {
                        search_state.error_message = Some("Pattern not found".to_string());
                    } else {
                        search_state.error_message = None;
                    }
                    return Ok(true);
                }
                Input { key: Key::Char('p'), ctrl: true, .. } | Input { key: Key::Up, .. } => {
                    // Find previous match, move cursor from current position
                    if !tui_textarea::TextArea::search_back(main_textarea, false) {
                        search_state.error_message = Some("Pattern not found".to_string());
                    } else {
                        search_state.error_message = None;
                    }
                    return Ok(true);
                }
                // Handle typing into the search box
                input => {
                     // Prevent Enter from adding newline in search box
                     if matches!(input, Input { key: Key::Enter, .. } | Input { key: Key::Char('m'), ctrl: true, ..}) {
                         return Ok(true); // Already handled Enter above, ignore Ctrl+M 
                     }

                    let modified = search_state.query_textarea.input(input);
                    if modified {
                        // Get current query from search textarea
                        let query = search_state.query_textarea.lines().get(0).map_or("", |s| s.as_str());
                        // Update the search pattern in the main textarea
                        match tui_textarea::TextArea::set_search_pattern(main_textarea, query) {
                            Ok(_) => search_state.error_message = None,
                            Err(e) => search_state.error_message = Some(e.to_string()),
                        }
                    }
                    return Ok(true); // Consumed input for search box
                }
            }
        }

        // --- Handle Esc separately to leave the editor (if not searching) --- 
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

        // Handle specific Ctrl combinations first
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

                // Activate Search with Ctrl+G
                KeyCode::Char('g') => {
                    app.editor.search_state.is_active = true;
                    app.editor.search_state.error_message = None; // Clear previous error
                    // Optionally pre-fill search box with selection or current word?
                    app.set_status_message("Search activated. Type query...".to_string());
                    return Ok(true);
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

        // Handle all other key events (including non-Ctrl) by passing to textarea 
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

        // Layout Definition 
        let editor_text_area: Rect; 
        let help_area: Rect;

        let search_active = app.editor.search_state.is_active;
        let compilation_error_present = app.editor.compilation_error.is_some();

        // Define constraints based on active panels
        let mut constraints = vec![Constraint::Min(0)]; // Editor content always present
        if search_active {
            constraints.push(Constraint::Length(3)); // Search box takes priority
        } else if compilation_error_present {
            constraints.push(Constraint::Length(5)); // Error panel if search not active
        }
        constraints.push(Constraint::Length(1)); // Help text always present

        let editor_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner_editor_area);

        // Assign areas based on layout
        editor_text_area = editor_chunks[0];
        let mut current_index = 1;
        if search_active || compilation_error_present { 
            let panel_area = editor_chunks[current_index];
            current_index += 1;

            if search_active {
                // --- Render Search Box --- 
                let search_state = &app.editor.search_state;
                let mut query_textarea = search_state.query_textarea.clone(); 
                if let Some(err_msg) = &search_state.error_message {
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title(format!(
                            " Search Query (Error: {}) (Esc: Cancel, Enter: Find, ^N/↓: Next, ^P/↑: Prev) ",
                            err_msg
                        ))
                        .style(Style::default().fg(Color::Red));
                    query_textarea.set_block(block);
                } // No need for else, default block is set in SearchState::new

                frame.render_widget(&query_textarea, panel_area); 
            } else {
                // --- Render Compilation Error Panel (only if search is not active) ---
                if let Some(error_msg) = &app.editor.compilation_error {
                    // Calculate line and column from character index
                    let mut error_line_num = 0;
                    let mut error_col_num = 0;
                    let mut char_idx_count = 0;
                    let editor_lines = app.editor.textarea.lines();

                    for (i, line) in editor_lines.iter().enumerate() {
                        let line_char_count = line.chars().count();
                        // Check if the 'from' index falls within this line (char indices)
                        if error_msg.from >= char_idx_count && error_msg.from < char_idx_count + line_char_count {
                            error_line_num = i;
                            error_col_num = error_msg.from - char_idx_count;
                            break;
                        }
                        // Add line length + 1 (for newline char) to cumulative count
                        char_idx_count += line_char_count + 1;
                        // If error index is exactly after the last char + newline, it's start of next line
                        if error_msg.from == char_idx_count {
                            error_line_num = i + 1;
                            error_col_num = 0;
                            break;
                        }
                    }

                    let error_block = Block::default()
                        .title(format!(
                            " Compilation Error ({}: Line {}, Col {}) ",
                            error_msg.lang, error_line_num + 1, error_col_num + 1
                        ))
                        .borders(Borders::ALL)
                        .border_type(BorderType::Plain)
                        .style(Style::default().fg(Color::Red));
                    let error_paragraph = Paragraph::new(error_msg.info.as_str())
                        .wrap(ratatui::widgets::Wrap { trim: true })
                        .block(error_block.clone());
                    frame.render_widget(error_paragraph, panel_area);
                    frame.render_widget(error_block, panel_area); // Render border over content
                }
            }
        }
        help_area = editor_chunks[current_index];
 
        let mut text_area = app.editor.textarea.clone();
        text_area.set_line_number_style(Style::default().fg(Color::DarkGray));

        // --- Render Main Editor --- 
        frame.render_widget(text_area.widget(), editor_text_area);
 
        // Indication des touches
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);

        // --- Render Help Text --- 
        let help_line = if search_active {
            Line::from(vec![
                Span::styled(" Esc ", key_style), Span::styled("Cancel | ", help_style),
                Span::styled(" Enter ", key_style), Span::styled("Find First & Close | ", help_style),
                Span::styled(" ^N/↓ ", key_style), Span::styled("Next Match | ", help_style),
                Span::styled(" ^P/↑ ", key_style), Span::styled("Prev Match", help_style),
            ])
        } else {
            Line::from(vec![
                Span::styled("Ctrl+S", key_style), Span::styled(": Send | ", help_style),
                Span::styled("Ctrl+E", key_style), Span::styled(": Toggle | ", help_style),
                Span::styled("Ctrl+G", key_style), Span::styled(": Search | ", help_style),
                Span::styled("Ctrl+Arrows", key_style), Span::styled(": Navigate Script | ", help_style),
                Span::styled("Edit", help_style),
            ])
        };

        let help = Paragraph::new(help_line)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
