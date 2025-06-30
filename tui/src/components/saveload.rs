//! Component responsible for handling the saving and loading of project snapshots.
//! It provides a user interface for listing existing projects, searching/filtering,
//! saving the current state, loading a previous state, and deleting projects.

use crate::app::App;
use crate::components::Component;
use crate::disk;
use crate::event::{AppEvent, Event};
use crate::utils::layout::centered_rect;
use crate::utils::styles::CommonStyles;
use chrono::{DateTime, Local, Utc};
use color_eyre::Result as EyreResult;
use corelib::schedule::action_timing::ActionTiming;
use corelib::server::client::ClientMessage;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Widget},
};
use tui_textarea::TextArea;

/// Represents the state of the Save/Load UI component.
///
/// This struct holds all the dynamic information required to render the Save/Load
/// screen, including the list of projects, user input, selection state, and
/// visibility of popups.
pub struct SaveLoadState {
    /// List of available projects.
    /// Each tuple contains: (name, creation_time, last_save_time, tempo, line_count).
    pub projects: Vec<(
        String,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
        Option<f32>,
        Option<usize>,
    )>,
    /// Index of the currently selected project in the filtered list.
    pub selected_index: usize,
    /// Text input area used when saving a new project or confirming overwrite.
    pub input_area: TextArea<'static>,
    /// Flag indicating whether the user is currently entering a project name to save.
    pub is_saving: bool,
    /// A message displayed to the user (e.g., status updates, errors).
    pub status_message: String,
    /// Flag indicating whether the project list is being filtered by a search query.
    pub is_searching: bool,
    /// The current string used to filter the project list.
    pub search_query: String,
    /// Flag indicating whether the help popup is visible.
    pub show_help: bool,
    /// Flag indicating whether the delete confirmation popup is visible.
    pub show_delete_confirmation: bool,
    /// The name of the project pending deletion confirmation.
    pub project_to_delete: Option<String>,
    /// Flag indicating whether the save overwrite confirmation popup is visible.
    pub show_save_overwrite_confirmation: bool,
    /// The name of the project pending save overwrite confirmation.
    pub project_to_overwrite: Option<String>,
}

impl Default for SaveLoadState {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveLoadState {
    /// Creates a new `SaveLoadState` with default values.
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

/// The main component for handling Save/Load functionality and UI.
///
/// This struct implements the `Component` trait to handle events and draw the UI.
/// It uses `SaveLoadState` to manage its internal state.
#[derive(Default)]
pub struct SaveLoadComponent;

impl SaveLoadComponent {
    /// Creates a new `SaveLoadComponent`.
    pub fn new() -> Self {
        Self
    }
}

/// Performs a simple case-insensitive fuzzy match.
///
/// Returns `true` if all characters in `query` appear sequentially in `text`.
/// An empty query always matches.
fn simple_fuzzy_match(query: &str, text: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let mut query_chars = query.chars().peekable();
    let mut text_chars = text.chars();

    while let Some(q_char) = query_chars.peek() {
        match text_chars.find(|t_char| t_char.eq_ignore_ascii_case(q_char)) {
            Some(_) => {
                query_chars.next();
            }
            None => {
                return false;
            }
        }
    }
    true
}

impl Component for SaveLoadComponent {
    /// Handles key events for the Save/Load component.
    ///
    /// This function manages different input modes (list navigation, saving, searching,
    /// help popup, confirmation dialogs) and triggers corresponding actions like
    /// loading, saving, deleting projects, or updating the UI state.
    fn handle_key_event(&self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
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
                _ => return Ok(true), // Consume all other keys when help is shown
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

                        // Spawn background task for deletion
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
                        state.status_message =
                            format!("Requesting snapshot to overwrite '{}'...", project_name);
                        state.is_saving = false;
                        state.input_area = TextArea::default();
                        state.input_area.insert_str(project_name); // Keep name for snapshot handler
                        state
                            .input_area
                            .set_block(Block::default().borders(Borders::NONE));
                        should_send_snapshot = true;
                    } else {
                        state.status_message =
                            "Error: No project specified for overwrite.".to_string();
                    }
                    state.show_save_overwrite_confirmation = false;
                    state.project_to_overwrite = None; // Clear confirmation state

