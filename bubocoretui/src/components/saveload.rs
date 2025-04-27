use crate::app::App;
use crate::components::Component;
use crate::disk;
use crate::event::{AppEvent, Event};
use bubocorelib::schedule::ActionTiming;
use bubocorelib::server::client::ClientMessage;
use chrono::{DateTime, Local, Utc};
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph},
};
use tui_textarea::TextArea;

/// State for the save/load component
pub struct SaveLoadState {
    /// Projects list (name, creation date, last save date, tempo, line_count)
    pub projects: Vec<(
        String,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
        Option<f32>,
        Option<usize>,
    )>,
    /// Selected project index
    pub selected_index: usize,
    /// Text input area for entering project name to save
    pub input_area: TextArea<'static>,
    /// Indicates if the user is entering text to save
    pub is_saving: bool,
    /// Status message (ex: "Project saved", "Loading error").
    pub status_message: String,
    /// Indicates if the user is filtering the project list
    pub is_searching: bool,
    /// The current search query string
    pub search_query: String,
    /// Indicates if the help popup is currently visible
    pub show_help: bool,
    /// Indicates if the delete confirmation popup is visible
    pub show_delete_confirmation: bool,
    /// Name of the project pending delete confirmation
    pub project_to_delete: Option<String>,
    /// Indicates if the save overwrite confirmation popup is visible
    pub show_save_overwrite_confirmation: bool,
    /// Name of the project pending save overwrite confirmation
    pub project_to_overwrite: Option<String>,
}

impl SaveLoadState {
    pub fn new() -> Self {
        let mut input_area = TextArea::default();
        input_area.set_block(Block::default().borders(Borders::NONE));
        Self {
            projects: Vec::new(),
            selected_index: 0,
            input_area,
            is_saving: false,
            status_message: String::new(),
            is_searching: false,
            search_query: String::new(),
            show_help: false,
            show_delete_confirmation: false,
            project_to_delete: None,
            show_save_overwrite_confirmation: false,
            project_to_overwrite: None,
        }
    }
}

/// Save/Load component
pub struct SaveLoadComponent;

impl SaveLoadComponent {
    pub fn new() -> Self {
        Self {}
    }
}

/// Performs a simple fuzzy match check.
/// Returns true if all characters in `query` appear in `text` in order, case-insensitive.
fn simple_fuzzy_match(query: &str, text: &str) -> bool {
    if query.is_empty() {
        return true; // Empty query matches everything
    }
    let mut query_chars = query.chars().peekable();
    let mut text_chars = text.chars();

    while let Some(q_char) = query_chars.peek() {
        // Find the next occurrence of the query char in the text
        match text_chars.find(|t_char| t_char.eq_ignore_ascii_case(q_char)) {
            Some(_) => {
                // Found the current query character, advance query
                query_chars.next(); // Consume the query char
            }
            None => {
                // Could not find the current query character in the rest of the text
                return false;
            }
        }
    }
    // If we consumed all query characters, it's a match
    true
}

impl Component for SaveLoadComponent {
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        let state = &mut app.interface.components.save_load_state;
        let key_code = key_event.code;
        let key_modifiers = key_event.modifiers;

        // --- Handle Help Popup Mode ---
        if state.show_help {
            match key_code {
                KeyCode::Esc | KeyCode::Char('?') => {
                    state.show_help = false;
                    state.status_message = "Closed help.".to_string();
                    return Ok(true);
                }
                _ => return Ok(true),
            }
        }

