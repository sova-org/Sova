use crate::App;
use crate::components::Component;
use crate::disk;
use crate::event::{AppEvent, Event};
use bubocorelib::server::client::ClientMessage;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
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

        // --- Handle Input Mode --- 
        if state.is_saving {
            match key_code {
                KeyCode::Esc => {
                    state.is_saving = false;
                    state.input_area.delete_line_by_head(); // Clear input
                    state.status_message = "Save cancelled.".to_string();
                    return Ok(true);
                }
                KeyCode::Enter => {
                    let project_name = state.input_area.lines()[0].trim().to_string();
                    
                    if project_name.is_empty() {
                        state.status_message = "Project name cannot be empty.".to_string();
                        // Ne pas quitter le mode sauvegarde si le nom est vide
                    } else {
                        // Préparer le statut et sortir du mode AVANT d'envoyer le message
                        state.status_message = format!("Requesting snapshot to save as '{}'...", project_name);
                        state.is_saving = false; 

                        // Cloner le nom avant l'appel réseau
                        let project_name_clone = project_name.clone();
                        
                        // Envoyer la requête snapshot
                        app.send_client_message(ClientMessage::GetSnapshot);

                        // Après l'appel réseau, on peut de nouveau modifier state
                        // Stocker temporairement le nom dans l'input area vidée
                        // (C'est une astuce, idéalement il faudrait un champ dédié)
                        let state_after_send = &mut app.interface.components.save_load_state;
                        state_after_send.input_area = TextArea::default();
                        state_after_send.input_area.insert_str(project_name_clone);
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
        match key_code {
            KeyCode::Up => {
                if !state.projects.is_empty() {
                    state.selected_index = state.selected_index.saturating_sub(1);
                }
                Ok(true)
            }
            KeyCode::Down => {
                if !state.projects.is_empty() {
                    let len = state.projects.len();
                    state.selected_index = (state.selected_index + 1).min(len.saturating_sub(1));
                }
                Ok(true)
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                state.status_message = "Requesting project list refresh...".to_string();
                state.is_refresh_pending = true; // Marquer pour rafraîchir au prochain before_draw
                Ok(true)
            }
            KeyCode::Char('l') => { 
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
            KeyCode::Char('s') => {
                state.is_saving = true;
                state.input_area = TextArea::default();
                state.input_area.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Save Project As (Enter to confirm, Esc to cancel)")
                        .style(Style::default().fg(Color::Yellow)),
                );
                state.status_message = "Enter project name to save.".to_string();
                Ok(true)
            }
            _ => Ok(false), // Not handled by this component
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

        // Layout: [ Main Area ]
        //         [ Help      ]
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Main area (list or input)
                Constraint::Length(1), // Help line
            ])
            .split(area);

        let main_area = chunks[0]; // Main area is now index 0
        let help_area = chunks[1]; // Help area is now index 1

        if state.is_saving {
            frame.render_widget(&state.input_area, main_area);
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

            let list_block = Block::default()
                .borders(Borders::ALL)
                .title("Load Project");

            let list = List::new(list_items)
                .block(list_block)
                .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::BOLD));

            frame.render_widget(list, main_area);
        }

        let help_text = if state.is_saving {
             "Enter: Confirm | Esc: Cancel"
        } else {
            "↑↓: Navigate | l: Load | s: Save | r: Refresh"
        };
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
