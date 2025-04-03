use crate::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    prelude::{Rect, Constraint, Layout, Direction, Modifier},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell},
};
use bubocorelib::server::client::ClientMessage;
use std::cmp::min;
use crate::app::GridSelection;

/// Représentation graphique du pattern en cours d'exécution sous la forme d'une grille
pub struct GridComponent {
    // editing_state removed
}

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
    ///   - `Space`: Sends a message to the server to toggle the enabled/disabled state of the selected step.
    ///   - `Enter`: Sends a message to request the script for the selected step.
    ///   - `<` / `,`: Decrease step length.
    ///   - `>` / `.`: Increase step length.
    ///   - `b`: Mark selected step as the sequence start.
    ///   - `e`: Mark selected step as the sequence end.
    ///   - `a`: Add a new sequence.
    ///   - `d`: Remove the last sequence.
    ///
    /// # Arguments
    ///
    /// * `app`: Mutable reference to the main application state (`App`).
    /// * `key_event`: The `KeyEvent` received from the terminal.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the key event was handled by this component.
    /// * `Ok(false)` if the key event was not handled.
    /// * `Err` if an error occurred during event handling.
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        // Get pattern data, but don't exit immediately if empty
        let pattern_opt = app.editor.pattern.as_ref();
        let num_cols = pattern_opt.map_or(0, |p| p.sequences.len());

        // Handle 'a' regardless of whether sequences exist
        if key_event.code == KeyCode::Char('a') {
             // Send the request to add a sequence; the server will create the default one.
            app.send_client_message(ClientMessage::SchedulerControl(
                bubocorelib::schedule::SchedulerMessage::AddSequence
            ));
            app.set_status_message("Requested adding sequence".to_string());
            return Ok(true); // Handled
        }

        // --- For other keys, require a pattern and at least one sequence ---
        let pattern = match pattern_opt {
             Some(p) if num_cols > 0 => p,
             _ => { return Ok(false); } // No pattern or no sequences, ignore other keys
        };

        // let mut current_cursor = app.interface.components.grid_cursor;
        let mut current_selection = app.interface.components.grid_selection;
        let mut handled = true; // Assume handled unless explicitly set otherwise

        // Extract shift modifier for easier checking
        let is_shift_pressed = key_event.modifiers.contains(KeyModifiers::SHIFT);

        match key_event.code {
            KeyCode::Esc => {
                if !current_selection.is_single() {
                    // Reset selection to single cell at the selection's start position
                    current_selection = GridSelection::single(current_selection.start.0, current_selection.start.1);
                    app.set_status_message("Selection reset to single cell (at start)".to_string());
                    // handled remains true
                } else {
                    // If already single cell, let ESC potentially be handled globally (e.g., navigation)
                    handled = false;
                }
            }
            // Edit step length ('l' removed)
            // Add/Remove steps
            KeyCode::Char('+') => {
                 let cursor_pos = current_selection.cursor_pos(); // Use cursor pos for single target
                 current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1); // Reset selection
                 if let Some(sequence) = pattern.sequences.get(cursor_pos.1) {
                    let mut updated_steps = sequence.steps.clone();
                    updated_steps.push(1.0);
                    app.send_client_message(ClientMessage::UpdateSequenceSteps(cursor_pos.1, updated_steps));
                    app.set_status_message("Requested adding step".to_string());
                } else { handled = false; }
            }
            KeyCode::Char('-') => {
                let cursor_pos = current_selection.cursor_pos();
                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1); // Reset selection
                let mut message_to_send: Option<ClientMessage> = None;
                let mut status_update: Option<String> = None;
                let mut new_cursor_row: Option<usize> = None;
                {
                    if let Some(sequence) = pattern.sequences.get(cursor_pos.1) {
                        if !sequence.steps.is_empty() {
                            let mut updated_steps = sequence.steps.clone();
                            updated_steps.pop();
                            message_to_send = Some(ClientMessage::UpdateSequenceSteps(cursor_pos.1, updated_steps));
                            status_update = Some("Requested removing last step".to_string());
                            let last_step_idx = sequence.steps.len() - 1;
                            if cursor_pos.0 == last_step_idx {
                                // We don't need to modify the selection here anymore, 
                                // just need to know if the step was the last one for status perhaps.
                                // The selection was already reset above.
                                // new_cursor_row = Some(cursor_pos.0.saturating_sub(1));
                            }
                        } else {
                            status_update = Some("Sequence is already empty".to_string());
                            handled = false;
                        }
                    } else { handled = false; }
                }
                if let Some(message) = message_to_send { app.send_client_message(message); }
                if let Some(status) = status_update { app.set_status_message(status); }
                // Cursor update handled by resetting selection above
                // if let Some(new_row) = new_cursor_row { current_selection.end.0 = new_row; current_selection.start.0 = new_row; }
            }
            // Edit Script (Enter only, 'e' removed)
            KeyCode::Enter => {
                // let (row_idx, col_idx) = current_cursor;
                let cursor_pos = current_selection.cursor_pos();
                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1); // Reset selection
                let (row_idx, col_idx) = cursor_pos;
                let status_update: Option<String>;

                if let Some(pattern) = &app.editor.pattern {
                    if let Some(sequence) = pattern.sequences.get(col_idx) {
                        if row_idx < sequence.steps.len() {
                            // Send request to server for the script content
                            app.send_client_message(ClientMessage::GetScript(col_idx, row_idx));
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
            // Increment/Decrement step length
            KeyCode::Char('>') | KeyCode::Char('.') => { // Period key
                let ((top, left), (bottom, right)) = current_selection.bounds();
                let mut modified_sequences: std::collections::HashMap<usize, Vec<f64>> = std::collections::HashMap::new();
                let mut steps_changed = 0;

                // Iterate over the selection bounds
                for col_idx in left..=right {
                    // Get a mutable reference *if needed*, but work on a clone first
                    if let Some(sequence) = pattern.sequences.get(col_idx) {
                        let mut current_steps = sequence.steps.clone(); // Clone the steps vector
                        let mut was_modified = false;
                        for row_idx in top..=bottom {
                            if row_idx < current_steps.len() { // Check against cloned length
                                let current_length = current_steps[row_idx];
                                let new_length = current_length + 0.25;
                                current_steps[row_idx] = new_length; // Modify the clone
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
            KeyCode::Char('<') | KeyCode::Char(',') => { // Comma key
                let ((top, left), (bottom, right)) = current_selection.bounds();
                let mut modified_sequences: std::collections::HashMap<usize, Vec<f64>> = std::collections::HashMap::new();
                let mut steps_changed = 0;

                for col_idx in left..=right {
                    if let Some(sequence) = pattern.sequences.get(col_idx) {
                        let mut current_steps = sequence.steps.clone();
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
            // Set Start/End Step
            KeyCode::Char('b') => {
                 // let (row_idx, col_idx) = current_cursor;
                 let cursor_pos = current_selection.cursor_pos();
                 current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1); // Reset selection
                 let (row_idx, col_idx) = cursor_pos;
                 if let Some(sequence) = pattern.sequences.get(col_idx) {
                     if row_idx < sequence.steps.len() {
                         // If already the start step, send None to reset
                         let start_step_val = if sequence.start_step == Some(row_idx) { None } else { Some(row_idx) };
                         app.send_client_message(ClientMessage::SetSequenceStartStep(col_idx, start_step_val));
                         app.set_status_message(format!("Requested setting start step to {:?} for Seq {}", start_step_val, col_idx));
                     } else {
                         app.set_status_message("Cannot set start step on empty slot".to_string());
                         handled = false;
                     }
                 } else { handled = false; }
            }
            KeyCode::Char('e') => {
                 // let (row_idx, col_idx) = current_cursor;
                 let cursor_pos = current_selection.cursor_pos();
                 current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1); // Reset selection
                 let (row_idx, col_idx) = cursor_pos;
                 if let Some(sequence) = pattern.sequences.get(col_idx) {
                     if row_idx < sequence.steps.len() {
                         // If already the end step, send None to reset
                         let end_step_val = if sequence.end_step == Some(row_idx) { None } else { Some(row_idx) };
                         app.send_client_message(ClientMessage::SetSequenceEndStep(col_idx, end_step_val));
                         app.set_status_message(format!("Requested setting end step to {:?} for Seq {}", end_step_val, col_idx));
                     } else {
                         app.set_status_message("Cannot set end step on empty slot".to_string());
                         handled = false;
                     }
                 } else { handled = false; }
            }
            // Navigation Arrows
            KeyCode::Down => {
                let mut end_pos = current_selection.end;
                if let Some(seq) = pattern.sequences.get(end_pos.1) {
                    let steps_in_col = seq.steps.len();
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
            KeyCode::Up => {
                let mut end_pos = current_selection.end;
                end_pos.0 = end_pos.0.saturating_sub(1);
                 if is_shift_pressed {
                     current_selection.end = end_pos;
                 } else {
                     current_selection = GridSelection::single(end_pos.0, end_pos.1);
                 }
            }
            KeyCode::Left => {
                let mut end_pos = current_selection.end;
                let next_col = end_pos.1.saturating_sub(1);
                if next_col != end_pos.1 { // Check if column actually changed
                     let steps_in_next_col = pattern.sequences.get(next_col).map_or(0, |s| s.steps.len());
                     end_pos.0 = min(end_pos.0, steps_in_next_col.saturating_sub(1)); // Adjust row
                     end_pos.1 = next_col;

                     if is_shift_pressed {
                         current_selection.end = end_pos;
                     } else {
                         current_selection = GridSelection::single(end_pos.0, end_pos.1);
                     }
                 } else {
                     handled = false; // Didn't move
                 }
            }
            KeyCode::Right => {
                let mut end_pos = current_selection.end;
                let next_col = min(end_pos.1 + 1, num_cols.saturating_sub(1)); // Ensure not out of bounds
                 if next_col != end_pos.1 { // Check if column actually changed
                     let steps_in_next_col = pattern.sequences.get(next_col).map_or(0, |s| s.steps.len());
                     end_pos.0 = min(end_pos.0, steps_in_next_col.saturating_sub(1)); // Adjust row
                     end_pos.1 = next_col;

                     if is_shift_pressed {
                         current_selection.end = end_pos;
                     } else {
                         current_selection = GridSelection::single(end_pos.0, end_pos.1);
                     }
                 } else {
                     handled = false; // Didn't move
                 }
            }
            // Toggle step enabled/disabled
            KeyCode::Char(' ') => {
                // Get selection bounds
                 let ((top, left), (bottom, right)) = current_selection.bounds();
                 let mut to_enable: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
                 let mut to_disable: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
                 let mut steps_toggled = 0;

                 for col_idx in left..=right {
                     if let Some(sequence) = pattern.sequences.get(col_idx) {
                         for row_idx in top..=bottom {
                             if row_idx < sequence.steps.len() {
                                 let is_enabled = sequence.is_step_enabled(row_idx);
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
            // Remove Last Sequence (d)
             KeyCode::Char('d') => { // Changed from Shift+-
                let cursor_pos = current_selection.cursor_pos();
                current_selection = GridSelection::single(cursor_pos.0, cursor_pos.1); // Reset selection
                let mut needs_cursor_update = false;
                let mut new_cursor_col : Option<usize> = None;
                let mut last_sequence_index_opt : Option<usize> = None;

                // --- Scope for cursor check ---
                // if let Some(pattern) = app.editor.pattern.as_mut() { // Use immutable borrow now
                if let Some(pattern) = &app.editor.pattern {
                     if pattern.sequences.len() > 0 {
                        let last_sequence_index = pattern.sequences.len() - 1;
                        last_sequence_index_opt = Some(last_sequence_index);

                        // Check cursor
                        // if current_cursor.1 == last_sequence_index {
                        if cursor_pos.1 == last_sequence_index { // Check against the reset cursor pos
                            needs_cursor_update = true;
                            new_cursor_col = Some(last_sequence_index.saturating_sub(1));
                        }

                        // Optimistic UI Removed: Do not remove locally
                        // pattern.remove_sequence(last_sequence_index);
                        // removed_locally = true;

                    } else {
                         app.set_status_message("No sequences to remove".to_string());
                         handled = false;
                    }
                } else {
                    app.set_status_message("Pattern not loaded".to_string());
                    handled = false;
                }

                // --- Server message and status update (only if operation seems valid) ---
                // if removed_locally { // Check handled instead
                if handled {
                     if let Some(last_sequence_index) = last_sequence_index_opt { // Should always be Some if handled is true
                        app.send_client_message(ClientMessage::SchedulerControl(
                            bubocorelib::schedule::SchedulerMessage::RemoveSequence(last_sequence_index)
                        ));
                        app.set_status_message(format!("Requested removing sequence {}", last_sequence_index));
                    }
                } // else: status already set if error occurred

                // --- Cursor update (if needed) ---
                if needs_cursor_update {
                    if let Some(new_col) = new_cursor_col {
                        // Get step count (can use immutable borrow)
                        let steps_in_new_col = app.editor.pattern.as_ref() // Immutable borrow is fine
                            .and_then(|p| p.sequences.get(new_col))
                            .map_or(0, |s| s.steps.len());

                        let new_row = min(cursor_pos.0, steps_in_new_col.saturating_sub(1)); // Use reset cursor pos row
                        // current_cursor = (new_row, new_col);
                        // Selection already reset, update is implicit
                    }
                }
            }
            _ => { handled = false; } // Ignore other keys
        }

        if handled {
            // app.interface.components.grid_cursor = current_cursor;
            app.interface.components.grid_selection = current_selection;
        }
        Ok(handled)
    }

    /// Draws the sequence grid UI component.
    ///
    /// Renders the main grid table showing sequence steps and their states (enabled/disabled).
    /// Highlights the currently selected cell based on `app.interface.components.grid_cursor`.
    /// Also renders a help line at the bottom showing available keybindings.
    /// Indicates Start (B) and End (E) steps for sequences.
    ///
    /// # Arguments
    ///
    /// * `app`: Immutable reference to the main application state (`App`).
    /// * `frame`: Mutable reference to the current terminal frame (`Frame`).
    /// * `area`: The `Rect` area allocated for this component to draw into.
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        // --- Main Grid Area Setup ---
        let outer_block = Block::default()
            .title(" Sequence Grid ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        let inner_area = outer_block.inner(area);
        frame.render_widget(outer_block.clone(), area); // Draw the outer border

        // Need at least some space to draw anything inside
        if inner_area.width < 1 || inner_area.height < 2 { return; }

        // Split inner area into table area and a small help line area at the bottom
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([ Constraint::Min(0), Constraint::Length(1) ]) // Table gets remaining space, help gets 1 line
            .split(inner_area);
        let table_area = main_chunks[0];
        let help_area = main_chunks[1];

        // --- Help Line ---
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        let help_spans = vec![
            Span::styled("Arrows", key_style), Span::raw(":Move | "),
            Span::styled("Space", key_style), Span::raw(":Toggle | "),
            Span::styled("Enter", key_style), Span::raw(":Edit Script | "),
            Span::styled("</>", key_style), Span::raw(":Len+/- | "),
            Span::styled("b", key_style), Span::raw("/"), Span::styled("e", key_style), Span::raw(":Set Start/End | "),
            Span::styled("+", key_style), Span::raw("/"), Span::styled("-", key_style), Span::raw(":Add/Rem Step | "),
            Span::styled("a", key_style), Span::raw("/"), Span::styled("d", key_style), Span::raw(":Add/Rem Seq"),
        ];

        frame.render_widget(Paragraph::new(Line::from(help_spans).style(help_style)).centered(), help_area);

        // --- Grid Table --- (Requires pattern data)
        if let Some(pattern) = &app.editor.pattern {
            let sequences = &pattern.sequences;
            if sequences.is_empty() {
                frame.render_widget(Paragraph::new("No sequences in pattern. Use 'a' to add.").yellow().centered(), table_area);
                return;
            }

            let num_sequences = sequences.len();
            // Determine the maximum number of steps across all sequences for table height
            let max_steps = sequences.iter().map(|seq| seq.steps.len()).max().unwrap_or(0);

            // Placeholder message if sequences exist but have no steps
            if max_steps == 0 && num_sequences > 0 {
                frame.render_widget(Paragraph::new("Sequences have no steps. Use '+' to add.").yellow().centered(), table_area);
                // Continue to draw header even if no steps
            }

            // --- Styling ---
            let header_style = Style::default().fg(Color::Black).bg(Color::Cyan).bold();
            let enabled_style = Style::default().fg(Color::Black).bg(Color::Green);
            let disabled_style = Style::default().fg(Color::Black).bg(Color::Red);
            let cursor_style = Style::default().fg(Color::White).bg(Color::Magenta).bold();
            let empty_cell_style = Style::default().bg(Color::Rgb(40, 40, 40)); // Dark background for empty slots
            let start_end_marker_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);

            // Get current cursor position from app state
            let (cursor_row, cursor_col) = app.interface.components.grid_selection.cursor_pos();
            let bar_char_active = "┃";
            let bar_char_inactive = " ";

            // Calculate column widths (distribute available width, min width 6)
            let col_width = if num_sequences > 0 { table_area.width / num_sequences as u16 } else { table_area.width };
            let widths: Vec<Constraint> = std::iter::repeat(Constraint::Min(col_width.max(6)))
                .take(num_sequences)
                .collect();

            // --- Table Header --- (SEQ 1, SEQ 2, ...)
            let header_cells = sequences.iter().enumerate()
                .map(|(i, _)| {
                     let text = format!("SEQ {}", i + 1);
                     Cell::from(Line::from(text).alignment(ratatui::layout::Alignment::Center))
                         .style(header_style)
                 });
            let header = Row::new(header_cells).height(1).style(header_style);

            // --- Table Rows --- (Iterate up to max_steps)
            let rows = (0..max_steps).map(|step_idx| {
                let cells = sequences.iter().enumerate().map(|(col_idx, seq)| {
                    if step_idx < seq.steps.len() {
                        // Cell for an existing step
                        let step_val = seq.steps[step_idx];
                        let is_enabled = seq.is_step_enabled(step_idx);
                        let base_style = if is_enabled { enabled_style } else { disabled_style };

                        // Check if this step is the currently playing one
                        let is_current_step = app.server.current_step_positions.as_ref()
                            .and_then(|positions| positions.get(col_idx))
                            .map_or(false, |&current| current == step_idx);

                        // Check if this step is the defined start or end step
                        let is_start_step = seq.start_step == Some(step_idx);
                        let is_end_step = seq.end_step == Some(step_idx);

                        // Determine if the vertical bar should be drawn for this step
                        let should_draw_bar = if let Some(start) = seq.start_step {
                            if let Some(end) = seq.end_step {
                                // Both set: draw between start and end
                                step_idx >= start && step_idx <= end
                            } else {
                                // Start set, End not set: draw from start to last step
                                step_idx >= start
                            }
                        } else {
                            if let Some(end) = seq.end_step {
                                // Start not set, End set: draw from 0 to end
                                step_idx <= end
                            } else {
                                // Neither set: never draw bar
                                false
                            }
                        };

                        // Format the string with markers
                        let bar_char = if should_draw_bar { bar_char_active } else { bar_char_inactive };
                        let play_marker = if is_current_step { ">" } else { " " };

                        // Create spans: Bar | Play Marker | Value
                        let bar_span = Span::styled(
                            bar_char,
                            if should_draw_bar { start_end_marker_style } else { Style::default() }
                        );
                        let play_marker_span = Span::raw(play_marker);
                        let value_span = Span::raw(format!("{:.2}", step_val));

                        let line_spans = vec![bar_span, play_marker_span, Span::raw(" "), value_span]; // Add space after play marker

                        // Apply cursor style if this is the selected cell
                        let final_style = if step_idx == cursor_row && col_idx == cursor_col {
                            cursor_style
                        } else {
                            base_style
                        };

                        // Determine if the cell is within the selection bounds
                        let ((top, left), (bottom, right)) = app.interface.components.grid_selection.bounds();
                        let is_selected = step_idx >= top && step_idx <= bottom && col_idx >= left && col_idx <= right;

                        // Apply cursor style if selected
                        let final_style = if is_selected {
                             cursor_style
                         } else {
                             base_style
                         };

                        Cell::from(Line::from(line_spans).alignment(ratatui::layout::Alignment::Center)).style(final_style)

                    } else {
                        // Cell for an empty slot below existing steps
                        Cell::from("").style(empty_cell_style)
                    }
                });
                Row::new(cells).height(1)
            });

            // Create and render the table
            let table = Table::new(rows, &widths)
                .header(header)
                .column_spacing(1);
            frame.render_widget(table, table_area);

        } else {
            // No pattern loaded message
            frame.render_widget(Paragraph::new("No pattern loaded from server.").yellow().centered(), table_area);
        }

        // Editing Popup removed
    }
}

// centered_rect function removed
