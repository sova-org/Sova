//! Editing input handling for the grid component.
//!
//! Handles frame editing operations like enabling/disabling, length setting,
//! inserting, deleting, and other frame manipulations.

use crate::app::App;
use crate::components::logs::LogLevel;
use color_eyre::Result as EyreResult;
use corelib::schedule::action_timing::ActionTiming;
use corelib::schedule::message::SchedulerMessage;
use corelib::server::client::ClientMessage;
use corelib::shared_types::GridSelection;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use tui_textarea::TextArea;

pub struct EditingHandler;

impl EditingHandler {
    /// Handle editing-related key events
    pub fn handle_editing(app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        let mut current_selection = app.interface.components.grid_selection;
        let mut handled = false;

        // Handle Shift+A for adding lines regardless of scene state
        if key_event.code == KeyCode::Char('A') && key_event.modifiers.contains(KeyModifiers::SHIFT) {
            app.send_client_message(ClientMessage::SchedulerControl(SchedulerMessage::AddLine));
            app.set_status_message("Requested adding line".to_string());
            return Ok(true);
        }

        // For other operations, we need a valid scene - check without borrowing
        let (has_scene, num_cols) = match &app.editor.scene {
            Some(s) => (s.lines.len() > 0, s.lines.len()),
            None => (false, 0),
        };

        if !has_scene {
            return Ok(false);
        }

        match key_event.code {
            KeyCode::Enter => {
                handled = Self::handle_enter_frame_edit(app, &mut current_selection);
            }
            KeyCode::Char(' ') => {
                handled = Self::handle_toggle_frames(app, current_selection);
            }
            KeyCode::Char('l') => {
                handled = Self::handle_set_frame_length(app, current_selection);
            }
            KeyCode::Char('B') => {
                handled = Self::handle_set_loop_range(app, current_selection);
            }
            KeyCode::Char('b') => {
                handled = Self::handle_clear_loop_range(app, current_selection);
            }
            KeyCode::Char('i') => {
                handled = Self::handle_insert_frame(app, &mut current_selection);
            }
            KeyCode::Char('n') => {
                handled = Self::handle_set_frame_name(app, current_selection);
            }
            KeyCode::Char('r') => {
                handled = Self::handle_set_frame_repetitions(app, current_selection);
            }
            KeyCode::Char('L') if key_event.modifiers.contains(KeyModifiers::SHIFT) => {
                handled = Self::handle_set_scene_length(app);
            }
            KeyCode::Char('X') => {
                handled = Self::handle_delete_line(app, &mut current_selection, num_cols);
            }
            KeyCode::Delete | KeyCode::Backspace => {
                handled = Self::handle_delete_frames(app, &mut current_selection);
            }
            _ => {}
        }

        // Update selection if it was modified
        if current_selection != app.interface.components.grid_selection {
            app.interface.components.grid_selection = current_selection;
            app.send_client_message(ClientMessage::UpdateGridSelection(current_selection));
        }

        Ok(handled)
    }

    fn handle_enter_frame_edit(app: &mut App, current_selection: &mut GridSelection) -> bool {
        let cursor_pos = current_selection.cursor_pos();
        *current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
        let (row_idx, col_idx) = cursor_pos;

        if let Some(scene) = &app.editor.scene {
            if let Some(line) = scene.lines.get(col_idx) {
                if row_idx < line.frames.len() {
                    let is_same_frame_as_editor = app.editor.active_line.line_index == col_idx
                        && app.editor.active_line.frame_index == row_idx;

                    if is_same_frame_as_editor {
                        // Switch back to editor
                        app.events
                            .sender
                            .send(crate::event::Event::App(
                                crate::event::AppEvent::SwitchToEditor,
                            ))
                            .unwrap_or_else(|e| {
                                app.add_log(LogLevel::Error, format!("Event send error: {}", e));
                            });
                        app.set_status_message(format!(
                            "Returning to editor for Line {}, Frame {}",
                            col_idx, row_idx
                        ));
                    } else {
                        // Fetch script from server
                        app.send_client_message(ClientMessage::GetScript(col_idx, row_idx));
                        app.send_client_message(ClientMessage::StartedEditingFrame(col_idx, row_idx));
                        app.set_status_message(format!(
                            "Requested script for Line {}, Frame {}",
                            col_idx, row_idx
                        ));
                    }
                    return true;
                }
            }
        }

        app.set_status_message("Cannot request script for invalid frame".to_string());
        false
    }

    fn handle_toggle_frames(app: &mut App, current_selection: GridSelection) -> bool {
        let ((top, left), (bottom, right)) = current_selection.bounds();
        let mut to_enable: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut to_disable: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut frames_toggled = 0;

        if let Some(scene) = &app.editor.scene {
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
            app.set_status_message(format!("Requested toggling {} frames", frames_toggled));
            true
        } else {
            app.set_status_message("No valid frames in selection to toggle".to_string());
            false
        }
    }

