use crate::components::devices::DevicesState;
use ratatui::{
    layout::Rect,
    widgets::{Block, BorderType, Borders, Paragraph},
    prelude::Widget,
    style::{Style, Color},
};

/// A widget that displays interactive prompts for various device management operations.
/// 
/// The widget handles different types of prompts:
/// - Naming a new virtual MIDI port
/// - Assigning a slot number to a device
/// - Creating a new OSC device (multi-step process)
/// 
/// # Fields
/// 
/// * `state` - A reference to the current state of the devices component, containing
///   the necessary input fields and flags to determine which prompt to display
pub struct PromptWidget<'a> {
    pub state: &'a DevicesState,
}

impl<'a> Widget for PromptWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let state = self.state;
        if state.confirmation_prompt.is_none() { 
            if state.is_naming_virtual {
                let input_widget = &state.virtual_port_input;
                let block = Block::default()
                    .title(" Virtual Port Name ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Yellow));
                ratatui::widgets::Widget::render(block.clone(), area, buf);
                // Render the input widget inside the block's inner area
                let inner_area = block.inner(area);
                ratatui::widgets::Widget::render(input_widget, inner_area, buf);
            } else if state.is_assigning_slot {
                let input_widget = &state.slot_assignment_input;
                let block = Block::default()
                    .title(" Assign Slot ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Yellow));
                ratatui::widgets::Widget::render(block.clone(), area, buf);
                let inner_area = block.inner(area);
                ratatui::widgets::Widget::render(input_widget, inner_area, buf);
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

                ratatui::widgets::Widget::render(block.clone(), area, buf);
                let inner_input_area = block.inner(area);

                match state.osc_creation_step {
                    0 => ratatui::widgets::Widget::render(&state.osc_name_input, inner_input_area, buf),
                    1 => ratatui::widgets::Widget::render(&state.osc_ip_input, inner_input_area, buf),
                    2 => ratatui::widgets::Widget::render(&state.osc_port_input, inner_input_area, buf),
                    _ => ratatui::widgets::Widget::render(Paragraph::new("Error"), inner_input_area, buf),
                }
            }
        }
    }
}