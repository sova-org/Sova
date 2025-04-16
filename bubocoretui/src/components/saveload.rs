use crate::App;
use crate::components::Component;
use crate::disk;
use crate::event::{AppEvent, Event};
use bubocorelib::server::client::ClientMessage;
use bubocorelib::schedule::ActionTiming;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, BorderType},
    Frame,
};
use tui_textarea::TextArea;
use chrono::{DateTime, Utc, Local};

/// State for the save/load component
pub struct SaveLoadState {
    /// Projects list (name, creation date, last save date)
    pub projects: Vec<(String, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>,
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
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        let state = &mut app.interface.components.save_load_state;
        let key_code = key_event.code;
        let key_modifiers = key_event.modifiers;

        // --- Handle Saving Input Mode --- 
        if state.is_saving {
            match key_code {
                // Cancel save
                KeyCode::Esc => {
                    state.is_saving = false;
                    state.input_area = TextArea::default();
                    state.input_area.set_block(
                        Block::default().borders(Borders::NONE)
                    );
                    state.status_message = "Save cancelled.".to_string();
                    return Ok(true);
                }
                // Confirm save
                KeyCode::Enter => {
                    let project_name = state.input_area.lines()[0].trim().to_string();
                    
                    if project_name.is_empty() {
                        state.status_message = "Project name cannot be empty.".to_string();
                    } else {
                        state.status_message = format!("Requesting snapshot to save as '{}'...", project_name);
                        state.is_saving = false;
                        let project_name_clone = project_name.clone();
                        app.send_client_message(ClientMessage::GetSnapshot);

                        let state_after_send = &mut app.interface.components.save_load_state;
                        state_after_send.input_area = TextArea::default();
                        state_after_send.input_area.insert_str(project_name_clone);
                        state_after_send.input_area.set_block(
                            Block::default().borders(Borders::NONE)
                        );
                    }
                    return Ok(true);
                }
                _ => {
                    let handled = state.input_area.input(key_event);
                    return Ok(handled);
                }
            }
        }

        // --- Handle Searching/Filtering Input Mode --- 
        if state.is_searching {
            match key_code {
                KeyCode::Esc | KeyCode::Enter => {
                    state.is_searching = false;
                    // Don't clear query on Enter, maybe user wants to refine?
                    if key_code == KeyCode::Esc {
                         state.search_query.clear();
                    }
                    state.status_message = "Exited search.".to_string();
                    // Reset selection when exiting search to avoid out-of-bounds if list shrinks
                    state.selected_index = 0;
                    return Ok(true);
                }
                KeyCode::Backspace => {
                    if !state.search_query.is_empty() {
                        state.search_query.pop();
                         // Reset selection when query changes
                         state.selected_index = 0;
                    }
                    return Ok(true);
                }
                KeyCode::Char(c) if !key_modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
                    state.search_query.push(c);
                     // Reset selection when query changes
                     state.selected_index = 0;
                    return Ok(true);
                }
                _ => { return Ok(false); } // Ignore other keys in search mode
            }
        }

