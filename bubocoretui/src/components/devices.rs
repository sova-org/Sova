use crate::App;
use crate::components::Component;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph},
};

pub struct DevicesState {
    pub selected_index: usize,
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
}

impl Component for DevicesComponent {
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        // Devices-specific key handling
        match key_event.code {
            KeyCode::Up => {
                if !app.server.devices.is_empty() {
                    app.interface.components.devices_state.selected_index = 
                        app.interface.components.devices_state.selected_index.saturating_sub(1);
                }
                Ok(true)
            }
            KeyCode::Down => {
                if !app.server.devices.is_empty() {
                    let len = app.server.devices.len();
                    app.interface.components.devices_state.selected_index = 
                        (app.interface.components.devices_state.selected_index + 1).min(len - 1);
                }
                Ok(true)
            }
            KeyCode::Enter => {
                // TODO: Implement device selection/connection
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        // Create the main block for the entire component area
        let block = Block::default()
            .title(" Available Devices ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::Cyan));
        
        // Calculate the inner area *after* drawing the block
        let inner_area = block.inner(area);
        frame.render_widget(block, area); // Render the block first

        // Split the inner area for list and help text
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // List takes available space
                Constraint::Length(1), // Help text takes 1 line
            ])
            .split(inner_area); // Split the inner area, not the original area

        let list_area = chunks[0];
        let help_area = chunks[1];

        // Create the list of devices
        let devices: Vec<ListItem> = app.server.devices
            .iter()
            .enumerate()
            .map(|(i, device)| {
                let style = if i == app.interface.components.devices_state.selected_index {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                ListItem::new(Text::from(device.clone())).style(style)
            })
            .collect();

        let list = List::new(devices)
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .block(Block::default()); // Add a default block for potential padding/styling if needed, though maybe not necessary

        // Render the list inside the calculated list_area
        frame.render_widget(list, list_area);

        // Render help text in the help_area
        let help_text = "↑↓: Navigate | Enter: Select";
        let help = Paragraph::new(Text::from(help_text))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}
