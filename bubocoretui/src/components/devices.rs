///! Manages the UI component for displaying and interacting with MIDI and OSC devices.

use crate::app::App;
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
use std::time::Instant;
use std::collections::HashMap; 

/// Maximum user-assignable slot ID (1-based). Slot 0 is used for logging.
const MAX_ASSIGNABLE_SLOT: usize = 16;

/// Stores the state for the Devices UI component.
pub struct DevicesState {
    /// The visually selected index in the current device list (MIDI or OSC).
    pub selected_index: usize,
    /// Flag indicating if the user is currently entering a name for a new virtual MIDI port.
    pub is_naming_virtual: bool,
    /// Text input area for the virtual MIDI port name.
    pub virtual_port_input: TextArea<'static>,
    /// Flag indicating if the user is currently assigning a slot number to the selected device.
    pub is_assigning_slot: bool,
    /// Text input area for the slot number.
    pub slot_assignment_input: TextArea<'static>,
    /// Message displayed at the bottom, indicating status or prompts.
    pub status_message: String,
    /// Index of the currently active tab (0 for MIDI, 1 for OSC).
    pub tab_index: usize,
    /// Stores the `selected_index` specifically for the MIDI tab when switching.
    pub midi_selected_index: usize,
    pub osc_selected_index: usize,
    /// Caches the current Slot ID -> Device Name mapping received from the server.
    pub slot_assignments: HashMap<usize, String>,
    /// Flag indicating if the connection/disconnection animation is active.
    pub animation_active: bool,
    /// Timestamp when the animation started.
    pub animation_start: Option<Instant>,
    /// ID of the device undergoing animation (currently unused).
    pub animation_device_id: Option<u32>,
    /// Stores the last few names used for creating virtual ports.
    pub recent_port_names: Vec<String>,

    /// Flag indicating if the user is in the multi-step OSC device creation process.
    pub is_creating_osc: bool,
    /// Text input area for the new OSC device name.
    pub osc_name_input: TextArea<'static>,
    /// Text input area for the new OSC device IP address.
    pub osc_ip_input: TextArea<'static>,
    /// Text input area for the new OSC device port.
    pub osc_port_input: TextArea<'static>,
    /// Current step within the OSC creation process (0: Name, 1: IP, 2: Port).
    pub osc_creation_step: usize,
}

impl DevicesState {
    /// Creates a new `DevicesState` with default values and initialized text areas.
    pub fn new() -> Self {
        let mut input_area = TextArea::default();
        input_area.set_block(
            Block::default()
                .borders(Borders::NONE)
        );
        let mut slot_input = TextArea::<'static>::default();
        slot_input.set_block(Block::default().borders(Borders::NONE));
        
        // Initialize OSC input areas
        let mut osc_name_input = TextArea::<'static>::default();
        osc_name_input.set_block(Block::default().borders(Borders::NONE));
        let mut osc_ip_input = TextArea::<'static>::default();
        osc_ip_input.set_block(Block::default().borders(Borders::NONE));
        let mut osc_port_input = TextArea::<'static>::default();
        osc_port_input.set_block(Block::default().borders(Borders::NONE));

        Self {
            selected_index: 0,
            is_naming_virtual: false,
            virtual_port_input: input_area,
            is_assigning_slot: false,
            slot_assignment_input: slot_input,
            status_message: String::new(),
            tab_index: 0,
            midi_selected_index: 0,
            osc_selected_index: 0,
            slot_assignments: HashMap::new(),
            animation_active: false,
            animation_start: None,
            animation_device_id: None,
            recent_port_names: Vec::new(),
            // Initialize OSC state
            is_creating_osc: false,
            osc_name_input,
            osc_ip_input,
            osc_port_input,
            osc_creation_step: 0,
        }
    }
    
    /// Returns the stored selected index based on the current `tab_index`.
    pub fn get_current_tab_selection(&self) -> usize {
        match self.tab_index {
            0 => self.midi_selected_index,
            1 => self.osc_selected_index,
            _ => 0,
        }
    }
     