        // --- Handle Delete Confirmation Mode ---
        if state.show_delete_confirmation {
            match key_code {
                // Confirm Delete
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    if let Some(proj_name) = state.project_to_delete.take() {
                        state.status_message = format!("Deleting project '{}'...", proj_name);
                        let event_sender = app.events.sender.clone();

                        // Adjust selected index *before* deletion potentially changes the list
                        let filtered_projects: Vec<_> = state
                            .projects
                            .iter()
                            .filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name))
                            .collect();
                        let num_filtered = filtered_projects.len();
                        // Find the index of the project *about to be deleted* in the filtered list
                        if let Some(index_to_delete) = filtered_projects
                            .iter()
                            .position(|(name, ..)| *name == proj_name)
                        {
                            // If the deleted item is the last one or beyond the current selection, move selection up
                            if state.selected_index >= index_to_delete && state.selected_index > 0 {
                                state.selected_index = state.selected_index.saturating_sub(1);
                            }
                            // Special case: if deleting the only item, reset index to 0
                            if num_filtered == 1 {
                                state.selected_index = 0;
                            }
                        }

                        tokio::spawn(async move {
                            match disk::delete_project(&proj_name).await {
                                Ok(_) => {
                                    let _ = event_sender
                                        .send(Event::App(AppEvent::ProjectDeleted(proj_name)));
                                }
                                Err(e) => {
                                    let _ = event_sender.send(Event::App(
                                        AppEvent::ProjectDeleteError(e.to_string()),
                                    ));
                                }
                            }
                        });
                    } else {
                        // Should not happen if state is managed correctly
                        state.status_message =
                            "Error: No project specified for deletion.".to_string();
                    }
                    state.show_delete_confirmation = false;
                    return Ok(true);
                }
                // Cancel Delete
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    state.show_delete_confirmation = false;
                    state.project_to_delete = None;
                    state.status_message = "Deletion cancelled.".to_string();
                    return Ok(true);
                }
                _ => return Ok(true), // Consume other keys
            }
        }

        // --- Handle Save Overwrite Confirmation Mode ---
        if state.show_save_overwrite_confirmation {
            match key_code {
                // Confirm Overwrite
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    let mut should_send_snapshot = false;
                    if let Some(project_name) = state.project_to_overwrite.as_ref() {
                        // Borrow immutably first
                        state.status_message =
                            format!("Requesting snapshot to overwrite '{}'...", project_name);
                        state.is_saving = false;
                        state.input_area = TextArea::default();
                        state.input_area.insert_str(project_name);
                        state
                            .input_area
                            .set_block(Block::default().borders(Borders::NONE));
                        should_send_snapshot = true;
                    } else {
                        state.status_message =
                            "Error: No project specified for overwrite.".to_string();
                    }
                    state.show_save_overwrite_confirmation = false;
                    state.project_to_overwrite = None; // Clear here

                    if should_send_snapshot {
                        app.send_client_message(ClientMessage::GetSnapshot);
                    }
                    return Ok(true);
                }
                // Cancel Overwrite
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    state.show_save_overwrite_confirmation = false;
                    state.project_to_overwrite = None;
                    state.is_saving = false;
                    state.input_area = TextArea::default();
                    state
                        .input_area
                        .set_block(Block::default().borders(Borders::NONE));
                    state.status_message = "Save cancelled.".to_string();
                    return Ok(true);
                }
                _ => return Ok(true), // Consume other keys
            }
        }

        // --- Handle Saving Input Mode ---
        if state.is_saving {
            match key_code {
                // Cancel save (but not overwrite confirmation - handled above)
                KeyCode::Esc => {
                    state.is_saving = false;
                    state.input_area = TextArea::default();
                    state
                        .input_area
                        .set_block(Block::default().borders(Borders::NONE));
                    state.status_message = "Save cancelled.".to_string();
                    return Ok(true);
                }
                // Confirm save
                KeyCode::Enter => {
                    let project_name = state.input_area.lines()[0].trim().to_string();
                    let mut should_send_snapshot = false;
                    let mut name_for_snapshot = String::new();

                    if project_name.is_empty() {
                        state.status_message = "Project name cannot be empty.".to_string();
                    } else {
                        // Check if project exists BEFORE requesting snapshot
                        let exists = state
                            .projects
                            .iter()
                            .any(|(name, ..)| *name == project_name);
                        if exists {
                            // Show confirmation popup
                            state.project_to_overwrite = Some(project_name);
                            state.show_save_overwrite_confirmation = true;
                            state.status_message = format!(
                                "Project '{}' already exists. Overwrite? (y/n)",
                                state.project_to_overwrite.as_ref().unwrap()
                            );
                            // Don't send snapshot yet
                        } else {
                            // Project doesn't exist, proceed with save
                            state.status_message =
                                format!("Requesting snapshot to save as '{}'...", project_name);
                            state.is_saving = false;
                            // Prepare state *before* snapshot request
                            state.input_area = TextArea::default(); // Clear input area now
                            state.input_area.insert_str(&project_name); // Store name for handle_event
                            state
                                .input_area
                                .set_block(Block::default().borders(Borders::NONE));

                            should_send_snapshot = true;
                            name_for_snapshot = project_name; // We don't need name_for_snapshot, state.input_area holds it
                        }
                    }

                    if should_send_snapshot {
                        app.send_client_message(ClientMessage::GetSnapshot);
                    }

                    return Ok(true);
                }
                _ => {
                    // Let TextArea handle other input like text typing, backspace etc.
                    // But prevent navigation keys from changing selection while typing name
                    match key_code {
                        KeyCode::Up
                        | KeyCode::Down
                        | KeyCode::PageUp
                        | KeyCode::PageDown
                        | KeyCode::Home
                        | KeyCode::End => {
                            // Potentially allow some text navigation within TextArea later if needed
                            return Ok(true); // Consume navigation keys
                        }
                        _ => {
                            let handled = state.input_area.input(key_event);
                            return Ok(handled);
                        }
                    }
                }
            }
        }

        // --- Handle Searching/Filtering Input Mode ---
        if state.is_searching {
            match key_code {
                KeyCode::Esc | KeyCode::Enter => {
                    state.is_searching = false;
                    if key_code == KeyCode::Esc {
                        state.search_query.clear();
                    }
                    state.status_message = "Exited search.".to_string();
                    state.selected_index = 0; // Reset selection
                    return Ok(true);
                }
                KeyCode::Backspace => {
                    if !state.search_query.is_empty() {
                        state.search_query.pop();
                        state.selected_index = 0; // Reset selection
                    }
                    return Ok(true);
                }
                KeyCode::Char(c)
                    if !key_modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    state.search_query.push(c);
                    state.selected_index = 0; // Reset selection
                    return Ok(true);
                }
                // Allow navigation while searching - useful for long lists
                KeyCode::Up => {
                    let num_filtered = state
                        .projects
                        .iter()
                        .filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name))
                        .count();
                    if num_filtered > 0 {
                        state.selected_index = state.selected_index.saturating_sub(1);
                    }
                    return Ok(true);
                }
                KeyCode::Down => {
                    let num_filtered = state
                        .projects
                        .iter()
                        .filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name))
                        .count();
                    if num_filtered > 0 {
                        state.selected_index =
                            (state.selected_index + 1).min(num_filtered.saturating_sub(1));
                    }
                    return Ok(true);
                }
                _ => {
                    return Ok(false);
                } // Ignore other keys in search mode for now
            }
        }

        // --- Handle List Navigation/Actions Mode (Default mode) ---
        match (key_code, key_modifiers) {
            // Toggle Help Popup
            (KeyCode::Char('?'), _) => {
                state.show_help = true;
                state.status_message = "Help popup opened (Esc or ? to close).".to_string();
                Ok(true)
            }
            // Enter search mode
            (KeyCode::Char('/'), _) => {
                state.is_searching = true;
                state.search_query.clear();
                state.status_message = "Enter search query (Esc/Enter to exit)...".to_string();
                state.selected_index = 0;
                Ok(true)
            }
            // Navigate up
            (KeyCode::Up, _) => {
                let num_filtered = state
                    .projects
                    .iter()
                    .filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name))
                    .count();
                if num_filtered > 0 {
                    state.selected_index = state.selected_index.saturating_sub(1);
                }
                Ok(true)
            }
            // Navigate down
            (KeyCode::Down, _) => {
                let num_filtered = state
                    .projects
                    .iter()
                    .filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name))
                    .count();
                if num_filtered > 0 {
                    state.selected_index =
                        (state.selected_index + 1).min(num_filtered.saturating_sub(1));
                }
                Ok(true)
            }
            // Load a project
            (KeyCode::Char('l'), modifiers) => {
                let timing = if modifiers.contains(KeyModifiers::CONTROL) {
                    ActionTiming::EndOfScene
                } else {
                    ActionTiming::Immediate
                };
                let filtered_projects: Vec<_> = state
                    .projects
                    .iter()
                    .filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name))
                    .collect();
                if let Some((project_name, _, _, _, _)) =
                    filtered_projects.get(state.selected_index)
                {
                    state.status_message =
                        format!("Loading project '{}' ({:?})...", project_name, timing);
                    let proj_name = (*project_name).clone();
                    let event_sender = app.events.sender.clone();
                    tokio::spawn(async move {
                        match disk::load_project(&proj_name).await {
                            Ok(snapshot) => {
                                let _ = event_sender
                                    .send(Event::App(AppEvent::LoadProject(snapshot, timing)));
                            }
                            Err(e) => {
                                let _ = event_sender
                                    .send(Event::App(AppEvent::ProjectLoadError(e.to_string())));
                            }
                        }
                    });
                    let _ = app.events.sender.send(Event::App(AppEvent::SwitchToGrid));
                } else {
                    state.status_message = "No project selected to load.".to_string();
                }
                Ok(true)
            }
            // Save a project (Enter saving mode)
            (KeyCode::Char('s'), _) => {
                state.is_saving = true;
                state.is_searching = false;
                state.search_query.clear();
                state.input_area = TextArea::default();
                state
                    .input_area
                    .set_block(Block::default().borders(Borders::NONE));
                state.status_message = "Enter project name to save.".to_string();
                Ok(true)
            }
            // Save/Overwrite selected project (Enter)
            (KeyCode::Enter, _) => {
                let filtered_projects: Vec<_> = state
                    .projects
                    .iter()
                    .filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name))
                    .collect();
                if let Some((project_name, _, _, _, _)) =
                    filtered_projects.get(state.selected_index)
                {
                    // Project selected, trigger overwrite confirmation directly
                    state.project_to_overwrite = Some((*project_name).clone());
                    state.show_save_overwrite_confirmation = true;
                    state.is_saving = false; // Not entering text input mode here
                    state.status_message = format!("Overwrite project '{}'? (y/n)", project_name);
                } else {
                    // No project selected (e.g., empty list) - maybe start a new save?
                    // Or just do nothing? Let's do nothing for now.
                    state.status_message = "No project selected.".to_string();
                }
                Ok(true)
            }
            // Delete a project (Show confirmation) - Changed to Backspace/Delete
            (KeyCode::Backspace, _) | (KeyCode::Delete, _) => {
                let filtered_projects: Vec<_> = state
                    .projects
                    .iter()
                    .filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name))
                    .collect();
                if let Some((project_name, _, _, _, _)) =
                    filtered_projects.get(state.selected_index)
                {
                    state.project_to_delete = Some((*project_name).clone());
                    state.show_delete_confirmation = true;
                    state.status_message = format!("Delete project '{}'? (y/n)", project_name);
                } else {
                    state.status_message = "No project selected to delete.".to_string();
                }
                Ok(true)
            }
            _ => Ok(false), // Pass unhandled keys up
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let state = &app.interface.components.save_load_state;

        let key_style = Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD);

        // --- Define Layout (Always calculate layout now) ---
        let input_prompt_height = 3;
        let constraints: Vec<Constraint>;
        let list_area: Rect;
        let mut input_area_opt: Option<Rect> = None;

        if state.is_saving || state.is_searching {
            // Layout: List Area, Input Area
            constraints = vec![Constraint::Min(0), Constraint::Length(input_prompt_height)];
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);
            list_area = chunks[0];
            input_area_opt = Some(chunks[1]);
        } else {
            // Layout: List Area only
            constraints = vec![Constraint::Min(0)];
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);
            list_area = chunks[0];
            // input_area_opt remains None
        };

        // --- Render Main View (Always) ---

        // --- Render List Area ---
        let list_title = if state.is_searching {
            format!(" Save/Load Project (Filter: {}) ", state.search_query)
        } else {
            " Save/Load Project ".to_string()
        };
        let list_block = Block::default()
            .borders(Borders::ALL)
            .title(list_title)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::White));
        frame.render_widget(list_block.clone(), list_area); // Render the block frame first
        let inner_list_area = list_block.inner(list_area); // Get the inner area *after* rendering the block

        // Render project list within the inner area
        // Filter projects (needed if list is shown)
        let filtered_projects: Vec<_> = state
            .projects
            .iter()
            .filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name))
            .cloned()
            .collect();
        // Clamp selected index (needed if list is shown)
        let num_filtered = filtered_projects.len();
        let current_selected_index = if num_filtered == 0 {
            0
        } else {
            state.selected_index.min(num_filtered - 1)
        };
        // Ensure list doesn't overwrite the help text area if drawn last
        let list_render_area = if !state.show_help && inner_list_area.height > 1 {
            Rect {
                height: inner_list_area.height - 1,
                ..inner_list_area
            } // Leave last line for help text
        } else {
            inner_list_area
        };
        render_project_list(
            frame,
            list_render_area,
            &filtered_projects,
            current_selected_index,
        );

        // --- Render Input Area (Search or Save, if applicable) ---
        if let Some(input_render_area) = input_area_opt {
            if state.is_searching {
                // Render Search Input
                let search_block = Block::default()
                    .borders(Borders::ALL)
                    .title(" Search Query (Type, Esc: Clear, Enter: Exit Keeping Filter) ")
                    .style(Style::default().fg(Color::Yellow));

                let search_paragraph = Paragraph::new(state.search_query.as_str())
                    .style(Style::default().fg(Color::White))
                    .block(search_block.clone())
                    .alignment(Alignment::Left)
                    .wrap(ratatui::widgets::Wrap { trim: false });

                frame.render_widget(search_paragraph, input_render_area);
                let cursor_x = input_render_area.x + 1 + state.search_query.chars().count() as u16;
                let cursor_y = input_render_area.y + 1;
                // Make sure cursor is only shown when actually searching
                if state.is_searching {
                    frame.set_cursor_position(Rect::new(cursor_x, cursor_y, 1, 1));
                }
            } else if state.is_saving {
                // Render Save Input
                let mut save_textarea = state.input_area.clone();
                save_textarea.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Save Project As (Enter: Confirm, Esc: Cancel) ")
                        .style(Style::default().fg(Color::Yellow)),
                );
                save_textarea.set_style(Style::default().fg(Color::White));
                frame.render_widget(&save_textarea, input_render_area);
                // Cursor is handled by TextArea widget
            }
        }

        // --- Render Help Text Bar --- -> Now Render Help Text INSIDE List Box
        // Only render this help text if the popup is NOT shown
        if !state.show_help {
            // Calculate the area for the help text in the bottom right of the inner list area
            let help_text_string = "?: Help "; // Include the space here
            let help_text_width = help_text_string.len() as u16; // Use the new string length
            // Ensure width doesn't exceed inner area width
            let actual_width = help_text_width.min(inner_list_area.width);
            if inner_list_area.width >= actual_width && inner_list_area.height > 0 {
                // Check if there's space (use >= for exact fit)
                let help_text_area = Rect::new(
                    inner_list_area.right().saturating_sub(actual_width), // x position
                    inner_list_area.bottom().saturating_sub(1),           // y position (last line)
                    actual_width,                                         // width
                    1,                                                    // height
                );

                // Create the styled spans: White '?' and Gray ': Help '
                let help_spans = vec![
                    Span::styled("?", Style::default().fg(Color::White)), // White '?'
                    Span::styled(": Help ", key_style), // Gray ': Help ' (using existing gray key_style)
                ];
                // Create the paragraph aligned to the right (within its small rect)
                let help_paragraph =
                    Paragraph::new(Line::from(help_spans)).alignment(Alignment::Right);
                // Render it in the calculated area inside the list box
                frame.render_widget(help_paragraph, help_text_area);
            }
        }

        // --- Render Help Popup (if active, drawn *last* to overlay) ---
        if state.show_help {
            let popup_area = centered_rect(60, 50, area); // Use the helper

            // Create the popup block with uniform padding
            let popup_block = Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .style(Style::default().fg(Color::White))
                .padding(Padding::uniform(1)); // Changed back to uniform padding of 1

            // Get the help text
            let help_lines = create_help_text(state);
            let help_paragraph = Paragraph::new(help_lines)
                .block(popup_block) // Add the block *here*
                .alignment(Alignment::Left)
                .wrap(ratatui::widgets::Wrap { trim: true });

            // Clear the area *before* rendering the popup paragraph
            frame.render_widget(Clear, popup_area);
            // Render the paragraph (which includes the block)
            frame.render_widget(help_paragraph, popup_area);

            // Hide the main cursor if the popup is shown and we are not in save mode
            if !state.is_saving {
                frame.set_cursor_position(Rect::default()); // Move cursor off-screen
            }
        }

        // --- Render Confirmation Popups (drawn *last* to overlay) ---
        if state.show_delete_confirmation {
            let popup_area = centered_rect(40, 20, area); // Smaller popup
            let project_name = state.project_to_delete.as_deref().unwrap_or("Error");
            let text = vec![
                Line::from(Span::styled(
                    format!("Really delete '{}'?", project_name),
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        " Enter",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::LightRed),
                    ),
                    Span::raw("/"),
                    Span::styled(
                        "Y",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::LightRed),
                    ),
                    Span::raw(": Yes"),
                    Span::raw("   "),
                    Span::styled(
                        "Esc",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Gray),
                    ),
                    Span::raw("/"),
                    Span::styled(
                        "N",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Gray),
                    ),
                    Span::raw(": No"),
                ]),
            ];
            let paragraph = Paragraph::new(text)
                .block(
                    Block::default()
                        .title(" Confirm Delete ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Double)
                        .style(Style::default().fg(Color::Red)),
                )
                .alignment(Alignment::Center)
                .wrap(ratatui::widgets::Wrap { trim: true });
            frame.render_widget(Clear, popup_area);
            frame.render_widget(paragraph, popup_area);
            frame.set_cursor_position(Rect::default()); // Hide main cursor
        } else if state.show_save_overwrite_confirmation {
            let popup_area = centered_rect(40, 20, area); // Smaller popup
            let project_name = state.project_to_overwrite.as_deref().unwrap_or("Error");
            let text = vec![
                Line::from(Span::styled(
                    format!("Overwrite '{}'?", project_name),
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        " Enter",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::LightGreen),
                    ),
                    Span::raw("/"),
                    Span::styled(
                        "Y",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::LightGreen),
                    ),
                    Span::raw(": Yes"),
                    Span::raw("   "),
                    Span::styled(
                        "Esc",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Gray),
                    ),
                    Span::raw("/"),
                    Span::styled(
                        "N",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Gray),
                    ),
                    Span::raw(": No"),
                ]),
            ];
            let paragraph = Paragraph::new(text)
                .block(
                    Block::default()
                        .title(" Confirm Overwrite ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Double)
                        .style(Style::default().fg(Color::Yellow)),
                )
                .alignment(Alignment::Center)
                .wrap(ratatui::widgets::Wrap { trim: true });
            frame.render_widget(Clear, popup_area);
            frame.render_widget(paragraph, popup_area);
            frame.set_cursor_position(Rect::default()); // Hide main cursor
        }
    }
}

