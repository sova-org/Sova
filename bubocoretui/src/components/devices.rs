use crate::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell, BorderType, Tabs},
};
// Import shared types from bubocorelib
use bubocorelib::shared_types::{DeviceInfo, DeviceKind};
// Import ClientMessage
use bubocorelib::server::client::ClientMessage;
use tui_textarea::TextArea;
use std::time::{Instant, Duration};
use std::convert::TryFrom;

pub struct DevicesState {
    pub selected_index: usize,
    /// Indique si l'utilisateur est en train de nommer un port MIDI virtuel
    pub is_naming_virtual: bool,
    /// Zone de texte pour entrer le nom du port MIDI virtuel
    pub virtual_port_input: TextArea<'static>,
    /// Message de statut pour la création du port virtuel
    pub status_message: String,
    /// Indique l'onglet actuellement sélectionné (0 = MIDI, 1 = OSC)
    pub tab_index: usize,
    /// Stocke les index de sélection par onglet
    pub midi_selected_index: usize,
    pub osc_selected_index: usize,
    /// Animation lors de la connexion
    pub animation_active: bool,
    pub animation_start: Option<Instant>,
    pub animation_device_id: Option<u32>,
    /// Historique des noms de ports virtuels
    pub recent_port_names: Vec<String>,
}

impl DevicesState {
    pub fn new() -> Self {
        let mut input_area = TextArea::default();
        input_area.set_block(
            Block::default()
                .borders(Borders::NONE)
        );
        
        Self {
            selected_index: 0,
            is_naming_virtual: false,
            virtual_port_input: input_area,
            status_message: String::new(),
            tab_index: 0, // Par défaut sur l'onglet MIDI
            midi_selected_index: 0,
            osc_selected_index: 0,
            animation_active: false,
            animation_start: None,
            animation_device_id: None,
            recent_port_names: Vec::new(),
        }
    }
    
    pub fn get_current_tab_selection(&self) -> usize {
        match self.tab_index {
            0 => self.midi_selected_index,
            1 => self.osc_selected_index,
            _ => 0,
        }
    }
    
    pub fn set_current_tab_selection(&mut self, index: usize) {
        match self.tab_index {
            0 => self.midi_selected_index = index,
            1 => self.osc_selected_index = index,
            _ => {},
        }
    }
    
    pub fn start_animation(&mut self, device_id: u32) {
        self.animation_active = true;
        self.animation_start = Some(Instant::now());
        self.animation_device_id = Some(device_id);
    }
    
    pub fn update_animation(&mut self) -> bool {
        if let Some(start_time) = self.animation_start {
            if start_time.elapsed() > Duration::from_millis(1500) {
                // Animation terminée après 1.5 secondes
                self.animation_active = false;
                self.animation_start = None;
                self.animation_device_id = None;
                return true;
            }
        }
        false
    }
    
    pub fn add_recent_port_name(&mut self, name: String) {
        // Ne pas ajouter de doublons
        if !self.recent_port_names.contains(&name) {
            self.recent_port_names.push(name);
            // Limiter la liste à 5 noms récents
            if self.recent_port_names.len() > 5 {
                self.recent_port_names.remove(0);
            }
        }
    }
}

pub struct DevicesComponent;

impl DevicesComponent {
    pub fn new() -> Self {
        Self {}
    }

    // Helper to get filtered device list and count before selection
    // Now returns Vec<DeviceInfo> directly, preserving IDs
    fn get_filtered_devices(app: &App) -> (Vec<DeviceInfo>, Vec<DeviceInfo>) {
        // Filter MIDI devices, excluding temporary and internal utility devices
        let midi_devices: Vec<DeviceInfo> = app.server.devices.iter()
            .filter(|d| {
                d.kind == DeviceKind::Midi 
                && !d.name.contains("BuboCore-Temp-Connector") 
                && !d.name.contains("BuboCore-Virtual-Creator")
            })
            .cloned()
            .collect();
            
        let osc_devices: Vec<DeviceInfo> = app.server.devices.iter()
            .filter(|d| d.kind == DeviceKind::Osc)
            .cloned()
            .collect();
            
        (midi_devices, osc_devices)
    }
    
