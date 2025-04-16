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

pub struct DevicesState {
    pub selected_index: usize,
    // TODO: Potentially add separate indices or logic for multi-pane selection
}

impl DevicesState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
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
                let virtual_device_name = "BuboVirtualOut1".to_string(); // Hardcoded for now
                app.set_status_message(format!("Requesting creation of virtual MIDI output: {}", virtual_device_name));
                app.send_client_message(ClientMessage::CreateVirtualMidiOutput(virtual_device_name));
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let outer_block = Block::default()
            .title(" Devices ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let inner_area = outer_block.inner(area);
        frame.render_widget(outer_block, area);

        // Main layout: Split horizontally for MIDI/OSC, plus a help row
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // Panes area
                Constraint::Length(1), // Help text
            ])
            .split(inner_area);

        let panes_area = main_chunks[0];
        let help_area = main_chunks[1];

        // Split the panes area horizontally
        let pane_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // MIDI Pane
                Constraint::Percentage(50), // OSC Pane
            ])
            .split(panes_area);

        let midi_area = pane_chunks[0];
        let osc_area = pane_chunks[1];

        // --- MIDI Pane ---
        let midi_block = Block::default().title(" MIDI ").borders(Borders::ALL).style(Style::default().fg(Color::Cyan));
        let midi_list_area = midi_block.inner(midi_area);
        frame.render_widget(midi_block, midi_area);

        // TEMP: Filter devices for MIDI pane
        // We need a stable way to map selected_index to the correct item across panes
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
                 Style::default().fg(Color::Yellow) // Available style (adjust as needed for OSC)
            };
             // Display ID along with name
             let display_text = format!("[{}] {}", device.id, device.name);
            ListItem::new(Text::from(display_text)).style(style)
        }).collect();

         let osc_list = List::new(osc_items)
            .block(Block::default());
        frame.render_widget(osc_list, osc_list_area);


        // --- Help Text ---
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        let help_spans = vec![
            Span::styled("↑↓", key_style), Span::styled(": Navigate | ", help_style),
            Span::styled("Enter", key_style), Span::styled(": Connect/Disconnect | ", help_style),
            Span::styled("Ctrl+N", key_style), Span::styled(": New Virtual Out", help_style),
        ];
        let help = Paragraph::new(Line::from(help_spans))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
