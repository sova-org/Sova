use crate::app::App;
use crate::app::{ClipboardFrameData, ClipboardState};
use crate::components::logs::LogLevel;
use bubocorelib::schedule::ActionTiming;
use bubocorelib::server::client::ClientMessage;
use bubocorelib::schedule::SchedulerMessage;
use bubocorelib::scene::Scene;
use bubocorelib::shared_types::{PastedFrameData, GridSelection};
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cmp::min;
use std::collections::HashSet;
use std::str::FromStr;
use tui_textarea::TextArea;

use super::GridComponent;

impl GridComponent {
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
                        let (row_idx, col_idx) =
                            app.interface.components.grid_selection.cursor_pos();
                        let insert_pos = row_idx + 1;
                        app.send_client_message(ClientMessage::InsertFrame(
                            col_idx,
                            insert_pos,
                            new_duration,
                            ActionTiming::Immediate,
                        ));
                        status_msg_to_set = Some(format!(
                            "Requested inserting frame with duration {:.2} at ({}, {})",
                            new_duration, col_idx, insert_pos
                        ));
                        exit_mode = true;
                    }
                    _ => {
                        let error_message = format!(
                            "Invalid duration: '{}'. Must be a positive number.",
                            input_str
                        );
                        // Set bottom message directly
                        app.interface.components.bottom_message = error_message.clone();
                        app.interface.components.bottom_message_timestamp =
                            Some(std::time::Instant::now());
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

        app.interface.components.is_inserting_frame_duration = is_active;
        app.interface.components.insert_duration_input = textarea;
        Ok(exit_mode || handled_textarea)
    }

    /// Handles input for setting frame lengths in the grid.
    fn handle_set_length_input(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        // Initialize state from app
        let mut is_active = app.interface.components.is_setting_frame_length;
        let mut textarea = app.interface.components.frame_length_input.clone();
        let mut status_msg_to_set = None;
        let mut exit_mode = false;
        let mut handled_textarea = false;

        match key_event.code {
            // Handle Escape key - cancel operation
            KeyCode::Esc => {
                status_msg_to_set = Some("Frame length setting cancelled.".to_string());
                exit_mode = true;
            }
            // Handle Enter key - process input and update frames
            KeyCode::Enter => {
                let input_str = textarea.lines()[0].trim();
                // Need scene access here
                if let Some(scene) = app.editor.scene.as_ref() {
                    let current_selection = app.interface.components.grid_selection;
                    match input_str.parse::<f64>() {
                        // Valid positive number entered
                        Ok(new_length) if new_length > 0.0 => {
                            let ((top, left), (bottom, right)) = current_selection.bounds();
                            let mut modified_lines: std::collections::HashMap<usize, Vec<f64>> =
                                std::collections::HashMap::new();
                            let mut frames_changed = 0;

                            // Update frames in selected area
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

                            // Send update messages for modified lines
                            for (col, updated_frames) in modified_lines {
                                app.send_client_message(ClientMessage::UpdateLineFrames(
                                    col,
                                    updated_frames,
                                    ActionTiming::Immediate,
                                ));
                            }

                            // Set appropriate status message
                            if frames_changed > 0 {
                                status_msg_to_set = Some(format!(
                                    "Set length to {:.2} for {} frame(s)",
                                    new_length, frames_changed
                                ));
                            } else {
                                status_msg_to_set =
                                    Some("No valid frames in selection to set length".to_string());
                            }
                            exit_mode = true;
                        }
                        // Invalid input
                        _ => {
                            let error_message = format!(
                                "Invalid frame length: '{}'. Must be positive number.",
                                input_str
                            );
                            app.interface.components.bottom_message = error_message.clone();
                            app.interface.components.bottom_message_timestamp =
                                Some(std::time::Instant::now());
                            status_msg_to_set = Some(error_message);
                            // Don't exit on error
                        }
                    }
                } else {
                    status_msg_to_set =
                        Some("Error: Scene not loaded while setting frame length.".to_string());
                    exit_mode = true; // Exit if scene isn't loaded
                }
            }
            // Handle all other keys - pass to textarea
            _ => {
                handled_textarea = textarea.input(key_event);
            }
        }

        // Update status message if needed
        if let Some(msg) = status_msg_to_set {
            app.set_status_message(msg);
        }

        // Reset textarea if exiting mode
        if exit_mode {
            is_active = false;
            textarea = TextArea::default();
        }

        // Update app state
        app.interface.components.is_setting_frame_length = is_active;
        app.interface.components.frame_length_input = textarea;

        Ok(exit_mode || handled_textarea)
    }

    /// Handles user input for setting the scene length.
    fn handle_set_scene_length_input(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        let mut is_active = app.interface.components.is_setting_scene_length;
        let mut textarea = app.interface.components.scene_length_input.clone();
        let mut status_msg_to_set = None;
        let mut exit_mode = false;
        let mut handled_textarea = false;

        match key_event.code {
            KeyCode::Esc => {
                status_msg_to_set = Some("Scene length setting cancelled.".to_string());
                exit_mode = true;
            }
            KeyCode::Enter => {
                let input_str = textarea.lines()[0].trim();
                match input_str.parse::<usize>() {
                    Ok(new_length) if new_length > 0 => {
                        app.send_client_message(ClientMessage::SetSceneLength(
                            new_length,
                            ActionTiming::EndOfScene,
                        ));
                        status_msg_to_set =
                            Some(format!("Requested setting scene length to {}", new_length));
                        exit_mode = true;
                    }
                    _ => {
                        let error_message = format!(
                            "Invalid length: '{}'. Must be a positive integer.",
                            input_str
                        );
                        app.interface.components.bottom_message = error_message.clone();
                        app.interface.components.bottom_message_timestamp =
                            Some(std::time::Instant::now());
                        status_msg_to_set = Some(error_message);
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
        app.interface.components.is_setting_scene_length = is_active;
        app.interface.components.scene_length_input = textarea;
        Ok(exit_mode || handled_textarea)
    }

    /// Handles input events for setting a frame's name in the grid.
    fn handle_set_name_input(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
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
                let name_to_send = if input_name.is_empty() {
                    None
                } else {
                    Some(input_name.clone())
                };
                app.send_client_message(ClientMessage::SetFrameName(
                    col_idx,
                    row_idx,
                    name_to_send.clone(),
                    ActionTiming::Immediate,
                ));

                status_msg_to_set = if let Some(name) = name_to_send {
                    Some(format!(
                        "Requested setting name to '{}' for frame ({}, {})",
                        name, col_idx, row_idx
                    ))
                } else {
                    Some(format!(
                        "Requested clearing name for frame ({}, {})",
                        col_idx, row_idx
                    ))
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

    pub fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        if app.interface.components.grid_show_help {
            match key_event.code {
                KeyCode::Esc | KeyCode::Char('?') => {
                    app.interface.components.grid_show_help = false;
                    app.set_status_message("Closed help.".to_string());
                    return Ok(true);
                }
                _ => return Ok(true), // Consume all other input when help is shown
            }
        }

        // Get scene data, but don't exit immediately if empty
        let scene_opt = app.editor.scene.as_ref();
        let num_cols = scene_opt.map_or(0, |p| p.lines.len());

        if app.interface.components.is_setting_scene_length {
            return self.handle_set_scene_length_input(app, key_event);
        }

        if app.interface.components.is_setting_frame_name {
            return self.handle_set_name_input(app, key_event);
        }
        if app.interface.components.is_inserting_frame_duration {
            return self.handle_insert_duration_input(app, key_event);
        }
        if app.interface.components.is_setting_frame_length {
            return self.handle_set_length_input(app, key_event);
        }
        // Handle 'a' regardless of whether lines exist
        if key_event.code == KeyCode::Char('A') && key_event.modifiers.contains(KeyModifiers::SHIFT)
        {
            app.send_client_message(ClientMessage::SchedulerControl(
                SchedulerMessage::AddLine,
            ));
            app.set_status_message("Requested adding line".to_string());
            return Ok(true);
        }
        let scene = match scene_opt {
            Some(p) if num_cols > 0 => p,
            _ => {
                return Ok(false); // No scene or no lines, cannot proceed with grid actions
            }
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
                    current_selection =
                        GridSelection::single(current_selection.start.0, current_selection.start.1);
                    app.set_status_message(
                        "Selection reset to single cell (at start)".to_string(),
                    );
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
                            let is_same_frame_as_editor = app.editor.active_line.line_index == col_idx
                                && app.editor.active_line.frame_index == row_idx;

                            if is_same_frame_as_editor {
                                // Just switch back to the editor, no need to fetch script
                                app.events
                                    .sender
                                    .send(crate::event::Event::App(crate::event::AppEvent::SwitchToEditor))
                                    .map_err(|e| {
                                        color_eyre::eyre::eyre!("Send Error (SwitchToEditor): {}", e)
                                    })?;
                                status_update = Some(format!(
                                    "Returning to editor for Line {}, Frame {}",
                                    col_idx, row_idx
                                ));
                            } else {
                                // Different frame, fetch script from server
                                app.send_client_message(ClientMessage::GetScript(col_idx, row_idx));
                                // Also notify server that we START editing this frame
                                app.send_client_message(ClientMessage::StartedEditingFrame(
                                    col_idx,
                                    row_idx,
                                ));
                                status_update = Some(format!(
                                    "Requested script for Line {}, Frame {}",
                                    col_idx, row_idx
                                ));
                            }
                        } else {
                            status_update =
                                Some("Cannot request script for an empty slot".to_string());
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

                if let Some(status) = status_update {
                    app.set_status_message(status);
                }
                // Note: We don't switch to the editor here. We wait for the server response
                // (unless we switched directly in the is_same_frame_as_editor block).
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
                    if can_set {
                        break;
                    } // Found one, no need to check other lines
                }

                if can_set {
                    app.interface.components.is_setting_frame_length = true;
                    // Pre-fill with the length of the first selected frame, or empty if none
                    let initial_text =
                        first_frame_length.map_or(String::new(), |len| format!("{:.2}", len));
                    app.interface.components.frame_length_input =
                        TextArea::new(vec![initial_text]);
                    app.set_status_message("Enter new frame length (e.g., 1.5):".to_string());
                } else {
                    app.set_status_message(
                        "Cannot set length: selection contains no frames.".to_string(),
                    );
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
                        let start_frame_val =
                            if line.start_frame == Some(row_idx) {
                                None
                            } else {
                                Some(row_idx)
                            };
                        app.send_client_message(ClientMessage::SetLineStartFrame(
                            col_idx,
                            start_frame_val,
                            ActionTiming::Immediate,
                        ));
                        app.set_status_message(format!(
                            "Requested setting start frame to {:?} for Line {}",
                            start_frame_val,
                            col_idx
                        ));
                    } else {
                        app.set_status_message(
                            "Cannot set start frame on empty slot".to_string(),
                        );
                        handled = false;
                    }
                } else {
                    handled = false;
                }
            }
            KeyCode::Char('e') => {
                let cursor_pos = current_selection.cursor_pos();
                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                let (row_idx, col_idx) = cursor_pos;
                if let Some(line) = scene.lines.get(col_idx) {
                    if row_idx < line.frames.len() {
                        let end_frame_val =
                            if line.end_frame == Some(row_idx) {
                                None
                            } else {
                                Some(row_idx)
                            };
                        app.send_client_message(ClientMessage::SetLineEndFrame(
                            col_idx,
                            end_frame_val,
                            ActionTiming::Immediate,
                        ));
                        app.set_status_message(format!(
                            "Requested setting end frame to {:?} for Line {}",
                            end_frame_val,
                            col_idx
                        ));
                    } else {
                        app.set_status_message("Cannot set end frame on empty slot".to_string());
                        handled = false;
                    }
                } else {
                    handled = false;
                }
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
                let mut to_enable: std::collections::HashMap<usize, Vec<usize>> =
                    std::collections::HashMap::new();
                let mut to_disable: std::collections::HashMap<usize, Vec<usize>> =
                    std::collections::HashMap::new();
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
                        app.send_client_message(ClientMessage::DisableFrames(
                            col,
                            rows,
                            ActionTiming::Immediate,
                        ));
                    }
                }
                for (col, rows) in to_enable {
                    if !rows.is_empty() {
                        app.send_client_message(ClientMessage::EnableFrames(
                            col,
                            rows,
                            ActionTiming::Immediate,
                        ));
                    }
                }

                if frames_toggled > 0 {
                    app.set_status_message(format!(
                        "Requested toggling {} frames",
                        frames_toggled
                    ));
                } else {
                    app.set_status_message("No valid frames in selection to toggle".to_string());
                    handled = false;
                }
            }
            // Remove the last frame from the line
            KeyCode::Char('D') if is_shift_pressed => {
                // Shift+D removes last line
                let cursor_pos = current_selection.cursor_pos();
                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                let mut last_line_index_opt: Option<usize> = None;

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
                            SchedulerMessage::RemoveLine(last_line_index, ActionTiming::Immediate),
                        ));
                        app.set_status_message(format!(
                            "Requested removing line {}",
                            last_line_index
                        ));
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
            KeyCode::Char('a') => {
                // 'a' duplicates (insert before)
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
                        app.interface.components.insert_duration_input =
                            TextArea::new(vec![initial_text]);
                        app.set_status_message(
                            "Enter duration for new frame (default 1.0):".to_string(),
                        );
                        handled = true;
                    } else {
                        app.add_log(
                            LogLevel::Warn,
                            format!(
                                "Cannot insert frame at invalid position {} in line {}",
                                insert_pos,
                                col_idx
                            ),
                        );
                        app.set_status_message(
                            "Cannot insert frame here (beyond end + 1)".to_string(),
                        );
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
                        current_scroll_offset_val =
                            (current_scroll_offset_val + page_size).min(max_scroll);

                        // Move cursor to the top of the new page (relative to current column)
                        let current_col = current_selection.cursor_pos().1;
                        let new_row = current_scroll_offset_val;
                        // Clamp row based on actual frames in target column
                        let frames_in_col =
                            scene.lines.get(current_col).map_or(0, |l| l.frames.len());
                        let clamped_row = new_row.min(frames_in_col.saturating_sub(1));
                        current_selection = GridSelection::single(clamped_row, current_col);
                        // Handled is true by default
                    } else {
                        handled = false;
                    } // Cannot scroll if no overflow or no visible height
                } else {
                    handled = false;
                } // Cannot scroll if render info is missing
            }
            KeyCode::PageUp => {
                if let Some(info) = render_info {
                    if info.visible_height > 0 {
                        let page_size = info.visible_height.saturating_sub(1).max(1);
                        current_scroll_offset_val =
                            current_scroll_offset_val.saturating_sub(page_size);

                        // Move cursor to the top of the new page
                        let current_col = current_selection.cursor_pos().1;
                        let new_row = current_scroll_offset_val;
                        // Clamp row based on actual frames in target column
                        let frames_in_col =
                            scene.lines.get(current_col).map_or(0, |l| l.frames.len());
                        let clamped_row = new_row.min(frames_in_col.saturating_sub(1));
                        current_selection = GridSelection::single(clamped_row, current_col);
                        // Handled is true by default
                    } else {
                        handled = false;
                    } // Cannot scroll if no visible height
                } else {
                    handled = false;
                } // Cannot scroll if render info is missing
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
                        let existing_name = line
                            .frame_names
                            .get(row_idx)
                            .cloned()
                            .flatten()
                            .unwrap_or_default();

                        app.interface.components.is_setting_frame_name = true;
                        app.interface.components.frame_name_input =
                            TextArea::new(vec![existing_name]);
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
            // --- Toggle Help Popup ---
            KeyCode::Char('?') => {
                app.interface.components.grid_show_help = true;
                app.set_status_message("Opened help (Esc or ? to close).".to_string());
                handled = true;
            }
            // --- Set Scene Length via Prompt ---
            KeyCode::Char('L') if is_shift_pressed => {
                if let Some(scene) = scene_opt {
                    app.interface.components.is_setting_scene_length = true;
                    let initial_text = format!("{}", scene.length());
                    app.interface.components.scene_length_input =
                        TextArea::new(vec![initial_text]);
                    app.set_status_message("Enter new scene length (beats):".to_string());
                    handled = true;
                } else {
                    app.set_status_message("Cannot set scene length: Scene not loaded.".to_string());
                    handled = false;
                }
            }
            _ => {
                handled = false;
            }
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

            if final_cursor_row < desired_offset {
                // Cursor moved above visible area
                desired_offset = final_cursor_row;
            } else if visible_height > 0 && final_cursor_row >= desired_offset + visible_height {
                // Cursor moved below visible area
                desired_offset =
                    final_cursor_row.saturating_sub(visible_height.saturating_sub(1));
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

    fn calculate_next_selection(
        &self,
        current_selection: GridSelection,
        key_code: KeyCode,
        is_shift_pressed: bool,
        scene: &Scene,
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
                    let frames_in_next_col =
                        scene.lines.get(next_col).map_or(0, |s| s.frames.len());
                    end_pos.0 = min(end_pos.0, frames_in_next_col.saturating_sub(1));
                    end_pos.1 = next_col;
                } else {
                    changed = false;
                }
            }
            KeyCode::Right => {
                let next_col = min(end_pos.1 + 1, num_cols.saturating_sub(1));
                if next_col != end_pos.1 {
                    // Check if column actually changed
                    let frames_in_next_col =
                        scene.lines.get(next_col).map_or(0, |s| s.frames.len());
                    end_pos.0 = min(end_pos.0, frames_in_next_col.saturating_sub(1)); // Adjust row
                    end_pos.1 = next_col;
                } else {
                    changed = false;
                }
            }
            _ => {
                changed = false;
            } // Should not happen as we match specific keys
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
        scene: &Scene,
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
        let handled;
        match app.clipboard.clone() {
            // Clone to work with the value
            ClipboardState::ReadyMulti { data } => {
                let (target_row, target_col) = current_selection.cursor_pos();
                *current_selection = GridSelection::single(target_row, target_col);

                let num_cols_pasted = data.len();
                let num_rows_pasted = data.get(0).map_or(0, |col| col.len());

                if num_cols_pasted > 0 && num_rows_pasted > 0 {
                    let paste_block_data = data
                        .into_iter()
                        .map(|col| {
                            col.into_iter()
                                .map(|frame| PastedFrameData {
                                    length: frame.length,
                                    is_enabled: frame.is_enabled,
                                    script_content: frame.script_content,
                                    name: frame.frame_name,
                                })
                                .collect()
                        })
                        .collect();

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
                    app.clipboard = ClipboardState::Empty;
                    handled = true;
                } else {
                    app.set_status_message("Cannot paste empty clipboard data.".to_string());
                    handled = false;
                }
            }
            ClipboardState::FetchingScripts { pending, .. } => {
                app.set_status_message(format!(
                    "Still fetching {} scripts from server to copy...",
                    pending.len()
                ));
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

        true
    }

    /// Handles the deletion of selected frames in the grid.
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

        let mut status_msg = "Cannot delete: Invalid state or no scene loaded".to_string();
        let scene_available = app.editor.scene.is_some();

        if scene_available {
            let local_scene = app.editor.scene.as_ref().unwrap();
            if local_scene.lines.is_empty() {
                status_msg = "Cannot delete: Scene has no lines".to_string();
            } else {
                for col_idx in left..=right {
                    if let Some(line) = local_scene.lines.get(col_idx) {
                        let line_len = line.frames.len();
                        if line_len == 0 {
                            continue;
                        }

                        let row_start = top;
                        let row_end = bottom;
                        let effective_start = row_start;
                        let effective_end = row_end.min(line_len.saturating_sub(1));

                        if effective_start <= effective_end {
                            let indices_in_col: Vec<usize> =
                                (effective_start..=effective_end).collect();
                            if !indices_in_col.is_empty() {
                                let indices_count = indices_in_col.len();
                                total_frames_deleted += indices_count;
                                lines_and_indices_to_remove.push((col_idx, indices_in_col));
                                // Adjust potential final cursor based on the *first* column affected
                                if col_idx == left {
                                    // Try to place cursor at the start row of deletion, or the frame before if it was the first frame
                                    let new_len = line_len.saturating_sub(indices_count);
                                    let target_row = effective_start.min(new_len.saturating_sub(1));
                                    final_cursor_pos = (target_row, col_idx);
                                }
                            }
                        }
                    } else {
                        status_msg = format!("Cannot delete: Invalid column index {}", col_idx);
                        lines_and_indices_to_remove.clear();
                        handled_delete = false;
                        break;
                    }
                }

                // Update status based on collected indices
                if !lines_and_indices_to_remove.is_empty() {
                    status_msg = format!(
                        "Requested deleting {} frame(s) across {} line(s)",
                        total_frames_deleted,
                        lines_and_indices_to_remove.len()
                    );
                    handled_delete = true;
                } else if handled_delete != false {
                    // Check renamed variable
                    status_msg = "Cannot delete: Selection contains no valid frames.".to_string();
                    handled_delete = false;
                }
                // else: status_msg was set by invalid column error, handled_delete is false
            }
        }

        if handled_delete {
            app.send_client_message(ClientMessage::RemoveFramesMultiLine {
                lines_and_indices: lines_and_indices_to_remove,
                timing: ActionTiming::Immediate,
            });
            app.set_status_message(status_msg.clone());
            app.add_log(LogLevel::Info, status_msg);
            *current_selection = GridSelection::single(final_cursor_pos.0, final_cursor_pos.1);
        } else {
            app.set_status_message(status_msg);
        }

        handled_delete
    }
} 