    // Génère un caractère d'animation basé sur le temps écoulé
    fn get_animation_char(elapsed_ms: u128) -> &'static str {
        match (elapsed_ms / 150) % 4 {
            0 => "◐",
            1 => "◓",
            2 => "◑",
            3 => "◒",
            _ => "◐",
        }
    }
}

impl Component for DevicesComponent {

    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        // Si l'utilisateur est en train de nommer un port MIDI virtuel
        if app.interface.components.devices_state.is_naming_virtual {
            match key_event.code {
                // Annuler la création
                KeyCode::Esc => {
                    app.interface.components.devices_state.is_naming_virtual = false;
                    app.interface.components.devices_state.virtual_port_input = TextArea::default();
                    app.interface.components.devices_state.virtual_port_input.set_block(
                        Block::default().borders(Borders::NONE)
                    );
                    app.interface.components.devices_state.status_message = "Creation cancelled.".to_string();
                    app.set_status_message("Virtual port creation cancelled.".to_string());
                    return Ok(true);
                }
                // Confirmer la création
                KeyCode::Enter => {
                    let virtual_port_name = app.interface.components.devices_state.virtual_port_input.lines()[0].trim().to_string();
                    
                    if virtual_port_name.is_empty() {
                        app.interface.components.devices_state.status_message = "Port name cannot be empty.".to_string();
                        app.set_status_message("Port name cannot be empty.".to_string());
                    } else {
                        // Ajouter le nom aux récents
                        app.interface.components.devices_state.add_recent_port_name(virtual_port_name.clone());
                        
                        // Mettre à jour l'état
                        app.interface.components.devices_state.is_naming_virtual = false;
                        app.interface.components.devices_state.status_message = format!("Creating port '{}' in progress...", virtual_port_name);
                        
                        // Réinitialiser le champ de saisie
                        app.interface.components.devices_state.virtual_port_input = TextArea::default();
                        app.interface.components.devices_state.virtual_port_input.set_block(
                            Block::default().borders(Borders::NONE)
                        );
                        
                        // Envoyer la demande de création au serveur
                        app.send_client_message(ClientMessage::CreateVirtualMidiOutput(virtual_port_name.clone()));
                        app.set_status_message(format!("Creating MIDI virtual port: {}", virtual_port_name));
                    }
                    return Ok(true);
                }
                // Navigation dans l'historique des noms récents
                KeyCode::Up => {
                    let current_text = app.interface.components.devices_state.virtual_port_input.lines()[0].trim();
                    let recent_names = &app.interface.components.devices_state.recent_port_names;
                    
                    // Vérifier s'il y a des noms récents
                    if recent_names.is_empty() {
                        return Ok(false);
                    }
                    
                    // Trouver le nom précédent dans l'historique
                    if let Some(idx) = recent_names.iter().position(|n| n == current_text) {
                        if idx < recent_names.len() - 1 {
                            let next_name = &recent_names[idx + 1];
                            let mut new_input = TextArea::new(vec![next_name.clone()]);
                            new_input.set_block(Block::default().borders(Borders::NONE));
                            app.interface.components.devices_state.virtual_port_input = new_input;
                        }
                    } else if !recent_names.is_empty() {
                        // Si le texte actuel n'est pas dans l'historique, afficher le plus récent
                        let latest_name = &recent_names[0];
                        let mut new_input = TextArea::new(vec![latest_name.clone()]);
                        new_input.set_block(Block::default().borders(Borders::NONE));
                        app.interface.components.devices_state.virtual_port_input = new_input;
                    }
                    return Ok(true);
                }
                KeyCode::Down => {
                    let current_text = app.interface.components.devices_state.virtual_port_input.lines()[0].trim();
                    let recent_names = &app.interface.components.devices_state.recent_port_names;
                    
                    // Vérifier s'il y a des noms récents
                    if recent_names.is_empty() {
                        return Ok(false);
                    }
                    
                    // Trouver le nom suivant dans l'historique
                    if let Some(idx) = recent_names.iter().position(|n| n == current_text) {
                        if idx > 0 {
                            let prev_name = &recent_names[idx - 1];
                            let mut new_input = TextArea::new(vec![prev_name.clone()]);
                            new_input.set_block(Block::default().borders(Borders::NONE));
                            app.interface.components.devices_state.virtual_port_input = new_input;
                        }
                    }
                    return Ok(true);
                }
                // Gérer les autres touches (saisie de texte)
                _ => {
                    let handled = app.interface.components.devices_state.virtual_port_input.input(key_event);
                    return Ok(handled);
                }
            }
        }

