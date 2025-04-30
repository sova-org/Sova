use crate::app::App;
use crate::components::Component;
use bubocorelib::server::client::ClientMessage;
use bubocorelib::shared_types::{DeviceInfo, DeviceKind};
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::components::{
    devices::device_table::DeviceTable,
    devices::utils::centered_rect,
    devices::prompt::PromptWidget,
    devices::help::HelpTextWidget,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Tabs, Wrap, Widget},
};
use std::collections::HashMap;
use std::time::Instant;
use tui_textarea::TextArea;

mod device_table;
mod utils;
mod prompt;
mod help;

/// Maximum user-assignable slot ID (1-based). Slot 0 is used for logging.
const MAX_ASSIGNABLE_SLOT: usize = 16;

/// Stores the state for the Devices UI component.
pub struct DevicesState {
    /// Index of the visually selected item in the current device list (MIDI or OSC tab).
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
    /// Stores the confirmation prompt message, e.g., "Disconnect 'Device'? (Y/N)".
    pub confirmation_prompt: Option<String>,
    /// Stores the action to be performed upon confirmation.
    pub pending_action: Option<ClientMessage>,
}

impl DevicesState {
    /// Creates a new `DevicesState` with default values and initialized text areas.
    pub fn new() -> Self {
        let mut input_area = TextArea::default();
        input_area.set_block(Block::default().borders(Borders::NONE));
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
            // Initialize new confirmation fields
            confirmation_prompt: None,
            pending_action: None,
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
        if !self.recent_port_names.contains(&name) {
            self.recent_port_names.push(name);
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
        let midi_devices: Vec<DeviceInfo> = app
            .server
            .devices
            .iter()
            .filter(|d| {
                d.kind == DeviceKind::Midi
                    && !d.name.contains("BuboCore-Temp-Connector")
                    && !d.name.contains("BuboCore-Virtual-Creator")
            })
            .cloned()
            .collect();

        let osc_devices: Vec<DeviceInfo> = app
            .server
            .devices
            .iter()
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

struct StatusBarWidget<'a> {
    message: &'a str,
}

impl<'a> Widget for StatusBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if !self.message.is_empty() {
            let status_style = Style::default().fg(Color::Yellow);
            let status_paragraph = Paragraph::new(self.message)
                .style(status_style)
                .alignment(Alignment::Center);
            ratatui::widgets::Widget::render(status_paragraph, area, buf);
        }
    }
}

// --- End of new HelpTextWidget ---

// --- Start of new ConfirmationDialogWidget ---

struct ConfirmationDialogWidget<'a> {
    prompt: &'a str,
    full_area: Rect,
}

impl<'a> Widget for ConfirmationDialogWidget<'a> {
    fn render(self, _area: Rect, buf: &mut ratatui::buffer::Buffer) {
        // Note: We ignore the `_area` passed to render because we need the full frame area
        // to calculate the centered position correctly.
        let popup_area = centered_rect(60, 25, self.full_area);

        let block = Block::default()
            .title(" Confirm Action ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .style(Style::default().fg(Color::Red));

        let confirm_key_style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);
        let cancel_key_style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);
        let text_style_popup = Style::default().fg(Color::Yellow);

        let text_lines = vec![
            Line::from(Span::styled(self.prompt, text_style_popup)),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Y", confirm_key_style),
                Span::raw("/"),
                Span::styled("Enter", confirm_key_style),
                Span::styled(": Confirm", confirm_key_style),
                Span::raw("   "),
                Span::styled("N", cancel_key_style),
                Span::raw("/"),
                Span::styled("Esc", cancel_key_style),
                Span::styled(": Cancel", cancel_key_style),
            ]),
        ];

        let prompt_paragraph = Paragraph::new(text_lines)
            .block(block.clone())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        // Clear the area first, then render the dialog
        ratatui::widgets::Widget::render(Clear, popup_area, buf);
        ratatui::widgets::Widget::render(prompt_paragraph, popup_area, buf);
    }
}

// --- End of new ConfirmationDialogWidget ---

impl Component for DevicesComponent {
    /// Handles key events for the Devices component, managing state changes and UI interactions.
    /// Returns `Ok(true)` if the key event was handled, `Ok(false)` otherwise.
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        let (midi_devices, osc_devices) = Self::get_filtered_devices(app);
        let mut status_message_to_set: Option<String> = None;
        let mut client_message_to_send: Option<ClientMessage> = None;
        let mut handled = false;

