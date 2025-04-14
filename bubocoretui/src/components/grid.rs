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
use bubocorelib::server::client::ClientMessage;
use bubocorelib::shared_types::GridSelection;
use std::cmp::min;
use crate::components::logs::LogLevel;
use crate::app::ClipboardState;

/// Component representing the pattern grid, what is currently being played/edited
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
    ///   - `+`: Sends a message to the server to add a default step (length 1.0) to the current sequence.
    ///   - `-`: Sends a message to the server to remove the last step from the current sequence.
    ///   - Arrow keys (`Up`, `Down`, `Left`, `Right`): Navigates the grid cursor.
    ///   - Shift + Arrow keys: Extend the selection range.
    ///   - `Space`: Sends a message to the server to toggle the enabled/disabled state of the selected step.
    ///   - `Enter`: Sends a message to request the script for the selected step and edit it.
    ///   - `<` / `,`: Decrease step length.
    ///   - `>` / `.`: Increase step length.
    ///   - `b`: Mark selected step as the sequence start.
    ///   - `e`: Mark selected step as the sequence end.
    ///   - `a`: Add a new sequence.
    ///   - `d`: Remove the last sequence.
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
        // Get pattern data, but don't exit immediately if empty
        let pattern_opt = app.editor.pattern.as_ref();
        let num_cols = pattern_opt.map_or(0, |p| p.lines.len());

        // Handle 'a' regardless of whether sequences exist
        if key_event.code == KeyCode::Char('a') {
             // Send the request to add a sequence; the server will create the default one.
            app.send_client_message(ClientMessage::SchedulerControl(
                bubocorelib::schedule::SchedulerMessage::AddSequence
            ));
            app.set_status_message("Requested adding sequence".to_string());
            return Ok(true);
        }

        // --- For other keys, require a pattern and at least one sequence ---
        let pattern = match pattern_opt {
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
            // Add a new step to the sequence (default length 1.0) after the cursor
            KeyCode::Char('+') => {
                 let cursor_pos = current_selection.cursor_pos();
                 current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1); // Keep selection single
                 let (row_idx, col_idx) = cursor_pos;
                 let insert_pos = row_idx + 1;

                 // Check if sequence exists (redundant with outer pattern check but safe)
                 if let Some(sequence) = pattern.lines.get(col_idx) {
                     // Check if the insert position is valid (can be equal to len for appending)
                     if insert_pos <= sequence.frames.len() {
                         app.send_client_message(ClientMessage::InsertStep(col_idx, insert_pos));
                         app.set_status_message(format!("Requested inserting step at ({}, {})", col_idx, insert_pos));
                     } else {
                         app.add_log(LogLevel::Warn, format!("Attempted to insert step at invalid position {} in sequence {}", insert_pos, col_idx));
                         app.set_status_message("Cannot insert step here".to_string());
                         handled = false;
                     }
                 } else {
                     app.set_status_message("Invalid sequence for adding step".to_string());
                     handled = false;
                 }
            }
            // Remove the step immediately AFTER the cursor position
            KeyCode::Char('-') => {
                let cursor_pos = current_selection.cursor_pos();
                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1); // Keep selection single
                let (row_idx, col_idx) = cursor_pos;
                let remove_pos = row_idx + 1;

                 // Check if sequence exists
                 if let Some(sequence) = pattern.lines.get(col_idx) {
                     // Check if the position to remove is valid
                     if remove_pos < sequence.frames.len() {
                         app.send_client_message(ClientMessage::RemoveStep(col_idx, remove_pos));
                         app.set_status_message(format!("Requested removing step at ({}, {})", col_idx, remove_pos));
                     } else {
                         app.set_status_message(format!("No step found at ({}, {}) to remove", col_idx, remove_pos));
                         handled = false; // Indicate nothing was done
                     }
                 } else {
                     app.set_status_message("Invalid sequence for removing step".to_string());
                     handled = false;
                 }
            }
            // Request the script for the selected step form the server and edit it
            KeyCode::Enter => {
                let cursor_pos = current_selection.cursor_pos();
                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                let (row_idx, col_idx) = cursor_pos;
                let status_update: Option<String>;
                if let Some(pattern) = &app.editor.pattern {
                    if let Some(sequence) = pattern.lines.get(col_idx) {
                        if row_idx < sequence.frames.len() {
                            // Send request to server for the script content
                            app.send_client_message(ClientMessage::GetScript(col_idx, row_idx));
                            // Also notify server that we START editing this step
                            app.send_client_message(ClientMessage::StartedEditingStep(col_idx, row_idx));
                            status_update = Some(format!("Requested script for Seq {}, Step {}", col_idx, row_idx));
                        } else {
                            status_update = Some("Cannot request script for an empty slot".to_string());
                            handled = false;
                        }
                    } else {
                         status_update = Some("Invalid sequence index".to_string());
                         handled = false;
                    }
                } else {
                    status_update = Some("Pattern not loaded".to_string());
                    handled = false;
                }

                if let Some(status) = status_update { app.set_status_message(status); }
                // Note: We don't switch to the editor here. We wait for the server response.
            }
            // Increment step length (fixed to 0.25 increments for now)
            KeyCode::Char('>') | KeyCode::Char('.') => {
                let ((top, left), (bottom, right)) = current_selection.bounds();
                let mut modified_sequences: std::collections::HashMap<usize, Vec<f64>> = std::collections::HashMap::new();
                let mut steps_changed = 0;

                // Iterate over the selection bounds
                for col_idx in left..=right {
                    if let Some(sequence) = pattern.lines.get(col_idx) {
                        let mut current_steps = sequence.frames.clone();
                        let mut was_modified = false;
                        for row_idx in top..=bottom {
                            if row_idx < current_steps.len() {
                                let current_length = current_steps[row_idx];
                                let new_length = current_length + 0.25;
                                current_steps[row_idx] = new_length;
                                was_modified = true;
                                steps_changed += 1;
                            }
                        }
                        if was_modified {
                            modified_sequences.insert(col_idx, current_steps);
                        }
                    }
                }

                // Send messages for modified sequences
                for (col, updated_steps) in modified_sequences {
                     app.send_client_message(ClientMessage::UpdateSequenceSteps(col, updated_steps));
                }

                if steps_changed > 0 {
                    app.set_status_message(format!("Requested increasing length for {} steps", steps_changed));
                } else {
                    app.set_status_message("No valid steps in selection to increase length".to_string());
                    handled = false;
                }
            }
            // Decrement step length (fixed to 0.25 increments for now)
            KeyCode::Char('<') | KeyCode::Char(',') => {
                let ((top, left), (bottom, right)) = current_selection.bounds();
                let mut modified_sequences: std::collections::HashMap<usize, Vec<f64>> = std::collections::HashMap::new();
                let mut steps_changed = 0;

                for col_idx in left..=right {
                    if let Some(sequence) = pattern.lines.get(col_idx) {
                        let mut current_steps = sequence.frames.clone();
                        let mut was_modified = false;
                        for row_idx in top..=bottom {
                            if row_idx < current_steps.len() {
                                let current_length = current_steps[row_idx];
                                let new_length = (current_length - 0.25).max(0.01); // Keep minimum
                                current_steps[row_idx] = new_length;
                                was_modified = true;
                                steps_changed += 1;
                            }
                        }
                        if was_modified {
                            modified_sequences.insert(col_idx, current_steps);
                        }
                    }
                }

                for (col, updated_steps) in modified_sequences {
                     app.send_client_message(ClientMessage::UpdateSequenceSteps(col, updated_steps));
                }

                if steps_changed > 0 {
                    app.set_status_message(format!("Requested decreasing length for {} steps", steps_changed));
                } else {
                    app.set_status_message("No valid steps in selection to decrease length".to_string());
                    handled = false;
                }
            }

            // Set the start step of the sequence
            KeyCode::Char('b') => {
                 let cursor_pos = current_selection.cursor_pos();
                 current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                 let (row_idx, col_idx) = cursor_pos;
                 if let Some(sequence) = pattern.lines.get(col_idx) {
                     if row_idx < sequence.frames.len() {
                         let start_step_val = if sequence.start_frame == Some(row_idx) { None } else { Some(row_idx) };
                         app.send_client_message(ClientMessage::SetSequenceStartStep(col_idx, start_step_val));
                         app.set_status_message(format!("Requested setting start step to {:?} for Seq {}", start_step_val, col_idx));
                     } else {
                         app.set_status_message("Cannot set start step on empty slot".to_string());
                         handled = false;
                     }
                 } else { handled = false; }
            }
            KeyCode::Char('e') => {
                 let cursor_pos = current_selection.cursor_pos();
                 current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                 let (row_idx, col_idx) = cursor_pos;
                 if let Some(sequence) = pattern.lines.get(col_idx) {
                     if row_idx < sequence.frames.len() {
                         let end_step_val = if sequence.end_frame == Some(row_idx) { None } else { Some(row_idx) };
                         app.send_client_message(ClientMessage::SetSequenceEndStep(col_idx, end_step_val));
                         app.set_status_message(format!("Requested setting end step to {:?} for Seq {}", end_step_val, col_idx));
                     } else {
                         app.set_status_message("Cannot set end step on empty slot".to_string());
                         handled = false;
                     }
                 } else { handled = false; }
            }
            // Down arrow key: Move the cursor one step down (if shift is pressed, extend the selection)
            KeyCode::Down => {
                let mut end_pos = current_selection.end;
                if let Some(seq) = pattern.lines.get(end_pos.1) {
                    let steps_in_col = seq.frames.len();
                    if steps_in_col > 0 {
                        end_pos.0 = min(end_pos.0 + 1, steps_in_col - 1);
                    }
                }
                if is_shift_pressed {
                     current_selection.end = end_pos;
                 } else {
                     current_selection = GridSelection::single(end_pos.0, end_pos.1);
                 }
            }
            // Up arrow key: Move the cursor one step up (if shift is pressed, decrease the selection)
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
                     let steps_in_next_col = pattern.lines.get(next_col).map_or(0, |s| s.frames.len());
                     end_pos.0 = min(end_pos.0, steps_in_next_col.saturating_sub(1));
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
                     let steps_in_next_col = pattern.lines.get(next_col).map_or(0, |s| s.frames.len());
                     end_pos.0 = min(end_pos.0, steps_in_next_col.saturating_sub(1)); // Adjust row
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
            // Enable / Disable steps
            KeyCode::Char(' ') => {
                 let ((top, left), (bottom, right)) = current_selection.bounds();
                 let mut to_enable: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
                 let mut to_disable: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
                 let mut steps_toggled = 0;

                 for col_idx in left..=right {
                     if let Some(sequence) = pattern.lines.get(col_idx) {
                         for row_idx in top..=bottom {
                             if row_idx < sequence.frames.len() {
                                 let is_enabled = sequence.is_frame_enabled(row_idx);
                                 if is_enabled {
                                     to_disable.entry(col_idx).or_default().push(row_idx);
                                 } else {
                                     to_enable.entry(col_idx).or_default().push(row_idx);
                                 }
                                 steps_toggled += 1;
                             }
                         }
                     }
                 }

                 // Send messages
                 for (col, rows) in to_disable {
                     if !rows.is_empty() {
                        app.send_client_message(ClientMessage::DisableSteps(col, rows));
                    }
                 }
                 for (col, rows) in to_enable {
                     if !rows.is_empty() {
                        app.send_client_message(ClientMessage::EnableSteps(col, rows));
                    }
                 }

                 if steps_toggled > 0 {
                     app.set_status_message(format!("Requested toggling {} steps", steps_toggled));
                 } else {
                     app.set_status_message("No valid steps in selection to toggle".to_string());
                     handled = false;
                 }
            }
            // Remove the last step from the sequence
            KeyCode::Char('d') => {
                let cursor_pos = current_selection.cursor_pos();
                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1);
                let mut last_sequence_index_opt : Option<usize> = None;

                if let Some(pattern) = &app.editor.pattern {
                     if pattern.lines.len() > 0 {
                        let last_sequence_index = pattern.lines.len() - 1;
                        last_sequence_index_opt = Some(last_sequence_index);
                    } else {
                         app.set_status_message("No sequences to remove".to_string());
                         handled = false;
                    }
                } else {
                    app.set_status_message("Pattern not loaded".to_string());
                    handled = false;
                }

                if handled {
                     if let Some(last_sequence_index) = last_sequence_index_opt {
                        app.send_client_message(ClientMessage::SchedulerControl(
                            bubocorelib::schedule::SchedulerMessage::RemoveSequence(last_sequence_index)
                        ));
                        app.set_status_message(format!("Requested removing sequence {}", last_sequence_index));
                    }
                }

            }
            // --- Copy SINGLE Cell Script Info ---
            KeyCode::Char('c') => {
                let (row_idx, col_idx) = current_selection.cursor_pos();
                current_selection = GridSelection::single(row_idx, col_idx);
                let mut handled_copy = false; // Local handled flag for this block

                if let Some(sequence) = pattern.lines.get(col_idx) {
                    if row_idx < sequence.frames.len() {
                        // Get length and enabled state locally first
                        let length = sequence.frames[row_idx];
                        let is_enabled = sequence.is_frame_enabled(row_idx);

                        // Send request to server for the script content
                        app.send_client_message(ClientMessage::GetScript(col_idx, row_idx));

                        // Update clipboard state to fetching script, storing len/state now
                        app.clipboard = ClipboardState::FetchingScript {
                            col: col_idx,
                            row: row_idx,
                            length,
                            is_enabled,
                        };
                        app.set_status_message(format!("Requesting script for copy: Seq {}, Step {}", col_idx, row_idx));
                        app.add_log(LogLevel::Info, format!("Requested script copy for ({}, {}). Length: {}, Enabled: {}", col_idx, row_idx, length, is_enabled));
                        handled_copy = true; // Successfully initiated copy
                    } else {
                        app.set_status_message("Cannot copy script info from an empty slot".to_string());
                        app.clipboard = ClipboardState::Empty; // Reset clipboard state
                        // handled_copy remains false
                    }
                } else {
                    app.set_status_message("Invalid sequence index for copy".to_string());
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

                         if let Some(target_sequence) = pattern.lines.get(target_col) {
                             if target_row < target_sequence.frames.len() {
                                 // 1. Paste Length
                                 let mut updated_steps = target_sequence.frames.clone();
                                 if target_row < updated_steps.len() { // Double check bounds
                                     updated_steps[target_row] = copied_data.length;
                                     app.send_client_message(ClientMessage::UpdateSequenceSteps(target_col, updated_steps));
                                     messages_sent += 1;
                                 }

                                 // 2. Paste Enabled/Disabled State
                                 if copied_data.is_enabled {
                                     app.send_client_message(ClientMessage::EnableSteps(target_col, vec![target_row]));
                                 } else {
                                     app.send_client_message(ClientMessage::DisableSteps(target_col, vec![target_row]));
                                 }
                                 messages_sent += 1;

                                 // 3. Paste Script Content
                                 if let Some(script) = &copied_data.script_content {
                                     app.send_client_message(ClientMessage::SetScript(
                                         target_col,
                                         target_row,
                                         script.clone(),
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
                             app.set_status_message("Invalid sequence index for paste".to_string());
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

    /// Draws the sequence grid UI component.
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

        // Main window
        let outer_block = Block::default()
            .title(" Pattern Grid ")
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
            Span::styled("+", key_style), Span::raw("/"), Span::styled("-", key_style), Span::raw(":Ins/Del Step  "),
            Span::styled("a", key_style), Span::raw("/"), Span::styled("d", key_style), Span::raw(":Add/Rem Seq  "),
            Span::styled("c", key_style), Span::raw("/"), Span::styled("p", key_style),
            Span::raw(":Copy/Paste Step"),
        ];

        // Split the help area into two rows
        let help_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(help_area);

        frame.render_widget(Paragraph::new(Line::from(help_spans_line1).style(help_style)).centered(), help_layout[0]);
        frame.render_widget(Paragraph::new(Line::from(help_spans_line2).style(help_style)).centered(), help_layout[1]);

        // Grid table (requiring pattern data)
        if let Some(pattern) = &app.editor.pattern {
            let sequences = &pattern.lines;
            if sequences.is_empty() {
                frame.render_widget(Paragraph::new("No sequences in pattern. Use 'a' to add.").yellow().centered(), table_area);
                return;
            }

            let num_sequences = sequences.len();
            // Determine the maximum number of steps across all sequences for table height
            let max_steps = sequences.iter().map(|seq| seq.frames.len()).max().unwrap_or(0);

            // Placeholder message if sequences exist but have no steps
            if max_steps == 0 && num_sequences > 0 {
                frame.render_widget(
                    Paragraph::new("Sequences have no steps. Use '+' to add.")
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
            let col_width = if num_sequences > 0 { table_area.width / num_sequences as u16 } else { table_area.width };
            let widths: Vec<Constraint> = std::iter::repeat(Constraint::Min(col_width.max(6)))
                .take(num_sequences)
                .collect();

            // Table Header (SEQ 1, SEQ 2, ...)
            let header_cells = sequences.iter().enumerate()
                .map(|(i, _)| {
                     let text = format!("SEQ {}", i + 1);
                     Cell::from(Line::from(text).alignment(ratatui::layout::Alignment::Center))
                         .style(header_style)
                 });
            let header = Row::new(header_cells).height(1).style(header_style);

            // Create Padding Row: use default style
            let padding_cells = std::iter::repeat(Cell::from("").style(Style::default())) 
                                  .take(num_sequences);
            let padding_row = Row::new(padding_cells).height(1); // Height 1 for one line of padding

            // Create Data Rows 
            let data_rows = (0..max_steps).map(|step_idx| {
                 let cells = sequences.iter().enumerate().map(|(col_idx, seq)| {
                    if step_idx < seq.frames.len() {
                        let step_val = seq.frames[step_idx];
                        let is_enabled = seq.is_frame_enabled(step_idx);
                        let base_style = if is_enabled { enabled_style } else { disabled_style };
                        let is_current_step = app.server.current_step_positions.as_ref()
                            .and_then(|positions| positions.get(col_idx))
                            .map_or(false, |&current| current == step_idx);
                        let should_draw_bar = if let Some(start) = seq.start_frame {
                            if let Some(end) = seq.end_frame { step_idx >= start && step_idx <= end }
                            else { step_idx >= start }
                        } else { if let Some(end) = seq.end_frame { step_idx <= end } else { false } };
                        let bar_char = if should_draw_bar { bar_char_active } else { bar_char_inactive };
                        let play_marker = if is_current_step { "▶" } else { " " };
                        let bar_span = Span::styled(bar_char, if should_draw_bar { start_end_marker_style } else { Style::default() });
                        let play_marker_span = Span::raw(play_marker);
                        let value_span = Span::raw(format!("{:.2}", step_val));
                        let ((top, left), (bottom, right)) = app.interface.components.grid_selection.bounds();
                        let is_selected_locally = step_idx >= top && step_idx <= bottom && col_idx >= left && col_idx <= right;
                        let is_local_cursor = (step_idx, col_idx) == app.interface.components.grid_selection.cursor_pos();

                        // Find if a peer's cursor is on this cell
                        let peer_on_cell: Option<(String, GridSelection)> = app.server.peer_sessions.iter()
                            .filter_map(|(name, peer_state)| peer_state.grid_selection.map(|sel| (name.clone(), sel)))
                            .find(|(_, peer_selection)| (step_idx, col_idx) == peer_selection.cursor_pos());

                        // Check if any peer is editing this specific cell *before* the main logic block
                        let is_being_edited_by_peer = app.server.peer_sessions.values()
                            .any(|peer_state| peer_state.editing_step == Some((col_idx, step_idx)));

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
                            .find(|(_, peer_selection)| (step_idx, col_idx) == peer_selection.cursor_pos());

                         let mut final_style;
                         let cell_content;
                         let cell_content_span; // Use a different name

                         let is_local_cursor = (step_idx, col_idx) == app.interface.components.grid_selection.cursor_pos();
                         let is_being_edited_by_peer = app.server.peer_sessions.values()
                                .any(|peer_state| peer_state.editing_step == Some((col_idx, step_idx)));

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
            frame.render_widget(Paragraph::new("No pattern loaded from server.").yellow().centered(), table_area);
        }
    }
}