                    // Request snapshot from core after confirming overwrite
                    if should_send_snapshot {
                        app.send_client_message(ClientMessage::GetSnapshot);
                    }
                    return Ok(true);
                }
                // Cancel Overwrite
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    state.show_save_overwrite_confirmation = false;
                    state.project_to_overwrite = None;
                    state.is_saving = false; // Exit save mode if cancelling overwrite
                    state.input_area = TextArea::default(); // Clear input area
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
                // Cancel save input
                KeyCode::Esc => {
                    state.is_saving = false;
                    state.input_area = TextArea::default();
                    state
                        .input_area
                        .set_block(Block::default().borders(Borders::NONE));
                    state.status_message = "Save cancelled.".to_string();
                    return Ok(true);
                }
                // Confirm save input
                KeyCode::Enter => {
                    let project_name = state.input_area.lines()[0].trim().to_string();
                    let mut should_send_snapshot = false;

                    if project_name.is_empty() {
                        state.status_message = "Project name cannot be empty.".to_string();
                    } else {
                        // Check if project already exists
                        let exists = state
                            .projects
                            .iter()
                            .any(|(name, ..)| *name == project_name);
                        if exists {
                            // Show overwrite confirmation popup
                            state.project_to_overwrite = Some(project_name);
                            state.show_save_overwrite_confirmation = true;
                            state.status_message = format!(
                                "Project '{}' already exists. Overwrite? (y/n)",
                                state.project_to_overwrite.as_ref().unwrap()
                            );
                            // Don't request snapshot yet, wait for confirmation
                        } else {
                            // Project doesn't exist, proceed with save
                            state.status_message =
                                format!("Requesting snapshot to save as '{}'...", project_name);
                            state.is_saving = false;
                            // Clear input area but keep the name in it for the snapshot handler
                            state.input_area = TextArea::default();
                            state.input_area.insert_str(&project_name);
                            state
                                .input_area
                                .set_block(Block::default().borders(Borders::NONE));

                            should_send_snapshot = true;
                        }
                    }

                    // Request snapshot if saving a new project
                    if should_send_snapshot {
                        app.send_client_message(ClientMessage::GetSnapshot);
                    }

                    return Ok(true);
                }
                // Handle text input within the TextArea
                _ => {
                    match key_code {
                        // Prevent list navigation keys from affecting the list while typing
                        KeyCode::Up
                        | KeyCode::Down
                        | KeyCode::PageUp
                        | KeyCode::PageDown
                        | KeyCode::Home
                        | KeyCode::End => {
                            return Ok(true);
                        }
                        // Let TextArea handle other keys (typing, backspace, etc.)
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
                // Exit search mode (ESC clears query, Enter keeps it)
                KeyCode::Esc | KeyCode::Enter => {
                    state.is_searching = false;
                    if key_code == KeyCode::Esc {
                        state.search_query.clear();
                    }
                    state.status_message = "Exited search.".to_string();
                    state.selected_index = 0; // Reset selection to top of potentially filtered list
                    return Ok(true);
                }
                // Handle backspace
                KeyCode::Backspace => {
                    if !state.search_query.is_empty() {
                        state.search_query.pop();
                        state.selected_index = 0; // Reset selection when query changes
                    }
                    return Ok(true);
                }
                // Handle character input for the search query
                KeyCode::Char(c)
                    if !key_modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    state.search_query.push(c);
                    state.selected_index = 0; // Reset selection when query changes
                    return Ok(true);
                }
                // Allow list navigation while searching
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
                // Ignore other keys in search mode
                _ => {
                    return Ok(false);
                }
            }
        }

