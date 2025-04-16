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
use tui_textarea::{Input, Key};

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
        let mut current_selection = app.interface.components.grid_selection;
        let mut handled = false; // Initialize handled to false
        let is_shift_pressed = key_event.modifiers.contains(KeyModifiers::SHIFT);

        // --- Handle Frame Length Edit Mode FIRST ---
        if app.interface.components.frame_length_edit_state.is_active {
            // Borrow necessary parts immutably first
            let scene_len = app.editor.scene.as_ref().map(|s| s.length());
            let target_cell_for_lookup = app.interface.components.frame_length_edit_state.target_cell;
            let current_frames_for_lookup = app.editor.scene.as_ref()
                .and_then(|s| s.lines.get(target_cell_for_lookup.1))
                .map(|l| l.frames.clone()); // Clone needed frames

            // Now borrow the state mutably
            let edit_state = &mut app.interface.components.frame_length_edit_state;

            match key_event.into() { // Convert to tui_textarea::Input
                Input { key: Key::Esc, .. } => {
                    edit_state.deactivate();
                    app.set_status_message("Frame length edit cancelled.".to_string());
                    handled = true; // Esc is handled here
                }
                Input { key: Key::Enter, .. } => {
                    let input_str = edit_state.input_area.lines().get(0).map_or("", |s| s.as_str()).trim();
                    let mut status_update = String::new();
                    let target_cell = edit_state.target_cell;
                    let mut should_deactivate = true;
                    let mut message_to_send: Option<ClientMessage> = None;

                    match input_str.parse::<f64>() {
                        Ok(new_length_input) => {
                            if new_length_input <= 0.0 {
                                edit_state.status_message = Some("Length must be positive.".to_string());
                                status_update = "Invalid length: Must be positive.".to_string();
                                should_deactivate = false;
                            } else {
                                // Use the immutably borrowed data
                                if let (Some(scene_total_length_f64), Some(line_frames)) = (scene_len.map(|l| l as f64), current_frames_for_lookup) {
                                     let (row_idx, col_idx) = target_cell;
                                     if row_idx < line_frames.len() {
                                        let sum_other_frames: f64 = line_frames.iter().enumerate()
                                            .filter(|(i, _)| *i != row_idx)
                                            .map(|(_, &len)| len)
                                            .sum();
                                        let max_allowed_length = (scene_total_length_f64 - sum_other_frames).max(0.01);
                                        let final_new_length = new_length_input.clamp(0.01, max_allowed_length);

                                        if (final_new_length - line_frames[row_idx]).abs() > f64::EPSILON {
                                            let mut updated_frames = line_frames.clone();
                                            updated_frames[row_idx] = final_new_length;
                                            message_to_send = Some(ClientMessage::UpdateLineFrames(
                                                col_idx, updated_frames, ActionTiming::Immediate
                                            ));
                                            status_update = format!(
                                                "Requested setting length of ({}, {}) to {:.2}", col_idx, row_idx, final_new_length
                                            );
                                        } else {
                                            status_update = format!(
                                                "Length for ({}, {}) already {:.2}", col_idx, row_idx, final_new_length
                                            );
                                        }
                                    } else {
                                        status_update = "Target frame index out of bounds.".to_string();
                                    }
                                } else {
                                    status_update = "Scene/Line data not available for validation.".to_string();
                                }
                            }
                        }
                        Err(_) => {
                            edit_state.status_message = Some("Invalid number format.".to_string());
                            status_update = "Invalid length: Please enter a number (e.g., 1.5).".to_string();
                            should_deactivate = false;
                        }
                    }

                    if should_deactivate {
                         edit_state.deactivate();
                    }
                    app.set_status_message(status_update);
                    if let Some(msg) = message_to_send {
                        app.send_client_message(msg);
                    }
                    handled = true; // Enter is handled here
                }
                input => {
                    if matches!(input, Input { key: Key::Enter, .. } | Input { key: Key::Char('m'), ctrl: true, ..}) {
                         handled = true; // Ignore Ctrl+M/Enter if not caught above
                    } else {
                        let modified = edit_state.input_area.input(input);
                        if modified {
                            edit_state.status_message = None;
                        }
                        handled = true; // Input into the text box is handled
                    }
                }
            }
            // If frame length edit was active, we've handled the key event here
            // Update selection state regardless of *which* key was pressed in this mode
             app.interface.components.grid_selection = current_selection;
             // We don't send updates while editing length, only potentially on Enter
             return Ok(true); // Return early as we handled the key in edit mode
        }

        // --- Normal Mode Key Handling (Frame Length Edit NOT active) ---

        match key_event.code {
             // Add line - Always possible
            KeyCode::Char('a') => {
                app.send_client_message(ClientMessage::SchedulerControl(
                    bubocorelib::schedule::SchedulerMessage::AddLine
                ));
                app.set_status_message("Requested adding line".to_string());
                handled = true;
            }
             // Remove line - Requires scene, but not necessarily frames
            KeyCode::Char('d') => {
                let mut last_line_index_opt : Option<usize> = None;
                let mut status = "Scene not loaded".to_string();
                if let Some(scene) = &app.editor.scene {
                     if scene.lines.len() > 0 {
                        last_line_index_opt = Some(scene.lines.len() - 1);
                        status = format!("Requested removing line {}", last_line_index_opt.unwrap());
                        handled = true;
                    } else {
                         status = "No lines to remove".to_string();
                         handled = false; // Technically handled, but did nothing
                    }
                } else {
                    handled = false;
                }

                if handled {
                     if let Some(last_line_index) = last_line_index_opt {
                        // Reset selection if necessary
                        if current_selection.end.1 >= last_line_index && last_line_index > 0 {
                            current_selection.end.1 = last_line_index - 1;
                            current_selection.start.1 = current_selection.start.1.min(last_line_index - 1);
                        } else if last_line_index == 0 {
                            current_selection = GridSelection::single(0,0); // Reset if deleting last line
                        }

                        app.send_client_message(ClientMessage::SchedulerControl(
                            bubocorelib::schedule::SchedulerMessage::RemoveLine(last_line_index, ActionTiming::Immediate)
                        ));
                    }
                }
                 app.set_status_message(status);
            }
             // Reset selection
            KeyCode::Esc => {
                if !current_selection.is_single() {
                    current_selection = GridSelection::single(current_selection.start.0, current_selection.start.1);
                    app.set_status_message("Selection reset to single cell (at start)".to_string());
                    handled = true;
                } else {
                    handled = false; // Already single, do nothing
                }
            }

            // --- Keys requiring scene and potentially lines/frames ---
            _ => {
                 // Get scene ONLY if needed for the remaining keys
                 if let Some(scene) = app.editor.scene.as_ref() {
                     let num_lines = scene.lines.len();
                     if num_lines > 0 {
                         // Match remaining keys that require a scene with lines
                         match key_event.code {
                            KeyCode::Char('+') => {
                                let cursor_pos = current_selection.cursor_pos();
                                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                                let (row_idx, col_idx) = cursor_pos;
                                let insert_pos = row_idx + 1;
                                let default_insert_length = 1.0;

                                if let Some(line) = scene.lines.get(col_idx) {
                                    if insert_pos <= line.frames.len() {
                                        let current_line_beats: f64 = line.frames.iter().sum();
                                        let potential_line_beats = current_line_beats + default_insert_length;
                                        let scene_total_length = scene.length() as f64;

                                        if potential_line_beats <= scene_total_length {
                                            app.send_client_message(ClientMessage::InsertFrame(col_idx, insert_pos, ActionTiming::Immediate));
                                            app.set_status_message(format!("Requested inserting frame at ({}, {})", col_idx, insert_pos));
                                            handled = true;
                                        } else {
                                            app.set_status_message(format!("Cannot insert frame: exceeds scene length ({:.2}/{:.2})", potential_line_beats, scene_total_length));
                                            // handled = false; // Keep false
                                        }
                                    } else {
                                        app.add_log(LogLevel::Warn, format!("Attempted to insert frame at invalid position {} in line {}", insert_pos, col_idx));
                                        app.set_status_message("Cannot insert frame here".to_string());
                                        // handled = false;
                                    }
                                } else {
                                    app.set_status_message("Invalid line for adding frame".to_string());
                                    // handled = false;
                                }
                            }
                            KeyCode::Char('-') => {
                                let cursor_pos = current_selection.cursor_pos();
                                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                                let (row_idx, col_idx) = cursor_pos;
                                let remove_pos = row_idx + 1;

                                if let Some(line) = scene.lines.get(col_idx) {
                                    if remove_pos < line.frames.len() {
                                        app.send_client_message(ClientMessage::RemoveFrame(col_idx, remove_pos, ActionTiming::Immediate));
                                        app.set_status_message(format!("Requested removing frame at ({}, {})", col_idx, remove_pos));
                                        handled = true;
                                    } else {
                                        app.set_status_message(format!("No frame found at ({}, {}) to remove", col_idx, remove_pos));
                                        // handled = false;
                                    }
                                } else {
                                    app.set_status_message("Invalid line for removing frame".to_string());
                                    // handled = false;
                                }
                            }
                            KeyCode::Enter => {
                                let cursor_pos = current_selection.cursor_pos();
                                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                                let (row_idx, col_idx) = cursor_pos;
                                let mut status_update = "Scene not loaded".to_string(); // Default message

                                if let Some(line) = scene.lines.get(col_idx) {
                                    if row_idx < line.frames.len() {
                                        app.send_client_message(ClientMessage::GetScript(col_idx, row_idx));
                                        app.send_client_message(ClientMessage::StartedEditingFrame(col_idx, row_idx));
                                        status_update = format!("Requested script for Line {}, Frame {}", col_idx, row_idx);
                                        handled = true;
                                    } else {
                                        status_update = "Cannot request script for an empty slot".to_string();
                                        // handled = false;
                                    }
                                } else {
                                     status_update = "Invalid line index".to_string();
                                     // handled = false;
                                }
                                app.set_status_message(status_update);
                            }
                             KeyCode::Char('b') => {
                                 let cursor_pos = current_selection.cursor_pos();
                                 current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                                 let (row_idx, col_idx) = cursor_pos;
                                 if let Some(line) = scene.lines.get(col_idx) {
                                     if row_idx < line.frames.len() {
                                         let start_frame_val = if line.start_frame == Some(row_idx) { None } else { Some(row_idx) };
                                         app.send_client_message(
                                            ClientMessage::SetLineStartFrame(col_idx, start_frame_val, ActionTiming::Immediate)
                                        );
                                         app.set_status_message(format!("Requested setting start frame to {:?} for Line {}", start_frame_val, col_idx));
                                         handled = true;
                                     } else {
                                         app.set_status_message("Cannot set start frame on empty slot".to_string());
                                         // handled = false;
                                     }
                                 } // else { handled = false; } Implicitly false if line doesn't exist
                            }
                             KeyCode::Char('e') => {
                                 let cursor_pos = current_selection.cursor_pos();
                                 current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                                 let (row_idx, col_idx) = cursor_pos;
                                 if let Some(line) = scene.lines.get(col_idx) {
                                     if row_idx < line.frames.len() {
                                         let end_frame_val = if line.end_frame == Some(row_idx) { None } else { Some(row_idx) };
                                         app.send_client_message(
                                            ClientMessage::SetLineEndFrame(col_idx, end_frame_val, ActionTiming::Immediate)
                                        );
                                         app.set_status_message(format!("Requested setting end frame to {:?} for Line {}", end_frame_val, col_idx));
                                         handled = true;
                                     } else {
                                         app.set_status_message("Cannot set end frame on empty slot".to_string());
                                         // handled = false;
                                     }
                                 } // else { handled = false; }
                            }
                            KeyCode::Down => {
                                let mut end_pos = current_selection.end;
                                let mut moved = false;
                                if let Some(line) = scene.lines.get(end_pos.1) {
                                    let frames_in_col = line.frames.len();
                                    if frames_in_col > 0 {
                                        let next_row = min(end_pos.0 + 1, frames_in_col - 1);
                                        if next_row != end_pos.0 {
                                             end_pos.0 = next_row;
                                             moved = true;
                                        }
                                    }
                                }
                                if moved {
                                    if is_shift_pressed { current_selection.end = end_pos; }
                                    else { current_selection = GridSelection::single(end_pos.0, end_pos.1); }
                                    handled = true;
                                } // else handled remains false
                            }
                            KeyCode::Up => {
                                let mut end_pos = current_selection.end;
                                let next_row = end_pos.0.saturating_sub(1);
                                if next_row != end_pos.0 {
                                     end_pos.0 = next_row;
                                     if is_shift_pressed { current_selection.end = end_pos; }
                                     else { current_selection = GridSelection::single(end_pos.0, end_pos.1); }
                                     handled = true;
                                 } // else handled remains false
                            }
                            KeyCode::Left => {
                                let mut end_pos = current_selection.end;
                                let next_col = end_pos.1.saturating_sub(1);
                                if next_col != end_pos.1 {
                                     let frames_in_next_col = scene.lines.get(next_col).map_or(0, |s| s.frames.len());
                                     end_pos.0 = min(end_pos.0, frames_in_next_col.saturating_sub(1));
                                     end_pos.1 = next_col;
                                     if is_shift_pressed { current_selection.end = end_pos; }
                                     else { current_selection = GridSelection::single(end_pos.0, end_pos.1); }
                                     handled = true;
                                 } // else handled remains false
                            }
                            KeyCode::Right => {
                                let mut end_pos = current_selection.end;
                                let next_col = min(end_pos.1 + 1, num_lines.saturating_sub(1));
                                 if next_col != end_pos.1 {
                                     let frames_in_next_col = scene.lines.get(next_col).map_or(0, |s| s.frames.len());
                                     end_pos.0 = min(end_pos.0, frames_in_next_col.saturating_sub(1));
                                     end_pos.1 = next_col;
                                     if is_shift_pressed { current_selection.end = end_pos; }
                                     else { current_selection = GridSelection::single(end_pos.0, end_pos.1); }
                                     handled = true;
                                 } // else handled remains false
                            }
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
                                                 if is_enabled { to_disable.entry(col_idx).or_default().push(row_idx); }
                                                 else { to_enable.entry(col_idx).or_default().push(row_idx); }
                                                 frames_toggled += 1;
                                                 handled = true; // Handled if we find a frame to toggle
                                             }
                                         }
                                     }
                                 }

                                 // Send messages only if needed
                                 let mut messages_sent = false;
                                 for (col, rows) in to_disable { if !rows.is_empty() { app.send_client_message(ClientMessage::DisableFrames(col, rows, ActionTiming::Immediate)); messages_sent = true; }}
                                 for (col, rows) in to_enable { if !rows.is_empty() { app.send_client_message(ClientMessage::EnableFrames(col, rows, ActionTiming::Immediate)); messages_sent = true; }}

                                 if messages_sent {
                                     app.set_status_message(format!("Requested toggling {} frames", frames_toggled));
                                 } else {
                                     app.set_status_message("No valid frames in selection to toggle".to_string());
                                     handled = false; // Reset handled if nothing was actually sent
                                 }
                            }
                             KeyCode::Char('c') => {
                                let (row_idx, col_idx) = current_selection.cursor_pos();
                                current_selection = GridSelection::single(row_idx, col_idx);
                                let mut handled_copy = false;

                                if let Some(line) = scene.lines.get(col_idx) {
                                    if row_idx < line.frames.len() {
                                        let length = line.frames[row_idx];
                                        let is_enabled = line.is_frame_enabled(row_idx);
                                        app.send_client_message(ClientMessage::GetScript(col_idx, row_idx));
                                        app.clipboard = ClipboardState::FetchingScript { col: col_idx, row: row_idx, length, is_enabled };
                                        app.set_status_message(format!("Requesting script for copy: Line {}, Frame {}", col_idx, row_idx));
                                        app.add_log(LogLevel::Info, format!("Requested script copy for ({}, {}). Length: {}, Enabled: {}", col_idx, row_idx, length, is_enabled));
                                        handled_copy = true;
                                    } else {
                                        app.set_status_message("Cannot copy script info from an empty slot".to_string());
                                        app.clipboard = ClipboardState::Empty;
                                    }
                                } else {
                                    app.set_status_message("Invalid line index for copy".to_string());
                                    app.clipboard = ClipboardState::Empty;
                                }
                                handled = handled_copy;
                            }
                             KeyCode::Char('p') => {
                                 match app.clipboard.clone() {
                                     ClipboardState::Ready(copied_data) => {
                                         let mut log_message = String::new();
                                         let (target_row, target_col) = current_selection.cursor_pos();
                                         current_selection = GridSelection::single(target_row, target_col);
                                         let mut messages_sent = 0;
                                         let mut script_pasted = false;

                                         if let Some(target_line) = scene.lines.get(target_col) {
                                             if target_row < target_line.frames.len() {
                                                 log_message = format!("Pasting frame data: Length = {:.2}, Enabled = {}, HasScript = {}", copied_data.length, copied_data.is_enabled, copied_data.script_content.is_some());
                                                 // 1. Paste Length
                                                 let mut updated_frames = target_line.frames.clone();
                                                 updated_frames[target_row] = copied_data.length;
                                                 app.send_client_message(ClientMessage::UpdateLineFrames(target_col, updated_frames, ActionTiming::Immediate));
                                                 messages_sent += 1;
                                                 // 2. Paste Enabled/Disabled State
                                                 if copied_data.is_enabled { app.send_client_message(ClientMessage::EnableFrames(target_col, vec![target_row], ActionTiming::Immediate)); }
                                                 else { app.send_client_message(ClientMessage::DisableFrames(target_col, vec![target_row], ActionTiming::Immediate)); }
                                                 messages_sent += 1;
                                                 // 3. Paste Script Content
                                                 if let Some(script) = &copied_data.script_content {
                                                     app.send_client_message(ClientMessage::SetScript(target_col, target_row, script.clone(), ActionTiming::Immediate));
                                                     messages_sent += 1;
                                                     script_pasted = true;
                                                 } else {
                                                     app.add_log(LogLevel::Warn, format!("Paste attempted for ({}, {}), but script content was not available in clipboard.", target_col, target_row));
                                                 }
                                                 app.set_status_message(format!(
                                                     "Pasted length & state to ({}, {}). {}",
                                                     target_col, target_row, if script_pasted { "Script pasted." } else { "Script paste skipped (not available)." }
                                                 ));
                                             } else { app.set_status_message("Cannot paste to an empty slot".to_string()); }
                                         } else { app.set_status_message("Invalid line index for paste".to_string()); }

                                         handled = messages_sent > 0;
                                         if !log_message.is_empty() { app.add_log(LogLevel::Debug, log_message); }
                                     }
                                     ClipboardState::FetchingScript { col, row, .. } => { app.set_status_message(format!("Still fetching script from ({}, {}) to copy...", col, row)); }
                                     ClipboardState::Empty => { app.set_status_message("Clipboard is empty. Use 'c' to copy first.".to_string()); }
                                 }
                                handled = true; // Pasting (or attempting to) is always handled
                             }
                            KeyCode::Char('l') => { // Edit Length
                                let cursor_pos = current_selection.cursor_pos();
                                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                                let (row_idx, col_idx) = cursor_pos;

                                if let Some(line) = scene.lines.get(col_idx) {
                                    if row_idx < line.frames.len() {
                                        let current_length = line.frames[row_idx];
                                        app.interface.components.frame_length_edit_state.activate(cursor_pos, current_length);
                                        app.set_status_message(format!("Editing length for ({}, {}). Enter value...", col_idx, row_idx));
                                        handled = true;
                                    } else {
                                        app.set_status_message("Cannot edit length of empty slot.".to_string());
                                        // handled = false;
                                    }
                                } else {
                                    app.set_status_message("Invalid line for editing length.".to_string());
                                    // handled = false;
                                }
                            }
                            _ => { /* handled remains false */ }
                         }
                     } else {
                         // Scene exists, but no lines. Only handle keys that don't need lines.
                          match key_event.code {
                              // You could potentially allow other keys here if they make sense
                              // without lines, but currently only 'a' and 'd' (and Esc) are
                              // handled outside this block.
                              _ => { /* handled remains false */ }
                          }
                     }
                 } // else scene doesn't exist - handled remains false for these keys
            } // End wildcard match arm
        } // End outer match

        // --- Update selection state if handled ---
        // This block needs to run *after* all potential handlers
        if handled {
            // If the selection changed, send update.
            if app.interface.components.grid_selection != current_selection {
                 app.interface.components.grid_selection = current_selection;
                 app.send_client_message(ClientMessage::UpdateGridSelection(current_selection));
                 app.add_log(LogLevel::Debug, format!("Sent grid selection update: {:?}", current_selection));
            } else {
                 // If handled but selection didn't change (e.g. Enter, +, - etc.)
                 // ensure local state is up-to-date even if no network message needed
                 app.interface.components.grid_selection = current_selection;
            }
        } else {
            // If not handled, but selection was potentially changed INTERNALLY by a handler
            // (like +, -, b, e which reset to single cell before potentially failing),
            // ensure the local state reflects this internal change.
            if app.interface.components.grid_selection != current_selection {
                app.interface.components.grid_selection = current_selection;
                 // OPTIONAL: Send update even if key wasn't primarily for movement,
                 // if selection changed internally? Decide based on desired behavior.
                 // For now, let's send it to keep client/server consistent.
                 app.send_client_message(ClientMessage::UpdateGridSelection(current_selection));
                 app.add_log(LogLevel::Debug, format!("Sent grid selection update (internal change): {:?}", current_selection));
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

        // --- Define Layout --- 
        let mut constraints = vec![Constraint::Min(0)]; // Table Area
        let is_editing_length = app.interface.components.frame_length_edit_state.is_active;
        if is_editing_length {
            constraints.push(Constraint::Length(3)); // Frame Length Input Area
        }
        constraints.push(Constraint::Length(2)); // Help Area

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner_area);

        let table_area = main_chunks[0];
        let mut current_chunk_index = 1;
        let length_input_area = if is_editing_length {
            let area = main_chunks[current_chunk_index];
            current_chunk_index += 1;
            Some(area)
        } else {
            None
        };
        let help_area = main_chunks[current_chunk_index];

        // --- Help line explaining keybindings --- 
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);

        // Line 1 (updated)
        let help_spans_line1 = vec![
            Span::raw("Move: "), Span::styled("↑↓←→ ", key_style),
            Span::raw("Toggle: "), Span::styled("Space ", key_style),
            Span::raw("Edit Script: "), Span::styled("Enter ", key_style),
            // Span::raw("Len: "), Span::styled("<", key_style), Span::raw("/"), Span::styled(">', key_style),
            Span::raw("Set Len: "), Span::styled("l ", key_style),
            Span::raw("Set Start/End: "), Span::styled("b", key_style), Span::raw("/"), Span::styled("e", key_style),
        ];

        // Line 2 (unchanged)
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

        // --- Grid table (requiring scene data) --- 
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
                        
                        let current_frame_for_line = app.server.current_frame_positions.as_ref()
                            .and_then(|positions| positions.get(col_idx))
                            .copied()
                            .unwrap_or(usize::MAX); // Use MAX as sentinel for unknown/past
                        
                        // Simplified play marker logic:
                        let is_head_on_this_frame = current_frame_for_line == frame_idx;
                        let play_marker = if is_head_on_this_frame { "▶" } else { " " };
                        let play_marker_span = Span::raw(play_marker);

                        // Hourglass logic (check if head is past, only for the last frame)
                        let last_frame_index = line.frames.len().saturating_sub(1);
                        let is_head_past_last_frame = current_frame_for_line == usize::MAX; // Check sentinel
                        let is_this_the_last_frame = frame_idx == last_frame_index;
                        
                        // Determine base content and style
                        let mut content_span;
                        let mut cell_base_style;
                        
                        if is_this_the_last_frame && is_head_past_last_frame {
                            content_span = Span::raw("⏳"); // Show hourglass when waiting for loop
                            cell_base_style = base_style.dim(); // Dim the style
                        } else {
                            content_span = Span::raw(format!("{:.2}", frame_val)); // Use frame_val from line.frames
                            cell_base_style = base_style;
                        }
                        
                        let ((top, left), (bottom, right)) = app.interface.components.grid_selection.bounds();
                        let is_selected_locally = frame_idx >= top && frame_idx <= bottom && col_idx >= left && col_idx <= right;
                        let is_local_cursor = (frame_idx, col_idx) == app.interface.components.grid_selection.cursor_pos();

                        // Find if a peer's cursor is on this cell
                        let peer_on_cell: Option<(String, GridSelection)> = app.server.peer_sessions.iter()
                            .filter_map(|(name, peer_state)| peer_state.grid_selection.map(|sel| (name.clone(), sel)))
                            .find(|(_, peer_selection)| (frame_idx, col_idx) == peer_selection.cursor_pos());

                        // Check if any peer is editing this specific cell
                        let is_being_edited_by_peer = app.server.peer_sessions.values()
                            .any(|peer_state| peer_state.editing_frame == Some((col_idx, frame_idx)));

                        // Determine final style and potentially override content based on selection/peer state
                        let mut final_style;
                        if is_local_cursor || is_selected_locally {
                            final_style = cursor_style;
                            // Keep original content_span if selected (could be frame value or hourglass)
                        } else if let Some((peer_name, _)) = peer_on_cell {
                            final_style = peer_cursor_style;
                            let name_fragment = peer_name.chars().take(4).collect::<String>();
                            content_span = Span::raw(format!("{:<4}", name_fragment)); // Override content with peer name
                        } else {
                            final_style = cell_base_style; // Use the base style determined earlier
                        }

                        // Apply Animation Overlay (if applicable)
                        if is_being_edited_by_peer && !(is_local_cursor || is_selected_locally) {
                            let phase = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() % 500;
                            let current_fg = final_style.fg.unwrap_or(Color::White);
                            let animated_fg = if phase < 250 { current_fg } else { Color::Red };
                            final_style = final_style.fg(animated_fg);
                        }

                        // Calculate the start/end bar display
                        let should_draw_bar = if let Some(start) = line.start_frame {
                            if let Some(end) = line.end_frame { frame_idx >= start && frame_idx <= end }
                            else { frame_idx >= start }
                        } else { if let Some(end) = line.end_frame { frame_idx <= end } else { false } };
                        let bar_char = if should_draw_bar { bar_char_active } else { bar_char_inactive };

                        // Construct Line and Cell
                        let bar_span = Span::styled(bar_char, if should_draw_bar { start_end_marker_style } else { Style::default() });
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

        // --- Render Frame Length Input Area (if active) --- 
        if let Some(input_render_area) = length_input_area {
            if is_editing_length {
                let edit_state = &app.interface.components.frame_length_edit_state;
                let mut textarea_clone = edit_state.input_area.clone(); // Clone for rendering

                // Update block title with status/error if present
                if let Some(status) = &edit_state.status_message {
                    textarea_clone.set_block(
                         ratatui::widgets::Block::default()
                             .borders(ratatui::widgets::Borders::ALL)
                             .title(format!(
                                 " Enter Frame Length ({}) (Esc: Cancel, Enter: Confirm) ",
                                 status
                             ))
                             .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red)) // Use red for error indication
                     );
                }
                frame.render_widget(&textarea_clone, input_render_area);
            }
        }
    }
}