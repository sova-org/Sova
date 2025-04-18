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
use bubocorelib::shared_types::{DeviceInfo, DeviceKind};
use bubocorelib::server::client::ClientMessage;
use tui_textarea::TextArea;
use std::time::{Instant, Duration};
use std::collections::HashMap; 

// Maximum assignable slot ID
const MAX_ASSIGNABLE_SLOT: usize = 16;

pub struct DevicesState {
    pub selected_index: usize,
    /// Indique si l'utilisateur est en train de nommer un port MIDI virtuel
    pub is_naming_virtual: bool,
    /// Zone de texte pour entrer le nom du port MIDI virtuel
    pub virtual_port_input: TextArea<'static>,
    /// Indique si l'utilisateur est en train d'assigner un slot
    pub is_assigning_slot: bool,
    /// Zone de texte pour entrer le numéro du slot à assigner
    pub slot_assignment_input: TextArea<'static>,
    /// Message de statut pour la création du port virtuel
    pub status_message: String,
    /// Indique l'onglet actuellement sélectionné (0 = MIDI, 1 = OSC)
    pub tab_index: usize,
    /// Stocke les index de sélection par onglet
    pub midi_selected_index: usize,
    pub osc_selected_index: usize,
    /// Stores the current Slot ID -> Device Name mapping (received from server).
    pub slot_assignments: HashMap<usize, String>,
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
        let mut slot_input = TextArea::<'static>::default();
        slot_input.set_block(Block::default().borders(Borders::NONE));
        