        // --- Handle List Navigation/Actions Mode (Default Mode) ---
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
            // Navigate up in the list
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
            // Navigate down in the list
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
            // Load selected project (Immediate or EndOfScene)
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
                    // Spawn background task for loading
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
                    // Switch back to Grid view after initiating load
                    let _ = app.events.sender.send(Event::App(AppEvent::SwitchToGrid));
                } else {
                    state.status_message = "No project selected to load.".to_string();
                }
                Ok(true)
            }
            // Enter saving mode to save a new project
            (KeyCode::Char('s'), _) => {
                state.is_saving = true;
                state.is_searching = false; // Exit search mode if active
                state.search_query.clear();
                state.input_area = TextArea::default();
                state
                    .input_area
                    .set_block(Block::default().borders(Borders::NONE));
                state.status_message = "Enter project name to save.".to_string();
                Ok(true)
            }
            // Trigger save/overwrite confirmation for the selected project
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
                    state.is_saving = false; // Not entering text input mode
                    state.status_message = format!("Overwrite project '{}'? (y/n)", project_name);
                } else {
                    // No project selected (e.g., empty list), do nothing
                    state.status_message = "No project selected.".to_string();
                }
                Ok(true)
            }
            // Trigger delete confirmation for the selected project
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
            // Pass unhandled keys up to the parent component/application
            _ => Ok(false),
        }
    }

    /// Draws the Save/Load UI component.
    ///
    /// This method renders the project list, input areas (save/search), help text,
    /// and popups (help, confirmations) based on the current `SaveLoadState`.
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let state = &app.interface.components.save_load_state;
        let key_style = CommonStyles::key_binding_themed(&app.client_config.theme);

        // --- Define Main Layout (List and optional Input Area) ---
        let input_prompt_height = 3; // Height reserved for search/save input
        let constraints: Vec<Constraint>;
        let list_area: Rect;
        let mut input_area_opt: Option<Rect> = None;

        if state.is_saving || state.is_searching {
            // Layout with Input Area at the bottom
            constraints = vec![Constraint::Min(0), Constraint::Length(input_prompt_height)];
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);
            list_area = chunks[0];
            input_area_opt = Some(chunks[1]);
        } else {
            // Layout with List Area only
            constraints = vec![Constraint::Min(0)];
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);
            list_area = chunks[0];
        };

        // --- Render List Area ---
        let list_title = if state.is_searching {
            format!(" Filter: {} ", state.search_query)
        } else {
            "".to_string()
        };
        let list_block = Block::default()
            .borders(Borders::ALL)
            .title(list_title)
            .border_type(BorderType::Thick)
            .style(CommonStyles::default_text_themed(&app.client_config.theme));
        let inner_list_area = list_block.inner(list_area); // Area inside the block borders
        frame.render_widget(list_block, list_area);

        // Filter projects based on search query
        let filtered_projects: Vec<_> = state
            .projects
            .iter()
            .filter(|(name, ..)| simple_fuzzy_match(&state.search_query, name))
            .cloned()
            .collect();
        // Clamp selected index to the bounds of the filtered list
        let num_filtered = filtered_projects.len();
        let current_selected_index = if num_filtered == 0 {
            0
        } else {
            state.selected_index.min(num_filtered - 1)
        };
        // Calculate actual render area for the list, leaving space for help text if needed
        let list_render_area = if !state.show_help && inner_list_area.height > 1 {
            Rect {
                height: inner_list_area.height - 1, // Reserve bottom line for help text
                ..inner_list_area
            }
        } else {
            inner_list_area
        };

        // Render the project list using the custom widget
        let project_list_widget =
            ProjectListWidget::new(&filtered_projects, current_selected_index);
        frame.render_widget(project_list_widget, list_render_area);

        // --- Render Input Area (Search or Save) ---
        if let Some(input_render_area) = input_area_opt {
            if state.is_searching {
                // Render Search Input Area
                let search_block = Block::default()
                    .borders(Borders::ALL)
                    .title(" Search Query (Type, Esc: Clear, Enter: Exit Keeping Filter) ")
                    .style(CommonStyles::warning_themed(&app.client_config.theme));

                let search_paragraph = Paragraph::new(state.search_query.as_str())
                    .style(CommonStyles::default_text_themed(&app.client_config.theme))
                    .block(search_block)
                    .alignment(Alignment::Left)
                    .wrap(ratatui::widgets::Wrap { trim: false });

                frame.render_widget(search_paragraph, input_render_area);

                // Set cursor position for search input
                if state.is_searching {
                    let cursor_x =
                        input_render_area.x + 1 + state.search_query.chars().count() as u16;
                    let cursor_y = input_render_area.y + 1;
                    // Ensure cursor stays within input area bounds
                    if cursor_x < input_render_area.right() - 1
                        && cursor_y < input_render_area.bottom() - 1
                    {
                        frame.set_cursor_position(Rect::new(cursor_x, cursor_y, 1, 1));
                    }
                }
            } else if state.is_saving {
                // Render Save Input Area using tui-textarea
                let mut save_textarea = state.input_area.clone();
                save_textarea.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Save Project As (Enter: Confirm, Esc: Cancel) ")
                        .style(CommonStyles::warning_themed(&app.client_config.theme)),
                );
                save_textarea
                    .set_style(CommonStyles::default_text_themed(&app.client_config.theme));
                frame.render_widget(&save_textarea, input_render_area);
                // Cursor visibility/position is handled by the TextArea widget itself
            }
        }

        // --- Render Short Help Text (Bottom right of list area) ---
        // Only shown when the main help popup is not active
        if !state.show_help {
            let help_text_string = "?: Help ";
            let help_text_width = help_text_string.len() as u16;
            let actual_width = help_text_width.min(inner_list_area.width);
            // Ensure there is space to render the help text
            if inner_list_area.width >= actual_width && inner_list_area.height > 0 {
                let help_text_area = Rect::new(
                    inner_list_area.right().saturating_sub(actual_width),
                    inner_list_area.bottom().saturating_sub(1), // Position on the last line
                    actual_width,
                    1,
                );
                let help_spans = vec![
                    Span::styled(
                        "?",
                        CommonStyles::default_text_themed(&app.client_config.theme),
                    ),
                    Span::styled(": Help ", key_style),
                ];
                let help_paragraph =
                    Paragraph::new(Line::from(help_spans)).alignment(Alignment::Right);
                frame.render_widget(help_paragraph, help_text_area);
            }
        }

        // --- Render Help Popup (Overlay) ---
        if state.show_help {
            let popup_area = centered_rect(60, 50, area); // Calculate centered area
            let help_lines = create_help_text(state); // Generate help text content
            let help_widget = HelpPopupWidget::new(help_lines); // Create help widget

            frame.render_widget(Clear, popup_area); // Clear background before drawing popup
            frame.render_widget(help_widget, popup_area);

            // Hide the main cursor when help popup is visible (unless in save mode)
            if !state.is_saving {
                frame.set_cursor_position(Rect::default());
            }
        }

        // --- Render Confirmation Popups (Overlays) ---
        if state.show_delete_confirmation {
            let popup_area = centered_rect(40, 20, area); // Smaller centered area
            let project_name = state.project_to_delete.as_deref().unwrap_or("Error");
            let confirm_widget = ConfirmationPopupWidget::new(
                " Confirm Delete ".to_string(),
                format!("Really delete '{}'?", project_name),
                CommonStyles::error(),
                true, // Destructive action style
            );
            frame.render_widget(Clear, popup_area);
            frame.render_widget(confirm_widget, popup_area);
            frame.set_cursor_position(Rect::default()); // Hide main cursor
        } else if state.show_save_overwrite_confirmation {
            let popup_area = centered_rect(40, 20, area);
            let project_name = state.project_to_overwrite.as_deref().unwrap_or("Error");
            let confirm_widget = ConfirmationPopupWidget::new(
                " Confirm Overwrite ".to_string(),
                format!("Overwrite '{}'?", project_name),
                CommonStyles::warning_themed(&app.client_config.theme),
                false, // Non-destructive action style
            );
            frame.render_widget(Clear, popup_area);
            frame.render_widget(confirm_widget, popup_area);
            frame.set_cursor_position(Rect::default()); // Hide main cursor
        }
    }
}