        // Vérifier et mettre à jour l'animation si active
        if app.interface.components.devices_state.animation_active {
            app.interface.components.devices_state.update_animation();
        }

        // Si l'utilisateur n'est pas en mode de nommage, gérer la navigation normale
        let (midi_devices, osc_devices) = Self::get_filtered_devices(app);
        
        // Naviguer entre les onglets avec M et O
        match key_event.code {
            KeyCode::Char('m') => {
                // Passer à l'onglet MIDI (0)
                if app.interface.components.devices_state.tab_index != 0 {
                    app.interface.components.devices_state.tab_index = 0;
                    app.interface.components.devices_state.selected_index = app.interface.components.devices_state.get_current_tab_selection();
                    return Ok(true);
                }
            }
            KeyCode::Char('o') => {
                // Passer à l'onglet OSC (1)
                if app.interface.components.devices_state.tab_index != 1 {
                    app.interface.components.devices_state.tab_index = 1;
                    app.interface.components.devices_state.selected_index = app.interface.components.devices_state.get_current_tab_selection();
                    return Ok(true);
                }
            }
            _ => {}
        }
        
        // Obtenir les périphériques pour l'onglet actuel
        let current_devices = match app.interface.components.devices_state.tab_index {
            0 => &midi_devices,
            1 => &osc_devices,
            _ => &midi_devices,
        };
        
        let total_devices = current_devices.len();