    /// Adds a virtual port name to the recent names history, maintaining a fixed size.
    /// Avoids adding duplicate names.
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

/// The UI component responsible for drawing the devices list and handling interactions.
pub struct DevicesComponent;

impl DevicesComponent {
    /// Creates a new `DevicesComponent`.
    pub fn new() -> Self {
        Self {}
    }

    /// Filters the main device list from the App state into separate MIDI and OSC lists.
    /// Excludes internal/temporary MIDI devices used by BuboCore itself.
    /// Returns tuple: `(midi_devices, osc_devices)`.
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
    
    /// Selects an animation character based on elapsed time for visual feedback.
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

    /// Handles key events for the Devices component, managing state changes and UI interactions.
    /// Returns `Ok(true)` if the key event was handled, `Ok(false)` otherwise.
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        let (midi_devices, osc_devices) = Self::get_filtered_devices(app);
        let mut status_message_to_set: Option<String> = None;
        let mut client_message_to_send: Option<ClientMessage> = None;
        let mut handled = false; 

        { // Scope for mutable borrow of state
            let state = &mut app.interface.components.devices_state;

            // --- Handle OSC Creation Input Mode --- 
            if state.is_creating_osc {
                let osc_handled_in_mode;
                match state.osc_creation_step {
                    0 => { // Name Input
                        match key_event.code {
                            KeyCode::Esc => {
                                state.is_creating_osc = false;
                                status_message_to_set = Some("OSC device creation cancelled.".to_string());
                                osc_handled_in_mode = true;
                            }
                            KeyCode::Enter => {
                                if !state.osc_name_input.lines()[0].trim().is_empty() {
                                    state.osc_creation_step = 1; 
                                    status_message_to_set = Some("Enter OSC IP Address...".to_string());
                                } else {
                                    status_message_to_set = Some("OSC name cannot be empty.".to_string());
                                }
                                osc_handled_in_mode = true;
                            }
                            _ => {
                                osc_handled_in_mode = state.osc_name_input.input(key_event);
                            }
                        }
                    }
                    1 => { // IP Input
                        match key_event.code {
                             KeyCode::Esc => {
                                state.osc_creation_step = 0; 
                                status_message_to_set = Some("Enter OSC Device Name".to_string());
                                 osc_handled_in_mode = true;
                            }
                            KeyCode::Enter => {
                                if !state.osc_ip_input.lines()[0].trim().is_empty() {
                                    state.osc_creation_step = 2; 
                                    status_message_to_set = Some("Enter OSC Port...".to_string());
                                 } else {
                                     status_message_to_set = Some("OSC IP cannot be empty.".to_string());
                                 }
                                osc_handled_in_mode = true;
                            }
                            _ => {
                                 osc_handled_in_mode = state.osc_ip_input.input(key_event);
                            }
                        }
                    }
                    2 => { // Port Input
                         match key_event.code {
                            KeyCode::Esc => {
                                state.osc_creation_step = 1; 
                                 status_message_to_set = Some("Enter OSC IP Address".to_string());
                                 osc_handled_in_mode = true;
                            }
                            KeyCode::Enter => {
                                let port_str = state.osc_port_input.lines()[0].trim();
                                match port_str.parse::<u16>() {
                                    Ok(port) if port > 0 => {
                                        let name = state.osc_name_input.lines()[0].trim().to_string();
                                        let ip = state.osc_ip_input.lines()[0].trim().to_string();
                                        status_message_to_set = Some(format!("Creating OSC '{}' @ {}:{}...", name, ip, port));
                                        client_message_to_send = Some(ClientMessage::CreateOscDevice(name, ip, port));
                                        // Reset and exit mode
                                        state.is_creating_osc = false;
                                        state.osc_creation_step = 0;
                                        state.osc_name_input = TextArea::default();
                                        state.osc_ip_input = TextArea::default();
                                        state.osc_port_input = TextArea::default();
                                    }
                                    _ => {
                                        status_message_to_set = Some("Invalid port (1-65535).".to_string());
                                    }
                                }
                                osc_handled_in_mode = true;
                            }
                            _ => {
                                 osc_handled_in_mode = state.osc_port_input.input(key_event);
                            }
                        }
                    }
                     _ => { state.is_creating_osc = false; osc_handled_in_mode = true; } // Should not happen
                }
                 // If handled within OSC mode, set the main handled flag
                 if osc_handled_in_mode { handled = true; }
             }
            // --- Handle Slot Assignment Input Mode --- 
            else if state.is_assigning_slot {
                 let slot_handled_in_mode;
                 let mut exit_assign_mode = false;
                 let mut temp_client_msg = None; // Temporary holder
                 match key_event.code {
                    KeyCode::Esc => {
                        status_message_to_set = Some("Slot assignment cancelled.".to_string());
                        exit_assign_mode = true;
                        slot_handled_in_mode = true;
                    }
                    KeyCode::Enter => {
                        let input_str = state.slot_assignment_input.lines()[0].trim();
                        match input_str.parse::<usize>() {
                            Ok(digit) if digit <= MAX_ASSIGNABLE_SLOT => {
                                // Determine the correct device list based on the current tab
                                let current_devices = match state.tab_index {
                                    0 => &midi_devices,
                                    1 => &osc_devices,
                                    _ => &Vec::new(), // Should not happen
                                };
                                
                                // Get the selected device from the CORRECT list
                                if let Some(selected_device) = current_devices.get(state.selected_index) {
                                    let device_name = selected_device.name.clone();
                                    let current_slot = selected_device.id; // ID from DeviceInfo
                                    let target_slot_assignee_name = state.slot_assignments.get(&digit).cloned();

                                    if digit == 0 { // Unassign
                                        if current_slot != 0 { // Only unassign if currently assigned
                                            status_message_to_set = Some(format!("Unassigning '{}' from Slot {}...", device_name, current_slot));
                                            // Send message to unassign the specific slot ID
                                            temp_client_msg = Some(ClientMessage::UnassignDeviceFromSlot(current_slot)); 
                                        } else {
                                            status_message_to_set = Some(format!("Device '{}' is not assigned to a slot.", device_name));
                                        }
                                    } else { // Assign (1-16)
                                        let target_slot_id = digit;
                                        if let Some(assignee) = target_slot_assignee_name {
                                            if assignee != device_name {
                                                status_message_to_set = Some(format!("Slot {} is already assigned to '{}'. Unassign first.", target_slot_id, assignee));
                                            } else {
                                                status_message_to_set = Some(format!("Device '{}' is already assigned to Slot {}.", device_name, target_slot_id));
                                            }
                                        } else if current_slot == target_slot_id {
                                            status_message_to_set = Some(format!("Device '{}' is already assigned to Slot {}.", device_name, target_slot_id));
                                        } else {
                                            status_message_to_set = Some(format!("Assigning '{}' to Slot {}...", device_name, target_slot_id));
                                            // Send message to assign the selected device name to the target slot
                                            temp_client_msg = Some(ClientMessage::AssignDeviceToSlot(target_slot_id, device_name)); 
                                        }
                                    }
                                } else {
                                    status_message_to_set = Some("No device selected (internal error?).".to_string());
                                }
                            }
                            _ => { // Parsing failed or number out of range
                                status_message_to_set = Some(format!("Invalid slot: '{}'. Must be 0-{}.", input_str, MAX_ASSIGNABLE_SLOT));
                            }
                        }
                        exit_assign_mode = true;
                        slot_handled_in_mode = true;
                    }
                     _ => { 
                         slot_handled_in_mode = state.slot_assignment_input.input(key_event);
                    }
                }
                if exit_assign_mode {
                     state.is_assigning_slot = false;
                     state.slot_assignment_input = TextArea::default(); 
                     state.slot_assignment_input.set_block(Block::default().borders(Borders::NONE));
                }
                 // If handled, set main flag and store potential client message
                 if slot_handled_in_mode { 
                      handled = true; 
                      client_message_to_send = temp_client_msg; 
                 }
             }
            // --- Handle Naming Virtual Port --- 
            else if state.is_naming_virtual {
                  let virtual_handled_in_mode;
                  let mut temp_client_msg = None; // Temporary holder
                  match key_event.code {
                    KeyCode::Esc => {
                        state.is_naming_virtual = false;
                        state.virtual_port_input = TextArea::default();
                        state.virtual_port_input.set_block(
                            Block::default().borders(Borders::NONE)
                        );
                        state.status_message = "Creation cancelled.".to_string();
                        status_message_to_set = Some("Virtual port creation cancelled.".to_string());
                        virtual_handled_in_mode = true;
                    }
                    KeyCode::Enter => {
                        let name = state.virtual_port_input.lines()[0].trim().to_string();
                        if name.is_empty() {
                            status_message_to_set = Some("Port name cannot be empty.".to_string());
                        } else {
                             state.add_recent_port_name(name.clone());
                             state.is_naming_virtual = false;
                             state.status_message = format!("Creating port '{}'...", name); // Internal ok
                             state.virtual_port_input = TextArea::default();
                             state.virtual_port_input.set_block(
                                 Block::default().borders(Borders::NONE)
                             );
                             temp_client_msg = Some(ClientMessage::CreateVirtualMidiOutput(name.clone()));
                             status_message_to_set = Some(format!("Creating MIDI virtual port: {}", name));
                        }
                        virtual_handled_in_mode = true;
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
                        
                        // Check if there are recent names
                        if recent_names.is_empty() {
                            return Ok(false);
                        }
                        
                        // Find the next name in the history
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
                 // If handled, set main flag and store potential client message
                 if virtual_handled_in_mode { 
                      handled = true; 
                      client_message_to_send = temp_client_msg; 
                  }
             }

            // --- Normal Key Handling (only if not handled by input modes) --- 
            if !handled {
                // Select devices based on tab
                let (current_devices, total_devices) = match state.tab_index {
                    0 => (&midi_devices, midi_devices.len()),
                    1 => (&osc_devices, osc_devices.len()),
                    _ => (&Vec::new(), 0),
                };

                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Up, _) => {
                        if total_devices > 0 {
                           let current_idx = state.get_current_tab_selection();
                           let next_idx = current_idx.saturating_sub(1);
                           state.selected_index = next_idx;
                           if state.tab_index == 0 { state.midi_selected_index = next_idx; }
                           else { state.osc_selected_index = next_idx; }
                           handled = true;
                        }
                    }
                    (KeyCode::Down, _) => {
                        if total_devices > 0 {
                            let current_idx = state.get_current_tab_selection();
                            let next_idx = (current_idx + 1).min(total_devices.saturating_sub(1));
                           state.selected_index = next_idx;
                           if state.tab_index == 0 { state.midi_selected_index = next_idx; }
                           else { state.osc_selected_index = next_idx; }
                            handled = true;
                         }
                    }
                    (KeyCode::Enter, _) => {
                         let current_idx = state.get_current_tab_selection();
                         if let Some(selected_device) = current_devices.get(current_idx) { 
                             let name = selected_device.name.clone();
                             match selected_device.kind {
                                 DeviceKind::Midi => {
                                     if selected_device.is_connected {
                                         status_message_to_set = Some(format!("Disconnecting MIDI '{}'...", name));
                                         client_message_to_send = Some(ClientMessage::DisconnectMidiDeviceByName(name)); 
                                     } else {
                                         status_message_to_set = Some(format!("Connecting MIDI '{}'...", name));
                                         client_message_to_send = Some(ClientMessage::ConnectMidiDeviceByName(name)); 
                                     }
                                 }
                                 DeviceKind::Osc => {
                                     status_message_to_set = Some(format!("Removing OSC '{}'...", name));
                                     client_message_to_send = Some(ClientMessage::RemoveOscDevice(name));
                                  }
                                  _ => { status_message_to_set = Some("Action not applicable.".to_string()); }
                             }
                         } else {
                              status_message_to_set = Some("No device selected.".to_string());
                         }
                         handled = true;
                    }
                    (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                        // Contextual Ctrl+N based on tab
                         if !state.is_assigning_slot && !state.is_creating_osc && !state.is_naming_virtual {
                            if state.tab_index == 0 { // MIDI Tab
                                 // Enter virtual naming mode
                                 state.is_naming_virtual = true;
                                 state.virtual_port_input = TextArea::default();
                                 state.virtual_port_input.set_block(Block::default().borders(Borders::NONE));
                                 status_message_to_set = Some("Enter Virtual MIDI Port Name...".to_string());
                                 handled = true;
                            } else if state.tab_index == 1 { // OSC Tab
                                 // Enter OSC creation mode
                                  state.is_creating_osc = true;
                                  state.osc_creation_step = 0;
                                  state.osc_name_input = TextArea::default();
                                  state.osc_ip_input = TextArea::default();
                                  state.osc_port_input = TextArea::default();
                                  status_message_to_set = Some("Enter OSC Device Name...".to_string());
                                  handled = true;
                            } else {
                                 // Tab index out of bounds or other state? Ignore.
                            }
                         }
                    }
                    (KeyCode::Char('s'), _) => {
                        // Enter slot assignment mode (if device selected)
                         let current_idx = state.get_current_tab_selection();
                         if current_devices.get(current_idx).is_some() {
                             state.is_assigning_slot = true;
                             state.slot_assignment_input = TextArea::default();
                             status_message_to_set = Some(format!("Assign Slot (0-{}):", MAX_ASSIGNABLE_SLOT));
                          } else {
                             status_message_to_set = Some("No device selected to assign slot.".to_string());
                          }
                          // This always handles the key, even if just to show status
                          handled = true;
                    }
                    // Tab switching
                     (KeyCode::Char('m'), _) => { // Match against tuple
                         if state.tab_index != 0 {
                            state.tab_index = 0;
                            state.selected_index = state.midi_selected_index;
                            handled = true; 
                         }
                     }
                     (KeyCode::Char('o'), _) => { // Match against tuple
                         if state.tab_index != 1 {
                             state.tab_index = 1;
                             state.selected_index = state.osc_selected_index;
                             handled = true; 
                         }
                     }
                    _ => { /* handled remains false */ }
                }
            } // end if !handled (normal mode)
        } // End of state borrow scope

