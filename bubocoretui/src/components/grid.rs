use crate::App;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use std::cmp::min;
use std::collections::HashSet;
use bubocorelib::schedule::ActionTiming;
use bubocorelib::server::client::ClientMessage;
use bubocorelib::scene::Line as SceneLine;
use bubocorelib::shared_types::GridSelection;
use crate::components::logs::LogLevel;
use crate::app::{ClipboardState, ClipboardFrameData};
use tui_textarea::TextArea;
use std::str::FromStr;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::Line;

// Styles utilisés pour le rendu du tableau
struct GridCellStyles {
    enabled: Style,
    disabled: Style,
    cursor: Style,
    peer_cursor: Style,
    empty: Style,
    start_end_marker: Style,
}

// --- Add struct for render info ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridRenderInfo {
    pub visible_height: usize,
    pub max_frames: usize,
}

/// Component representing the scene grid, what is currently being played/edited
pub struct GridComponent;

// --- Refactor: Helper structure for layout areas ---
struct GridLayoutAreas {
    table_area: Rect,
    help_area: Rect,
    length_prompt_area: Rect,
    insert_prompt_area: Rect,
    name_prompt_area: Rect,
}

impl GridComponent {
    /// Creates a new [`GridComponent`] instance.
    pub fn new() -> Self {
        Self {}
    }

    fn cell_styles() -> GridCellStyles {
        GridCellStyles {
            enabled: Style::default().fg(Color::White).bg(Color::Green),
            disabled: Style::default().fg(Color::White).bg(Color::Red),
            cursor: Style::default().fg(Color::White).bg(Color::Yellow).bold(),
            peer_cursor: Style::default().bg(Color::White).fg(Color::Black),
            empty: Style::default().bg(Color::DarkGray),
            start_end_marker: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        }
    }

    // --- Refactor: Helper for rendering a grid cell ---
    fn render_grid_cell(
        &self,
        frame_idx: usize,
        col_idx: usize,
        line: Option<&SceneLine>,
        app: &App,
    ) -> Cell<'static> {
        let styles = Self::cell_styles();
        let bar_char_active = "▌";
        let bar_char_inactive = " ";