        match key_event.code {
            KeyCode::Up => {
                if total_devices > 0 {
                    let new_index = app.interface.components.devices_state.selected_index.saturating_sub(1);
                    app.interface.components.devices_state.selected_index = new_index;
                    app.interface.components.devices_state.set_current_tab_selection(new_index);
                }
                Ok(true)
            }
            KeyCode::Down => {
                 if total_devices > 0 {
                    let new_index = (app.interface.components.devices_state.selected_index + 1).min(total_devices.saturating_sub(1));
                    app.interface.components.devices_state.selected_index = new_index;
                    app.interface.components.devices_state.set_current_tab_selection(new_index);
                 }
                Ok(true)
            }
            KeyCode::Enter => {
                let selected_index = app.interface.components.devices_state.selected_index;
                if let Some(selected_device) = current_devices.get(selected_index) {
                    // Use device ID for connect/disconnect messages
                    let device_id = selected_device.id;
                    let device_name = &selected_device.name;

                    if selected_device.kind == DeviceKind::Midi {
                        if selected_device.is_connected {
                            app.set_status_message(format!("Disconnecting {} ('{}')...", device_id, device_name));
                            app.send_client_message(ClientMessage::DisconnectMidiDevice(device_id));
                        } else {
                            // Démarrer l'animation de connexion si possible
                            if let Ok(device_id_u32) = u32::try_from(device_id) {
                                app.interface.components.devices_state.start_animation(device_id_u32);
                            }
                            
                            // Envoyer la demande de connexion au serveur
                            app.set_status_message(format!("Connecting to {} ('{}')...", device_id, device_name));
                            app.send_client_message(ClientMessage::ConnectMidiDevice(device_id));
                        }
                    } else {
                        app.set_status_message(format!("Connect/disconnect not implemented for {:?} devices.", selected_device.kind));
                    }
                } else {
                    app.set_status_message("No device selected.".to_string());
                }
                Ok(true)
            }
            // Handle Ctrl+N to create a new virtual device
            KeyCode::Char('n') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                // Activer le mode de nommage
                app.interface.components.devices_state.is_naming_virtual = true;
                app.interface.components.devices_state.virtual_port_input = TextArea::default();
                app.interface.components.devices_state.virtual_port_input.set_block(
                    Block::default().borders(Borders::NONE)
                );
                app.interface.components.devices_state.status_message = "Enter the MIDI virtual port name".to_string();
                app.set_status_message("Creating a new MIDI virtual port...".to_string());
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        // Vérifier et mettre à jour l'animation si active
        let devices_state = &app.interface.components.devices_state;
        let animation_char = if devices_state.animation_active {
            if let Some(start_time) = devices_state.animation_start {
                let elapsed = start_time.elapsed().as_millis();
                Some(Self::get_animation_char(elapsed))
            } else {
                None
            }
        } else {
            None
        };
        
        // Définir des hauteurs fixes pour les zones spéciales
        let input_prompt_height = if devices_state.is_naming_virtual { 3 } else { 0 };
        let status_height = if !devices_state.status_message.is_empty() { 1 } else { 0 };
        
        // Créer une mise en page pour les zones externes
        let outer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),                       // Zone principale (avec taille minimale)
                Constraint::Length(input_prompt_height),  // Zone de saisie (si visible)
                Constraint::Length(status_height),        // Message de statut (si présent)
            ])
            .split(area);
            
        let main_area = outer_chunks[0];
        let input_area = if input_prompt_height > 0 { Some(outer_chunks[1]) } else { None };
        let status_area = if status_height > 0 { 
            if input_prompt_height > 0 { Some(outer_chunks[2]) } else { Some(outer_chunks[1]) }
        } else { None };

        // --- Dessiner le bloc principal ---
        let outer_block = Block::default()
            .title(" Devices ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::White));

        let inner_area = outer_block.inner(main_area);
        frame.render_widget(outer_block, main_area);
        
        // Espace minimal requis
        if inner_area.width < 10 || inner_area.height < 7 { // Augmenter hauteur minimale pour l'aide
            return;
        }
        
        // Diviser la zone interne pour réserver de l'espace pour l'aide en bas
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),      // Zone de contenu
                Constraint::Length(2),   // Zone d'aide (2 lignes)
            ])
            .split(inner_area);
            
        let content_area = inner_chunks[0];
        let help_area = inner_chunks[1];

        // --- Onglets MIDI / OSC ---
        let tabs_height = 2;
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(tabs_height),          // Onglets
                Constraint::Min(0),                       // Contenu
            ])
            .split(content_area);
            
        let tabs_area = content_layout[0];
        let devices_area = content_layout[1];
        
        // Dessiner les onglets
        let tab_titles = vec!["MIDI", "OSC"];
        let tabs = Tabs::new(tab_titles.iter().map(|t| Line::from(*t)).collect::<Vec<Line>>())
            .select(devices_state.tab_index)
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .divider("|")
            .style(Style::default().fg(Color::White));
            
        frame.render_widget(tabs, tabs_area);
        
        // Récupérer les listes filtrées de périphériques
        let (midi_devices, osc_devices) = Self::get_filtered_devices(app);
        
        // Dessiner le contenu de l'onglet actif
        if devices_state.tab_index == 0 {
            // --- Onglet MIDI ---
            let headers = vec!["ID", "Statut", "Nom", "Type"];
            let col_widths = [
                Constraint::Length(5),    // ID
                Constraint::Length(8),    // Statut
                Constraint::Min(20),      // Nom
                Constraint::Length(10),   // Type
            ];
            
            let header_cells = headers.iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            let header = Row::new(header_cells)
                .style(Style::default().bg(Color::DarkGray))
                .height(1);
                
            let rows = midi_devices.iter().enumerate().map(|(i, device)| {
                let is_selected = i == devices_state.selected_index;
                // Convertir device.id en u32 pour comparaison
                let device_id_u32 = u32::try_from(device.id).unwrap_or(0);
                let is_animated = animation_char.is_some() && devices_state.animation_device_id == Some(device_id_u32);
                
                // Icônes d'état
                let status_text = if is_animated {
                    animation_char.unwrap_or("◯")
                } else if device.is_connected {
                    "▶ Connected"
                } else {
                    "◯ Available"
                };
                
                let status_color = if device.is_connected {
                    Color::Green
                } else {
                    Color::Yellow
                };
                
                let row_style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                
                let id_cell = Cell::from(format!("{}", device.id));
                let status_cell = Cell::from(status_text).style(Style::default().fg(status_color));
                let name_cell = Cell::from(device.name.as_str());
                let type_cell = Cell::from("MIDI");
                
                Row::new(vec![id_cell, status_cell, name_cell, type_cell])
                    .style(row_style)
                    .height(1)
            });
            
            let table = Table::new(rows, col_widths)
                .header(header)
                .block(Block::default().borders(Borders::NONE))
                .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));
                
            frame.render_widget(table, devices_area);
            
        } else {
            // --- Onglet OSC ---
            let headers = vec!["ID", "Statut", "Nom", "Adresse"];
            let col_widths = [
                Constraint::Length(5),    // ID
                Constraint::Length(8),    // Statut
                Constraint::Min(15),      // Nom
                Constraint::Min(15),      // Adresse
            ];
            
            let header_cells = headers.iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)));
            let header = Row::new(header_cells)
                .style(Style::default().bg(Color::DarkGray))
                .height(1);
                
            let rows = osc_devices.iter().enumerate().map(|(i, device)| {
                let is_selected = i == devices_state.selected_index;
                
                let status_text = if device.is_connected {
                    "▶ Active"
                } else {
                    "◯ Inactive"
                };
                
                let status_color = if device.is_connected {
                    Color::Green
                } else {
                    Color::Yellow
                };
                
                let row_style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                
                let id_cell = Cell::from(format!("{}", device.id));
                let status_cell = Cell::from(status_text).style(Style::default().fg(status_color));
                let name_cell = Cell::from(device.name.as_str());
                let addr_cell = Cell::from("127.0.0.1:8000"); // Simuler une adresse pour l'exemple
                
                Row::new(vec![id_cell, status_cell, name_cell, addr_cell])
                    .style(row_style)
                    .height(1)
            });
            
            let table = Table::new(rows, col_widths)
                .header(header)
                .block(Block::default().borders(Borders::NONE))
                .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));
                
            frame.render_widget(table, devices_area);
        }

        // Afficher la zone de saisie de texte si l'utilisateur est en train de nommer un port virtuel
        if let Some(area) = input_area {
            let mut virtual_input = devices_state.virtual_port_input.clone();
            virtual_input.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" MIDI Virtual Port Name (Enter: Confirm, Esc: Cancel) ")
                    .style(Style::default().fg(Color::Yellow))
            );
            virtual_input.set_style(Style::default().fg(Color::White));
            frame.render_widget(&virtual_input, area);
        }
        
        // Afficher le message de statut s'il est présent
        if let Some(area) = status_area {
            let status_style = Style::default().fg(Color::Yellow);
            let status_paragraph = Paragraph::new(devices_state.status_message.as_str())
                .style(status_style)
                .alignment(Alignment::Center);
            frame.render_widget(status_paragraph, area);
        }

        // --- Aide contextuelle (2 lignes) ---
        let _help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        
        if devices_state.is_naming_virtual {
            // Aide pour le mode de saisie
            let help_line1 = Line::from(vec![
                Span::styled("Enter", key_style), Span::raw(": Confirm | "),
                Span::styled("Esc", key_style), Span::raw(": Cancel")
            ]);
            let help_line2 = Line::from(vec![
                Span::styled("↑↓", key_style), Span::raw(": Browse through history")
            ]);
            
            let help_text = vec![help_line1, help_line2];
            let help = Paragraph::new(help_text)
                .alignment(Alignment::Center);
            frame.render_widget(help, help_area);
        } else {
            // Aide pour le mode normal
            let help_line1 = Line::from(vec![
                Span::styled("↑↓", key_style), Span::raw(": Navigate | "),
                Span::styled("M", key_style), Span::raw("/"), Span::styled("O", key_style), Span::raw(": MIDI/OSC | "),
                Span::styled("Enter", key_style), Span::raw(": Connect/Disconnect")
            ]);
            let help_line2 = Line::from(vec![
                Span::styled("Ctrl+N", key_style), Span::raw(": New virtual port")
            ]);
            
            let help_text = vec![help_line1, help_line2];
            let help = Paragraph::new(help_text)
                .alignment(Alignment::Center);
            frame.render_widget(help, help_area);
        }
    }
}
