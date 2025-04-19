use crate::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    prelude::{Rect, Constraint, Layout, Direction, Modifier},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell, BorderType},
};
use bubocorelib::schedule::ActionTiming;
use bubocorelib::server::client::ClientMessage;
use bubocorelib::shared_types::GridSelection;
use bubocorelib::scene::Line as SceneLine;
use std::cmp::min;
use std::collections::HashSet;
use crate::components::logs::LogLevel;
use crate::app::{ClipboardState, ClipboardFrameData};
use tui_textarea::TextArea;
use std::str::FromStr;

// Styles utilisés pour le rendu du tableau
struct GridCellStyles {
    enabled: Style,
    disabled: Style,
    cursor: Style,
    peer_cursor: Style,
    empty: Style,
    start_end_marker: Style,
}

/// Component representing the scene grid, what is currently being played/edited
pub struct GridComponent;

impl GridComponent {
    /// Creates a new [`GridComponent`] instance.
    pub fn new() -> Self {
        Self {}
    }

    // --- Refactor: Helpers for TextArea input modes ---
    fn handle_textarea_input(
        &self,
        textarea: &mut TextArea,
        key_event: KeyEvent,
        on_enter: impl Fn(&str) -> Option<String>,
        on_cancel: impl Fn() -> String,
    ) -> (bool, Option<String>, bool) {
        let mut exit_mode = false;
        let mut status_msg = None;
        let mut handled_textarea = false;
        match key_event.code {
            KeyCode::Esc => {
                status_msg = Some(on_cancel());
                exit_mode = true;
            }
            KeyCode::Enter => {
                let input_str = textarea.lines()[0].trim();
                status_msg = on_enter(input_str);
                if status_msg.is_some() {
                    exit_mode = true;
                }
            }
            _ => {
                handled_textarea = textarea.input(key_event);
            }
        }
        (exit_mode || handled_textarea, status_msg, exit_mode)
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
                let mut content_span;
                let cell_base_style;
                if is_this_the_last_frame && is_head_past_last_frame {
                    content_span = Span::raw("⏳");
                    cell_base_style = base_style.dim();
                } else {
                    content_span = Span::raw(format!("{:.2}", frame_val));
                    cell_base_style = base_style;
                }
                let ((top, left), (bottom, right)) = app.interface.components.grid_selection.bounds();
                let is_selected_locally = frame_idx >= top && frame_idx <= bottom && col_idx >= left && col_idx <= right;
                let is_local_cursor = (frame_idx, col_idx) == app.interface.components.grid_selection.cursor_pos();
                let peer_on_cell: Option<(String, GridSelection)> = app.server.peer_sessions.iter()
                    .filter_map(|(name, peer_state)| peer_state.grid_selection.map(|sel| (name.clone(), sel)))
                    .find(|(_, peer_selection)| (frame_idx, col_idx) == peer_selection.cursor_pos());
                let is_being_edited_by_peer = app.server.peer_sessions.values()
                    .any(|peer_state| peer_state.editing_frame == Some((col_idx, frame_idx)));
                let mut final_style;
                if is_local_cursor || is_selected_locally {
                    final_style = styles.cursor;
                } else if let Some((peer_name, _)) = peer_on_cell {
                    final_style = styles.peer_cursor;
                    let name_fragment = peer_name.chars().take(4).collect::<String>();
                    content_span = Span::raw(format!("{:<4}", name_fragment));
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
                let line_spans = vec![bar_span, play_marker_span, Span::raw(" "), content_span];
                let cell_content = Line::from(line_spans).alignment(ratatui::layout::Alignment::Center);
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
}

impl Component for GridComponent {


    /// Handles key events directed to the grid component.
    ///
    /// Handles:
    /// - Grid-specific actions:
    ///   - `+`: Sends a message to the server to add a default frame (length 1.0) to the current line.
    ///   - `-`: Sends a message to the server to remove the last frame from the current line.
    ///   - Arrow keys (`Up`, `Down`, `Left`, `Right`): Navigates the grid cursor.
    ///   - Shift + Arrow keys: Extend the selection range.
    ///   - `Space`: Sends a message to the server to toggle the enabled/disabled state of the selected frame.
    ///   - `Enter`: Sends a message to request the script for the selected frame and edit it.
    ///   - `l`: Set frame length via prompt.
    ///   - `b`: Mark selected frame as the line start.
    ///   - `e`: Mark selected frame as the line end.
    ///   - `a`: Add a new line.
    ///   - `d`: Remove the last line.
    ///   - `c`: Copy the selected cells to the clipboard.
    ///   - `p`: Paste cells from the clipboard to the grid.
    ///
    /// # Arguments
    ///
    /// * `app`: Mutable reference to the main application state (`App`).
    /// * `key_event`: The `KeyEvent` received from the terminal.
    ///
    /// # Returns
    /// 
    /// * `EyreResult<bool>` - Whether the key event was handled by this component.
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        // Get scene data, but don't exit immediately if empty
        let scene_opt = app.editor.scene.as_ref();
        let num_cols = scene_opt.map_or(0, |p| p.lines.len());
        let current_selection = app.interface.components.grid_selection; // Read current selection early

        // --- Handle Frame Duration Input Mode First ---
        if app.interface.components.is_inserting_frame_duration {
            let mut status_msg_to_set = None;
            let mut exit_insert_mode = false;
            let mut handled_textarea = false;

            match key_event.code {
                KeyCode::Esc => {
                    status_msg_to_set = Some("Frame insertion cancelled.".to_string());
                    exit_insert_mode = true;
                }
                KeyCode::Enter => {
                    let input_str = app.interface.components.insert_duration_input.lines()[0].trim();
                    match f64::from_str(input_str) {
                        Ok(new_duration) if new_duration > 0.0 => {
                            let (row_idx, col_idx) = current_selection.cursor_pos(); // Use the selection from *before* entering the mode
                            let insert_pos = row_idx + 1;

                            // Send the insertion request with the new duration
                            // NOTE: Assumes ClientMessage::InsertFrame now takes duration
                            // TEMPORARY FIX: Revert to old signature until ClientMessage is updated.
                            app.send_client_message(ClientMessage::InsertFrame(
                                col_idx,
                                insert_pos,
                                new_duration, // TODO: Add this back after updating ClientMessage
                                ActionTiming::Immediate
                            ));

                            status_msg_to_set = Some(format!(
                                "Requested inserting frame with duration {:.2} at ({}, {})",
                                new_duration, col_idx, insert_pos
                            ));
                            exit_insert_mode = true; // Exit on successful insertion
                        }
                        _ => { // Parsing failed or value <= 0.0
                            let error_message = format!(
                                "Invalid duration: '{}'. Must be a positive number.", input_str
                            );
                            app.interface.components.bottom_message = error_message.clone(); // Update immediately
                            app.interface.components.bottom_message_timestamp = Some(std::time::Instant::now());
                            status_msg_to_set = Some(error_message);
                            // Do not exit mode on invalid input
                        }
                    }
                }
                _ => { // Pass other inputs to the textarea
                    handled_textarea = app.interface.components.insert_duration_input.input(key_event);
                }
            }

            // --- Apply Actions After Input Handling ---
            if let Some(msg) = status_msg_to_set {
                app.set_status_message(msg);
            }
            if exit_insert_mode {
                app.interface.components.is_inserting_frame_duration = false;
                app.interface.components.insert_duration_input = TextArea::default(); // Clear the input field
            }
            return Ok(exit_insert_mode || handled_textarea); // Return true if handled
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
        let mut current_selection = app.interface.components.grid_selection;
        let mut handled = true;

        // Extract shift modifier for easier checking
        let is_shift_pressed = key_event.modifiers.contains(KeyModifiers::SHIFT);

        // --- Handle Frame Length Input Mode First ---
        if app.interface.components.is_setting_frame_length {
            let mut status_msg_to_set = None;
            let mut client_msg_to_send = None;
            let mut exit_length_mode = false;
            let mut handled_textarea = false;

            match key_event.code {
                KeyCode::Esc => {
                    status_msg_to_set = Some("Frame length setting cancelled.".to_string());
                    exit_length_mode = true;
                }
                KeyCode::Enter => {
                    let input_str = app.interface.components.frame_length_input.lines()[0].trim();
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
                                client_msg_to_send = Some(ClientMessage::UpdateLineFrames(
                                    col, updated_frames, ActionTiming::Immediate
                                ));
                                // Note: This sends multiple messages if multiple lines selected.
                                // Consider batching if necessary, but server handles individual line updates.
                                app.send_client_message(client_msg_to_send.clone().unwrap());
                            }

                            if frames_changed > 0 {
                                status_msg_to_set = Some(format!("Set length to {:.2} for {} frame(s)", new_length, frames_changed));
                            } else {
                                status_msg_to_set = Some("No valid frames in selection to set length".to_string());
                            }
                        }
                        _ => { // Parsing failed or value <= 0.0
                            let error_message = format!("Invalid frame length: '{}'. Must be positive number.", input_str);
                            app.interface.components.bottom_message = error_message.clone(); // Update immediately
                            app.interface.components.bottom_message_timestamp = Some(std::time::Instant::now());
                            status_msg_to_set = Some(error_message);
                            // Do not exit mode on invalid input
                        }
                    }
                    if client_msg_to_send.is_some() { // Exit only on success
                       exit_length_mode = true;
                    }
                }
                _ => { // Pass other inputs to the textarea
                    handled_textarea = app.interface.components.frame_length_input.input(key_event);
                }
            }

            // --- Apply Actions After Input Handling ---
            if let Some(msg) = status_msg_to_set {
                app.set_status_message(msg);
            }
            // Client messages are sent inside the Enter handler for now.
            if exit_length_mode {
                app.interface.components.is_setting_frame_length = false;
                app.interface.components.frame_length_input = TextArea::default();
            }
            return Ok(exit_length_mode || handled_textarea);
        }