/// A widget for rendering the list of projects.
struct ProjectListWidget<'a> {
    /// Slice of project data tuples.
    projects: &'a [(
        String,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
        Option<f32>,
        Option<usize>,
    )],
    /// Index of the currently selected project.
    selected_index: usize,
}

impl<'a> ProjectListWidget<'a> {
    /// Creates a new `ProjectListWidget`.
    fn new(
        projects: &'a [(
            String,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
            Option<f32>,
            Option<usize>,
        )],
        selected_index: usize,
    ) -> Self {
        Self {
            projects,
            selected_index,
        }
    }
}

impl<'a> Widget for ProjectListWidget<'a> {
    /// Renders the project list into the buffer.
    fn render(self, area: Rect, buf: &mut Buffer) {
        let list_items: Vec<ListItem> = self
            .projects
            .iter()
            .enumerate()
            .map(|(i, (name, created_at, updated_at, tempo, line_count))| {
                // Format project name (left-aligned)
                let mut spans = vec![Span::styled(
                    format!("{:<25}", name),
                    Style::default().fg(if i == self.selected_index {
                        Color::Black
                    } else {
                        Color::White
                    }),
                )];
                let meta_style_label = CommonStyles::description();
                let meta_style_value = Style::default().fg(if i == self.selected_index {
                    Color::Black
                } else {
                    Color::Gray
                });

                // Format tempo
                spans.push(Span::styled(" Tempo: ", meta_style_label));
                let tempo_str = tempo.map_or_else(|| "N/A".to_string(), |t| format!("{:.1}", t));
                spans.push(Span::styled(format!("{:<6}", tempo_str), meta_style_value));

                // Format line count
                spans.push(Span::styled(" Lines: ", meta_style_label));
                let lines_str = line_count.map_or_else(|| "N/A".to_string(), |lc| lc.to_string());
                spans.push(Span::styled(format!("{:<4}", lines_str), meta_style_value));

                // Format timestamp (Saved or Created)
                let time_style = CommonStyles::description().add_modifier(Modifier::ITALIC);
                let time_format = "%Y-%m-%d %H:%M";
                if let Some(updated) = updated_at {
                    let local_updated: DateTime<Local> = (*updated).into();
                    spans.push(Span::styled(
                        format!(" (Saved: {})", local_updated.format(time_format)),
                        time_style,
                    ));
                } else if let Some(created) = created_at {
                    let local_created: DateTime<Local> = (*created).into();
                    spans.push(Span::styled(
                        format!(" (Created: {})", local_created.format(time_format)),
                        time_style,
                    ));
                }

                // Apply selection highlight style
                let item_style = if i == self.selected_index {
                    Style::default().fg(Color::Black).bg(Color::White)
                } else {
                    Style::default()
                };

                // Create 3-line entry with content on middle line
                let lines = vec![
                    Line::from(""), // Empty top line
                    Line::from(vec![Span::raw(" ")].into_iter().chain(spans).collect::<Vec<_>>()), // Content on middle line with left padding
                    Line::from(""), // Empty bottom line
                ];

                ListItem::new(lines).style(item_style)
            })
            .collect();

        // Create the Ratatui List widget
        let list = List::new(list_items);
        // Render the list using fully qualified syntax to avoid ambiguity
        ratatui::widgets::Widget::render(list, area, buf);
    }
}