    fn handle_set_frame_length(app: &mut App, current_selection: GridSelection) -> bool {
        let ((top, left), (bottom, right)) = current_selection.bounds();
        let mut first_frame_length: Option<f64> = None;
        let mut can_set = false;

        // Check if selection contains at least one valid frame
        if let Some(scene) = &app.editor.scene {
            for col_idx in left..=right {
                if let Some(line) = scene.lines.get(col_idx) {
                    for row_idx in top..=bottom {
                        if row_idx < line.frames.len() {
                            can_set = true;
                            if first_frame_length.is_none() {
                                first_frame_length = Some(line.frames[row_idx]);
                            }
                            break;
                        }
                    }
                    if can_set {
                        break;
                    }
                }
            }
        }

        if can_set {
            app.interface.components.is_setting_frame_length = true;
            let initial_text = first_frame_length.map_or(String::new(), |len| format!("{:.2}", len));
            let mut textarea = TextArea::new(vec![initial_text]);
            textarea.move_cursor(tui_textarea::CursorMove::End);
            app.interface.components.frame_length_input = textarea;
            app.set_status_message("Enter new frame length (e.g., 1.5):".to_string());
            true
        } else {
            app.set_status_message("Cannot set length: selection contains no frames.".to_string());
            false
        }
    }

    fn handle_set_loop_range(app: &mut App, current_selection: GridSelection) -> bool {
        let ((top, left), (bottom, right)) = current_selection.bounds();
        let mut lines_affected = 0;

        for col_idx in left..=right {
            app.send_client_message(ClientMessage::SetLineStartFrame(
                col_idx,
                Some(top),
                ActionTiming::EndOfScene,
            ));
            app.send_client_message(ClientMessage::SetLineEndFrame(
                col_idx,
                Some(bottom),
                ActionTiming::EndOfScene,
            ));
            lines_affected += 1;
        }

        if lines_affected > 0 {
            app.set_status_message(format!(
                "Requested setting Start={} End={} for Lines {}..{}",
                top, bottom, left, right
            ));
            true
        } else {
            app.set_status_message("No lines in selection to set start/end.".to_string());
            false
        }
    }

    fn handle_clear_loop_range(app: &mut App, current_selection: GridSelection) -> bool {
        let ((_, left), (_, right)) = current_selection.bounds();
        let mut lines_affected = 0;

        for col_idx in left..=right {
            app.send_client_message(ClientMessage::SetLineStartFrame(
                col_idx,
                None,
                ActionTiming::EndOfScene,
            ));
            app.send_client_message(ClientMessage::SetLineEndFrame(
                col_idx,
                None,
                ActionTiming::EndOfScene,
            ));
            lines_affected += 1;
        }

        if lines_affected > 0 {
            app.set_status_message(format!(
                "Requested clearing loop for Lines {}..{}",
                left, right
            ));
            true
        } else {
            app.set_status_message("No lines in selection to clear loop.".to_string());
            false
        }
    }

    fn handle_insert_frame(app: &mut App, current_selection: &mut GridSelection) -> bool {
        let (row_idx, col_idx) = current_selection.cursor_pos();
        *current_selection = GridSelection::single(row_idx, col_idx);
        let insert_pos = row_idx + 1;

        if let Some(scene) = &app.editor.scene {
            if let Some(line) = scene.lines.get(col_idx) {
                if insert_pos <= line.frames.len() {
                    app.interface.components.is_inserting_frame_duration = true;
                    app.interface.components.insert_duration_input = TextArea::new(vec!["1.0".to_string()]);
                    app.set_status_message("Enter duration for new frame (default 1.0):".to_string());
                    true
                } else {
                    app.set_status_message("Cannot insert frame here (beyond end + 1)".to_string());
                    false
                }
            } else {
                app.set_status_message("Cannot insert frame: Line does not exist.".to_string());
                false
            }
        } else {
            app.set_status_message("Cannot insert frame: Scene not loaded.".to_string());
            false
        }
    }

    fn handle_set_frame_name(app: &mut App, current_selection: GridSelection) -> bool {
        let cursor_pos = current_selection.cursor_pos();
        let (row_idx, col_idx) = cursor_pos;

        if let Some(scene) = &app.editor.scene {
            if let Some(line) = scene.lines.get(col_idx) {
                if row_idx < line.frames.len() {
                    let existing_name = line
                        .frame_names
                        .get(row_idx)
                        .cloned()
                        .flatten()
                        .unwrap_or_default();

                    app.interface.components.is_setting_frame_name = true;
                    let mut textarea = TextArea::new(vec![existing_name]);
                    textarea.move_cursor(tui_textarea::CursorMove::End);
                    app.interface.components.frame_name_input = textarea;
                    app.set_status_message("Enter new frame name (empty clears):".to_string());
                    true
                } else {
                    app.set_status_message("Cannot name an empty frame slot.".to_string());
                    false
                }
            } else {
                app.set_status_message("Cannot name frame: Invalid line.".to_string());
                false
            }
        } else {
            app.set_status_message("Cannot name frame: Scene not loaded.".to_string());
            false
        }
    }

