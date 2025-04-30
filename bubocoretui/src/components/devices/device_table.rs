use ratatui::{
    widgets::{Block, Borders, Cell, Row, Table, Widget}, 
    prelude::{Constraint, Color, Rect, Style, Modifier}
};
use bubocorelib::shared_types::DeviceInfo;

/// A widget that displays a table of devices (MIDI or OSC) with their properties.
/// 
/// The table shows different columns and styling based on the device type (MIDI or OSC).
/// It supports selection highlighting and connection/disconnection animations.
/// 
/// # Fields
/// 
/// * `devices` - A slice of device information to display in the table
/// * `selected_index` - The index of the currently selected device in the list
/// * `tab_index` - The active tab (0 for MIDI, 1 for OSC) which determines table layout
/// * `animation_char` - Optional animation character to show during device connection/disconnection
/// * `animation_device_id` - Optional ID of the device currently undergoing animation
pub struct DeviceTable<'a> {
    pub devices: &'a [DeviceInfo],
    pub selected_index: usize,
    pub tab_index: usize,
    pub animation_char: Option<&'static str>,
    pub animation_device_id: Option<u32>,
}

impl<'a> Widget for DeviceTable<'a> {

    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let headers: Vec<&str>;
        let col_widths: Vec<Constraint>;
        let highlight_color: Color;

        if self.tab_index == 0 { // MIDI
            headers = vec!["Slot", "Status", "Name", "Type"];
            col_widths = vec![
                Constraint::Length(6),
                Constraint::Length(8),
                Constraint::Min(20),
                Constraint::Length(10),
            ];
            highlight_color = Color::Yellow;
        } else { // OSC
            headers = vec!["Slot", "Status", "Name", "Address"];
             col_widths = vec![
                Constraint::Length(6),
                Constraint::Length(8),
                Constraint::Min(15),
                Constraint::Min(18),
            ];
            highlight_color = Color::Magenta;
        }

        let header_cells = headers.iter().map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(highlight_color)
                    .add_modifier(Modifier::BOLD),
            )
        });
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::DarkGray))
            .height(1);

        let rows = self.devices
            .iter()
            .enumerate()
            .map(|(visual_index, device)| {
                let is_selected = visual_index == self.selected_index;
                let slot_id = device.id;
                let device_id_u32 = 0; // Placeholder, adapt if needed
                let is_animated = self.animation_char.is_some()
                    && self.animation_device_id == Some(device_id_u32);

                let row_style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                let slot_display = if slot_id == 0 {
                    "--".to_string()
                } else {
                    format!("{}", slot_id)
                };
                let slot_cell = Cell::from(slot_display);
                let name_cell = Cell::from(device.name.as_str());


                if self.tab_index == 0 { // MIDI specific cells
                    let status_text = if is_animated {
                        self.animation_char.unwrap_or("◯")
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
                    let status_cell =
                        Cell::from(status_text).style(Style::default().fg(status_color));
                    let type_cell = Cell::from("MIDI");
                     Row::new(vec![slot_cell, status_cell, name_cell, type_cell])
                        .style(row_style)
                        .height(1)

                } else { // OSC specific cells
                     let status_text = "Active"; // Assuming OSC is always "Active" if listed
                     let status_color = Color::Cyan;
                     let status_cell =
                        Cell::from(status_text).style(Style::default().fg(status_color));
                    let addr_display = device.address.clone().unwrap_or_else(|| "N/A".to_string());
                    let addr_cell = Cell::from(addr_display);
                    Row::new(vec![slot_cell, status_cell, name_cell, addr_cell])
                        .style(row_style)
                        .height(1)
                }
            });

        let table = Table::new(rows, col_widths)
            .header(header)
            .block(Block::default().borders(Borders::NONE));

        // Use render_widget from trait WidgetRef, which Widget implements via Deref
        ratatui::widgets::Widget::render(table, area, buf);
    }
}
