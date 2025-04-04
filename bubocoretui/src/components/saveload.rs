use crate::App;
use crate::components::Component;
use crate::disk;
use crate::event::{AppEvent, Event};
use bubocorelib::server::client::ClientMessage;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
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
    /// Indicates if a refresh is pending
    pub is_refresh_pending: bool,
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
            is_refresh_pending: true,
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

impl Component for SaveLoadComponent {
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        let state = &mut app.interface.components.save_load_state;
        let key_code = key_event.code;
        let key_modifiers = key_event.modifiers;
        if state.is_saving {
            match key_code {
                // Cancel save
                KeyCode::Esc => {
                    state.is_saving = false;
                    state.input_area.delete_line_by_head();
                    state.status_message = "Save cancelled.".to_string();
                    state.input_area.set_block(
                        Block::default().borders(Borders::NONE)
                    );
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

        // Handle List Navigation/Actions Mode
        match (key_code, key_modifiers) {
            // Navigate up
            (KeyCode::Up, _) => {
                if !state.projects.is_empty() {
                    state.selected_index = state.selected_index.saturating_sub(1);
                }
                Ok(true)
            }
            // Navigate down
            (KeyCode::Down, _) => {
                if !state.projects.is_empty() {
                    let len = state.projects.len();
                    state.selected_index = (state.selected_index + 1).min(len.saturating_sub(1));
                }
                Ok(true)
            }
            // Refresh the project list
            (KeyCode::Char('r'), _) => {
                state.status_message = "Requesting project list refresh...".to_string();
                state.is_refresh_pending = true; 
                Ok(true)
            }
            // Load a project
            (KeyCode::Char('l'), _) => {
                if let Some((project_name, _, _)) = state.projects.get(state.selected_index) {
                    state.status_message = format!("Loading project '{}'...", project_name);
                    let event_sender = app.events.sender.clone();
                    let proj_name = project_name.clone();
                    tokio::spawn(async move {
                        match disk::load_project(&proj_name).await {
                            Ok(snapshot) => {
                                let _ = event_sender.send(Event::App(AppEvent::SnapshotLoaded(snapshot)));
                            }
                            Err(e) => {
                                let _ = event_sender.send(Event::App(AppEvent::ProjectLoadError(e.to_string())));
                            }
                        }
                    });
                } else {
                    state.status_message = "No project selected to load.".to_string();
                }
                Ok(true)
            }
            // Save a project
            (KeyCode::Char('s'), _) => {
                state.is_saving = true;
                state.input_area = TextArea::default();
                state.input_area.set_block(
                    Block::default().borders(Borders::NONE)
                );
                state.status_message = "Enter project name to save.".to_string();
                Ok(true)
            }
            (KeyCode::Char('d'), crossterm::event::KeyModifiers::CONTROL) => {
                if let Some((project_name, _, _)) = state.projects.get(state.selected_index) {
                    state.status_message = format!("Deleting project '{}'...", project_name);
                    let event_sender = app.events.sender.clone();
                    let proj_name = project_name.clone();

                    if state.selected_index >= state.projects.len().saturating_sub(1) {
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

    fn before_draw(&mut self, app: &mut App) -> EyreResult<()> {
        let state = &mut app.interface.components.save_load_state;
        if state.is_refresh_pending {
            state.is_refresh_pending = false;
            state.status_message = "Refreshing project list...".to_string();

            let event_sender = app.events.sender.clone();
            tokio::spawn(async move {
                let result = disk::list_projects().await; 
                let event_result = result.map_err(|e| e.to_string());
                let _ = event_sender.send(Event::App(AppEvent::ProjectListLoaded(event_result)));
            });
        }
        Ok(())
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let state = &app.interface.components.save_load_state;

        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);

        if state.is_saving {
            // Saving Mode: Render Input Area 
            let save_block = Block::default()
                .title(" Save Project As ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow));

            frame.render_widget(save_block.clone(), area);
            let inner_area = save_block.inner(area);

            // Layout for input + help
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(inner_area);

            let input_render_area = chunks[0];
            let help_area = chunks[1];

            // Render the text area widget within the designated space
            frame.render_widget(&state.input_area, input_render_area);

            // Help text for saving mode
            let help_spans = vec![
                 Span::styled("Enter", key_style), Span::styled(": Confirm Save | ", help_style),
                 Span::styled("Esc", key_style), Span::styled(": Cancel", help_style),
            ];
            let help_paragraph = Paragraph::new(Line::from(help_spans))
                .alignment(Alignment::Center);
            frame.render_widget(help_paragraph, help_area);

        } else {
            // List Mode: Render Project List
            let list_block = Block::default()
                .borders(Borders::ALL)
                .title(" Save/Load Project ")
                .border_type(BorderType::Thick)
                .style(Style::default().fg(Color::White));

            frame.render_widget(list_block.clone(), area);
            let inner_area = list_block.inner(area);

             // Layout for list + help
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .split(inner_area);

            let list_render_area = chunks[0];
            let help_area = chunks[1];

            // Render the project list with metadata
            let list_items: Vec<ListItem> = state.projects.iter().enumerate().map(|(i, (name, created_at, updated_at))| {
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

                 let item_style = if i == state.selected_index {
                     Style::default().fg(Color::Black).bg(Color::Cyan)
                 } else {
                     Style::default()
                 };

                ListItem::new(Line::from(spans)).style(item_style)
            }).collect();

            let list = List::new(list_items)
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));

            frame.render_widget(list, list_render_area);

             // Help text for list mode
            let help_spans = vec![
                 Span::styled("↑↓", key_style), Span::styled(": Navigate | ", help_style),
                 Span::styled("l", key_style), Span::styled(": Load | ", help_style),
                 Span::styled("s", key_style), Span::styled(": Save | ", help_style),
                 Span::styled("r", key_style), Span::styled(": Refresh | ", help_style),
                 Span::styled("Ctrl+d", key_style), Span::styled(": Delete", help_style),
            ];
            let help_paragraph = Paragraph::new(Line::from(help_spans))
                .alignment(Alignment::Center);
            frame.render_widget(help_paragraph, help_area);
        }
    }
}