        Self {
            selected_index: 0,
            is_naming_virtual: false,
            virtual_port_input: input_area,
            is_assigning_slot: false,
            slot_assignment_input: slot_input,
            status_message: String::new(),
            tab_index: 0, // Par défaut sur l'onglet MIDI
            midi_selected_index: 0,
            osc_selected_index: 0,
            slot_assignments: HashMap::new(), // Initialize as empty
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
        // Get device list before borrowing state mutably
        let (midi_devices, _osc_devices) = Self::get_filtered_devices(app);

        // Borrow state mutably
        let state = &mut app.interface.components.devices_state;
        
        // Handle Slot Assignment Input Mode
        if state.is_assigning_slot {
            let mut status_msg_to_set = None;
            let mut client_msg_to_send = None;
            let mut exit_assign_mode = false;
            let mut handled_textarea = false;

            {
                // Scope for borrowing state
                match key_event.code {
                    KeyCode::Esc => {
                        status_msg_to_set = Some("Slot assignment cancelled.".to_string());
                        exit_assign_mode = true;
                    }
                    KeyCode::Enter => {
                        let input_str = state.slot_assignment_input.lines()[0].trim();
                        match input_str.parse::<usize>() {
                            Ok(digit) if digit <= MAX_ASSIGNABLE_SLOT => {
                                if let Some(selected_device) = midi_devices.get(state.selected_index) {
                                    let device_name = selected_device.name.clone();
                                    let current_slot = selected_device.id;
                                    let target_slot_assignee_name = state.slot_assignments.get(&digit).cloned();

                                    if digit == 0 { // Unassign
                                        if current_slot != 0 {
                                            status_msg_to_set = Some(format!("Unassigning '{}' from Slot {}...", device_name, current_slot));
                                            client_msg_to_send = Some(ClientMessage::UnassignDeviceFromSlot(current_slot));
                                        } else {
                                            status_msg_to_set = Some(format!("Device '{}' is not assigned to a slot.", device_name));
                                        }
                                    } else { // Assign (1-16)
                                        let target_slot_id = digit;
                                        if let Some(assignee) = target_slot_assignee_name {
                                            if assignee != device_name {
                                                status_msg_to_set = Some(format!("Slot {} is already assigned to '{}'. Unassign first.", target_slot_id, assignee));
                                            } else {
                                                status_msg_to_set = Some(format!("Device '{}' is already assigned to Slot {}.", device_name, target_slot_id));
                                            }
                                        } else if current_slot == target_slot_id {
                                            status_msg_to_set = Some(format!("Device '{}' is already assigned to Slot {}.", device_name, target_slot_id));
                                        } else {
                                            status_msg_to_set = Some(format!("Assigning '{}' to Slot {}...", device_name, target_slot_id));
                                            client_msg_to_send = Some(ClientMessage::AssignDeviceToSlot(target_slot_id, device_name));
                                        }
                                    }
                                } else {
                                    status_msg_to_set = Some("No device selected (error state?).".to_string());
                                }
                            }
                            _ => { // Parsing failed or number out of range
                                let error_message = format!("Invalid slot number: '{}'. Must be 0-{}.", input_str, MAX_ASSIGNABLE_SLOT);
                                state.status_message = error_message.clone(); // Update internal status immediately
                                status_msg_to_set = Some(error_message);
                            }
                        }
                        exit_assign_mode = true;
                    }
                    _ => { // Pass other inputs (digits, backspace) to the textarea
                         handled_textarea = state.slot_assignment_input.input(key_event);
                    }
                }
            } // state borrow ends here
            
            // --- Apply Actions After State Borrow (if any) ---
            if let Some(msg) = status_msg_to_set {
                app.set_status_message(msg);
            }
            if let Some(msg) = client_msg_to_send {
                app.send_client_message(msg);
            }
            if exit_assign_mode {
                 // Re-borrow state briefly to exit the mode
                 let state = &mut app.interface.components.devices_state;
                 state.is_assigning_slot = false;
                 state.slot_assignment_input = TextArea::default(); 
                 state.slot_assignment_input.set_block(Block::default().borders(Borders::NONE));
            }
            // Return true if we handled Esc/Enter or the textarea input
            return Ok(exit_assign_mode || handled_textarea);
        }

        // --- Handle Naming Virtual Port --- (Needs to be checked after slot assignment)
        if state.is_naming_virtual {
            match key_event.code {
                KeyCode::Esc => {
                    state.is_naming_virtual = false;
                    state.virtual_port_input = TextArea::default();
                    state.virtual_port_input.set_block(
                        Block::default().borders(Borders::NONE)
                    );
                    state.status_message = "Creation cancelled.".to_string();
                    app.set_status_message("Virtual port creation cancelled.".to_string());
                    return Ok(true);
                }
                KeyCode::Enter => {
                    let virtual_port_name = state.virtual_port_input.lines()[0].trim().to_string();
                    
                    if virtual_port_name.is_empty() {
                        state.status_message = "Port name cannot be empty.".to_string();
                        app.set_status_message("Port name cannot be empty.".to_string());
                    } else {
                        // Ajouter le nom aux récents
                        state.add_recent_port_name(virtual_port_name.clone());
                        
                        // Mettre à jour l'état
                        state.is_naming_virtual = false;
                        state.status_message = format!("Creating port '{}' in progress...", virtual_port_name);
                        
                        // Réinitialiser le champ de saisie
                        state.virtual_port_input = TextArea::default();
                        state.virtual_port_input.set_block(
                            Block::default().borders(Borders::NONE)
                        );
                        
                        // Envoyer la demande de création au serveur
                        app.send_client_message(ClientMessage::CreateVirtualMidiOutput(virtual_port_name.clone()));
                        app.set_status_message(format!("Creating MIDI virtual port: {}", virtual_port_name));
                    }
                    return Ok(true);
                }
                KeyCode::Up => {
                    let current_text = state.virtual_port_input.lines()[0].trim();
                    let recent_names = &state.recent_port_names;
                    
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
                            state.virtual_port_input = new_input;
                        }
                    } else if !recent_names.is_empty() {
                        // Si le texte actuel n'est pas dans l'historique, afficher le plus récent
                        let latest_name = &recent_names[0];
                        let mut new_input = TextArea::new(vec![latest_name.clone()]);
                        new_input.set_block(Block::default().borders(Borders::NONE));
                        state.virtual_port_input = new_input;
                    }
                    return Ok(true);
                }
                KeyCode::Down => {
                    let current_text = state.virtual_port_input.lines()[0].trim();
                    let recent_names = &state.recent_port_names;
                    
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
                            state.virtual_port_input = new_input;
                        }
                    }
                    return Ok(true);
                }
                _ => {
                    let handled = state.virtual_port_input.input(key_event);
                    return Ok(handled);
                }
            }
        }

        // --- Update Animation --- (No changes needed)
        if state.animation_active {
            state.update_animation();
        }

        // --- Handle Tab Switching --- (No changes needed)
        match key_event.code {
            KeyCode::Char('m') => {
                if state.tab_index != 0 {
                    state.tab_index = 0;
                    state.selected_index = state.get_current_tab_selection();
                    return Ok(true); // Tab changed, return early
                }
            }
            KeyCode::Char('o') => {
                if state.tab_index != 1 {
                    state.tab_index = 1;
                    state.selected_index = state.get_current_tab_selection();
                    return Ok(true); // Tab changed, return early
                }
            }
            _ => {}
        }
        
        // --- Select and Sort Devices based on current tab ---
        let displayed_devices = match state.tab_index {
             0 => midi_devices, // Use pre-fetched list
             // 1 => _osc_devices, // Handle OSC later
             _ => Vec::new(),
        };
        let mut sorted_displayed_devices = displayed_devices;
        // Sort displayed devices same way as in draw(): Assigned first, then Unassigned
        sorted_displayed_devices.sort_by(|a, b| {
             match (a.id, b.id) {
                 (0, 0) => a.name.cmp(&b.name), // Both unassigned: sort by name
                 (0, _) => std::cmp::Ordering::Greater, // Unassigned goes after assigned
                 (_, 0) => std::cmp::Ordering::Less, // Assigned goes before unassigned
                 (id_a, id_b) => id_a.cmp(&id_b), // Both assigned: sort by slot ID
             }
        });
        let total_devices_displayed = sorted_displayed_devices.len();

        // --- Normal Key Handling ---
         // Borrow selected_index mutably *here*, after potential early returns
         let current_selected_index = &mut state.selected_index;
 
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Up, _) => {
                if total_devices_displayed > 0 {
                    *current_selected_index = current_selected_index.saturating_sub(1);
                    // Update tab-specific index
                    if state.tab_index == 0 { state.midi_selected_index = *current_selected_index; }
                    // else if state.tab_index == 1 { state.osc_selected_index = *current_selected_index; }
                }
                Ok(true)
            }
            (KeyCode::Down, _) => {
                 if total_devices_displayed > 0 {
                    *current_selected_index = (*current_selected_index + 1).min(total_devices_displayed.saturating_sub(1));
                    // Update tab-specific index
                    if state.tab_index == 0 { state.midi_selected_index = *current_selected_index; }
                    // else if state.tab_index == 1 { state.osc_selected_index = *current_selected_index; }
                 }
                Ok(true)
            }
            (KeyCode::Enter, _) => {
                // Connect/Disconnect logic (doesn't conflict with state borrow anymore)
                let mut status_message = "No device selected.".to_string(); 
                // Get device name using the immutable `sorted_displayed_devices` and the index from `state`
                if let Some(selected_device) = sorted_displayed_devices.get(*current_selected_index) { 
                        let device_name = selected_device.name.clone();
                    if selected_device.kind == DeviceKind::Midi {
                        if selected_device.is_connected {
                                 status_message = format!("Disconnecting '{}'...", device_name);
                                 app.send_client_message(ClientMessage::DisconnectMidiDeviceByName(device_name)); 
                        } else {
                                 status_message = format!("Connecting to '{}'...", device_name);
                                 app.send_client_message(ClientMessage::ConnectMidiDeviceByName(device_name)); 
                        }
                    } else {
                             status_message = format!("Connect/disconnect not implemented for {:?} devices.", selected_device.kind);
                    }
                }
                app.set_status_message(status_message);
                Ok(true)
            }
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                // Create Virtual Port logic (only modifies state, doesn't read app)
                state.is_naming_virtual = true;
                state.virtual_port_input = TextArea::default();
                state.virtual_port_input.set_block(
                    Block::default().borders(Borders::NONE)
                );
                state.status_message = "Enter the MIDI virtual port name".to_string();
                app.set_status_message("Creating a new MIDI virtual port...".to_string());
                Ok(true)
            }
            // --- NEW: Enter Slot Assignment Mode --- 
            (KeyCode::Char('s'), _) => {
                if !state.is_naming_virtual { // Don't allow if naming virtual port
                    let status_msg_to_set;
                    let mut can_assign = false;
                    if sorted_displayed_devices.get(state.selected_index).is_some() {
                        can_assign = true;
                        state.is_assigning_slot = true;
                        state.slot_assignment_input = TextArea::default(); // Clear previous input
                        state.slot_assignment_input.set_block(Block::default().borders(Borders::NONE));
                        let status_msg = format!("Assign Slot (0-{}):", MAX_ASSIGNABLE_SLOT);
                        state.status_message = status_msg.clone(); // Update internal state
                        status_msg_to_set = Some(status_msg);
                    } else {
                        status_msg_to_set = Some("No device selected to assign slot.".to_string());
                    }
                    
                    // Set status message after potentially modifying state
                    if let Some(msg) = status_msg_to_set {
                         app.set_status_message(msg);
                    }
                    return Ok(can_assign); // Return true only if we actually entered assign mode
                 } else {
                    return Ok(false); // Let virtual naming handle 's' if active
                 }
            }
            _ => Ok(false), // Default: key not handled
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let state = &app.interface.components.devices_state;
        
        // --- Animation Update (No changes needed) ---
        let animation_char = if state.animation_active {
            if let Some(start_time) = state.animation_start {
                let elapsed = start_time.elapsed().as_millis();
                Some(Self::get_animation_char(elapsed))
            } else {
                None
            }
        } else {
            None
        };
        
        // --- Layout Definitions --- 
        let mut input_prompt_height = 0;
        if state.is_naming_virtual || state.is_assigning_slot {
            input_prompt_height = 3;
        }
        let status_height = if !state.status_message.is_empty() { 1 } else { 0 };
        
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
            .select(state.tab_index)
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .divider("|")
            .style(Style::default().fg(Color::White));
            
        frame.render_widget(tabs, tabs_area);
        
        // Récupérer les listes filtrées de périphériques ICI, dans draw
        let (midi_devices, osc_devices) = Self::get_filtered_devices(app);
        
        // Dessiner le contenu de l'onglet actif
        if state.tab_index == 0 {
            // --- Onglet MIDI ---
            let headers = vec!["Slot", "Statut", "Nom", "Type"];
            let col_widths = [
                Constraint::Length(6),    // Slot width
                Constraint::Length(8),    // Statut
                Constraint::Min(20),      // Nom
                Constraint::Length(10),   // Type
            ];
            
            let header_cells = headers.iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            let header = Row::new(header_cells)
                .style(Style::default().bg(Color::DarkGray))
                .height(1);
                
            let rows = midi_devices.iter().enumerate().map(|(visual_index, device)| {
                let is_selected = visual_index == state.selected_index;
                let slot_id = device.id;
                let device_id_u32 = 0; // Animation not linked to slot ID anymore
                let is_animated = animation_char.is_some() && state.animation_device_id == Some(device_id_u32); // TODO: Fix animation tracking if needed
                
                let status_text = if is_animated {
                    animation_char.unwrap_or("◯")
                } else if device.is_connected {
                    "▶ Connected"
                } else {
                    "◯ Available"
                };
                let status_color = if device.is_connected { Color::Green } else { Color::Yellow };
                
                let row_style = if is_selected { Style::default().bg(Color::Blue).fg(Color::White) } else { Style::default() };
                
                let slot_display = if slot_id == 0 { "--".to_string() } else { format!("{}", slot_id) };
                let slot_cell = Cell::from(slot_display);
                let status_cell = Cell::from(status_text).style(Style::default().fg(status_color));
                let name_cell = Cell::from(device.name.as_str());
                let type_cell = Cell::from("MIDI");
                
                Row::new(vec![slot_cell, status_cell, name_cell, type_cell])
                    .style(row_style)
                    .height(1)
            });
            
            let table = Table::new(rows, col_widths)
                .header(header)
                .block(Block::default().borders(Borders::NONE));
                
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
                let is_selected = i == state.selected_index;
                
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

        // Afficher la zone de saisie de texte si l'utilisateur est en train de nommer un port virtuel OU d'assigner un slot
        if let Some(area) = input_area {
            if state.is_naming_virtual {
                let mut virtual_input = state.virtual_port_input.clone();
            virtual_input.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" MIDI Virtual Port Name (Enter: Confirm, Esc: Cancel) ")
                    .style(Style::default().fg(Color::Yellow))
            );
            virtual_input.set_style(Style::default().fg(Color::White));
            frame.render_widget(&virtual_input, area);
            } else if state.is_assigning_slot {
                 let mut slot_input_area = state.slot_assignment_input.clone();
                 slot_input_area.set_block(
                     Block::default()
                         .borders(Borders::ALL)
                         .border_type(BorderType::Rounded)
                         .title(format!(" Assign Slot (0-{}, Enter: Confirm, Esc: Cancel) ", MAX_ASSIGNABLE_SLOT))
                         .style(Style::default().fg(Color::Yellow))
                 );
                 slot_input_area.set_style(Style::default().fg(Color::White));
                 frame.render_widget(&slot_input_area, area);
            }
        }
        
        // Afficher le message de statut s'il est présent
        if let Some(area) = status_area {
            let status_style = Style::default().fg(Color::Yellow);
            let status_paragraph = Paragraph::new(state.status_message.as_str())
                .style(status_style)
                .alignment(Alignment::Center);
            frame.render_widget(status_paragraph, area);
        }

        // --- Render Help Text --- (Update for slot assignment)
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        let help_spans1;
        let help_spans2;

        if state.is_naming_virtual {
             // Help for naming mode (No changes needed)
             help_spans1 = vec![
                 Span::styled("Enter", key_style), Span::raw(": Confirm | "),
                 Span::styled("Esc", key_style), Span::raw(": Cancel")
             ];
             help_spans2 = vec![
                 Span::styled("↑↓", key_style), Span::raw(": Browse through history")
             ];
        } else if state.is_assigning_slot {
            // Help for slot assignment mode
            help_spans1 = vec![
                Span::styled("Enter", key_style), Span::raw(": Confirm | "),
                Span::styled("Esc", key_style), Span::raw(": Cancel | "),
                Span::styled("0-9", key_style), Span::raw(": Enter Slot Number"),
            ];
            help_spans2 = vec![Span::raw("")]; // Second line empty for this mode
        } else {
            // Help for normal mode (Update for 's')
            help_spans1 = vec![
                Span::styled("↑↓", key_style), Span::raw(": Navigate | "),
                Span::styled("M", key_style), Span::raw("/ "), Span::styled("O", key_style), Span::raw(": MIDI/OSC | "),
                Span::styled("Enter", key_style), Span::raw(": Connect/Disconnect"),
            ];
            help_spans2 = vec![
                Span::styled("s", key_style), Span::raw(": Assign Slot | "),
                Span::styled("Ctrl+N", key_style), Span::raw(": New virtual port"),
            ];
        }
        let help_text = vec![Line::from(help_spans1), Line::from(help_spans2)];
        let help = Paragraph::new(help_text).alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