        // --- Post-Action Updates --- 
        // Set status message if it was determined
        if let Some(msg) = status_message_to_set { app.set_status_message(msg); }
        // Send client message if it was determined
        if let Some(msg) = client_message_to_send { app.send_client_message(msg); }
        
        // Return based on whether any action handled the key
        Ok(handled)
    }

    /// Draws the Devices component UI.
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
        if state.is_naming_virtual || state.is_assigning_slot || state.is_creating_osc {
            input_prompt_height = 3;
        }
        let status_height = if !state.status_message.is_empty() { 1 } else { 0 };
        
        let outer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5), // Main zone (with minimum size)
                Constraint::Length(input_prompt_height), // Input zone (if visible)
                Constraint::Length(status_height), // Status message (if present)
            ])
            .split(area);
            
        let main_area = outer_chunks[0];
        let input_area = if input_prompt_height > 0 { Some(outer_chunks[1]) } else { None };
        let status_area = if status_height > 0 { 
            if input_prompt_height > 0 { Some(outer_chunks[2]) } else { Some(outer_chunks[1]) }
        } else { None };

        // --- Draw the main block ---
        let outer_block = Block::default()
            .title(" Devices ")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .style(Style::default().fg(Color::White));

        let inner_area = outer_block.inner(main_area);
        frame.render_widget(outer_block, main_area);
        
        if inner_area.width < 10 || inner_area.height < 7 {
            return;
        }
        
        // Divide the inner area to reserve space for the help at the bottom
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3), // Content zone
                Constraint::Length(2), // Help zone (2 lines)
            ])
            .split(inner_area);
            
        let content_area = inner_chunks[0];
        let help_area = inner_chunks[1];

        // --- Onglets MIDI / OSC ---
        let tabs_height = 2;
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(tabs_height), // Panes
                Constraint::Min(0), // Content
            ])
            .split(content_area);
            
        let tabs_area = content_layout[0];
        let devices_area = content_layout[1];
        
        // Draw the panes
        let tab_titles = vec!["MIDI", "OSC"];
        let tabs = Tabs::new(tab_titles.iter().map(|t| Line::from(*t)).collect::<Vec<Line>>())
            .select(state.tab_index)
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .divider("|")
            .style(Style::default().fg(Color::White));
            
        frame.render_widget(tabs, tabs_area);
        
        // Get the filtered lists of devices HERE, in draw
        let (midi_devices, osc_devices) = Self::get_filtered_devices(app);
        
        // Draw the content of the active pane
        if state.tab_index == 0 {
            // --- MIDI Pane ---
            let headers = vec!["Slot", "Statut", "Nom", "Type"];
            let col_widths = [
                Constraint::Length(6),    // Slot width
                Constraint::Length(8),    // Status
                Constraint::Min(20),      // Name
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
                let is_animated = animation_char.is_some() && state.animation_device_id == Some(device_id_u32);
                
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
            // --- OSC Pane ---
            let headers = vec!["Slot", "Status", "Name", "Address"];
            let col_widths = [
                Constraint::Length(6),    // Slot ID
                Constraint::Length(8),    // Status (Active/Inactive?)
                Constraint::Min(15),      // Name
                Constraint::Min(18),      // Address (IP:Port)
            ];
            
            let header_cells = headers.iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)));
            let header = Row::new(header_cells)
                .style(Style::default().bg(Color::DarkGray))
                .height(1);
                
            let rows = osc_devices.iter().enumerate().map(|(visual_index, device)| {
                let is_selected = visual_index == state.selected_index;
                let slot_id = device.id; // Assumes DeviceInfo includes slot ID for OSC too
                
                // OSC doesn't have a persistent connection, maybe just "Active"?
                let status_text = "Active"; 
                let status_color = Color::Cyan;
                
                let row_style = if is_selected { Style::default().bg(Color::Blue).fg(Color::White) } else { Style::default() };
                
                let slot_display = if slot_id == 0 { "--".to_string() } else { format!("{}", slot_id) };
                let slot_cell = Cell::from(slot_display);
                let status_cell = Cell::from(status_text).style(Style::default().fg(status_color));
                let name_cell = Cell::from(device.name.as_str());
                // Use the actual address from DeviceInfo
                let addr_display = device.address.clone().unwrap_or_else(|| "N/A".to_string());
                let addr_cell = Cell::from(addr_display); 
                
                Row::new(vec![slot_cell, status_cell, name_cell, addr_cell])
                    .style(row_style)
                    .height(1)
            });
            
            let table = Table::new(rows, col_widths)
                .header(header)
                .block(Block::default().borders(Borders::NONE));
                
            frame.render_widget(table, devices_area);
        }

        // Display the text input zone if the user is naming a virtual port OR assigning a slot
        if let Some(area) = input_area {
            if state.is_naming_virtual {
                let input_widget = &state.virtual_port_input;
                let block = Block::default().title(" Virtual Port Name ").borders(Borders::ALL).style(Style::default().fg(Color::Yellow));
                frame.render_widget(block.clone(), area);
                frame.render_widget(input_widget, block.inner(area));
            } else if state.is_assigning_slot {
                 let input_widget = &state.slot_assignment_input;
                 let block = Block::default().title(" Assign Slot ").borders(Borders::ALL).style(Style::default().fg(Color::Yellow));
                 frame.render_widget(block.clone(), area);
                 frame.render_widget(input_widget, block.inner(area));
            } else if state.is_creating_osc {
                 let title = match state.osc_creation_step {
                      0 => " OSC Name (Enter: Next, Esc: Cancel) ",
                      1 => " OSC IP Address (Enter: Next, Esc: Back) ",
                      2 => " OSC Port (Enter: Create, Esc: Back) ",
                      _ => " Invalid State ",
                 };
                 let block = Block::default()
                         .borders(Borders::ALL)
                         .border_type(BorderType::Plain)
                         .title(title)
                         .style(Style::default().fg(Color::Magenta)); 
                         
                  frame.render_widget(block.clone(), area);
                  let inner_area = block.inner(area);

                  // Render the specific widget based on the step
                  match state.osc_creation_step {
                      0 => frame.render_widget(&state.osc_name_input, inner_area),
                      1 => frame.render_widget(&state.osc_ip_input, inner_area),
                      2 => frame.render_widget(&state.osc_port_input, inner_area),
                      _ => frame.render_widget(Paragraph::new("Error"), inner_area), // Correctly render Paragraph
                  }
            }
        }
        
        // Display the status message if it is present
        if let Some(area) = status_area {
            let status_style = Style::default().fg(Color::Yellow);
            let status_paragraph = Paragraph::new(state.status_message.as_str())
                .style(status_style)
                .alignment(Alignment::Center);
            frame.render_widget(status_paragraph, area);
        }

        // --- Render Help Text ---
        let key_style = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);
        let text_style = Style::default().fg(Color::DarkGray);
        let help_spans1;
        let help_spans2;

        if state.is_naming_virtual {
             // Help for naming mode
             help_spans1 = vec![
                 Span::styled("Enter", key_style), Span::styled(": Confirm | ", text_style),
                 Span::styled("Esc", key_style), Span::styled(": Cancel", text_style),
             ];
             help_spans2 = vec![
                 Span::styled("↑↓", key_style), Span::styled(": Browse through history", text_style),
             ];
        } else if state.is_assigning_slot {
            // Help for slot assignment mode
            help_spans1 = vec![
                Span::styled("Enter", key_style), Span::styled(": Confirm | ", text_style),
                Span::styled("Esc", key_style), Span::styled(": Cancel | ", text_style),
                Span::styled("0-9", key_style), Span::styled(": Enter Slot Number", text_style),
            ];
            help_spans2 = vec![Span::raw("")]; // Second line empty for this mode
        } else if state.is_creating_osc {
            // Help for OSC creation mode
            help_spans1 = vec![
                Span::styled("Enter", key_style), Span::styled(": Next/Confirm | ", text_style),
                Span::styled("Esc", key_style), Span::styled(": Back/Cancel", text_style),
            ];
            help_spans2 = vec![Span::raw("")]; // Second line empty for this mode
        } else {
            // Help for normal mode
            help_spans1 = vec![
                Span::styled("↑↓", key_style), Span::styled(": Navigate | ", text_style),
                Span::styled("M", key_style), Span::styled("/", text_style), Span::styled("O", key_style), Span::styled(": MIDI/OSC | ", text_style),
                Span::styled("s", key_style), Span::styled(": Assign Slot", text_style),
            ];
             help_spans2 = vec![
                Span::styled("Enter", key_style), Span::styled(": Connect(MIDI)/Remove(OSC) | ", text_style),
                Span::styled("Ctrl+N", key_style), Span::styled(": New MIDI/OSC Device", text_style),
            ];
        }
        let help_text = vec![Line::from(help_spans1), Line::from(help_spans2)];
        let help = Paragraph::new(help_text).alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
