use crate::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
// Import shared types from bubocorelib
use bubocorelib::shared_types::{DeviceInfo, DeviceKind};
// Import ClientMessage
use bubocorelib::server::client::ClientMessage;
use tui_textarea::TextArea;

pub struct DevicesState {
    pub selected_index: usize,
    /// Indique si l'utilisateur est en train de nommer un port MIDI virtuel
    pub is_naming_virtual: bool,
    /// Zone de texte pour entrer le nom du port MIDI virtuel
    pub virtual_port_input: TextArea<'static>,
    /// Message de statut pour la création du port virtuel
    pub status_message: String,
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
        let midi_devices: Vec<DeviceInfo> = app.server.devices.iter().filter(|d| d.kind == DeviceKind::Midi).cloned().collect();
        let osc_devices: Vec<DeviceInfo> = app.server.devices.iter().filter(|d| d.kind == DeviceKind::Osc).cloned().collect();
        (midi_devices, osc_devices)
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
                    app.interface.components.devices_state.status_message = "Création annulée.".to_string();
                    app.set_status_message("Création de port virtuel annulée.".to_string());
                    return Ok(true);
                }
                // Confirmer la création
                KeyCode::Enter => {
                    let virtual_port_name = app.interface.components.devices_state.virtual_port_input.lines()[0].trim().to_string();
                    
                    if virtual_port_name.is_empty() {
                        app.interface.components.devices_state.status_message = "Le nom du port ne peut pas être vide.".to_string();
                        app.set_status_message("Le nom du port ne peut pas être vide.".to_string());
                    } else {
                        // Envoyer la demande de création au serveur
                        app.send_client_message(ClientMessage::CreateVirtualMidiOutput(virtual_port_name.clone()));
                        
                        // Mettre à jour l'état
                        app.interface.components.devices_state.is_naming_virtual = false;
                        app.interface.components.devices_state.status_message = format!("Création du port '{}' en cours...", virtual_port_name);
                        app.set_status_message(format!("Création du port MIDI virtuel: {}", virtual_port_name));
                        
                        // Réinitialiser le champ de saisie
                        app.interface.components.devices_state.virtual_port_input = TextArea::default();
                        app.interface.components.devices_state.virtual_port_input.set_block(
                            Block::default().borders(Borders::NONE)
                        );
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

        // Si l'utilisateur n'est pas en mode de nommage, gérer la navigation normale
        let (midi_devices, osc_devices) = Self::get_filtered_devices(app);
        // Combine lists to get a unified index across both panes for selection logic
        let all_devices_ordered: Vec<DeviceInfo> = midi_devices.iter().chain(osc_devices.iter()).cloned().collect();
        let total_devices = all_devices_ordered.len();

        match key_event.code {
            KeyCode::Up => {
                if total_devices > 0 {
                    app.interface.components.devices_state.selected_index =
                        app.interface.components.devices_state.selected_index.saturating_sub(1);
                }
                Ok(true)
            }
            KeyCode::Down => {
                 if total_devices > 0 {
                    app.interface.components.devices_state.selected_index =
                        (app.interface.components.devices_state.selected_index + 1).min(total_devices.saturating_sub(1));
                 }
                Ok(true)
            }
            KeyCode::Enter => {
                let selected_index = app.interface.components.devices_state.selected_index;
                if let Some(selected_device) = all_devices_ordered.get(selected_index) {
                    // Use device ID for connect/disconnect messages
                    let device_id = selected_device.id;
                    let device_name = &selected_device.name; // Keep name for status message

                    if selected_device.kind == DeviceKind::Midi {
                        if selected_device.is_connected {
                            app.set_status_message(format!("Requesting disconnect for ID {} ('{}')...", device_id, device_name));
                            app.send_client_message(ClientMessage::DisconnectMidiDevice(device_id));
                        } else {
                            app.set_status_message(format!("Requesting connect for ID {} ('{}')...", device_id, device_name));
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
                app.interface.components.devices_state.status_message = "Entrez le nom du port MIDI virtuel".to_string();
                app.set_status_message("Création d'un nouveau port MIDI virtuel...".to_string());
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let devices_state = &app.interface.components.devices_state;
        
        // Définir la mise en page en fonction de l'état
        let input_prompt_height = if devices_state.is_naming_virtual { 3 } else { 0 };
        let help_height = 1;
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),                       // Zone principale
                Constraint::Length(input_prompt_height),  // Zone de saisie (si visible)
                Constraint::Length(help_height),          // Zone d'aide
            ])
            .split(area);
            
        let main_area = chunks[0];
        let help_area = if devices_state.is_naming_virtual {
            chunks[2]
        } else {
            chunks[1]
        };
        
        // Dessiner l'interface principale (listes de périphériques)
        let outer_block = Block::default()
            .title(" Devices ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let inner_area = outer_block.inner(main_area);
        frame.render_widget(outer_block, main_area);

        // Split the inner area horizontally for MIDI/OSC
        let pane_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // MIDI Pane
                Constraint::Percentage(50), // OSC Pane
            ])
            .split(inner_area);

        let midi_area = pane_chunks[0];
        let osc_area = pane_chunks[1];

        // --- MIDI Pane ---
        let midi_block = Block::default().title(" MIDI ").borders(Borders::ALL).style(Style::default().fg(Color::Cyan));
        let midi_list_area = midi_block.inner(midi_area);
        frame.render_widget(midi_block, midi_area);

        // TEMP: Filter devices for MIDI pane
        let (midi_devices, osc_devices) = Self::get_filtered_devices(app);
        let mut current_global_index = 0; // Track index across both lists

        let midi_items: Vec<ListItem> = midi_devices.iter().map(|device| {
            let list_index = current_global_index;
            current_global_index += 1;

            let style = if list_index == app.interface.components.devices_state.selected_index {
                Style::default().bg(Color::Blue).fg(Color::White) // Selected style
            } else if device.is_connected {
                 Style::default().fg(Color::Green) // Connected style
            } else {
                 Style::default().fg(Color::Yellow) // Available style
            };
            // Display ID along with name
            let display_text = format!("[{}] {}", device.id, device.name);
            ListItem::new(Text::from(display_text)).style(style)
        }).collect();

        let midi_list = List::new(midi_items)
            .block(Block::default()); // No extra block needed inside the pane block
        frame.render_widget(midi_list, midi_list_area);


        // --- OSC Pane ---
        let osc_block = Block::default().title(" OSC ").borders(Borders::ALL).style(Style::default().fg(Color::Magenta));
        let osc_list_area = osc_block.inner(osc_area);
        frame.render_widget(osc_block, osc_area);

        let osc_items: Vec<ListItem> = osc_devices.iter().map(|device| {
             let list_index = current_global_index;
            current_global_index += 1;

            let style = if list_index == app.interface.components.devices_state.selected_index {
                Style::default().bg(Color::Blue).fg(Color::White) // Selected style
            } else if device.is_connected {
                 Style::default().fg(Color::Green) // Connected style
            } else {
                 Style::default().fg(Color::Yellow) // Available style
            };
             // Display ID along with name
             let display_text = format!("[{}] {}", device.id, device.name);
            ListItem::new(Text::from(display_text)).style(style)
        }).collect();

         let osc_list = List::new(osc_items)
            .block(Block::default());
        frame.render_widget(osc_list, osc_list_area);

        // Afficher la zone de saisie de texte si l'utilisateur est en train de nommer un port virtuel
        if devices_state.is_naming_virtual {
            let input_area = chunks[1];
            let mut virtual_input = devices_state.virtual_port_input.clone();
            virtual_input.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Nom du port MIDI virtuel (Entrée: Confirmer, Échap: Annuler) ")
                    .style(Style::default().fg(Color::Yellow))
            );
            virtual_input.set_style(Style::default().fg(Color::White));
            frame.render_widget(&virtual_input, input_area);
        }

        // --- Help Text ---
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        
        let help_spans = if devices_state.is_naming_virtual {
            vec![
                Span::styled("Entrée", key_style), Span::styled(": Confirmer | ", help_style),
                Span::styled("Échap", key_style), Span::styled(": Annuler", help_style),
            ]
        } else {
            vec![
                Span::styled("↑↓", key_style), Span::styled(": Naviguer | ", help_style),
                Span::styled("Entrée", key_style), Span::styled(": Connecter/Déconnecter | ", help_style),
                Span::styled("Ctrl+N", key_style), Span::styled(": Nouveau port virtuel", help_style),
            ]
        };
        
        let help = Paragraph::new(Line::from(help_spans))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