        // --- Normal Grid Key Handling ---
        match key_event.code {
            // Reset selection to single cell at the selection's start position
            KeyCode::Esc => {
                if !current_selection.is_single() {
                    current_selection = GridSelection::single(current_selection.start.0, current_selection.start.1);
                    app.set_status_message("Selection reset to single cell (at start)".to_string());
                } else {
                    handled = false;
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
            KeyCode::Down => {
                let mut end_pos = current_selection.end;
                if let Some(line) = scene.lines.get(end_pos.1) {
                    let frames_in_col = line.frames.len();
                    if frames_in_col > 0 {
                        end_pos.0 = min(end_pos.0 + 1, frames_in_col - 1);
                    }
                }
                if is_shift_pressed {
                     current_selection.end = end_pos;
                 } else {
                     current_selection = GridSelection::single(end_pos.0, end_pos.1);
                 }
            }
            // Up arrow key: Move the cursor one frame up (if shift is pressed, decrease the selection)
            KeyCode::Up => {
                let mut end_pos = current_selection.end;
                end_pos.0 = end_pos.0.saturating_sub(1);
                 if is_shift_pressed {
                     current_selection.end = end_pos;
                 } else {
                     current_selection = GridSelection::single(end_pos.0, end_pos.1);
                 }
            }
            // Left arrow key: Move the cursor one column to the left (if shift is pressed, decrease the selection)
            KeyCode::Left => {
                let mut end_pos = current_selection.end;
                let next_col = end_pos.1.saturating_sub(1);
                if next_col != end_pos.1 {
                     let frames_in_next_col = scene.lines.get(next_col).map_or(0, |s| s.frames.len());
                     end_pos.0 = min(end_pos.0, frames_in_next_col.saturating_sub(1));
                     end_pos.1 = next_col;

                     if is_shift_pressed {
                         current_selection.end = end_pos;
                     } else {
                         current_selection = GridSelection::single(end_pos.0, end_pos.1);
                     }
                 } else {
                     handled = false; 
                 }
            }
            // Right arrow key: Move the cursor one column to the right (if shift is pressed, increase the selection)
            KeyCode::Right => {
                let mut end_pos = current_selection.end;
                let next_col = min(end_pos.1 + 1, num_cols.saturating_sub(1)); // Ensure not out of bounds
                 if next_col != end_pos.1 { // Check if column actually changed
                     let frames_in_next_col = scene.lines.get(next_col).map_or(0, |s| s.frames.len());
                     end_pos.0 = min(end_pos.0, frames_in_next_col.saturating_sub(1)); // Adjust row
                     end_pos.1 = next_col;

                     if is_shift_pressed {
                         current_selection.end = end_pos;
                     } else {
                         current_selection = GridSelection::single(end_pos.0, end_pos.1);
                     }
                 } else {
                     handled = false; 
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
            KeyCode::Char('D') if key_event.modifiers.contains(KeyModifiers::SHIFT) => { // Shift+D removes last line
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
            // --- Copy SINGLE Cell Script Info ---
            KeyCode::Char('c') => {
                 // --- Refactored Copy Logic --- 
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
                                 messages_to_send.push(ClientMessage::GetScript(col_idx, row_idx));
                                 pending_scripts.insert((col_idx, row_idx));
                                 ClipboardFrameData {
                                     length: line.frames[row_idx],
                                     is_enabled: line.is_frame_enabled(row_idx),
                                     script_content: None,
                                 }
                             } else {
                                 // Empty slot in valid line
                                 ClipboardFrameData { length: 0.0, is_enabled: false, script_content: None }
                             }
                         } else {
                             // Invalid line
                             ClipboardFrameData { length: 0.0, is_enabled: false, script_content: None }
                         };
                         col_vec.push(frame_data);
                     }
                     collected_data.push(col_vec);
                 }

                 if has_valid_frames {
                     for msg in messages_to_send {
                         app.send_client_message(msg);
                     }
                     app.clipboard = ClipboardState::FetchingScripts {
                         pending: pending_scripts,
                         collected_data,
                         origin_top_left: (src_top, src_left),
                     };
                     app.set_status_message(format!("Requesting scripts for copy [({}, {})..({}, {})]...", src_left, src_top, src_right, src_bottom));
                     handled = true;
                 } else {
                     app.set_status_message("Cannot copy: Selection contains no valid frames.".to_string());
                     app.clipboard = ClipboardState::Empty;
                     handled = false;
                 }
            }
            KeyCode::Char('p') => {
                 match app.clipboard.clone() { // Clone to work with the value
                     ClipboardState::ReadyMulti { data } => {
                          let (target_row, target_col) = current_selection.cursor_pos();
                          current_selection = GridSelection::single(target_row, target_col); // Ensure single cell selection
 
                          let num_cols_pasted = data.len();
                          let num_rows_pasted = data.get(0).map_or(0, |col| col.len());
 
                          if num_cols_pasted > 0 && num_rows_pasted > 0 {
                              // Convert TUI ClipboardFrameData to shared PastedFrameData for the message
                              let paste_block_data = data.into_iter().map(|col|
                                 col.into_iter().map(|frame| bubocorelib::shared_types::PastedFrameData {
                                     length: frame.length,
                                     is_enabled: frame.is_enabled,
                                     script_content: frame.script_content,
                                 }).collect()
                              ).collect();
 
                               app.send_client_message(ClientMessage::PasteDataBlock {
                                  data: paste_block_data,
                                  target_row,
                                  target_col,
                                  timing: ActionTiming::Immediate,
                               });
                               app.set_status_message(format!("Requested pasting {}x{} block at ({}, {})...", num_cols_pasted, num_rows_pasted, target_col, target_row));
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
            }
            // --- Duplicate Frame Before Cursor ---
            KeyCode::Char('a') => { // 'a' duplicates before
                let ((top, left), (bottom, right)) = current_selection.bounds();
                let target_cursor_row = top; // Target is the start of the selection
                let target_cursor_col = left;
 
                app.send_client_message(ClientMessage::RequestDuplicationData {
                    src_top: top,
                    src_left: left,
                    src_bottom: bottom,
                    src_right: right,
                    target_cursor_row,
                    target_cursor_col,
                    insert_before: true,
                    timing: ActionTiming::Immediate,
                });
                app.set_status_message(format!("Requested duplication (before) for selection [({}, {})..({}, {})]", left, top, right, bottom));
                // Cursor adjustment might happen after server confirmation/update?
                handled = true;
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
                let ((top, left), (bottom, right)) = current_selection.bounds();
                let target_cursor_row = bottom + 1; // Target after the selection
                let target_cursor_col = left;
 
                app.send_client_message(ClientMessage::RequestDuplicationData {
                    src_top: top,
                    src_left: left,
                    src_bottom: bottom,
                    src_right: right,
                    target_cursor_row,
                    target_cursor_col,
                    insert_before: false, // Insert after
                    timing: ActionTiming::Immediate,
                });
                app.set_status_message(format!("Requested duplication (after) for selection [({}, {})..({}, {})]", left, top, right, bottom));
                // Cursor adjustment?
                handled = true;
            }
            // --- Delete Selected Frame(s) ---
            KeyCode::Delete | KeyCode::Backspace => {
                let mut handled_delete = false;
                let mut lines_and_indices_to_remove: Vec<(usize, Vec<usize>)> = Vec::new();
                let mut total_frames_deleted = 0;
                let ((top, left), (bottom, right)) = current_selection.bounds();
                let mut final_cursor_pos = (top.saturating_sub(1), left); // Default: cell before top-left

                // --- Scope for immutable borrow of scene --- 
                let mut status_msg = "Cannot delete: Invalid state".to_string(); // Default error

                if let Some(local_scene) = &app.editor.scene { // Borrow scene immutably
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
                                let effective_end = row_end.min(line_len -1);

                                if effective_start <= effective_end { // Check if there's overlap
                                    let indices_in_col: Vec<usize> = (effective_start..=effective_end).collect();
                                    if !indices_in_col.is_empty() {
                                        let indices_count = indices_in_col.len(); // Calculate length before move
                                        total_frames_deleted += indices_count; // Use the calculated count
                                        lines_and_indices_to_remove.push((col_idx, indices_in_col)); // Move the Vec now
                                        // Try to adjust cursor based on the first affected column
                                        if col_idx == left {
                                            final_cursor_pos = (effective_start.saturating_sub(1).min(line_len.saturating_sub(indices_count + 1)), col_idx);
                                        }
                                    }
                                }
                                // If effective_start > effective_end, it means the selection was completely outside this line's bounds
                            } else {
                                // This column index itself is invalid, should ideally not happen if bounds are correct
                                status_msg = format!("Cannot delete: Invalid column index {}", col_idx);
                                lines_and_indices_to_remove.clear(); // Also clear the collected indices
                                break;
                            }
                        }
                        if !lines_and_indices_to_remove.is_empty() {
                            status_msg = format!("Requested deleting {} frame(s) across {} line(s)", total_frames_deleted, lines_and_indices_to_remove.len());
                            handled_delete = true;
                        } else { // Possible means scene wasn't empty and no invalid col index, but selection didn't overlap any frames
                             status_msg = "Cannot delete: Selection contains no valid frames.".to_string();
                             handled_delete = false;
                        }
                         // If !possible, status_msg is already set
                    }
                } // --- End of immutable borrow scope ---

                // --- Now perform actions requiring mutable app --- 
                if handled_delete {
                    // Send the single multi-line message
                    app.send_client_message(ClientMessage::RemoveFramesMultiLine {
                        lines_and_indices: lines_and_indices_to_remove,
                        timing: ActionTiming::Immediate,
                    });
                    app.set_status_message(status_msg.clone()); // Use the message determined earlier
                    app.add_log(LogLevel::Info, status_msg);

                    // Adjust selection after deletion request - move cursor to 'top' if possible, or previous frame.
                    current_selection = GridSelection::single(final_cursor_pos.0, final_cursor_pos.1);
                } else {
                    // Set the error status message determined in the borrow scope
                    app.set_status_message(status_msg);
                }

                handled = handled_delete; // Use the final flag
            }
            _ => { handled = false; } 
        }

        if handled {
            // If the selection changed and we handled the event, send update to server.
            if app.interface.components.grid_selection != current_selection {
                 app.interface.components.grid_selection = current_selection;
                 app.send_client_message(ClientMessage::UpdateGridSelection(current_selection));
            } else {
                 // Even if selection is same (e.g. pressing enter on same cell), update state
                 // No need to send network message if selection didn't change
                app.interface.components.grid_selection = current_selection;
            }
        } else {
            // If not handled, still need to potentially update the selection if it was changed internally
            // (e.g. clicking + or - resets selection to cursor pos)
            if app.interface.components.grid_selection != current_selection {
                app.interface.components.grid_selection = current_selection;
                 // Send update even if key wasn't primarily for movement, if selection changed
                 app.send_client_message(ClientMessage::UpdateGridSelection(current_selection));
            }
        }
        Ok(handled)
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
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {

        // Get the current scene length from the scene object
        let scene_length = app.editor.scene.as_ref().map_or(0, |s| s.length());

        // Main window title with length
        let title = format!(" Scene Grid (Length: {}) ", scene_length);
        let outer_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::White));
        let inner_area = outer_block.inner(area);
        frame.render_widget(outer_block.clone(), area);

