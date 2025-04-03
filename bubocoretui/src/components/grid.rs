use crate::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    prelude::{Rect, Constraint, Layout, Direction, Modifier},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell, Clear},
};
use bubocorelib::server::client::ClientMessage;
use std::cmp::min;
use tui_textarea::{TextArea, Input};

/// Représentation graphique du pattern en cours d'exécution sous la forme d'une grille
pub struct GridComponent {
    /// Stores the state required for the step length editing popup.
    ///
    /// When `Some((row, col, textarea))`, the popup is active for the step at `(row, col)`,
    /// and `textarea` holds the current input.
    /// When `None`, the popup is hidden.
    editing_state: Option<(usize, usize, TextArea<'static>)>, // (row, col, textarea)
}

impl GridComponent {
    /// Creates a new [`GridComponent`] instance.
    ///
    /// Initializes the component with no active editing popup (`editing_state` is `None`).
    pub fn new() -> Self {
        Self {
            editing_state: None,
        }
    }
}

impl Component for GridComponent {
    /// Handles key events directed to the grid component.
    ///
    /// This function first checks if the editing popup is active. If it is, key events
    /// are routed to the [`TextArea`] for editing the step length. `Enter` confirms the edit,
    /// `Esc` cancels it, and other keys modify the text.
    ///
    /// If the popup is not active (normal mode), it handles:
    /// - Grid-specific actions:
    ///   - `l`: Enters editing mode for the currently selected step's length.
    ///   - `+`: Sends a message to the server to add a default step (length 1.0) to the current sequence.
    ///   - `-`: Sends a message to the server to remove the last step from the current sequence.
    ///          Adjusts the cursor if it was pointing at the removed step.
    ///   - Arrow keys (`Up`, `Down`, `Left`, `Right`): Navigates the grid cursor, ensuring it stays within bounds
    ///     and adjusts the row based on the number of steps in the target column.
    ///   - `Space`: Sends a message to the server to toggle the enabled/disabled state of the selected step.
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

        let mut current_cursor = app.interface.components.grid_cursor;
        let mut handled = true; // Assume handled unless explicitly set otherwise

