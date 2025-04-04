use crate::App;
use crate::components::Component;
use crate::disk;
use crate::event::{AppEvent, Event};
use bubocorelib::server::client::ClientMessage;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use tui_textarea::TextArea;

/// État du composant de sauvegarde/chargement.
pub struct SaveLoadState {
    /// Liste des noms de projets disponibles sur le disque.
    pub projects: Vec<String>,
    /// Index du projet sélectionné dans la liste.
    pub selected_index: usize,
    /// Champ de texte pour entrer le nom du projet à sauvegarder.
    pub input_area: TextArea<'static>,
    /// Indique si l'utilisateur est en train d'entrer du texte pour sauvegarder.
    pub is_saving: bool,
    /// Message de statut (ex: "Projet sauvegardé", "Erreur de chargement").
    pub status_message: String,
    /// Indique si un rafraîchissement de la liste est en attente.
    pub is_refresh_pending: bool,
}

impl SaveLoadState {
    pub fn new() -> Self {
        let mut input_area = TextArea::default();
        input_area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Save Project As")
                .style(Style::default().fg(Color::Yellow)),
        );
        Self {
            projects: Vec::new(),
            selected_index: 0,
            input_area,
            is_saving: false,
            status_message: String::new(),
            is_refresh_pending: true,
        }
    }

    /// Charge la liste des projets depuis le disque.
    pub async fn load_projects_list(&mut self) {
        match disk::list_projects().await {
            Ok(projects) => {
                self.projects = projects;
                self.selected_index = self.selected_index.min(self.projects.len().saturating_sub(1));
                self.status_message = format!("{} projects found.", self.projects.len());
            }
            Err(e) => {
                self.projects.clear();
                self.selected_index = 0;
                self.status_message = format!("Error listing projects: {}", e);
            }
        }
    }
}

/// Composant UI pour la sauvegarde et le chargement de projets.
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
                KeyCode::Esc => {
                    state.is_saving = false;
                    state.input_area.delete_line_by_head();
                    state.status_message = "Save cancelled.".to_string();
                    state.input_area.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Save Project As")
                            .style(Style::default().fg(Color::Yellow)),
                    );
                    return Ok(true);
                }
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
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Save Project As")
                                .style(Style::default().fg(Color::Yellow)),
                        );
                    }
                    return Ok(true);
                }
                _ => {
                    // Forward key to text area
                    let handled = state.input_area.input(key_event);
                    return Ok(handled);
                }
            }
        }

        // --- Handle List Navigation/Actions Mode --- 
        match (key_code, key_modifiers) {
            (KeyCode::Up, _) => {
                if !state.projects.is_empty() {
                    state.selected_index = state.selected_index.saturating_sub(1);
                }
                Ok(true)
            }
            (KeyCode::Down, _) => {
                if !state.projects.is_empty() {
                    let len = state.projects.len();
                    state.selected_index = (state.selected_index + 1).min(len.saturating_sub(1));
                }
                Ok(true)
            }
            (KeyCode::Char('r'), _) => {
                state.status_message = "Requesting project list refresh...".to_string();
                state.is_refresh_pending = true; // Marquer pour rafraîchir au prochain before_draw
                Ok(true)
            }
            (KeyCode::Char('l'), _) => { 
                if let Some(project_name) = state.projects.get(state.selected_index) {
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
            (KeyCode::Char('s'), _) => {
                state.is_saving = true;
                state.input_area = TextArea::default();
                state.input_area.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Save Project As (Enter: confirm, Esc: cancel)")
                        .style(Style::default().fg(Color::Yellow)),
                );
                state.status_message = "Enter project name to save.".to_string();
                Ok(true)
            }
            (KeyCode::Char('d'), crossterm::event::KeyModifiers::CONTROL) => {
                if let Some(project_name) = state.projects.get(state.selected_index) {
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
            state.is_refresh_pending = false; // Marquer comme traité
            state.status_message = "Refreshing project list...".to_string();
            
            // Lancer la tâche async pour rafraîchir
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

        let help_text = if state.is_saving {
             "Enter: Confirm Save | Esc: Cancel"
        } else {
            "↑↓: Navigate | l: Load | s: Save | r: Refresh | Ctrl+d: Delete"
        };

        let main_block = if state.is_saving {
             state.input_area.block().cloned().unwrap_or_else(|| {
                 Block::default()
                     .borders(Borders::ALL)
                     .title("Save Project As (Enter: confirm, Esc: cancel)")
                     .style(Style::default().fg(Color::Yellow))
             })
        } else {
            Block::default()
                .borders(Borders::ALL)
                .title(" Save/Load Project ")
        };

        frame.render_widget(main_block.clone(), area);

        let inner_area = main_block.inner(area);

        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
            ].as_ref())
            .split(inner_area);

        let content_area = inner_chunks[0];
        let help_area = inner_chunks[1];

        if state.is_saving {
            frame.render_widget(&state.input_area, content_area);
        } else {
            let list_items: Vec<ListItem> = state
                .projects
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let style = if i == state.selected_index {
                        Style::default().fg(Color::Black).bg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(name.as_str()).style(style)
                })
                .collect();

            let list = List::new(list_items)
                .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::BOLD));

            frame.render_widget(list, content_area);
        }

        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);

        let help_spans = if state.is_saving {
             vec![
                 Span::styled("Enter", key_style), Span::styled(": Confirm Save | ", help_style),
                 Span::styled("Esc", key_style), Span::styled(": Cancel", help_style),
             ]
         } else {
             vec![
                 Span::styled("↑↓", key_style), Span::styled(": Navigate | ", help_style),
                 Span::styled("l", key_style), Span::styled(": Load | ", help_style),
                 Span::styled("s", key_style), Span::styled(": Save | ", help_style),
                 Span::styled("r", key_style), Span::styled(": Refresh | ", help_style),
                 Span::styled("Ctrl+d", key_style), Span::styled(": Delete", help_style),
             ]
         };

        let help_paragraph = Paragraph::new(Line::from(help_spans))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, help_area);
    }
}