        // --- Handle List Navigation/Actions Mode (when not saving or searching) --- 
        match (key_code, key_modifiers) {
            // Enter search mode
            (KeyCode::Char('/'), _) => {
                state.is_searching = true;
                state.search_query.clear();
                state.status_message = "Enter search query (Esc/Enter to exit)...".to_string();
                state.selected_index = 0; // Reset selection
                Ok(true)
            }
            // Navigate up (works on filtered list implicitly via draw)
            (KeyCode::Up, _) => {
                 let num_filtered = state.projects.iter().filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name)).count();
                if num_filtered > 0 {
                    state.selected_index = state.selected_index.saturating_sub(1);
                }
                Ok(true)
            }
            // Navigate down (works on filtered list implicitly via draw)
            (KeyCode::Down, _) => {
                 let num_filtered = state.projects.iter().filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name)).count();
                if num_filtered > 0 {
                    state.selected_index = (state.selected_index + 1).min(num_filtered.saturating_sub(1));
                }
                Ok(true)
            }
            // Load a project (needs to find the correct project from the filtered view)
            (KeyCode::Char('l'), modifiers) => {
                let timing = if modifiers.contains(KeyModifiers::CONTROL) {
                    ActionTiming::EndOfScene
                } else {
                    ActionTiming::Immediate
                };

                // Get the currently selected project *from the filtered list*
                let filtered_projects: Vec<_> = state.projects.iter()
                    .filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name))
                    .collect();

                if let Some((project_name, _, _)) = filtered_projects.get(state.selected_index) {
                    state.status_message = format!("Loading project '{}' ({:?})...", project_name, timing);
                    let proj_name = (*project_name).clone(); // Clone the name from the tuple reference
                    let event_sender = app.events.sender.clone();

                    tokio::spawn(async move {
                        match disk::load_project(&proj_name).await {
                            Ok(snapshot) => {
                                let _ = event_sender.send(Event::App(AppEvent::LoadProject(snapshot, timing)));
                            }
                            Err(e) => {
                                let _ = event_sender.send(Event::App(AppEvent::ProjectLoadError(e.to_string())));
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
                state.is_searching = false; // Exit search mode if active
                state.search_query.clear();
                state.input_area = TextArea::default();
                state.input_area.set_block(
                    Block::default().borders(Borders::NONE)
                );
                state.status_message = "Enter project name to save.".to_string();
                Ok(true)
            }
            // Delete a project (needs to find the correct project from the filtered view)
            (KeyCode::Char('d'), crossterm::event::KeyModifiers::CONTROL) => {
                // Get the currently selected project *from the filtered list*
                let filtered_projects: Vec<_> = state.projects.iter()
                    .filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name))
                    .collect();

                if let Some((project_name, _, _)) = filtered_projects.get(state.selected_index) {
                    state.status_message = format!("Deleting project '{}'...", project_name);
                    let event_sender = app.events.sender.clone();
                    let proj_name = (*project_name).clone(); // Clone the name

                    // Adjust selected index *before* deletion
                    let num_filtered = filtered_projects.len();
                    if state.selected_index >= num_filtered.saturating_sub(1) {
                         state.selected_index = state.selected_index.saturating_sub(1);
                    }

                    tokio::spawn(async move {
                        match disk::delete_project(&proj_name).await {
                            Ok(_) => {
                                let _ = event_sender.send(Event::App(AppEvent::ProjectDeleted(proj_name)));
                            }
                            Err(e) => {
                                let _ = event_sender.send(Event::App(AppEvent::ProjectDeleteError(e.to_string())));
                            }
                        }
                    });
                } else {
                    state.status_message = "No project selected to delete.".to_string();
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

   
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let state = &app.interface.components.save_load_state;

        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);

        // Filter projects (needed if list is shown)
        let filtered_projects: Vec<_> = state.projects.iter()
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

        // --- Define Layout based on state ---
        let input_prompt_height = 3; // Height for save or search prompt
        let help_height = 1;
        let constraints: Vec<Constraint>;
        let list_area: Rect;
        let mut input_area: Option<Rect> = None; // *** Make mutable ***
        let help_area: Rect;

        if state.is_saving || state.is_searching {
            // Layout: List Area, Input Area, Help Area
            constraints = vec![
                Constraint::Min(0),
                Constraint::Length(input_prompt_height),
                Constraint::Length(help_height),
            ];
            let chunks = Layout::default().direction(Direction::Vertical).constraints(constraints).split(area);
            list_area = chunks[0];
            input_area = Some(chunks[1]); // Assign the middle chunk
            help_area = chunks[2];
        } else {
             // Layout: List Area, Help Area
             constraints = vec![
                Constraint::Min(0),
                Constraint::Length(help_height),
            ];
             let chunks = Layout::default().direction(Direction::Vertical).constraints(constraints).split(area);
             list_area = chunks[0];
             // input_area remains None
             help_area = chunks[1];
        };

        // --- Render List Area (Always) ---
        let list_title = if state.is_searching {
             // Show filter in title when searching and list is visible
             format!(" Save/Load Project (Filter: {}) ", state.search_query)
         } else {
             " Save/Load Project ".to_string()
         };
        let list_block = Block::default()
            .borders(Borders::ALL)
            .title(list_title)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::White));
        frame.render_widget(list_block.clone(), list_area);
        let inner_list_area = list_block.inner(list_area);
        // Call the helper function (assuming it's defined outside impl)
        render_project_list(frame, inner_list_area, &filtered_projects, current_selected_index);


        // --- Render Input Area (Search or Save, if applicable) ---
        if let Some(input_render_area) = input_area {
            if state.is_searching {
                 // Render Search Input
                 let search_block = Block::default()
                    .borders(Borders::ALL)
                    .title(" Search Query (Type, Esc: Clear, Enter: Exit Keeping Filter) ")
                    .style(Style::default().fg(Color::Yellow));

                let search_paragraph = Paragraph::new(state.search_query.as_str())
                    .style(Style::default().fg(Color::White)) // Ensure text visibility
                    .block(search_block.clone())
                    .alignment(Alignment::Left)
                    .wrap(ratatui::widgets::Wrap { trim: false });

                frame.render_widget(search_paragraph, input_render_area);
                // Show cursor manually at the end of the query string
                 frame.set_cursor(
                     input_render_area.x + 1 + state.search_query.chars().count() as u16,
                     input_render_area.y + 1
                 );
            } else if state.is_saving {
                 // Render Save Input
                 let mut save_textarea = state.input_area.clone();
                 save_textarea.set_block(
                     Block::default()
                         .borders(Borders::ALL)
                         .title(" Save Project As (Enter: Confirm, Esc: Cancel) ")
                         .style(Style::default().fg(Color::Yellow))
                 );
                 save_textarea.set_style(Style::default().fg(Color::White));
                 frame.render_widget(save_textarea.widget(), input_render_area);
            }
        }

        // --- Render Help Text ---
        let help_spans = if state.is_saving {
             vec![
                 Span::styled("Enter", key_style), Span::styled(": Confirm Save | ", help_style),
                 Span::styled("Esc", key_style), Span::styled(": Cancel", help_style),
            ]
        } else if state.is_searching {
             // Help text when search prompt is active
            vec![
                 Span::styled("Esc", key_style), Span::styled(": Clear & Exit | ", help_style),
                 Span::styled("Enter", key_style), Span::styled(": Keep Filter & Exit | ", help_style),
                 Span::styled("Type", key_style), Span::styled(": Update Query", help_style),
             ]
        } else { // Listing mode help text
            vec![
                 Span::styled("↑↓", key_style), Span::styled(": Navigate | ", help_style),
                 Span::styled("/", key_style), Span::styled(": Search | ", help_style),
                 Span::styled("l", key_style), Span::styled(": Load Now | ", help_style),
                 Span::styled("Ctrl+L", key_style), Span::styled(": Load next cycle | ", help_style),
                 Span::styled("s", key_style), Span::styled(": Save | ", help_style),
                 Span::styled("Ctrl+d", key_style), Span::styled(": Delete", help_style),
             ]
        };
        let help_paragraph = Paragraph::new(Line::from(help_spans))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, help_area);
    }
}

/// Helper function to render the project list.
fn render_project_list(
    frame: &mut Frame,
    area: Rect,
    projects: &[(String, Option<DateTime<Utc>>, Option<DateTime<Utc>>)],
    selected_index: usize,
) {
    let list_items: Vec<ListItem> = projects.iter().enumerate().map(|(i, (name, created_at, updated_at))| {
        let mut spans = vec![Span::styled(name, Style::default().fg(Color::White))];
        let time_style = Style::default().fg(Color::DarkGray);
        let time_format = "%Y-%m-%d %H:%M";
    
        if let Some(created) = created_at {
            let local_created: DateTime<Local> = (*created).into();
            spans.push(Span::styled(format!(" (Created: {})", local_created.format(time_format)), time_style));
        }
        if let Some(updated) = updated_at {
            let local_updated: DateTime<Local> = (*updated).into();
            spans.push(Span::styled(format!(" (Saved: {})", local_updated.format(time_format)), time_style));
        }
    
        let item_style = if i == selected_index {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default()
        };
    
        ListItem::new(Line::from(spans)).style(item_style)
    }).collect();

    let list = List::new(list_items);
    frame.render_widget(list, area);
}