        // Need at least some space to draw anything inside
        if inner_area.width < 1 || inner_area.height < 2 { return; }

        // Determine heights based on which prompts are active
        let help_height = 2;
        let length_prompt_height = if app.interface.components.is_setting_frame_length { 3 } else { 0 };
        let insert_prompt_height = if app.interface.components.is_inserting_frame_duration { 3 } else { 0 };
        let prompt_height = length_prompt_height + insert_prompt_height; // Total prompt height

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

        // Split the prompt area if both prompts could potentially be active (though unlikely simultaneously)
        // Or just render one based on which flag is true
        let prompt_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(length_prompt_height),
                Constraint::Length(insert_prompt_height),
            ]).split(prompt_area);

        let length_prompt_area = prompt_layout[0];
        let insert_prompt_area = prompt_layout[1];

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
            frame.render_widget(&length_input_area, length_prompt_area);
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
            frame.render_widget(&insert_input_area, insert_prompt_area);
        }

        // Help line explaining keybindings
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);

        // Line 1
        let help_spans_line1 = vec![
            Span::raw("Move: "), Span::styled("↑↓←→ ", key_style),
            Span::raw("Toggle: "), Span::styled("Space ", key_style),
            Span::raw("Edit Script: "), Span::styled("Enter ", key_style),
            Span::raw("Set Len: "), Span::styled("l ", key_style),
            Span::raw("Set Start/End: "), Span::styled("b", key_style), Span::raw("/"), Span::styled("e", key_style),
        ];

        // Line 2
        let help_spans_line2 = vec![
            Span::styled("Shift+Arrows", key_style), Span::raw(":Select  "),
            Span::styled("Esc", key_style), Span::raw(":Reset Sel  "),
            Span::styled("i", key_style), Span::raw(":Ins Frame(+) "), // Updated help for 'i'
            Span::styled("Del/Bksp", key_style), Span::raw(":Del Frame "), // Removed 'After'
            Span::styled("a", key_style), Span::raw("/"), Span::styled("d", key_style), Span::raw(":Dup Before/After  "),
            Span::styled("c", key_style), Span::raw("/"), Span::styled("p", key_style),
            Span::raw(":Copy/Paste "), // Shortened
            Span::raw("  "), // Added spacing
            Span::styled("Shift+A/D", key_style), Span::raw(":Add/Rem Line"),
        ];

        // Split the help area into two rows
        let help_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(help_area);

        frame.render_widget(Paragraph::new(Line::from(help_spans_line1).style(help_style)).centered(), help_layout[0]);
        frame.render_widget(Paragraph::new(Line::from(help_spans_line2).style(help_style)).centered(), help_layout[1]);

        // Grid table (requiring scene data)
        if let Some(scene) = &app.editor.scene {
            let lines = &scene.lines;
            if lines.is_empty() {
                frame.render_widget(Paragraph::new("No lines in scene. Use 'Shift+A' to add.").yellow().centered(), table_area);
                return;
            }

            let num_lines = lines.len();
            // Determine the maximum number of frames across all lines for table height
            let max_frames = lines.iter().map(|line| line.frames.len()).max().unwrap_or(0);

            // Placeholder message if lines exist but have no frames
            if max_frames == 0 && num_lines > 0 {
                frame.render_widget(
                    Paragraph::new("Lines have no frames. Use 'i' to insert.")
                    .yellow()
                    .centered(), 
                    table_area
                );
                // Don't return here, still draw the header
            }

            // Various styles for the table
            let header_style = Style::default().fg(Color::White).bg(Color::Blue).bold();

            // Calculate column widths (distribute available width, min width 6)
            let col_width = if num_lines > 0 { table_area.width / num_lines as u16 } else { table_area.width };
            let widths: Vec<Constraint> = std::iter::repeat(Constraint::Min(col_width.max(6)))
                .take(num_lines)
                .collect();

            // Table Header (LINE 1, LINE 2, ...)
            let header_cells = lines.iter().enumerate()
                .map(|(i, line)| {
                     let length_display = match line.custom_length {
                        Some(len) => format!("({:.1}b)", len),
                        None => "(Scene)".to_string(),
                     };
                     let speed_display = format!("x{:.1}", line.speed_factor);
                     let text = format!("LINE {} {} {}", i + 1, length_display, speed_display);
                     Cell::from(Line::from(text).alignment(ratatui::layout::Alignment::Center))
                         .style(header_style)
                 });
            let header = Row::new(header_cells).height(1).style(header_style);

            // Create Padding Row: use default style
            let padding_cells = std::iter::repeat(Cell::from("").style(Style::default())) 
                                  .take(num_lines);
            let padding_row = Row::new(padding_cells).height(1); // Height 1 for one line of padding

            // Create Data Rows 
            let data_rows = (0..max_frames.max(1)) // Ensure at least one row is drawn if max_frames is 0
            .map(|frame_idx| {
                 let cells = lines.iter().enumerate().map(|(col_idx, line)| {
                    self.render_grid_cell(frame_idx, col_idx, Some(line), app)
                 });
                 Row::new(cells).height(1)
             });

             // Combine Padding and Data Rows 
             let combined_rows = std::iter::once(padding_row).chain(data_rows);

            // Create and render the table
            let table = Table::new(combined_rows, &widths)
                .header(header)
                .column_spacing(1);
            frame.render_widget(table, table_area);

        } else {
            frame.render_widget(Paragraph::new("No scene loaded from server.").yellow().centered(), table_area);
        }
    }
}