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
use std::cmp::min;
use crate::components::logs::LogLevel;
use crate::app::ClipboardState;

/// Component representing the scene grid, what is currently being played/edited
pub struct GridComponent;

impl GridComponent {
    /// Creates a new [`GridComponent`] instance.
    pub fn new() -> Self {
        Self {}
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
    ///   - `<` / `,`: Decrease frame length.
    ///   - `>` / `.`: Increase frame length.
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

        // Handle 'a' regardless of whether lines exist
        if key_event.code == KeyCode::Char('a') {
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
            // Add a new frame to the line (default length 1.0) after the cursor
            KeyCode::Char('+') => {
                 let cursor_pos = current_selection.cursor_pos();
                 current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1); // Keep selection single
                 let (row_idx, col_idx) = cursor_pos;
                 let insert_pos = row_idx + 1;
                 let default_insert_length = 1.0;

                 // Check if line exists and if insertion is valid
                 if let Some(line) = scene.lines.get(col_idx) {
                     // Check if the insert position is valid (can be equal to len for appending)
                     if insert_pos <= line.frames.len() {
                         // Check if adding the frame exceeds scene length
                         let current_line_beats: f64 = line.frames.iter().sum();
                         let potential_line_beats = current_line_beats + default_insert_length;
                         let scene_total_length = scene.length() as f64;

                         if potential_line_beats <= scene_total_length {
                             app.send_client_message(ClientMessage::InsertFrame(col_idx, insert_pos, ActionTiming::Immediate));
                             app.set_status_message(format!("Requested inserting frame at ({}, {})", col_idx, insert_pos));
                         } else {
                             app.set_status_message(format!("Cannot insert frame: exceeds scene length ({:.2}/{:.2})", potential_line_beats, scene_total_length));
                             handled = false;
                         }
                     } else {
                         app.add_log(LogLevel::Warn, format!("Attempted to insert frame at invalid position {} in line {}", insert_pos, col_idx));
                         app.set_status_message("Cannot insert frame here".to_string());
                         handled = false;
                     }
                 } else {
                     app.set_status_message("Invalid line for adding frame".to_string());
                     handled = false;
                 }
            }
            // Remove the frame immediately AFTER the cursor position
            KeyCode::Char('-') => {
                let cursor_pos = current_selection.cursor_pos();
                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1); // Keep selection single
                let (row_idx, col_idx) = cursor_pos;
                let remove_pos = row_idx + 1;

                 // Check if line exists
                 if let Some(line) = scene.lines.get(col_idx) {
                     // Check if the position to remove is valid
                     if remove_pos < line.frames.len() {
                         app.send_client_message(ClientMessage::RemoveFrame(col_idx, remove_pos, ActionTiming::Immediate));
                         app.set_status_message(format!("Requested removing frame at ({}, {})", col_idx, remove_pos));
                     } else {
                         app.set_status_message(format!("No frame found at ({}, {}) to remove", col_idx, remove_pos));
                         handled = false; // Indicate nothing was done
                     }
                 } else {
                     app.set_status_message("Invalid line for removing frame".to_string());
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
            // Increment frame length (fixed to 0.25 increments for now)
            KeyCode::Char('>') | KeyCode::Char('.') => {
                let ((top, left), (bottom, right)) = current_selection.bounds();
                let mut modified_lines: std::collections::HashMap<usize, Vec<f64>> = std::collections::HashMap::new();
                let mut frames_changed = 0;
                let scene_total_length = scene.length() as f64; // Get total scene length as f64

                // Iterate over the selection bounds
                for col_idx in left..=right {
                    if let Some(line) = scene.lines.get(col_idx) {
                        let mut current_frames = line.frames.clone();
                        let mut was_modified = false;
                        for row_idx in top..=bottom {
                            if row_idx < current_frames.len() {
                                let current_length = current_frames[row_idx];

                                // Calculate sum of lengths of *other* frames in this line
                                let sum_other_frames: f64 = current_frames.iter().enumerate()
                                    .filter(|(i, _)| *i != row_idx)
                                    .map(|(_, &len)| len)
                                    .sum();

                                // Calculate max allowed length for *this* frame
                                let max_allowed_length = (scene_total_length - sum_other_frames).max(0.01); // Ensure minimum length

                                let new_length = (current_length + 0.25).min(max_allowed_length); // Clamp against max

                                // Only count as changed if the value actually increases
                                if new_length > current_length {
                                    current_frames[row_idx] = new_length;
                                    was_modified = true;
                                    frames_changed += 1;
                                }
                            }
                        }
                        if was_modified {
                            modified_lines.insert(col_idx, current_frames);
                        }
                    }
                }

                // Send messages for modified lines
                for (col, updated_frames) in modified_lines {
                     app.send_client_message(
                        ClientMessage::UpdateLineFrames(col, updated_frames, ActionTiming::Immediate)
                    );
                }

                if frames_changed > 0 {
                    app.set_status_message(format!("Requested increasing length for {} frames (max {})", frames_changed, scene_total_length));
                } else {
                    app.set_status_message("No valid frames/space to increase length".to_string());
                    handled = false;
                }
            }
            // Decrement frame length (fixed to 0.25 increments for now)
            KeyCode::Char('<') | KeyCode::Char(',') => {
                let ((top, left), (bottom, right)) = current_selection.bounds();
                let mut modified_lines: std::collections::HashMap<usize, Vec<f64>> = std::collections::HashMap::new();
                let mut frames_changed = 0;

                for col_idx in left..=right {
                    if let Some(line) = scene.lines.get(col_idx) {
                        let mut current_frames = line.frames.clone();
                        let mut was_modified = false;
                        for row_idx in top..=bottom {
                            if row_idx < current_frames.len() {
                                let current_length = current_frames[row_idx];
                                let new_length = (current_length - 0.25).max(0.01); // Keep minimum
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
                     app.send_client_message(
                        ClientMessage::UpdateLineFrames(
                            col, updated_frames, ActionTiming::Immediate
                        )
                    );
                }

                if frames_changed > 0 {
                    app.set_status_message(format!("Requested decreasing length for {} frames", frames_changed));
                } else {
                    app.set_status_message("No valid frames in selection to decrease length".to_string());
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
            KeyCode::Char('d') => {
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
                let (row_idx, col_idx) = current_selection.cursor_pos();
                current_selection = GridSelection::single(row_idx, col_idx);
                let mut handled_copy = false; // Local handled flag for this block

                if let Some(line) = scene.lines.get(col_idx) {
                    if row_idx < line.frames.len() {
                        // Get length and enabled state locally first
                        let length = line.frames[row_idx];
                        let is_enabled = line.is_frame_enabled(row_idx);

                        // Send request to server for the script content
                        app.send_client_message(ClientMessage::GetScript(col_idx, row_idx));

                        // Update clipboard state to fetching script, storing len/state now
                        app.clipboard = ClipboardState::FetchingScript {
                            col: col_idx,
                            row: row_idx,
                            length,
                            is_enabled,
                        };
                        app.set_status_message(format!("Requesting script for copy: Line {}, Frame {}", col_idx, row_idx));
                        app.add_log(LogLevel::Info, format!("Requested script copy for ({}, {}). Length: {}, Enabled: {}", col_idx, row_idx, length, is_enabled));
                        handled_copy = true; // Successfully initiated copy
                    } else {
                        app.set_status_message("Cannot copy script info from an empty slot".to_string());
                        app.clipboard = ClipboardState::Empty; // Reset clipboard state
                        // handled_copy remains false
                    }
                } else {
                    app.set_status_message("Invalid line index for copy".to_string());
                    app.clipboard = ClipboardState::Empty; // Reset clipboard state
                    // handled_copy remains false
                }
                handled = handled_copy; // Set the main handled flag based on copy success
            }
            KeyCode::Char('p') => {
                 match app.clipboard.clone() { // Clone to work with the value
                     ClipboardState::Ready(copied_data) => {
                         let (target_row, target_col) = current_selection.cursor_pos();
                         current_selection = GridSelection::single(target_row, target_col); // Ensure single cell selection
                         let mut messages_sent = 0;
                         let mut script_pasted = false;

                         if let Some(target_line) = scene.lines.get(target_col) {
                             if target_row < target_line.frames.len() {
                                 // 1. Paste Length
                                 let mut updated_frames = target_line.frames.clone();
                                 if target_row < updated_frames.len() { // Double check bounds
                                     updated_frames[target_row] = copied_data.length;
                                     app.send_client_message(
                                        ClientMessage::UpdateLineFrames(
                                            target_col, updated_frames, ActionTiming::Immediate
                                        )
                                    );
                                     messages_sent += 1;
                                 }

                                 // 2. Paste Enabled/Disabled State
                                 if copied_data.is_enabled {
                                     app.send_client_message(ClientMessage::EnableFrames(target_col, vec![target_row], ActionTiming::Immediate));
                                 } else {
                                     app.send_client_message(ClientMessage::DisableFrames(target_col, vec![target_row], ActionTiming::Immediate));
                                 }
                                 messages_sent += 1;

                                 // 3. Paste Script Content
                                 if let Some(script) = &copied_data.script_content {
                                     app.send_client_message(ClientMessage::SetScript(
                                         target_col,
                                         target_row,
                                         script.clone(),
                                         ActionTiming::Immediate
                                     ));
                                     messages_sent += 1;
                                     script_pasted = true;
                                 } else {
                                     // Script wasn't fetched or available during copy
                                     app.add_log(LogLevel::Warn, format!("Paste attempted for ({}, {}), but script content was not available in clipboard.", target_col, target_row));
                                 };

                                 app.set_status_message(format!(
                                     "Pasted length & state to ({}, {}). {}",
                                     target_col, target_row,
                                     if script_pasted { "Script pasted." } else { "Script paste skipped (not available)." }
                                 ));
                                  app.add_log(LogLevel::Info, format!(
                                     "Pasted length ({}) & state ({}) from ({},{}) to ({}, {}). Script pasted: {}",
                                     copied_data.length, copied_data.is_enabled, copied_data.source_col, copied_data.source_row, target_col, target_row,
                                     script_pasted
                                 ));

                             } else {
                                 app.set_status_message("Cannot paste to an empty slot".to_string());
                                 handled = false;
                             }
                         } else {
                             app.set_status_message("Invalid line index for paste".to_string());
                             handled = false;
                         }
                         // Mark handled if we sent any messages
                         handled = messages_sent > 0;
                     }
                     ClipboardState::FetchingScript { col, row, .. } => {
                         app.set_status_message(format!("Still fetching script from ({}, {}) to copy...", col, row));
                         handled = false;
                     }
                     ClipboardState::Empty => {
                         app.set_status_message("Clipboard is empty. Use 'c' to copy first.".to_string());
                         handled = false;
                     }
                 }
            }
            _ => { handled = false; } 
        }

        if handled {
            // If the selection changed and we handled the event, send update to server.
            if app.interface.components.grid_selection != current_selection {
                 app.interface.components.grid_selection = current_selection;
                 app.send_client_message(ClientMessage::UpdateGridSelection(current_selection));
                 app.add_log(LogLevel::Debug, format!("Sent grid selection update: {:?}", current_selection)); // Use Debug
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
                 app.add_log(LogLevel::Debug, format!("Sent grid selection update (internal change): {:?}", current_selection)); // Use Debug
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

        // Split inner area into table area and a small help line area at the bottom
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            // Allocate 2 lines for help text
            .constraints([ Constraint::Min(0), Constraint::Length(2) ])
            .split(inner_area);
        let table_area = main_chunks[0];
        let help_area = main_chunks[1];

        // Help line explaining keybindings
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);

        // Line 1
        let help_spans_line1 = vec![
            Span::raw("Move: "), Span::styled("↑↓←→ ", key_style),
            Span::raw("Toggle: "), Span::styled("Space ", key_style),
            Span::raw("Edit Script: "), Span::styled("Enter ", key_style),
            Span::raw("Len: "), Span::styled("<", key_style), Span::raw("/"), Span::styled(">", key_style),
            Span::raw("Set Start/End: "), Span::styled("b", key_style), Span::raw("/"), Span::styled("e", key_style),
        ];

        // Line 2
        let help_spans_line2 = vec![
            Span::styled("Shift+Arrows", key_style), Span::raw(":Select  "),
            Span::styled("Esc", key_style), Span::raw(":Reset Sel  "),
            Span::styled("+", key_style), Span::raw("/"), Span::styled("-", key_style), Span::raw(":Ins/Del Frame  "),
            Span::styled("a", key_style), Span::raw("/"), Span::styled("d", key_style), Span::raw(":Add/Rem Line "),
            Span::styled("c", key_style), Span::raw("/"), Span::styled("p", key_style),
            Span::raw(":Copy/Paste Frame"),
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
                frame.render_widget(Paragraph::new("No lines in scene. Use 'a' to add.").yellow().centered(), table_area);
                return;
            }

            let num_lines = lines.len();
            // Determine the maximum number of frames across all lines for table height
            let max_frames = lines.iter().map(|line| line.frames.len()).max().unwrap_or(0);

            // Placeholder message if lines exist but have no frames
            if max_frames == 0 && num_lines > 0 {
                frame.render_widget(
                    Paragraph::new("Lines have no frames. Use '+' to add.")
                    .yellow()
                    .centered(), 
                    table_area
                );
            }

            // Various styles for the table
            let header_style = Style::default().fg(Color::White).bg(Color::Blue).bold();
            let enabled_style = Style::default().fg(Color::White).bg(Color::Green);
            let disabled_style = Style::default().fg(Color::White).bg(Color::Red);
            let cursor_style = Style::default().fg(Color::White).bg(Color::Yellow).bold();
            let peer_cursor_style = Style::default().bg(Color::White).fg(Color::Black); // White BG, Black FG for peer cursor
            let empty_cell_style = Style::default().bg(Color::DarkGray);
            let start_end_marker_style = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);

            // Define characters for the start/end range bar
            let bar_char_active = "▌"; 
            let bar_char_inactive = " ";

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
            let data_rows = (0..max_frames).map(|frame_idx| {
                 let cells = lines.iter().enumerate().map(|(col_idx, line)| {
                    if frame_idx < line.frames.len() {
                        let frame_val = line.frames[frame_idx];
                        let is_enabled = line.is_frame_enabled(frame_idx);
                        let base_style = if is_enabled { enabled_style } else { disabled_style };
                        let is_current_frame = app.server.current_frame_positions.as_ref()
                            .and_then(|positions| positions.get(col_idx))
                            .map_or(false, |&current| current == frame_idx);
                        let should_draw_bar = if let Some(start) = line.start_frame {
                            if let Some(end) = line.end_frame { frame_idx >= start && frame_idx <= end }
                            else { frame_idx >= start }
                        } else { if let Some(end) = line.end_frame { frame_idx <= end } else { false } };
                        let bar_char = if should_draw_bar { bar_char_active } else { bar_char_inactive };
                        let play_marker = if is_current_frame { "▶" } else { " " };
                        let bar_span = Span::styled(bar_char, if should_draw_bar { start_end_marker_style } else { Style::default() });
                        let play_marker_span = Span::raw(play_marker);
                        let value_span = Span::raw(format!("{:.2}", frame_val));
                        let ((top, left), (bottom, right)) = app.interface.components.grid_selection.bounds();
                        let is_selected_locally = frame_idx >= top && frame_idx <= bottom && col_idx >= left && col_idx <= right;
                        let is_local_cursor = (frame_idx, col_idx) == app.interface.components.grid_selection.cursor_pos();

                        // Find if a peer's cursor is on this cell
                        let peer_on_cell: Option<(String, GridSelection)> = app.server.peer_sessions.iter()
                            .filter_map(|(name, peer_state)| peer_state.grid_selection.map(|sel| (name.clone(), sel)))
                            .find(|(_, peer_selection)| (frame_idx, col_idx) == peer_selection.cursor_pos());

                        // Check if any peer is editing this specific cell *before* the main logic block
                        let is_being_edited_by_peer = app.server.peer_sessions.values()
                            .any(|peer_state| peer_state.editing_frame == Some((col_idx, frame_idx)));

                        // Determine final style and content based on state
                        let mut final_style;
                        let content_span;

                        // 1. Determine Base Style & Content
                        if is_local_cursor || is_selected_locally {
                            final_style = cursor_style;
                            content_span = value_span;
                        } else if let Some((peer_name, _)) = peer_on_cell {
                            final_style = peer_cursor_style;
                            let name_fragment = peer_name.chars().take(4).collect::<String>();
                            content_span = Span::raw(format!("{:<4}", name_fragment)); // Pad to left align
                        } else {
                            final_style = base_style;
                            content_span = value_span;
                        }

                        // 2. Apply Animation Overlay (if applicable)
                        if is_being_edited_by_peer && !(is_local_cursor || is_selected_locally) {
                            // Use milliseconds for faster animation (e.g., 500ms cycle)
                            let phase = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() % 500;
                            let current_fg = final_style.fg.unwrap_or(Color::White); // Get FG from determined style
                            let animated_fg = if phase < 250 { current_fg } else { Color::Red }; // Flash Red
                            final_style = final_style.fg(animated_fg); // Apply animation to the correct base style
                        }

                        // 3. Construct Line and Cell
                        let line_spans = vec![bar_span, play_marker_span, Span::raw(" "), content_span];
                        let cell_content = Line::from(line_spans).alignment(ratatui::layout::Alignment::Center);

                        Cell::from(cell_content).style(final_style)
                    } else {
                        // Empty Cell Logic 
                        let peer_on_cell: Option<(String, GridSelection)> = app.server.peer_sessions.iter()
                            .filter_map(|(name, peer_state)| peer_state.grid_selection.map(|sel| (name.clone(), sel)))
                            .find(|(_, peer_selection)| (frame_idx, col_idx) == peer_selection.cursor_pos());

                         let mut final_style;
                         let cell_content;
                         let cell_content_span; // Use a different name

                         let is_local_cursor = (frame_idx, col_idx) == app.interface.components.grid_selection.cursor_pos();
                         let is_being_edited_by_peer = app.server.peer_sessions.values()
                                .any(|peer_state| peer_state.editing_frame == Some((col_idx, frame_idx)));

                         // 1. Determine Base Style & Content Span
                         if is_local_cursor {
                             final_style = cursor_style;
                             cell_content_span = Span::raw(""); // Empty content
                         } else if let Some((peer_name, _)) = peer_on_cell {
                             final_style = peer_cursor_style;
                             let name_fragment = peer_name.chars().take(4).collect::<String>();
                             cell_content_span = Span::raw(format!("{:<4}", name_fragment));
                         } else {
                             final_style = empty_cell_style;
                             cell_content_span = Span::raw("");
                         }

                         // 2. Apply Animation Overlay (if applicable and not local cursor)
                         if is_being_edited_by_peer && !is_local_cursor && cell_content_span.width() > 0 { // Only animate if there's peer name content
                            // Use milliseconds for faster animation (e.g., 500ms cycle)
                             let phase = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() % 500;
                             let current_fg = final_style.fg.unwrap_or(Color::White); // Should be Black from peer_cursor_style
                             let animated_fg = if phase < 250 { current_fg } else { Color::Red }; // Flash Red
                             final_style = final_style.fg(animated_fg);
                         }

                         // 3. Construct Line and Cell
                         cell_content = Line::from(cell_content_span).alignment(ratatui::layout::Alignment::Center);
                         Cell::from(cell_content).style(final_style)
                    }
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