    fn handle_set_frame_repetitions(app: &mut App, current_selection: GridSelection) -> bool {
        let (row_idx, col_idx) = current_selection.cursor_pos();

        if let Some(scene) = &app.editor.scene {
            if let Some(line) = scene.lines.get(col_idx) {
                if let Some(repetitions) = line.frame_repetitions.get(row_idx) {
                    let mut textarea = TextArea::from(vec![repetitions.to_string()]);
                    textarea.move_cursor(tui_textarea::CursorMove::End);
                    app.interface.components.frame_repetitions_input = textarea;
                } else {
                    app.interface.components.frame_repetitions_input = TextArea::from(vec!["1"]);
                }
            } else {
                app.interface.components.frame_repetitions_input = TextArea::from(vec!["1"]);
            }
        } else {
            app.interface.components.frame_repetitions_input = TextArea::from(vec!["1"]);
        }

        app.interface.components.is_setting_frame_repetitions = true;
        app.set_status_message("Enter frame repetitions (positive integer).".to_string());
        true
    }

    fn handle_set_scene_length(app: &mut App) -> bool {
        if let Some(scene) = &app.editor.scene {
            app.interface.components.is_setting_scene_length = true;
            let initial_text = format!("{}", scene.length());
            app.interface.components.scene_length_input = TextArea::new(vec![initial_text]);
            app.set_status_message("Enter new scene length (beats):".to_string());
            true
        } else {
            app.set_status_message("Cannot set scene length: Scene not loaded.".to_string());
            false
        }
    }

    fn handle_delete_line(app: &mut App, current_selection: &mut GridSelection, num_cols: usize) -> bool {
        let cursor_pos = current_selection.cursor_pos();
        *current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
        let (_, line_idx_to_delete) = cursor_pos;

        if num_cols > 1 {
            if let Some(scene) = &app.editor.scene {
                let new_line_idx = line_idx_to_delete.saturating_sub(1);
                let frames_in_new_line = scene.lines.get(new_line_idx).map_or(0, |l| l.frames.len());
                let new_row_idx = cursor_pos.0.min(frames_in_new_line.saturating_sub(1)).max(0);

                app.send_client_message(ClientMessage::SchedulerControl(
                    SchedulerMessage::RemoveLine(line_idx_to_delete, ActionTiming::Immediate),
                ));
                app.set_status_message(format!("Requested removing line {}", line_idx_to_delete));

                *current_selection = GridSelection::single(new_row_idx, new_line_idx);
                true
            } else {
                app.set_status_message("Cannot remove line: Scene not loaded.".to_string());
                false
            }
        } else {
            app.set_status_message("Cannot remove the last line.".to_string());
            false
        }
    }

    fn handle_delete_frames(app: &mut App, current_selection: &mut GridSelection) -> bool {
        let mut lines_and_indices_to_remove: Vec<(usize, Vec<usize>)> = Vec::new();
        let mut total_frames_deleted = 0;
        let ((top, left), (bottom, right)) = current_selection.bounds();
        let mut final_cursor_pos = (top.saturating_sub(1), left);

        if let Some(scene) = &app.editor.scene {
            if scene.lines.is_empty() {
                app.set_status_message("Cannot delete: Scene has no lines".to_string());
                return false;
            }

            for col_idx in left..=right {
                if let Some(line) = scene.lines.get(col_idx) {
                    let line_len = line.frames.len();
                    if line_len == 0 {
                        continue;
                    }

                    // Check if trying to delete the last frame
                    if line_len == 1 && top == 0 && bottom == 0 {
                        app.set_status_message(format!("Cannot delete the last frame of line {}.", col_idx));
                        return false;
                    }

                    let effective_start = top;
                    let effective_end = bottom.min(line_len.saturating_sub(1));

                    if effective_start <= effective_end {
                        let indices_in_col: Vec<usize> = (effective_start..=effective_end).collect();
                        if !indices_in_col.is_empty() {
                            let indices_count = indices_in_col.len();
                            total_frames_deleted += indices_count;
                            lines_and_indices_to_remove.push((col_idx, indices_in_col));

                            if col_idx == left {
                                let new_len = line_len.saturating_sub(indices_count);
                                let target_row = effective_start.min(new_len.saturating_sub(1));
                                final_cursor_pos = (target_row, col_idx);
                            }
                        }
                    }
                }
            }

            if !lines_and_indices_to_remove.is_empty() {
                let num_lines_affected = lines_and_indices_to_remove.len();
                app.send_client_message(ClientMessage::RemoveFramesMultiLine {
                    lines_and_indices: lines_and_indices_to_remove,
                    timing: ActionTiming::Immediate,
                });
                app.set_status_message(format!(
                    "Requested deleting {} frame(s) across {} line(s)",
                    total_frames_deleted,
                    num_lines_affected
                ));
                *current_selection = GridSelection::single(final_cursor_pos.0, final_cursor_pos.1);
                true
            } else {
                app.set_status_message("Cannot delete: Selection contains no valid frames.".to_string());
                false
            }
        } else {
            app.set_status_message("Cannot delete: No scene loaded".to_string());
            false
        }
    }
}