/// Helper function to render the project list.
fn render_project_list(
    frame: &mut Frame,
    area: Rect,
    projects: &[(
        String,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
        Option<f32>,
        Option<usize>,
    )],
    selected_index: usize,
) {
    let list_items: Vec<ListItem> = projects
        .iter()
        .enumerate()
        .map(|(i, (name, created_at, updated_at, tempo, line_count))| {
            let mut spans = vec![Span::styled(
                format!("{:<25}", name),
                Style::default().fg(Color::White),
            )]; // Left align name with padding

            // Style for metadata
            let meta_style_label = Style::default().fg(Color::DarkGray);
            let meta_style_value = Style::default().fg(Color::Gray);

            // Tempo
            spans.push(Span::styled(" Tempo: ", meta_style_label));
            let tempo_str = tempo.map_or_else(|| "N/A".to_string(), |t| format!("{:.1}", t));
            spans.push(Span::styled(format!("{:<6}", tempo_str), meta_style_value)); // Pad tempo

            // Line Count
            spans.push(Span::styled(" Lines: ", meta_style_label));
            let lines_str = line_count.map_or_else(|| "N/A".to_string(), |lc| lc.to_string());
            spans.push(Span::styled(format!("{:<4}", lines_str), meta_style_value)); // Pad line count

            // Timestamps
            let time_style = Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC);
            let time_format = "%Y-%m-%d %H:%M";

            if let Some(updated) = updated_at {
                let local_updated: DateTime<Local> = (*updated).into();
                spans.push(Span::styled(
                    format!(" (Saved: {})", local_updated.format(time_format)),
                    time_style,
                ));
            } else if let Some(created) = created_at {
                // Show created only if updated is missing
                let local_created: DateTime<Local> = (*created).into();
                spans.push(Span::styled(
                    format!(" (Created: {})", local_created.format(time_format)),
                    time_style,
                ));
            }

            let item_style = if i == selected_index {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(spans)).style(item_style)
        })
        .collect();

    let list = List::new(list_items);
    frame.render_widget(list, area);
}