        match key_event.code {
            // Edit step length
            KeyCode::Char('l') => {
                let (row_idx, col_idx) = current_cursor;
                if let Some(sequence) = pattern.sequences.get(col_idx) {
                    if row_idx < sequence.steps.len() {
                        let current_length = sequence.steps[row_idx].to_string();
                        let mut textarea = TextArea::new(vec![current_length]);
                        textarea.set_cursor_line_style(Style::default());
                        textarea.set_block(Block::default().borders(Borders::ALL).title("Edit Length"));
                        self.editing_state = Some((row_idx, col_idx, textarea));
                        app.set_status_message("Editing step length. Enter to confirm, Esc to cancel.".to_string());
                    } else {
                        app.set_status_message("Cannot edit length of an empty slot".to_string());
                        handled = false;
                    }
                } else {
                    handled = false; // Should not happen
                }
            }
            // Add/Remove steps
            KeyCode::Char('+') => {
                if let Some(sequence) = pattern.sequences.get(current_cursor.1) {
                    let mut updated_steps = sequence.steps.clone();
                    updated_steps.push(1.0);
                    app.send_client_message(ClientMessage::UpdateSequenceSteps(current_cursor.1, updated_steps));
                    app.set_status_message("Requested adding step".to_string());
                } else { handled = false; }
            }
            KeyCode::Char('-') => {
                let mut message_to_send: Option<ClientMessage> = None;
                let mut status_update: Option<String> = None;
                let mut new_cursor_row: Option<usize> = None;
                {
                    if let Some(sequence) = pattern.sequences.get(current_cursor.1) {
                        if !sequence.steps.is_empty() {
                            let mut updated_steps = sequence.steps.clone();
                            updated_steps.pop();
                            message_to_send = Some(ClientMessage::UpdateSequenceSteps(current_cursor.1, updated_steps));
                            status_update = Some("Requested removing last step".to_string());
                            let last_step_idx = sequence.steps.len() - 1;
                            if current_cursor.0 == last_step_idx {
                                new_cursor_row = Some(current_cursor.0.saturating_sub(1));
                            }
                        } else {
                            status_update = Some("Sequence is already empty".to_string());
                            handled = false;
                        }
                    } else { handled = false; }
                }
                if let Some(message) = message_to_send { app.send_client_message(message); }
                if let Some(status) = status_update { app.set_status_message(status); }
                if let Some(new_row) = new_cursor_row { current_cursor.0 = new_row; }
            }
            // Edit Script (e or Enter)
            KeyCode::Char('e') | KeyCode::Enter => {
                let (row_idx, col_idx) = current_cursor;
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
                let (row_idx, col_idx) = current_cursor;
                if let Some(pattern) = app.editor.pattern.as_mut() { // Need mutable access to pattern
                    if let Some(sequence) = pattern.sequences.get_mut(col_idx) {
                        if row_idx < sequence.steps.len() {
                            let current_length = sequence.steps[row_idx];
                            let new_length = current_length + 0.25;
                            sequence.steps[row_idx] = new_length;
                            let updated_steps = sequence.steps.clone();
                            app.send_client_message(ClientMessage::UpdateSequenceSteps(col_idx, updated_steps));
                            app.set_status_message(format!("Increased step ({},{}) length to {:.2}", col_idx, row_idx, new_length));
                        } else { handled = false; }
                    } else { handled = false; }
                } else { handled = false; }
            }
            KeyCode::Char('<') | KeyCode::Char(',') => { // Comma key
                let (row_idx, col_idx) = current_cursor;
                if let Some(pattern) = app.editor.pattern.as_mut() { // Need mutable access
                    if let Some(sequence) = pattern.sequences.get_mut(col_idx) {
                        if row_idx < sequence.steps.len() {
                            let current_length = sequence.steps[row_idx];
                            // Ensure length doesn't go below a small positive value (e.g., 0.01)
                            let new_length = (current_length - 0.25).max(0.01);
                            sequence.steps[row_idx] = new_length;
                            let updated_steps = sequence.steps.clone();
                            app.send_client_message(ClientMessage::UpdateSequenceSteps(col_idx, updated_steps));
                            app.set_status_message(format!("Decreased step ({},{}) length to {:.2}", col_idx, row_idx, new_length));
                        } else { handled = false; }
                    } else { handled = false; }
                } else { handled = false; }
            }
            // Navigation Arrows
            KeyCode::Down => {
                if let Some(seq) = pattern.sequences.get(current_cursor.1) {
                    let steps_in_col = seq.steps.len();
                    if steps_in_col > 0 {
                        current_cursor.0 = min(current_cursor.0 + 1, steps_in_col - 1);
                    }
                }
            }
            KeyCode::Up => { current_cursor.0 = current_cursor.0.saturating_sub(1); }
            KeyCode::Left => {
                let next_col = current_cursor.1.saturating_sub(1);
                if next_col != current_cursor.1 {
                    let steps_in_next_col = pattern.sequences.get(next_col).map_or(0, |s| s.steps.len());
                    current_cursor.0 = min(current_cursor.0, steps_in_next_col.saturating_sub(1));
                    current_cursor.1 = next_col;
                }
            }
            KeyCode::Right => {
                let next_col = min(current_cursor.1 + 1, num_cols - 1);
                if next_col != current_cursor.1 {
                    let steps_in_next_col = pattern.sequences.get(next_col).map_or(0, |s| s.steps.len());
                    current_cursor.0 = min(current_cursor.0, steps_in_next_col.saturating_sub(1));
                    current_cursor.1 = next_col;
                }
            }
            // Toggle step enabled/disabled
            KeyCode::Char(' ') => {
                let (row_idx, col_idx) = current_cursor;
                let mut message_opt: Option<ClientMessage> = None;
                let mut status_opt: Option<String> = None;
                {
                    if let Some(sequence) = pattern.sequences.get(col_idx) {
                        if row_idx < sequence.steps.len() {
                            let is_enabled = sequence.is_step_enabled(row_idx);
                            message_opt = Some(if is_enabled {
                                ClientMessage::DisableStep(col_idx, row_idx)
                            } else {
                                ClientMessage::EnableStep(col_idx, row_idx)
                            });
                            status_opt = Some(format!("Sent toggle request for step [Seq: {}, Step: {}]", col_idx, row_idx));
                        } else {
                            status_opt = Some("Cannot toggle an empty slot".to_string());
                            handled = false;
                        }
                    } else { handled = false; }
                }
                if let Some(message) = message_opt { app.send_client_message(message); }
                if let Some(status) = status_opt { app.set_status_message(status); }
            }
            // Remove Last Sequence (d)
             KeyCode::Char('d') => { // Changed from Shift+-
                let mut needs_cursor_update = false;
                let mut new_cursor_col : Option<usize> = None;
                let mut last_sequence_index_opt : Option<usize> = None;
                // let mut removed_locally = false; // Remove this flag

                // --- Scope for cursor check --- 
                // if let Some(pattern) = app.editor.pattern.as_mut() { // Use immutable borrow now
                if let Some(pattern) = &app.editor.pattern {
                     if pattern.sequences.len() > 0 {
                        let last_sequence_index = pattern.sequences.len() - 1;
                        last_sequence_index_opt = Some(last_sequence_index);

                        // Check cursor
                        if current_cursor.1 == last_sequence_index {
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
                        
                        let new_row = min(current_cursor.0, steps_in_new_col.saturating_sub(1));
                        current_cursor = (new_row, new_col);
                    }
                }
            }
            _ => { handled = false; } // Ignore other keys
        }

        if handled {
            app.interface.components.grid_cursor = current_cursor;
        }
        Ok(handled)
    }