        if let Some(line) = line {
            if frame_idx < line.frames.len() {
                let frame_val = line.frames[frame_idx];
                // --- Use line.frame_names --- 
                let frame_name = line.frame_names.get(frame_idx).cloned().flatten();
                // -----------------------------
                let is_enabled = line.is_frame_enabled(frame_idx);
                let base_style = if is_enabled { styles.enabled } else { styles.disabled };
                let current_frame_for_line = app.server.current_frame_positions.as_ref()
                    .and_then(|positions| positions.get(col_idx))
                    .copied()
                    .unwrap_or(usize::MAX);
                let is_head_on_this_frame = current_frame_for_line == frame_idx;
                let play_marker = if is_head_on_this_frame { "▶" } else { " " };
                let play_marker_span = Span::raw(play_marker);
                let last_frame_index = line.frames.len().saturating_sub(1);
                let is_head_past_last_frame = current_frame_for_line == usize::MAX;
                let is_this_the_last_frame = frame_idx == last_frame_index;

                // --- Cell Content Logic ---
                let content_spans = if is_this_the_last_frame && is_head_past_last_frame {
                    vec![Span::raw("⏳")]
                } else {
                    let len_str = format!("[{:.2}]", frame_val);
                    if let Some(name) = frame_name {
                        let name_span = Span::raw(format!("{} ", name));
                        let len_span = Span::raw(len_str);
                        vec![name_span, len_span]
                    } else {
                        vec![Span::raw(len_str)]
                    }
                };
                let cell_base_style = if is_this_the_last_frame && is_head_past_last_frame {
                    base_style.dim()
                } else {
                    base_style
                };
                // --- End Cell Content Logic ---

                let ((top, left), (bottom, right)) = app.interface.components.grid_selection.bounds();
                let is_selected_locally = frame_idx >= top && frame_idx <= bottom && col_idx >= left && col_idx <= right;
                let is_local_cursor = (frame_idx, col_idx) == app.interface.components.grid_selection.cursor_pos();
                let peer_on_cell: Option<(String, GridSelection)> = app.server.peer_sessions.iter()
                    .filter_map(|(name, peer_state)| peer_state.grid_selection.map(|sel| (name.clone(), sel)))
                    .find(|(_, peer_selection)| (frame_idx, col_idx) == peer_selection.cursor_pos());
                let is_being_edited_by_peer = app.server.peer_sessions.values()
                    .any(|peer_state| peer_state.editing_frame == Some((col_idx, frame_idx)));
                let mut final_style;
                let mut final_content_spans = content_spans;

                if is_local_cursor || is_selected_locally {
                    final_style = styles.cursor;
                } else if let Some((peer_name, _)) = peer_on_cell {
                    final_style = styles.peer_cursor;
                    let name_fragment = peer_name.chars().take(4).collect::<String>();
                    final_content_spans = vec![Span::raw(format!("{:<4}", name_fragment))];
                } else {
                    final_style = cell_base_style;
                }
                if is_being_edited_by_peer && !(is_local_cursor || is_selected_locally) {
                    let phase = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() % 500;
                    let current_fg = final_style.fg.unwrap_or(Color::White);
                    let animated_fg = if phase < 250 { current_fg } else { Color::Red };
                    final_style = final_style.fg(animated_fg);
                }
                let should_draw_bar = if let Some(start) = line.start_frame {
                    if let Some(end) = line.end_frame { frame_idx >= start && frame_idx <= end }
                    else { frame_idx >= start }
                } else { if let Some(end) = line.end_frame { frame_idx <= end } else { false } };
                let bar_char = if should_draw_bar { bar_char_active } else { bar_char_inactive };
                let bar_span = Span::styled(bar_char, if should_draw_bar { styles.start_end_marker } else { Style::default() });
                let mut cell_line_spans = vec![bar_span, play_marker_span, Span::raw(" ")];
                cell_line_spans.extend(final_content_spans);

                // --- Alignment ---
                // Change alignment to Right
                let cell_content = Line::from(cell_line_spans).alignment(ratatui::layout::Alignment::Right);
                // --- ---

                Cell::from(cell_content).style(final_style)
            } else {
                // Empty cell in a valid line
                self.render_empty_grid_cell(frame_idx, col_idx, app, &styles)
            }
        } else {
            // Invalid line (should not happen)
            self.render_empty_grid_cell(frame_idx, col_idx, app, &styles)
        }
    }

    fn render_empty_grid_cell(
        &self,
        frame_idx: usize,
        col_idx: usize,
        app: &App,
        styles: &GridCellStyles,
    ) -> Cell<'static> {
        let mut final_style;
        let cell_content_span;
        let is_local_cursor = (frame_idx, col_idx) == app.interface.components.grid_selection.cursor_pos();
        let peer_on_cell: Option<(String, GridSelection)> = app.server.peer_sessions.iter()
            .filter_map(|(name, peer_state)| peer_state.grid_selection.map(|sel| (name.clone(), sel)))
            .find(|(_, peer_selection)| (frame_idx, col_idx) == peer_selection.cursor_pos());
        let is_being_edited_by_peer = app.server.peer_sessions.values()
            .any(|peer_state| peer_state.editing_frame == Some((col_idx, frame_idx)));
        if is_local_cursor {
            final_style = styles.cursor;
            cell_content_span = Span::raw("");
        } else if let Some((peer_name, _)) = peer_on_cell {
            final_style = styles.peer_cursor;
            let name_fragment = peer_name.chars().take(4).collect::<String>();
            cell_content_span = Span::raw(format!("{:<4}", name_fragment));
        } else {
            final_style = styles.empty;
            cell_content_span = Span::raw("");
        }
        if is_being_edited_by_peer && !is_local_cursor && cell_content_span.width() > 0 {
            let phase = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() % 500;
            let current_fg = final_style.fg.unwrap_or(Color::White);
            let animated_fg = if phase < 250 { current_fg } else { Color::Red };
            final_style = final_style.fg(animated_fg);
        }
        let cell_content = Line::from(cell_content_span).alignment(ratatui::layout::Alignment::Center);
        Cell::from(cell_content).style(final_style)
    }

    // --- Refactor: Handle key events when inserting frame duration ---
    fn handle_insert_duration_input(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        let mut is_active = app.interface.components.is_inserting_frame_duration;
        let mut textarea = app.interface.components.insert_duration_input.clone();
        let mut status_msg_to_set = None;
        let mut exit_mode = false;
        let mut handled_textarea = false;

        match key_event.code {
            KeyCode::Esc => {
                status_msg_to_set = Some("Frame insertion cancelled.".to_string());
                exit_mode = true;
            }
            KeyCode::Enter => {
                let input_str = textarea.lines()[0].trim();
                match f64::from_str(input_str) {
                    Ok(new_duration) if new_duration > 0.0 => {
                        let (row_idx, col_idx) = app.interface.components.grid_selection.cursor_pos();
                        let insert_pos = row_idx + 1;
                        app.send_client_message(ClientMessage::InsertFrame(
                            col_idx,
                            insert_pos,
                            new_duration,
                            ActionTiming::Immediate
                        ));
                        status_msg_to_set = Some(format!(
                            "Requested inserting frame with duration {:.2} at ({}, {})",
                            new_duration, col_idx, insert_pos
                        ));
                        exit_mode = true;
                    }
                    _ => {
                        let error_message = format!(
                            "Invalid duration: '{}'. Must be a positive number.", input_str
                        );
                        // Set bottom message directly
                        app.interface.components.bottom_message = error_message.clone();
                        app.interface.components.bottom_message_timestamp = Some(std::time::Instant::now());
                        status_msg_to_set = Some(error_message); // Also set status bar briefly
                        // Don't exit mode on error, allow user to correct
                    }
                }
            }
            _ => {
                handled_textarea = textarea.input(key_event);
            }
        }

        if let Some(msg) = status_msg_to_set {
            app.set_status_message(msg);
        }

        if exit_mode {
            is_active = false;
            textarea = TextArea::default();
        }

        // Update app state
        app.interface.components.is_inserting_frame_duration = is_active;
        app.interface.components.insert_duration_input = textarea;

        Ok(exit_mode || handled_textarea)
    }

    // --- Refactor: Handle key events when setting frame length ---
    fn handle_set_length_input(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        let mut is_active = app.interface.components.is_setting_frame_length;
        let mut textarea = app.interface.components.frame_length_input.clone();
        let mut status_msg_to_set = None;
        let mut exit_mode = false;
        let mut handled_textarea = false;

        match key_event.code {
            KeyCode::Esc => {
                status_msg_to_set = Some("Frame length setting cancelled.".to_string());
                exit_mode = true;
            }
            KeyCode::Enter => {
                let input_str = textarea.lines()[0].trim();
                // Need scene access here
                if let Some(scene) = app.editor.scene.as_ref() {
                    let current_selection = app.interface.components.grid_selection;
                    match input_str.parse::<f64>() {
                        Ok(new_length) if new_length > 0.0 => {
                            let ((top, left), (bottom, right)) = current_selection.bounds();
                            let mut modified_lines: std::collections::HashMap<usize, Vec<f64>> = std::collections::HashMap::new();
                            let mut frames_changed = 0;
                            for col_idx in left..=right {
                                if let Some(line) = scene.lines.get(col_idx) {
                                    let mut current_frames = line.frames.clone();
                                    let mut was_modified = false;
                                    for row_idx in top..=bottom {
                                        if row_idx < current_frames.len() {
                                            current_frames[row_idx] = new_length;
                                            was_modified = true;
                                            frames_changed += 1;
                                        }
                                    }
                                    if was_modified {
                                        modified_lines.insert(col_idx, current_frames);
                                    }
                                }
                            }
                            for (col, updated_frames) in modified_lines {
                                app.send_client_message(ClientMessage::UpdateLineFrames(
                                    col, updated_frames, ActionTiming::Immediate
                                ));
                            }
                            if frames_changed > 0 {
                                status_msg_to_set = Some(format!(
                                    "Set length to {:.2} for {} frame(s)", new_length, frames_changed
                                ));
                            } else {
                                status_msg_to_set = Some("No valid frames in selection to set length".to_string());
                            }
                            exit_mode = true;
                        }
                        _ => {
                            let error_message = format!(
                                "Invalid frame length: '{}'. Must be positive number.", input_str
                            );
                            app.interface.components.bottom_message = error_message.clone();
                            app.interface.components.bottom_message_timestamp = Some(std::time::Instant::now());
                            status_msg_to_set = Some(error_message);
                            // Don't exit on error
                        }
                    }
                } else {
                    status_msg_to_set = Some("Error: Scene not loaded while setting frame length.".to_string());
                    exit_mode = true; // Exit if scene isn't loaded
                }
            }
            _ => {
                handled_textarea = textarea.input(key_event);
            }
        }

        if let Some(msg) = status_msg_to_set {
            app.set_status_message(msg);
        }

        if exit_mode {
            is_active = false;
            textarea = TextArea::default();
        }

        // Update app state
        app.interface.components.is_setting_frame_length = is_active;
        app.interface.components.frame_length_input = textarea;

        Ok(exit_mode || handled_textarea)
    }

    // --- Add Handler for Frame Name Input ---
    fn handle_set_name_input(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        let mut is_active = app.interface.components.is_setting_frame_name;
        let mut textarea = app.interface.components.frame_name_input.clone();
        let mut status_msg_to_set = None;
        let mut exit_mode = false;
        let mut handled_textarea = false;

        match key_event.code {
            KeyCode::Esc => {
                status_msg_to_set = Some("Frame naming cancelled.".to_string());
                exit_mode = true;
            }
            KeyCode::Enter => {
                let input_name = textarea.lines()[0].trim().to_string();
                let (row_idx, col_idx) = app.interface.components.grid_selection.cursor_pos();

                // Send message to server, None if input is empty
                let name_to_send = if input_name.is_empty() { None } else { Some(input_name.clone()) };
                app.send_client_message(ClientMessage::SetFrameName(
                    col_idx,
                    row_idx,
                    name_to_send.clone(),
                    ActionTiming::Immediate
                ));

                status_msg_to_set = if let Some(name) = name_to_send {
                     Some(format!("Requested setting name to '{}' for frame ({}, {})", name, col_idx, row_idx))
                } else {
                     Some(format!("Requested clearing name for frame ({}, {})", col_idx, row_idx))
                };
                exit_mode = true;
            }
            _ => {
                handled_textarea = textarea.input(key_event);
            }
        }

        if let Some(msg) = status_msg_to_set {
            app.set_status_message(msg);
        }

        if exit_mode {
            is_active = false;
            textarea = TextArea::default();
        }

        // Update app state
        app.interface.components.is_setting_frame_name = is_active;
        app.interface.components.frame_name_input = textarea;

        Ok(exit_mode || handled_textarea)
    }

    pub fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        // Get scene data, but don't exit immediately if empty
        let scene_opt = app.editor.scene.as_ref();
        let num_cols = scene_opt.map_or(0, |p| p.lines.len());

        // --- Handle Frame Name Input Mode First ---
        if app.interface.components.is_setting_frame_name {
             return self.handle_set_name_input(app, key_event);
        }

        // --- Handle Frame Duration Input Mode ---
        if app.interface.components.is_inserting_frame_duration {
            return self.handle_insert_duration_input(app, key_event);
        }

        // --- Handle Frame Length Input Mode ---
        if app.interface.components.is_setting_frame_length {
            return self.handle_set_length_input(app, key_event);
        }

        // Handle 'a' regardless of whether lines exist
        if key_event.code == KeyCode::Char('A') && key_event.modifiers.contains(KeyModifiers::SHIFT) { // Shift+A adds line
            // Send the request to add a line; the server will create the default one.
            app.send_client_message(ClientMessage::SchedulerControl(
                bubocorelib::schedule::SchedulerMessage::AddLine
            ));
            app.set_status_message("Requested adding line".to_string());
            return Ok(true);
        }

        // --- For other keys, require a scene and at least one line ---
        let scene = match scene_opt {
            Some(p) if num_cols > 0 => p,
            _ => { return Ok(false); }
        };

        // Get the current selection
        let initial_selection = app.interface.components.grid_selection; // Store initial
        let mut current_selection = initial_selection; // Work with mutable copy
        let mut handled = true;

        // Extract shift modifier for easier checking
        let is_shift_pressed = key_event.modifiers.contains(KeyModifiers::SHIFT);

        // --- Retrieve render info and current scroll offset for key handling ---
        let render_info = app.interface.components.last_grid_render_info;
        let mut current_scroll_offset_val = app.interface.components.grid_scroll_offset;

        // --- Normal Grid Key Handling ---
        match key_event.code {
            // Reset selection to single cell at the selection's start position
            KeyCode::Esc => {
                if !current_selection.is_single() {
                    current_selection = GridSelection::single(current_selection.start.0, current_selection.start.1);
                    app.set_status_message("Selection reset to single cell (at start)".to_string());
                } else {
                    handled = false; // Esc doesn't do anything if selection is already single
                }
            }
            // Request the script for the selected frame form the server and edit it
            KeyCode::Enter => {
                let cursor_pos = current_selection.cursor_pos();
                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                let (row_idx, col_idx) = cursor_pos;
                let status_update: Option<String>;
                if let Some(scene) = &app.editor.scene {
                    if let Some(line) = scene.lines.get(col_idx) {
                        if row_idx < line.frames.len() {
                            // Send request to server for the script content
                            app.send_client_message(ClientMessage::GetScript(col_idx, row_idx));
                            // Also notify server that we START editing this frame
                            app.send_client_message(ClientMessage::StartedEditingFrame(col_idx, row_idx));
                            status_update = Some(format!("Requested script for Line {}, Frame {}", col_idx, row_idx));
                        } else {
                            status_update = Some("Cannot request script for an empty slot".to_string());
                            handled = false;
                        }
                    } else {
                        status_update = Some("Invalid line index".to_string());
                        handled = false;
                    }
                } else {
                    status_update = Some("Scene not loaded".to_string());
                    handled = false;
                }

                if let Some(status) = status_update { app.set_status_message(status); }
                // Note: We don't switch to the editor here. We wait for the server response.
            }
            // Set frame length via prompt
            KeyCode::Char('l') => {
                let ((top, left), (bottom, right)) = current_selection.bounds();
                let mut first_frame_length: Option<f64> = None;
                let mut can_set = false;

                // Check if selection contains at least one valid frame
                for col_idx in left..=right {
                     if let Some(line) = scene.lines.get(col_idx) {
                         for row_idx in top..=bottom {
                             if row_idx < line.frames.len() {
                                 can_set = true;
                                 if first_frame_length.is_none() {
                                    first_frame_length = Some(line.frames[row_idx]);
                                 }
                                 break; // Found one, no need to check further in this line
                             }
                         }
                     }
                     if can_set { break; } // Found one, no need to check other lines
                }

                if can_set {
                    app.interface.components.is_setting_frame_length = true;
                    // Pre-fill with the length of the first selected frame, or empty if none
                    let initial_text = first_frame_length.map_or(String::new(), |len| format!("{:.2}", len));
                    app.interface.components.frame_length_input = TextArea::new(vec![initial_text]);
                    app.set_status_message("Enter new frame length (e.g., 1.5):".to_string());
                } else {
                    app.set_status_message("Cannot set length: selection contains no frames.".to_string());
                    handled = false;
                }
            }
            // Set the start frame of the line
            KeyCode::Char('b') => {
                 let cursor_pos = current_selection.cursor_pos();
                 current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                 let (row_idx, col_idx) = cursor_pos;
                 if let Some(line) = scene.lines.get(col_idx) {
                     if row_idx < line.frames.len() {
                         let start_frame_val = if line.start_frame == Some(row_idx) { None } else { Some(row_idx) };
                         app.send_client_message(
                            ClientMessage::SetLineStartFrame(
                                col_idx, start_frame_val,
                                ActionTiming::Immediate)
                            );
                         app.set_status_message(format!("Requested setting start frame to {:?} for Line {}", start_frame_val, col_idx));
                     } else {
                         app.set_status_message("Cannot set start frame on empty slot".to_string());
                         handled = false;
                     }
                 } else { handled = false; }
            }
            KeyCode::Char('e') => {
                 let cursor_pos = current_selection.cursor_pos();
                 current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                 let (row_idx, col_idx) = cursor_pos;
                 if let Some(line) = scene.lines.get(col_idx) {
                     if row_idx < line.frames.len() {
                         let end_frame_val = if line.end_frame == Some(row_idx) { None } else { Some(row_idx) };
                         app.send_client_message(
                            ClientMessage::SetLineEndFrame(
                                col_idx, end_frame_val,
                                ActionTiming::Immediate)
                            );
                         app.set_status_message(format!("Requested setting end frame to {:?} for Line {}", end_frame_val, col_idx));
                     } else {
                         app.set_status_message("Cannot set end frame on empty slot".to_string());
                         handled = false;
                     }
                 } else { handled = false; }
            }
            // Down arrow key: Move the cursor one frame down (if shift is pressed, extend the selection)
            KeyCode::Down |
            // Up arrow key: Move the cursor one frame up (if shift is pressed, decrease the selection)
            KeyCode::Up |
            // Left arrow key: Move the cursor one column to the left (if shift is pressed, decrease the selection)
            KeyCode::Left |
            // Right arrow key: Move the cursor one column to the right (if shift is pressed, increase the selection)
            KeyCode::Right => {
                let (next_selection, changed) = self.calculate_next_selection(
                    current_selection,
                    key_event.code, // Pass the specific arrow key code
                    is_shift_pressed,
                    scene, // Pass the scene reference
                    num_cols,
                );
                if changed {
                    current_selection = next_selection;
                } else {
                    handled = false; // Indicate no effective movement occurred
                }
            }
            // Enable / Disable frames
            KeyCode::Char(' ') => {
        let ((top, left), (bottom, right)) = current_selection.bounds();
                 let mut to_enable: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
                 let mut to_disable: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
                 let mut frames_toggled = 0;

        for col_idx in left..=right {
            if let Some(line) = scene.lines.get(col_idx) {
                for row_idx in top..=bottom {
                    if row_idx < line.frames.len() {
                                 let is_enabled = line.is_frame_enabled(row_idx);
                                 if is_enabled {
                                     to_disable.entry(col_idx).or_default().push(row_idx);
                                 } else {
                                     to_enable.entry(col_idx).or_default().push(row_idx);
                                 }
                                 frames_toggled += 1;
                             }
                         }
                     }
                 }

                 // Send messages
                 for (col, rows) in to_disable {
                     if !rows.is_empty() {
                        app.send_client_message(ClientMessage::DisableFrames(col, rows, ActionTiming::Immediate));
                    }
                 }
                 for (col, rows) in to_enable {
                     if !rows.is_empty() {
                        app.send_client_message(ClientMessage::EnableFrames(col, rows, ActionTiming::Immediate));
                    }
                 }

                 if frames_toggled > 0 {
                     app.set_status_message(format!("Requested toggling {} frames", frames_toggled));
        } else {
                     app.set_status_message("No valid frames in selection to toggle".to_string());
                     handled = false;
                 }
            }
            // Remove the last frame from the line
            KeyCode::Char('D') if is_shift_pressed => { // Shift+D removes last line
                let cursor_pos = current_selection.cursor_pos();
                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                let mut last_line_index_opt : Option<usize> = None;

                if let Some(scene) = &app.editor.scene {
                     if scene.lines.len() > 0 {
                        let last_line_index = scene.lines.len() - 1;
                        last_line_index_opt = Some(last_line_index);
                    } else {
                         app.set_status_message("No lines to remove".to_string());
                         handled = false;
                    }
                } else {
                    app.set_status_message("Scene not loaded".to_string());
                    handled = false;
                }

                if handled {
                     if let Some(last_line_index) = last_line_index_opt {
                        app.send_client_message(ClientMessage::SchedulerControl(
                            bubocorelib::schedule::SchedulerMessage::RemoveLine(last_line_index, ActionTiming::Immediate)
                        ));
                        app.set_status_message(format!("Requested removing line {}", last_line_index));
                    }
                }

            }
            // --- Copy Action --- 
            KeyCode::Char('c') => {
                match self.handle_copy_action(current_selection, scene) {
                    Ok((new_clipboard_state, status_msg, messages_to_send)) => {
                        app.clipboard = new_clipboard_state;
                        app.set_status_message(status_msg);
                        for msg in messages_to_send {
                            app.send_client_message(msg);
                        }
                        handled = true;
                    }
                    Err(status_msg) => {
                        app.set_status_message(status_msg);
                        app.clipboard = ClipboardState::Empty;
                        handled = false;
                    }
                }
            }
            // --- Paste Action ---
            KeyCode::Char('p') => {
                handled = self.handle_paste_action(app, &mut current_selection);
            }
            // --- Duplicate Frame Before Cursor ---
            KeyCode::Char('a') => { // 'a' duplicates (insert before)
                handled = self.handle_duplicate_action(app, current_selection, true);
            }
            // --- Insert Frame After Cursor (with Duration Prompt) ---
            KeyCode::Char('i') => {
                let (row_idx, col_idx) = current_selection.cursor_pos();
                // Make selection single cell *before* entering input mode
                current_selection = GridSelection::single(row_idx, col_idx);
                let insert_pos = row_idx + 1;

                // Check if line exists and if insertion is valid
                if let Some(line) = scene.lines.get(col_idx) {
                    // Check if the insert position is valid (can be equal to len for appending)
                    if insert_pos <= line.frames.len() {
                        // Enter the insert duration mode
                        app.interface.components.is_inserting_frame_duration = true;
                        // Pre-fill with default duration "1.0"
                        let initial_text = "1.0".to_string();
                        app.interface.components.insert_duration_input = TextArea::new(vec![initial_text]);
                        app.set_status_message("Enter duration for new frame (default 1.0):".to_string());
                        handled = true;
            } else {
                        app.add_log(LogLevel::Warn, format!("Cannot insert frame at invalid position {} in line {}", insert_pos, col_idx));
                        app.set_status_message("Cannot insert frame here (beyond end + 1)".to_string());
                        handled = false;
            }
        } else {
                    // Only allow insertion if the line *exists* (col_idx is valid)
                    // Inserting into a non-existent line doesn't make sense here.
                    // AddLine should be used first.
                    app.set_status_message("Cannot insert frame: Line does not exist.".to_string());
                    handled = false;
                }
            }
            KeyCode::Char('d') => {
                handled = self.handle_duplicate_action(app, current_selection, false);
            }
            // --- Delete Selected Frame(s) ---
            KeyCode::Delete | KeyCode::Backspace => {
                handled = self.handle_delete_action(app, &mut current_selection);
            }
            KeyCode::PageDown => {
                if let Some(info) = render_info {
                    if info.visible_height > 0 && info.max_frames > info.visible_height {
                        let page_size = info.visible_height.saturating_sub(1).max(1);
                        let max_scroll = info.max_frames.saturating_sub(info.visible_height);
                        current_scroll_offset_val = (current_scroll_offset_val + page_size).min(max_scroll);

                        // Move cursor to the top of the new page (relative to current column)
                        let current_col = current_selection.cursor_pos().1;
                        let new_row = current_scroll_offset_val;
                        // Clamp row based on actual frames in target column
                        let frames_in_col = scene.lines.get(current_col).map_or(0, |l| l.frames.len());
                        let clamped_row = new_row.min(frames_in_col.saturating_sub(1));
                        current_selection = GridSelection::single(clamped_row, current_col);
                        // Handled is true by default
                    } else { handled = false; } // Cannot scroll if no overflow or no visible height
                } else { handled = false; } // Cannot scroll if render info is missing
            }
            KeyCode::PageUp => {
                if let Some(info) = render_info {
                    if info.visible_height > 0 {
                        let page_size = info.visible_height.saturating_sub(1).max(1);
                        current_scroll_offset_val = current_scroll_offset_val.saturating_sub(page_size);

                        // Move cursor to the top of the new page
                        let current_col = current_selection.cursor_pos().1;
                        let new_row = current_scroll_offset_val;
                        // Clamp row based on actual frames in target column
                        let frames_in_col = scene.lines.get(current_col).map_or(0, |l| l.frames.len());
                        let clamped_row = new_row.min(frames_in_col.saturating_sub(1));
                        current_selection = GridSelection::single(clamped_row, current_col);
                        // Handled is true by default
                    } else { handled = false; } // Cannot scroll if no visible height
                } else { handled = false; } // Cannot scroll if render info is missing
            }
            // Set frame name via prompt
            KeyCode::Char('n') => {
                // Ensure selection is single cell *before* entering mode
                let cursor_pos = current_selection.cursor_pos();
                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                let (row_idx, col_idx) = cursor_pos;

                if let Some(line) = scene.lines.get(col_idx) {
                    if row_idx < line.frames.len() {
                        // Get existing name to pre-fill input
                        let existing_name = line.frame_names.get(row_idx).cloned().flatten().unwrap_or_default();

                        app.interface.components.is_setting_frame_name = true;
                        app.interface.components.frame_name_input = TextArea::new(vec![existing_name]);
                        app.set_status_message("Enter new frame name (empty clears):".to_string());
                        handled = true;
                    } else {
                        app.set_status_message("Cannot name an empty frame slot.".to_string());
                        handled = false;
                    }
                } else {
                    app.set_status_message("Cannot name frame: Invalid line.".to_string());
                    handled = false;
                }
            }
            _ => { handled = false; } 
        }

        // Start with the offset potentially modified by PageUp/Down
        let mut final_scroll_offset = current_scroll_offset_val;

        // --- Adjust scroll based on final cursor position --- 
        // This ensures that after any cursor movement (arrows, delete, paste etc.),
        // the view scrolls if the cursor is now outside the visible area.
        let final_cursor_row = current_selection.cursor_pos().0;
        if let Some(info) = render_info {
             let visible_height = info.visible_height;
             let max_frames = info.max_frames;
             let mut desired_offset = final_scroll_offset; // Start from potentially updated offset

             if final_cursor_row < desired_offset { // Cursor moved above visible area
                 desired_offset = final_cursor_row;
             } else if visible_height > 0 && final_cursor_row >= desired_offset + visible_height { // Cursor moved below visible area
                 desired_offset = final_cursor_row.saturating_sub(visible_height.saturating_sub(1));
             }

             // Clamp desired_offset 
             let max_scroll = max_frames.saturating_sub(visible_height);
             final_scroll_offset = desired_offset.min(max_scroll); 
        }

        let scroll_changed = final_scroll_offset != current_scroll_offset_val;

        // --- Final state update --- 
        let selection_changed = initial_selection != current_selection;

        if selection_changed || scroll_changed {
            // Update actual app state
            app.interface.components.grid_scroll_offset = final_scroll_offset;
            app.interface.components.grid_selection = current_selection; // Update selection state

            if selection_changed {
                app.send_client_message(ClientMessage::UpdateGridSelection(current_selection));
            }
        }

        // Return true if an action was handled OR if selection/scroll changed
        Ok(handled || selection_changed || scroll_changed)
    }

    /// Draws the line grid UI component.
    /// 
    /// # Arguments
    /// 
    /// * `app`: Immutable reference to the main application state (`App`).
    /// * `frame`: Mutable reference to the current terminal frame (`Frame`).
    /// * `area`: The `Rect` area allocated for this component to draw into.
    /// 
    /// # Returns
    /// 
    /// * `()`
    pub fn draw(&self, app: &mut App, frame: &mut Frame, area: Rect) {

        // Get the current scene length from the scene object
        let scene_length = app.editor.scene.as_ref().map_or(0, |s| s.length());

        // --- 1. Render Outer Block and Calculate Layout ---
        let layout_areas = match self.calculate_layout(app, area) {
             Some(areas) => areas,
             None => {
                 // Render a simple block even if area is too small, but nothing inside
                 let outer_block = Block::default().borders(Borders::ALL).title(" Grid ");
                 frame.render_widget(outer_block, area);
                 return;
             }
        };

        // --- Calculate max_frames (needed for outer block potentially) ---
        let max_frames = app.editor.scene.as_ref()
            .map_or(0, |s| s.lines.iter().map(|line| line.frames.len()).max().unwrap_or(0));

        // --- Calculate visible height ---
        let table_height = layout_areas.table_area.height as usize;
        let header_rows = 1;
        let padding_rows = 1;
        let visible_height = table_height.saturating_sub(header_rows + padding_rows);

        // --- Scrolling (Offset fixed to 0 for now, key handling deferred) ---
        // Read current offset and clamp based on current render info
        let max_scroll = max_frames.saturating_sub(visible_height);
        app.interface.components.grid_scroll_offset = app.interface.components.grid_scroll_offset.min(max_scroll);
        let scroll_offset = app.interface.components.grid_scroll_offset; // Use the potentially clamped value
        let render_info = GridRenderInfo { visible_height, max_frames }; // For title indicators
        // Store render info back into app state
        app.interface.components.last_grid_render_info = Some(render_info);

        // --- Render outer block (now separate) ---
        self.render_outer_block(frame, area, scene_length, scroll_offset, Some(render_info));

        // --- 2. Render Input Prompts ---
        self.render_input_prompts(app, frame, &layout_areas);

        // --- 3. Render Help Text ---
        self.render_help_text(frame, &layout_areas);

        // --- 4. Render Grid Table (or empty state) ---
        if let Some(scene) = &app.editor.scene {
             // Pass clamped scroll_offset and calculated visible_height
             self.render_grid_table(app, frame, &layout_areas, scene, scroll_offset, visible_height);
        } else {
             self.render_empty_state(frame, &layout_areas, "No scene loaded from server.");
             // Ensure render info is cleared if no scene
             app.interface.components.last_grid_render_info = None;
        }
    }

    // --- Refactor: Helper to calculate layout ---
    fn calculate_layout(
         &self,
         app: &App, // No longer needs mutable app
         area: Rect,
     ) -> Option<GridLayoutAreas> {
 
         // Need at least some space for borders + title + content (Thick border = 2 horiz, 2 vert)
         if area.width < 2 || area.height < 2 {
             return None;
         }

         // Calculate the actual inner area after accounting for Thick borders
         let inner_area = area.inner(Margin { vertical: 1, horizontal: 1 });

         // Check if inner area is valid for content
         if inner_area.width < 1 || inner_area.height < 1 { 
             return None;
         }

         // Determine heights based on which prompts are active
         let help_height = 3;
         let length_prompt_height = if app.interface.components.is_setting_frame_length { 3 } else { 0 };
         let insert_prompt_height = if app.interface.components.is_inserting_frame_duration { 3 } else { 0 };
         let name_prompt_height = if app.interface.components.is_setting_frame_name { 3 } else { 0 };
         let prompt_height = length_prompt_height + insert_prompt_height + name_prompt_height; // Total prompt height

         // Split inner area: Table takes remaining space, prompt(s), help text
         let main_chunks = Layout::default()
             .direction(Direction::Vertical)
             .constraints([
                 Constraint::Min(0), // Table area
                 Constraint::Length(prompt_height), // Combined Prompt area (0 if inactive)
                 Constraint::Length(help_height), // Help area
             ])
             .split(inner_area);

         let table_area = main_chunks[0];
         let prompt_area = main_chunks[1]; // This area now holds both prompts
         let help_area = main_chunks[2];

         // Split the prompt area if both prompts could potentially be active
         let prompt_layout = Layout::default()
             .direction(Direction::Vertical)
             .constraints([
                 Constraint::Length(length_prompt_height),
                 Constraint::Length(insert_prompt_height),
                 Constraint::Length(name_prompt_height),
             ])
             .split(prompt_area);

         let length_prompt_area = prompt_layout[0];
         let insert_prompt_area = prompt_layout[1];
         let name_prompt_area = prompt_layout[2];

         Some(GridLayoutAreas {
             table_area,
             help_area,
             length_prompt_area,
             insert_prompt_area,
             name_prompt_area,
         })
     }

    // --- Refactor: Helper to render the outer block with scroll indicators ---
    fn render_outer_block(
        &self,
        frame: &mut Frame,
        area: Rect,
        scene_length: usize,
        scroll_offset: usize, // Current offset
        render_info: Option<GridRenderInfo>, // Contains max_frames, visible_height
    ) {
        let mut title = format!(" Scene Grid (Length: {}) ", scene_length);
        if let Some(info) = render_info {
            if info.max_frames > info.visible_height {
                // Calculate max_scroll accurately here
                let max_scroll = info.max_frames.saturating_sub(info.visible_height);
                let scroll_perc = if max_scroll > 0 {
                    (scroll_offset * 100) / max_scroll
                } else { 0 };
                title = format!(
                    " Scene Grid L:{} F:{} {} {}{} {}% ", 
                    scene_length,                                                  // 1
                    info.max_frames,                                               // 2
                    if scroll_offset > 0 { '↑' } else { ' ' },                      // 3
                    if scroll_offset + info.visible_height < info.max_frames { '↓' } else { ' ' }, // 4
                    scroll_perc,                                                   // 5
                    "" // Need a 6th argument for the last placeholder, maybe scroll position like "(row {}/{})" later?
                ); 
            }
        }
        let outer_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::White));
        let inner_area = outer_block.inner(area);
        frame.render_widget(outer_block.clone(), area);

        // Need at least some space to draw anything inside
        if inner_area.width < 1 || inner_area.height < 2 {
            return;
        }
    }

    // --- Refactor: Helper to render input prompts ---
    fn render_input_prompts(&self, app: &App, frame: &mut Frame, layout: &GridLayoutAreas) {
         // Render input prompt for setting length if active
        if app.interface.components.is_setting_frame_length {
            let mut length_input_area = app.interface.components.frame_length_input.clone();
            length_input_area.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Set Frame Length (Enter: Confirm, Esc: Cancel) ")
                    .style(Style::default().fg(Color::Yellow))
            );
            length_input_area.set_style(Style::default().fg(Color::White));
            frame.render_widget(&length_input_area, layout.length_prompt_area);
        }

        // Render input prompt for inserting frame if active
        if app.interface.components.is_inserting_frame_duration {
            let mut insert_input_area = app.interface.components.insert_duration_input.clone();
            insert_input_area.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Insert Frame Duration (Enter: Confirm, Esc: Cancel) ")
                    .style(Style::default().fg(Color::Cyan)) // Different color for distinction
            );
            insert_input_area.set_style(Style::default().fg(Color::White));
            frame.render_widget(&insert_input_area, layout.insert_prompt_area);
        }

        // --- Render name input prompt ---
        if app.interface.components.is_setting_frame_name {
            let mut name_input_area = app.interface.components.frame_name_input.clone();
            name_input_area.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Set Frame Name (Enter: Confirm, Esc: Cancel) ")
                    .style(Style::default().fg(Color::Magenta)) // Different color
            );
            name_input_area.set_style(Style::default().fg(Color::White));
            frame.render_widget(&name_input_area, layout.name_prompt_area); // <-- Use name prompt area
        }
    }

    // --- Refactor: Helper to render help text ---
    fn render_help_text(&self, frame: &mut Frame, layout: &GridLayoutAreas) {
         let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);

        // Line 1
        let help_spans_line1 = vec![
            Span::raw("Move: "), Span::styled("↑↓←→ ", key_style),
            Span::raw(" | Select: "), Span::styled("Shift+↑↓←→ ", key_style),
            Span::raw(" | Edit: "), Span::styled("Enter ", key_style),
            Span::raw(" | En/Dis: "), Span::styled("Space ", key_style),
            Span::raw(" | Reset Sel: "), Span::styled("Esc ", key_style),
        ];

        // Line 2
        let help_spans_line2 = vec![
            Span::raw("Length: "), Span::styled("l ", key_style),
            Span::raw(" | Name: "), Span::styled("n ", key_style),
            Span::raw(" | Start/End: "), Span::styled("b", key_style), Span::raw("/"), Span::styled("e ", key_style),
            Span::raw(" | Ins Frame: "), Span::styled("i ", key_style),
            Span::raw(" | Del Frame: "), Span::styled("Del/Bksp ", key_style), 
        ];

        // Line 3
        let help_spans_line3 = vec![
            Span::raw("Dup Bef/Aft: "), Span::styled("a", key_style), Span::raw("/"), Span::styled("d ", key_style),
            Span::raw(" | Copy/Paste: "), Span::styled("c", key_style), Span::raw("/"), Span::styled("p ", key_style),
            Span::raw(" | Add/Rem Line: "), Span::styled("Shift+A", key_style), Span::raw("/ "), Span::styled("Shift+D", key_style),
        ];

        // Split the help area into three rows
        let help_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                 Constraint::Length(1),
                 Constraint::Length(1),
                 Constraint::Length(1)
            ])
            .split(layout.help_area);

        frame.render_widget(Paragraph::new(Line::from(help_spans_line1).style(help_style)).centered(), help_layout[0]);
        frame.render_widget(Paragraph::new(Line::from(help_spans_line2).style(help_style)).centered(), help_layout[1]);
        frame.render_widget(Paragraph::new(Line::from(help_spans_line3).style(help_style)).centered(), help_layout[2]);
    }

    // --- Refactor: Helper to render the grid table ---
    fn render_grid_table(
        &self,
        app: &App,
        frame: &mut Frame,
        layout: &GridLayoutAreas,
        scene: &bubocorelib::scene::Scene,
        scroll_offset: usize,
        visible_height: usize,
    ) {
        let lines = &scene.lines;
        let num_lines = lines.len();
        if num_lines == 0 {
            // This case should technically be handled by the caller checking scene.lines
            self.render_empty_state(frame, layout, "No lines in scene. Shift+A to add.");
            return;
        }

        let max_frames = lines.iter().map(|line| line.frames.len()).max().unwrap_or(0);

        // Use passed-in values
        let start_row = scroll_offset;
        let end_row = scroll_offset.saturating_add(visible_height);

        if max_frames == 0 && visible_height > 0 { // Check if space exists before showing this
            self.render_empty_state(frame, layout, "Lines have no frames. 'i' to insert.");
            // Still draw the header below even if no frames
        } else if visible_height == 0 {
            // Not enough space to draw even one data row
            self.render_empty_state(frame, layout, "Area too small for grid data");
             // Still draw header if possible
            if layout.table_area.height < 1 { return; } // Cannot even draw header
        }

        // Table Styles and Header
        let header_style = Style::default().fg(Color::White).bg(Color::Blue).bold();
        let header_cells = lines.iter().enumerate().map(|(i, line)| {
            let length_display = line.custom_length.map_or("(Scene)".to_string(), |len| format!("({:.1}b)", len));
            let speed_display = format!("x{:.1}", line.speed_factor);
            let text = format!("LINE {} {} {}", i + 1, length_display, speed_display);
            Cell::from(Line::from(text).alignment(ratatui::layout::Alignment::Center)).style(header_style)
        });
        let header = Row::new(header_cells).height(1).style(header_style);

        // Padding Row
        let padding_cells = std::iter::repeat(Cell::from("").style(Style::default())).take(num_lines);
        let padding_row = Row::new(padding_cells).height(1);

        // Data Rows - Iterate only over visible range
        let data_rows = (start_row..end_row.min(max_frames)) // Use calculated range
            .map(|frame_idx| {
                let cells = lines.iter().enumerate()
                   .map(|(col_idx, line)| self.render_grid_cell(frame_idx, col_idx, Some(line), app));
                Row::new(cells).height(1)
            });

        // Combine Rows
        let combined_rows = std::iter::once(padding_row).chain(data_rows);

        // Calculate Column Widths
        let col_width = if num_lines > 0 { layout.table_area.width / num_lines as u16 } else { layout.table_area.width };
        let widths: Vec<Constraint> = std::iter::repeat(Constraint::Min(col_width.max(6)))
            .take(num_lines)
            .collect();

        // Create and Render Table
        let table = Table::new(combined_rows, &widths)
            .header(header)
            .column_spacing(1);
        frame.render_widget(table, layout.table_area);
    }

    // --- Refactor: Helper to render empty/placeholder states ---
    fn render_empty_state(&self, frame: &mut Frame, layout: &GridLayoutAreas, message: &str) {
        frame.render_widget(
            Paragraph::new(message).yellow().centered(),
            layout.table_area, // Render the message in the table area
        );
    }

    fn calculate_next_selection(
        &self,
        current_selection: GridSelection,
        key_code: KeyCode,
        is_shift_pressed: bool,
        scene: &bubocorelib::scene::Scene,
        num_cols: usize,
    ) -> (GridSelection, bool) {
        let mut end_pos = current_selection.end;
        let mut changed = true; // Assume changed, set to false if no movement occurs

        match key_code {
            KeyCode::Down => {
                if let Some(line) = scene.lines.get(end_pos.1) {
                    let frames_in_col = line.frames.len();
                    if frames_in_col > 0 {
                        end_pos.0 = min(end_pos.0 + 1, frames_in_col - 1);
                    } else {
                        changed = false; // Cannot move down in empty line
                    }
                } else {
                    changed = false; // Column index invalid
                }
            }
            KeyCode::Up => {
                end_pos.0 = end_pos.0.saturating_sub(1);
            }
            KeyCode::Left => {
                let next_col = end_pos.1.saturating_sub(1);
                if next_col != end_pos.1 {
                    let frames_in_next_col = scene.lines.get(next_col).map_or(0, |s| s.frames.len());
                    end_pos.0 = min(end_pos.0, frames_in_next_col.saturating_sub(1));
                    end_pos.1 = next_col;
                } else {
                    changed = false;
                }
            }
            KeyCode::Right => {
                let next_col = min(end_pos.1 + 1, num_cols.saturating_sub(1));
                if next_col != end_pos.1 { // Check if column actually changed
                    let frames_in_next_col = scene.lines.get(next_col).map_or(0, |s| s.frames.len());
                    end_pos.0 = min(end_pos.0, frames_in_next_col.saturating_sub(1)); // Adjust row
                    end_pos.1 = next_col;
                } else {
                    changed = false;
                }
            }
            _ => { changed = false; } // Should not happen as we match specific keys
        }

        // Update selection based on shift state and if movement occurred
        let final_selection = if changed {
            if is_shift_pressed {
                // Modify end position of existing selection
                let mut modified_selection = current_selection;
                modified_selection.end = end_pos;
                modified_selection
            } else {
                // Move cursor (start and end are the same)
                GridSelection::single(end_pos.0, end_pos.1)
            }
        } else {
            current_selection // No change
        };

        // Check if the final selection state is actually different from the original
        let actually_changed = final_selection != current_selection;

        (final_selection, actually_changed)
    }

    fn handle_copy_action(
        &self,
        current_selection: GridSelection,
        scene: &bubocorelib::scene::Scene,
    ) -> Result<(ClipboardState, String, Vec<ClientMessage>), String> {
        let ((src_top, src_left), (src_bottom, src_right)) = current_selection.bounds();
        let mut collected_data: Vec<Vec<ClipboardFrameData>> = Vec::new();
        let mut pending_scripts = HashSet::new();
        let mut messages_to_send = Vec::new();
        let mut has_valid_frames = false;

        for col_idx in src_left..=src_right {
            let mut col_vec = Vec::new();
            let line_opt = scene.lines.get(col_idx);
            for row_idx in src_top..=src_bottom {
                let frame_data = if let Some(line) = line_opt {
                    if row_idx < line.frames.len() {
                        has_valid_frames = true;
                        // Only request script if not already requested (redundant? GetScript is idempotent)
                        if pending_scripts.insert((col_idx, row_idx)) {
                             messages_to_send.push(ClientMessage::GetScript(col_idx, row_idx));
                        }
                        ClipboardFrameData {
                            length: line.frames[row_idx],
                            is_enabled: line.is_frame_enabled(row_idx),
                            script_content: None, // Will be filled later
                            frame_name: line.frame_names.get(row_idx).cloned().flatten(),
                        }
                    } else {
                        // Empty slot in valid line
                        ClipboardFrameData::default()
                    }
                } else {
                    // Invalid line (column index out of bounds, shouldn't happen with bounds check?)
                    ClipboardFrameData::default()
                };
                col_vec.push(frame_data);
            }
            collected_data.push(col_vec);
        }

        if has_valid_frames {
            let new_clipboard_state = ClipboardState::FetchingScripts {
                pending: pending_scripts,
                collected_data,
                origin_top_left: (src_top, src_left),
            };
            let status_msg = format!(
                "Requesting scripts for copy [({}, {})..({}, {})]...",
                src_left, src_top, src_right, src_bottom
            );
            Ok((new_clipboard_state, status_msg, messages_to_send))
        } else {
            Err("Cannot copy: Selection contains no valid frames.".to_string())
        }
    }

    fn handle_paste_action(
        &mut self,
        app: &mut App,
        current_selection: &mut GridSelection, // Mutable to reset selection
    ) -> bool {
        let mut handled = false;
        match app.clipboard.clone() { // Clone to work with the value
            ClipboardState::ReadyMulti { data } => {
                let (target_row, target_col) = current_selection.cursor_pos();
                *current_selection = GridSelection::single(target_row, target_col); // Reset selection

                let num_cols_pasted = data.len();
                let num_rows_pasted = data.get(0).map_or(0, |col| col.len());

                if num_cols_pasted > 0 && num_rows_pasted > 0 {
                    // Convert TUI ClipboardFrameData to shared PastedFrameData
                    let paste_block_data = data.into_iter().map(|col|
                        col.into_iter().map(|frame| bubocorelib::shared_types::PastedFrameData {
                            length: frame.length,
                            is_enabled: frame.is_enabled,
                            script_content: frame.script_content,
                            name: frame.frame_name, // <-- Correct field name is 'name'
                        }).collect()
                    ).collect();

                    app.send_client_message(ClientMessage::PasteDataBlock {
                        data: paste_block_data,
                        target_row,
                        target_col,
                        timing: ActionTiming::Immediate,
                    });
                    app.set_status_message(format!(
                        "Requested pasting {}x{} block at ({}, {})...",
                        num_cols_pasted, num_rows_pasted, target_col, target_row
                    ));
                    app.clipboard = ClipboardState::Empty; // Clear clipboard after sending paste request
                    handled = true;
                } else {
                    app.set_status_message("Cannot paste empty clipboard data.".to_string());
                    handled = false;
                }
            }
            ClipboardState::FetchingScripts { pending, .. } => {
                app.set_status_message(format!("Still fetching {} scripts from server to copy...", pending.len()));
                handled = false;
            }
            ClipboardState::Empty => {
                app.set_status_message("Clipboard is empty. Use 'c' to copy first.".to_string());
                handled = false;
            }
        }
        handled
    }

    fn handle_duplicate_action(
        &mut self,
        app: &mut App,
        current_selection: GridSelection,
        insert_before: bool,
    ) -> bool {
        let ((top, left), (bottom, right)) = current_selection.bounds();

        let (target_cursor_row, target_cursor_col, desc) = if insert_before {
            (top, left, "before")
        } else {
            // Target after the selection. If selection spans multiple columns,
            // the target column remains the leftmost one (left).
            (bottom + 1, left, "after")
        };

        app.send_client_message(ClientMessage::RequestDuplicationData {
            src_top: top,
            src_left: left,
            src_bottom: bottom,
            src_right: right,
            target_cursor_row,
            target_cursor_col,
            insert_before,
            timing: ActionTiming::Immediate,
        });

        app.set_status_message(format!(
            "Requested duplication ({}) for selection [({}, {})..({}, {})]",
            desc, left, top, right, bottom
        ));

        // Note: Cursor position adjustment might be better handled
        // after receiving confirmation/update from the server.
        true // Assume handled (request sent)
    }

    fn handle_delete_action(
        &mut self,
        app: &mut App,
        current_selection: &mut GridSelection,
    ) -> bool {
        let mut handled_delete = false;
        let mut lines_and_indices_to_remove: Vec<(usize, Vec<usize>)> = Vec::new();
        let mut total_frames_deleted = 0;
        let ((top, left), (bottom, right)) = current_selection.bounds();
        // Determine potential cursor position after deletion (start with pos before top-left of selection)
        let mut final_cursor_pos = (top.saturating_sub(1), left);

        // --- Scope for immutable borrow of scene --- 
        let mut status_msg = "Cannot delete: Invalid state or no scene loaded".to_string(); // Default error
        let scene_available = app.editor.scene.is_some();

        if scene_available {
            let local_scene = app.editor.scene.as_ref().unwrap(); // Safe unwrap due to check above
            if local_scene.lines.is_empty() {
                status_msg = "Cannot delete: Scene has no lines".to_string();
            } else {
                for col_idx in left..=right {
                    if let Some(line) = local_scene.lines.get(col_idx) {
                        let line_len = line.frames.len();
                        if line_len == 0 { continue; } // Skip empty lines

                        // Determine effective rows to delete in this column
                        let row_start = top;
                        let row_end = bottom;

                        // Clamp deletion range to valid indices for this line
                        let effective_start = row_start;
                        let effective_end = row_end.min(line_len.saturating_sub(1)); // Use saturating_sub

                        if effective_start <= effective_end { 
                            let indices_in_col: Vec<usize> = (effective_start..=effective_end).collect();
                            if !indices_in_col.is_empty() {
                                let indices_count = indices_in_col.len();
                                total_frames_deleted += indices_count;
                                lines_and_indices_to_remove.push((col_idx, indices_in_col));
                                // Adjust potential final cursor based on the *first* column affected
                                if col_idx == left {
                                    // Try to place cursor at the start row of deletion, or the frame before if it was the first frame
                                    // Needs careful calculation relative to remaining frames
                                    // Simplified: Place cursor at row index *before* the deleted block start, 
                                    // clamped by the new potential length (line_len - indices_count)
                                    let new_len = line_len.saturating_sub(indices_count);
                                    let target_row = effective_start.min(new_len.saturating_sub(1)); // Aim for start, clamp if needed
                                    //let target_row = effective_start.saturating_sub(1).min(line_len.saturating_sub(indices_count + 1)); // Old calculation
                                    final_cursor_pos = (target_row, col_idx);
                                }
                            }
                        }
                        // If effective_start > effective_end, selection didn't overlap this line
                    } else {
                        status_msg = format!("Cannot delete: Invalid column index {}", col_idx);
                        lines_and_indices_to_remove.clear();
                        handled_delete = false; // Ensure we don't proceed
                        break;
                    }
                }

                // Update status based on collected indices
                if !lines_and_indices_to_remove.is_empty() {
                    status_msg = format!(
                        "Requested deleting {} frame(s) across {} line(s)",
                        total_frames_deleted, lines_and_indices_to_remove.len()
                    );
                    handled_delete = true;
                } else if handled_delete != false { // Only set this if we didn't hit an invalid col error
                    status_msg = "Cannot delete: Selection contains no valid frames.".to_string();
                    handled_delete = false;
                }
                // else: status_msg was set by invalid column error, handled_delete is false
            }
        } // --- End of immutable borrow scope ---

        // --- Perform actions requiring mutable app --- 
        if handled_delete {
            // Send the single multi-line message
            app.send_client_message(ClientMessage::RemoveFramesMultiLine {
                lines_and_indices: lines_and_indices_to_remove,
                timing: ActionTiming::Immediate,
            });
            app.set_status_message(status_msg.clone());
            app.add_log(LogLevel::Info, status_msg);

            // Adjust selection after deletion request
            *current_selection = GridSelection::single(final_cursor_pos.0, final_cursor_pos.1);
        } else {
            // Set the error status message determined earlier
            app.set_status_message(status_msg);
        }

        handled_delete // Return the final flag
    }
}