/// Creates the help text lines based on the current state.
fn create_help_text(state: &SaveLoadState) -> Vec<Line<'static>> {
    let key_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::White);

    let mut lines = vec![];

    if state.show_delete_confirmation {
        lines.push(Line::from(vec![
            Span::styled("  Enter/Y ", key_style),
            Span::styled(": Confirm Delete", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Esc/N   ", key_style),
            Span::styled(": Cancel Delete", desc_style),
        ]));
    } else if state.show_save_overwrite_confirmation {
        lines.push(Line::from(vec![
            Span::styled("  Enter/Y ", key_style),
            Span::styled(": Confirm Overwrite", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Esc/N   ", key_style),
            Span::styled(": Cancel Overwrite", desc_style),
        ]));
    } else if state.is_saving {
        lines.push(Line::from(vec![
            Span::styled("  Enter   ", key_style),
            Span::styled(": Confirm Save (or prompt overwrite)", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Esc     ", key_style),
            Span::styled(": Cancel Save", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  (Type)  ", key_style),
            Span::styled(": Enter project name", desc_style),
        ]));
    } else if state.is_searching {
        lines.push(Line::from(vec![
            Span::styled("  Esc     ", key_style),
            Span::styled(": Clear search & Exit", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Enter   ", key_style),
            Span::styled(": Keep filter & Exit", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ↑ / ↓   ", key_style),
            Span::styled(": Navigate while searching", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Backspace ", key_style),
            Span::styled(": Delete character", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  (Type)  ", key_style),
            Span::styled(": Update search query", desc_style),
        ]));
    } else {
        // List view mode
        lines.push(Line::from(vec![
            Span::styled("  ↑ / ↓   ", key_style),
            Span::styled(": Navigate List", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  /       ", key_style),
            Span::styled(": Start Search/Filter", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Enter   ", key_style),
            Span::styled(": Save/Overwrite Selected Project", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  l       ", key_style),
            Span::styled(": Load Project (Immediate)", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Ctrl+L  ", key_style),
            Span::styled(": Load Project (End of Scene)", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  s       ", key_style),
            Span::styled(": Save New Project (Enter Name)", desc_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Del/Bksp", key_style),
            Span::styled(": Delete Selected Project (Confirm)", desc_style),
        ]));
    }

    // Common keys shown when specific popups are not active
    if !state.show_delete_confirmation && !state.show_save_overwrite_confirmation {
        lines.push(Line::from(" "));
        lines.push(Line::from(vec![
            Span::styled("  ?       ", key_style),
            Span::styled(": Toggle this Help", desc_style),
        ]));
        if state.show_help {
            // Only show Esc binding when help is visible
            lines.push(Line::from(vec![
                Span::styled("  Esc     ", key_style),
                Span::styled(": Close Help", desc_style),
            ]));
        }
    }

    lines
}

/// Helper function to create a centered rect.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