    /// Draws the sequence grid UI component.
    ///
    /// Renders the main grid table showing sequence steps and their states (enabled/disabled).
    /// Highlights the currently selected cell based on `app.interface.components.grid_cursor`.
    /// Also renders a help line at the bottom showing available keybindings.
    ///
    /// If the `editing_state` is `Some`, it renders a centered popup window containing the
    /// [`TextArea`] for editing the selected step's length.
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

        // --- Help Line --- TODO: Make this dynamic based on context (editing vs normal)
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        let mut help_spans = vec![
            Span::styled("Arrows", key_style), Span::raw(":Move | "),
            Span::styled("Space", key_style), Span::raw(":Toggle | "),
            Span::styled("+", key_style), Span::raw("/"), Span::styled("-", key_style), Span::raw(":Add/Rem Step | "),
            Span::styled("a", key_style), Span::raw("/"), Span::styled("d", key_style), Span::raw(":Add/Rem Seq | "),
            Span::styled("</>", key_style), Span::raw(":Len+/- | "),
            Span::styled("E", key_style), Span::raw(":Edit Script | "),
        ];
        // Append editing help if popup is active
        if self.editing_state.is_some() {
            help_spans.extend(vec![
                Span::raw(" | EDITING: "),
                Span::styled("Enter", key_style), Span::raw(":Confirm | "),
                Span::styled("Esc", key_style), Span::raw(":Cancel"),
            ]);
        }
        frame.render_widget(Paragraph::new(Line::from(help_spans).style(help_style)).centered(), help_area);