        {
            // Scope for mutable borrow of state
            let state = &mut app.interface.components.devices_state;

            if let Some(_prompt) = &state.confirmation_prompt {
                match key_event.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        if let Some(action) = state.pending_action.take() {
                            client_message_to_send = Some(action);
                            status_message_to_set = Some("Action confirmed.".to_string());
                        } else {
                            status_message_to_set =
                                Some("Confirmation error (no pending action).".to_string());
                        }
                        state.confirmation_prompt = None;
                        handled = true;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        state.confirmation_prompt = None;
                        state.pending_action = None;
                        status_message_to_set = Some("Action cancelled.".to_string());
                        handled = true;
                    }
                    _ => {
                        handled = true;
                    }
                }
            } else if state.is_creating_osc {
                let osc_handled_in_mode;
                match state.osc_creation_step {
                    0 => {
                        // Name Input
                        match key_event.code {
                            KeyCode::Esc => {
                                state.is_creating_osc = false;
                                status_message_to_set =
                                    Some("OSC device creation cancelled.".to_string());
                                osc_handled_in_mode = true;
                            }
                            KeyCode::Enter => {
                                if !state.osc_name_input.lines()[0].trim().is_empty() {
                                    state.osc_creation_step = 1;
                                    status_message_to_set =
                                        Some("Enter OSC IP Address...".to_string());
                                } else {
                                    status_message_to_set =
                                        Some("OSC name cannot be empty.".to_string());
                                }
                                osc_handled_in_mode = true;
                            }
                            _ => {
                                osc_handled_in_mode = state.osc_name_input.input(key_event);
                            }
                        }
                    }
                    1 => {
                        // IP Input
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
                                    status_message_to_set =
                                        Some("OSC IP cannot be empty.".to_string());
                                }
                                osc_handled_in_mode = true;
                            }
                            _ => {
                                osc_handled_in_mode = state.osc_ip_input.input(key_event);
                            }
                        }
                    }
                    2 => {
                        // Port Input
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
                                        let name =
                                            state.osc_name_input.lines()[0].trim().to_string();
                                        let ip = state.osc_ip_input.lines()[0].trim().to_string();
                                        status_message_to_set = Some(format!(
                                            "Creating OSC '{}' @ {}:{}...",
                                            name, ip, port
                                        ));
                                        client_message_to_send =
                                            Some(ClientMessage::CreateOscDevice(name, ip, port));
                                        state.is_creating_osc = false;
                                        state.osc_creation_step = 0;
                                        state.osc_name_input = TextArea::default();
                                        state.osc_ip_input = TextArea::default();
                                        state.osc_port_input = TextArea::default();
                                    }
                                    _ => {
                                        status_message_to_set =
                                            Some("Invalid port (1-65535).".to_string());
                                    }
                                }
                                osc_handled_in_mode = true;
                            }
                            _ => {
                                osc_handled_in_mode = state.osc_port_input.input(key_event);
                            }
                        }
                    }
                    _ => {
                        state.is_creating_osc = false;
                        osc_handled_in_mode = true;
                    }
                }
                if osc_handled_in_mode {
                    handled = true;
                }
            } else if state.is_assigning_slot {
                let slot_handled_in_mode;
                let mut exit_assign_mode = false;
                let mut temp_client_msg = None;
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
                                let current_devices = match state.tab_index {
                                    0 => &midi_devices,
                                    1 => &osc_devices,
                                    _ => &Vec::new(),
                                };

                                if let Some(selected_device) =
                                    current_devices.get(state.selected_index)
                                {
                                    let device_name = selected_device.name.clone();
                                    let current_slot = selected_device.id;
                                    let target_slot_assignee_name =
                                        state.slot_assignments.get(&digit).cloned();

                                    if digit == 0 {
                                        // Unassign
                                        if current_slot != 0 {
                                            // Only unassign if currently assigned
                                            status_message_to_set = Some(format!(
                                                "Unassigning '{}' from Slot {}...",
                                                device_name, current_slot
                                            ));
                                            temp_client_msg = Some(
                                                ClientMessage::UnassignDeviceFromSlot(current_slot),
                                            );
                                        } else {
                                            status_message_to_set = Some(format!(
                                                "Device '{}' is not assigned to a slot.",
                                                device_name
                                            ));
                                        }
                                    } else {
                                        // Assign (1-16)
                                        let target_slot_id = digit;
                                        if let Some(assignee) = target_slot_assignee_name {
                                            if assignee != device_name {
                                                status_message_to_set = Some(format!(
                                                    "Slot {} is already assigned to '{}'. Unassign first.",
                                                    target_slot_id, assignee
                                                ));
                                            } else {
                                                status_message_to_set = Some(format!(
                                                    "Device '{}' is already assigned to Slot {}.",
                                                    device_name, target_slot_id
                                                ));
                                            }
                                        } else if current_slot == target_slot_id {
                                            status_message_to_set = Some(format!(
                                                "Device '{}' is already assigned to Slot {}.",
                                                device_name, target_slot_id
                                            ));
                                        } else {
                                            status_message_to_set = Some(format!(
                                                "Assigning '{}' to Slot {}...",
                                                device_name, target_slot_id
                                            ));
                                            temp_client_msg =
                                                Some(ClientMessage::AssignDeviceToSlot(
                                                    target_slot_id,
                                                    device_name,
                                                ));
                                        }
                                    }
                                } else {
                                    status_message_to_set =
                                        Some("No device selected (internal error?).".to_string());
                                }
                            }
                            _ => {
                                // Parsing failed or number out of range
                                status_message_to_set = Some(format!(
                                    "Invalid slot: '{}'. Must be 0-{}.",
                                    input_str, MAX_ASSIGNABLE_SLOT
                                ));
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
                    state
                        .slot_assignment_input
                        .set_block(Block::default().borders(Borders::NONE));
                }
                if slot_handled_in_mode {
                    handled = true;
                    client_message_to_send = temp_client_msg;
                }
            } else if state.is_naming_virtual {
                let virtual_handled_in_mode;
                let mut temp_client_msg = None;
                match key_event.code {
                    KeyCode::Esc => {
                        state.is_naming_virtual = false;
                        state.virtual_port_input = TextArea::default();
                        state
                            .virtual_port_input
                            .set_block(Block::default().borders(Borders::NONE));
                        state.status_message = "Creation cancelled.".to_string();
                        status_message_to_set =
                            Some("Virtual port creation cancelled.".to_string());
                        virtual_handled_in_mode = true;
                    }
                    KeyCode::Enter => {
                        let name = state.virtual_port_input.lines()[0].trim().to_string();
                        if name.is_empty() {
                            status_message_to_set = Some("Port name cannot be empty.".to_string());
                        } else {
                            state.add_recent_port_name(name.clone());
                            state.is_naming_virtual = false;
                            state.status_message = format!("Creating port '{}'...", name);
                            state.virtual_port_input = TextArea::default();
                            state
                                .virtual_port_input
                                .set_block(Block::default().borders(Borders::NONE));
                            temp_client_msg =
                                Some(ClientMessage::CreateVirtualMidiOutput(name.clone()));
                            status_message_to_set =
                                Some(format!("Creating MIDI virtual port: {}", name));
                        }
                        virtual_handled_in_mode = true;
                    }
                    KeyCode::Up => {
                        let current_text = state.virtual_port_input.lines()[0].trim();
                        let recent_names = &state.recent_port_names;

                        if recent_names.is_empty() {
                            return Ok(false);
                        }

                        let next_name = if let Some(idx) =
                            recent_names.iter().position(|n| n == current_text)
                        {
                            if idx > 0 {
                                // Move towards the start of the vec (older entries)
                                Some(&recent_names[idx - 1])
                            } else {
                                None
                            } // Already at the oldest
                        } else if !recent_names.is_empty() {
                            recent_names.last()
                        } else {
                            None
                        };

                        if let Some(name_to_set) = next_name {
                            let mut new_input = TextArea::new(vec![name_to_set.clone()]);
                            new_input.set_block(Block::default().borders(Borders::NONE));
                            state.virtual_port_input = new_input;
                        }
                        virtual_handled_in_mode = true;
                    }
                    KeyCode::Down => {
                        let current_text = state.virtual_port_input.lines()[0].trim();
                        let recent_names = &state.recent_port_names;

                        if recent_names.is_empty() {
                            return Ok(false);
                        }

                        let next_name = if let Some(idx) =
                            recent_names.iter().position(|n| n == current_text)
                        {
                            if idx < recent_names.len() - 1 {
                                Some(&recent_names[idx + 1])
                            } else {
                                None
                            } // Already at the newest
                        } else if !recent_names.is_empty() {
                            None
                        } else {
                            None
                        };

                        if let Some(name_to_set) = next_name {
                            let mut new_input = TextArea::new(vec![name_to_set.clone()]);
                            new_input.set_block(Block::default().borders(Borders::NONE));
                            state.virtual_port_input = new_input;
                        }
                        virtual_handled_in_mode = true;
                    }
                    _ => {
                        virtual_handled_in_mode = state.virtual_port_input.input(key_event);
                    }
                }
                if virtual_handled_in_mode {
                    handled = true;
                    client_message_to_send = temp_client_msg;
                }
            } else if !handled {
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
                            if state.tab_index == 0 {
                                state.midi_selected_index = next_idx;
                            } else {
                                state.osc_selected_index = next_idx;
                            }
                            handled = true;
                        }
                    }
                    (KeyCode::Down, _) => {
                        if total_devices > 0 {
                            let current_idx = state.get_current_tab_selection();
                            let next_idx = (current_idx + 1).min(total_devices.saturating_sub(1));
                            state.selected_index = next_idx;
                            if state.tab_index == 0 {
                                state.midi_selected_index = next_idx;
                            } else {
                                state.osc_selected_index = next_idx;
                            }
                            handled = true;
                        }
                    }
                    (KeyCode::Enter, _) => {
                        let current_idx = state.get_current_tab_selection();
                        if let Some(selected_device) = current_devices.get(current_idx) {
                            match selected_device.kind {
                                DeviceKind::Midi => {
                                    if !selected_device.is_connected {
                                        let name = selected_device.name.clone();
                                        status_message_to_set =
                                            Some(format!("Connecting MIDI '{}'...", name));
                                        client_message_to_send =
                                            Some(ClientMessage::ConnectMidiDeviceByName(name));
                                    } else {
                                        status_message_to_set = Some(format!(
                                            "MIDI '{}' already connected.",
                                            selected_device.name
                                        ));
                                    }
                                }
                                DeviceKind::Osc => {
                                    status_message_to_set =
                                        Some("Use Backspace to remove OSC devices.".to_string());
                                }
                                _ => {
                                    status_message_to_set =
                                        Some("Action not applicable.".to_string());
                                }
                            }
                        } else {
                            status_message_to_set = Some("No device selected.".to_string());
                        }
                        handled = true;
                    }
                    (KeyCode::Backspace, _) | (KeyCode::Delete, _) => {
                        let current_idx = state.get_current_tab_selection();
                        if let Some(selected_device) = current_devices.get(current_idx) {
                            let name = selected_device.name.clone();
                            match selected_device.kind {
                                DeviceKind::Midi => {
                                    if selected_device.is_connected {
                                        state.confirmation_prompt =
                                            Some(format!("Disconnect MIDI '{}'?", name));
                                        state.pending_action = Some(
                                            ClientMessage::DisconnectMidiDeviceByName(name.clone()),
                                        );
                                        status_message_to_set =
                                            Some("Confirmation required.".to_string());
                                    } else {
                                        status_message_to_set =
                                            Some(format!("MIDI '{}' is not connected.", name));
                                    }
                                }
                                DeviceKind::Osc => {
                                    state.confirmation_prompt =
                                        Some(format!("Remove OSC '{}'?", name));
                                    state.pending_action =
                                        Some(ClientMessage::RemoveOscDevice(name.clone()));
                                    status_message_to_set =
                                        Some("Confirmation required.".to_string());
                                }
                                _ => {
                                    status_message_to_set =
                                        Some("Action not applicable.".to_string());
                                }
                            }
                        } else {
                            status_message_to_set = Some("No device selected.".to_string());
                        }
                        handled = true;
                    }
                    (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                        if !state.is_assigning_slot
                            && !state.is_creating_osc
                            && !state.is_naming_virtual
                        {
                            if state.tab_index == 0 {
                                // MIDI Tab
                                state.is_naming_virtual = true;
                                state.virtual_port_input = TextArea::default();
                                state
                                    .virtual_port_input
                                    .set_block(Block::default().borders(Borders::NONE));
                                status_message_to_set =
                                    Some("Enter Virtual MIDI Port Name...".to_string());
                                handled = true;
                            } else if state.tab_index == 1 {
                                // OSC Tab
                                state.is_creating_osc = true;
                                state.osc_creation_step = 0;
                                state.osc_name_input = TextArea::default();
                                state.osc_ip_input = TextArea::default();
                                state.osc_port_input = TextArea::default();
                                status_message_to_set =
                                    Some("Enter OSC Device Name...".to_string());
                                handled = true;
                            }
                        }
                    }
                    (KeyCode::Char('s'), _) => {
                        let current_idx = state.get_current_tab_selection();
                        if current_devices.get(current_idx).is_some() {
                            state.is_assigning_slot = true;
                            state.slot_assignment_input = TextArea::default();
                            status_message_to_set =
                                Some(format!("Assign Slot (0-{}):", MAX_ASSIGNABLE_SLOT));
                        } else {
                            status_message_to_set =
                                Some("No device selected to assign slot.".to_string());
                        }
                        handled = true;
                    }
                    (KeyCode::Char('m'), _) => {
                        if state.tab_index != 0 {
                            state.tab_index = 0;
                            state.selected_index = state.midi_selected_index;
                            handled = true;
                        }
                    }
                    (KeyCode::Char('o'), _) => {
                        if state.tab_index != 1 {
                            state.tab_index = 1;
                            state.selected_index = state.osc_selected_index;
                            handled = true;
                        }
                    }
                    _ => {}
                }
            } // end if !handled (normal mode)
        } // End of state borrow scope

        if let Some(msg) = status_message_to_set {
            app.set_status_message(msg);
        }
        if let Some(msg) = client_message_to_send {
            app.send_client_message(msg);
        }

        Ok(handled)
    }

    /// Draws the Devices component UI.
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let state = &app.interface.components.devices_state;

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

        let mut prompt_height = 0;
        if state.is_naming_virtual || state.is_assigning_slot || state.is_creating_osc {
            prompt_height = 3;
        }
        let status_height = if !state.status_message.is_empty() {
            1
        } else {
            0
        };

        let outer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(prompt_height),
                Constraint::Length(status_height),
            ])
            .split(area);

        let main_area = outer_chunks[0];
        let prompt_area = if prompt_height > 0 {
            Some(outer_chunks[1])
        } else {
            None
        };
        let status_area = if status_height > 0 {
            if prompt_height > 0 {
                Some(outer_chunks[2])
            } else {
                Some(outer_chunks[1])
            }
        } else {
            None
        };

        let outer_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .style(Style::default().fg(Color::White));

        let inner_area = outer_block.inner(main_area);
        frame.render_widget(outer_block, main_area);

        if inner_area.width < 10 || inner_area.height < 7 {
            return;
        }

        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(2)])
            .split(inner_area);

        let content_area = inner_chunks[0];
        let help_area = inner_chunks[1];

        let tabs_height = 2;
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(tabs_height), Constraint::Min(0)])
            .split(content_area);

        let tabs_area = content_layout[0];
        let devices_area = content_layout[1];

        let tab_titles = vec!["MIDI", "OSC"];
        let tabs = Tabs::new(
            tab_titles
                .iter()
                .map(|t| Line::from(*t))
                .collect::<Vec<Line>>(),
        )
        .select(state.tab_index)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|")
        .style(Style::default().fg(Color::White));

        frame.render_widget(tabs, tabs_area);

        let (midi_devices, osc_devices) = Self::get_filtered_devices(app);

        let devices_to_render = if state.tab_index == 0 {
            &midi_devices
        } else {
            &osc_devices
        };

        let device_table = DeviceTable {
            devices: devices_to_render,
            selected_index: state.selected_index,
            tab_index: state.tab_index,
            animation_char,
            animation_device_id: state.animation_device_id,
        };

        frame.render_widget(device_table, devices_area);

        if let Some(input_prompt_area) = prompt_area {
            let prompt_widget = PromptWidget { state };
            frame.render_widget(prompt_widget, input_prompt_area);
        }

        if let Some(status_render_area) = status_area {
            let status_widget = StatusBarWidget { message: &state.status_message };
            frame.render_widget(status_widget, status_render_area);
        }

        let help_widget = HelpTextWidget {
            is_naming_virtual: state.is_naming_virtual,
            is_assigning_slot: state.is_assigning_slot,
            is_creating_osc: state.is_creating_osc,
        };
        frame.render_widget(help_widget, help_area);

        if let Some(prompt) = &state.confirmation_prompt {
            let dialog_widget = ConfirmationDialogWidget { prompt, full_area: area };
            frame.render_widget(dialog_widget, area); // Pass the full area

            // Still need to manage cursor position outside the widget
            if !state.is_naming_virtual && !state.is_assigning_slot && !state.is_creating_osc {
                frame.set_cursor_position(Rect::default());
            }
        }
    }
}