/// A widget for rendering confirmation popups (Delete/Overwrite).
struct ConfirmationPopupWidget {
    title: String,
    prompt: String,
    style: Style,
    is_destructive: bool, // Affects the styling of the 'Yes' option
}

impl ConfirmationPopupWidget {
    /// Creates a new `ConfirmationPopupWidget`.
    fn new(title: String, prompt: String, style: Style, is_destructive: bool) -> Self {
        Self {
            title,
            prompt,
            style,
            is_destructive,
        }
    }
}

impl Widget for ConfirmationPopupWidget {
    /// Renders the confirmation popup into the buffer.
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Determine styles based on whether the action is destructive
        let (yes_style, no_style) = if self.is_destructive {
            (
                Style::default().bold().fg(Color::LightRed),
                Style::default().bold().fg(Color::Gray),
            )
        } else {
            (
                Style::default().bold().fg(Color::LightGreen),
                Style::default().bold().fg(Color::Gray),
            )
        };

        // Construct the text lines for the popup
        let text = vec![
            Line::from(Span::styled(self.prompt, self.style)),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Enter", yes_style),
                Span::raw("/"),
                Span::styled("Y", yes_style),
                Span::raw(": Yes"),
                Span::raw("   "),
                Span::styled("Esc", no_style),
                Span::raw("/"),
                Span::styled("N", no_style),
                Span::raw(": No"),
            ]),
        ];
        // Create the paragraph with a styled block
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(self.title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .style(self.style),
            )
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        // Clear the area before rendering the popup
        Clear.render(area, buf);
        paragraph.render(area, buf);
    }
}

/// A widget for rendering the help popup.
struct HelpPopupWidget {
    /// The lines of help text to display.
    lines: Vec<Line<'static>>,
}

impl HelpPopupWidget {
    /// Creates a new `HelpPopupWidget`.
    fn new(lines: Vec<Line<'static>>) -> Self {
        Self { lines }
    }
}

impl Widget for HelpPopupWidget {
    /// Renders the help popup into the buffer.
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create the block frame for the popup
        let popup_block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .style(Style::default().fg(Color::White))
            .padding(Padding::uniform(1));

        // Create the paragraph containing the help lines
        let help_paragraph = Paragraph::new(self.lines)
            .block(popup_block)
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        // Clear the area before rendering the popup
        Clear.render(area, buf);
        help_paragraph.render(area, buf);
    }
}

/// Creates the dynamic help text lines based on the current UI state.
///
/// The displayed keybindings change depending on whether the user is in the list view,
/// saving, searching, or confirming an action.
fn create_help_text(state: &SaveLoadState) -> Vec<Line<'static>> {
    let key_style = Style::default().fg(Color::Green).bold();
    let desc_style = Style::default().fg(Color::White);

    let mut lines = vec![];

    // Specific help text for different modes/popups
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
        // Default help text for the main list view mode
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

    // Common keys shown unless a confirmation popup is active
    if !state.show_delete_confirmation && !state.show_save_overwrite_confirmation {
        lines.push(Line::from(" ")); // Spacer line
        lines.push(Line::from(vec![
            Span::styled("  ?       ", key_style),
            Span::styled(": Toggle this Help", desc_style),
        ]));
        // Only show Esc binding to close help when help is visible
        if state.show_help {
            lines.push(Line::from(vec![
                Span::styled("  Esc     ", key_style),
                Span::styled(": Close Help", desc_style),
            ]));
        }
    }

    lines
}