        // --- Grid Table --- (Requires pattern data)
        if let Some(pattern) = &app.editor.pattern {
            let sequences = &pattern.sequences;
            if sequences.is_empty() {
                frame.render_widget(Paragraph::new("No sequences in pattern. Add one?").yellow().centered(), table_area);
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

            // --- Styling --- TODO: Move styles to a theme/config struct?
            let header_style = Style::default().fg(Color::Black).bg(Color::Cyan).bold();
            let enabled_style = Style::default().fg(Color::Black).bg(Color::Green);
            let disabled_style = Style::default().fg(Color::Black).bg(Color::Red);
            let cursor_style = Style::default().fg(Color::White).bg(Color::Magenta).bold();
            let empty_cell_style = Style::default().bg(Color::Rgb(40, 40, 40)); // Dark background for empty slots

            // Get current cursor position from app state
            let (cursor_row, cursor_col) = app.interface.components.grid_cursor;

            // Calculate column widths (distribute available width, min width 6)
            let col_width = if num_sequences > 0 { table_area.width / num_sequences as u16 } else { table_area.width };
            let widths: Vec<Constraint> = std::iter::repeat(Constraint::Min(col_width.max(6)))
                .take(num_sequences)
                .collect();

            // --- Table Header --- (SEQ 1, SEQ 2, ...)
            let header_cells = sequences.iter().enumerate()
                .map(|(i, _)| Cell::from(format!("SEQ {}", i + 1)).style(header_style));
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

                        // Format the string with a '>' prefix if it's the current step
                        let step_val_str = if is_current_step {
                            format!("> {:.2}", step_val)
                        } else {
                            format!("  {:.2}", step_val) // Add padding for alignment
                        };

                        // Apply cursor style if this is the selected cell
                        let final_style = if step_idx == cursor_row && col_idx == cursor_col {
                            cursor_style
                        } else {
                            base_style
                        };
                        Cell::from(Line::from(step_val_str).centered()).style(final_style)
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

        // --- Editing Popup --- (Render overlay if editing_state is Some)
        if let Some((_row, _col, textarea)) = &self.editing_state {
            let popup_width: u16 = 20; // Fixed width for the popup
            let popup_height: u16 = 3; // Fixed height (1 for border, 1 for text, 1 for border)

            // Calculate centered position for the popup
            let popup_area = centered_rect(
                popup_width, 
                popup_height, 
                frame.area()
            );

            // Style the popup block
            let popup_block = Block::default()
                .title(" Edit Length ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::DarkGray)); // Background for the popup itself


            // Render the popup:
            // 1. Clear the area behind the popup
            // 2. Render the popup's borders and background
            // 3. Render the textarea widget inside the popup's inner area
            frame.render_widget(Clear, popup_area); // Clear the space
            frame.render_widget(popup_block.clone(), popup_area); // Draw the block
            let inner_popup_area = popup_block.inner(popup_area);
            // Ensure inner area is valid before rendering textarea
            if inner_popup_area.width > 0 && inner_popup_area.height > 0 {
                frame.render_widget(textarea, inner_popup_area);
            }
        }
    }
}

/// Creates a [`Rect`] centered within a given area (`r`) with fixed dimensions.
///
/// Calculates the top-left corner (`x`, `y`) to center the rectangle of `width` and `height`
/// within the container `r`. Ensures the resulting rectangle does not exceed the bounds of `r`.
///
/// # Arguments
/// * `width`: The desired fixed width of the centered rectangle.
/// * `height`: The desired fixed height of the centered rectangle.
/// * `r`: The container [`Rect`] within which to center the new rectangle.
///
/// # Returns
/// The calculated centered [`Rect`].
fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    // Calculate the center coordinates of the container rectangle
    let center_y = r.y + r.height / 2;
    let center_x = r.x + r.width / 2;

    // Calculate the top-left corner coordinates for the new rectangle
    // Saturating subtraction prevents underflow if width/height is larger than container
    let rect_y = center_y.saturating_sub(height / 2);
    let rect_x = center_x.saturating_sub(width / 2);

    // Create the rectangle, ensuring its dimensions don't exceed the container's dimensions
    Rect {
        x: rect_x,
        y: rect_y,
        width: width.min(r.width),   // Use the smaller of desired width and container width
        height: height.min(r.height), // Use the smaller of desired height and container height
    }
}
