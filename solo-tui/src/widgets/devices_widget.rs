use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{buffer::Buffer, layout::{Constraint, Margin, Rect}, style::{Color, Style, Stylize}, symbols::scrollbar, text::Text, widgets::{Cell, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState}};
use sova_core::protocol::DeviceDirection;

use crate::{app::AppState, event::AppEvent, popup::PopupValue};

#[derive(Debug, Default)]
pub struct DevicesWidget {
    state: TableState,
    scroll_state: ScrollbarState,
}

impl DevicesWidget {

    pub fn process_event(&mut self, state: &mut AppState, event: KeyEvent) {
        match event.code {
            KeyCode::Up => {
                self.state.select_previous();
                if let Some(i) = self.state.selected() {
                    self.scroll_state = self.scroll_state.position(i * 3);
                }
            }
            KeyCode::Down => {
                self.state.select_next();
                if let Some(i) = self.state.selected() {
                    self.scroll_state = self.scroll_state.position(i * 3);
                }
            }
            KeyCode::Char('a') => {
                let Some(selected) = self.state.selected() else {
                    return;
                };
                let dev = &state.devices[selected];
                let name = dev.name.clone();
                state.events.send(AppEvent::Popup(
                    "Assign device".to_owned(),
                    format!("Which slot to assign device {} ?", dev.name), 
                    PopupValue::Int(1), 
                    Box::new(move |state, x| {
                        let _ = state.device_map.assign_slot(i64::from(x) as usize, &name);
                        state.refresh_devices();
                    })
                ));
            }
            KeyCode::Char('u') => {
                let Some(selected) = self.state.selected() else {
                    return;
                };
                let dev = &state.devices[selected];
                if let Some(id) = dev.slot_id {
                    let _ = state.device_map.unassign_slot(id);
                }
            }
            KeyCode::Char('o') => {
                Self::create_osc_out(state);
            }
            KeyCode::Char('m') => {
                let Some(selected) = self.state.selected() else {
                    return;
                };
                Self::connect_midi(selected, state);
            }
            _ => ()
        }
    }

    pub fn get_help() -> &'static str {
        "\
        A: Assign      O: Create OSC Out\n\
        U: Unassign    M: Connect Midi Out\n\
        "
    }

    pub fn connect_midi(selected : usize, state: &mut AppState) {
        let dev = &state.devices[selected];
        if let Err(s) = state.device_map.connect_midi_by_name(&dev.name) {
            state.events.send(AppEvent::Negative(s));
        } else {
            state.events.send(AppEvent::Positive(format!("Connected MIDI device {}", dev.name)));
        }
    }

    pub fn create_osc_out(state: &mut AppState) {
        let ev = AppEvent::Popup(
            "Create OSC Out".to_owned(), 
            "Configure a new OSC Output (name:ip:port)".to_owned(), 
            PopupValue::Text(String::default()), 
            Box::new(|state, x| {
                let input = String::from(x);
                let vec : Vec<&str> = input.split(":").collect();
                if vec.len() != 3 {
                    state.events.send(AppEvent::Negative("Wrong address format !".to_owned()));
                    return;
                }
                match state.device_map.create_osc_output_device(vec[0], vec[1], vec[2].parse().unwrap_or_default()) {
                    Ok(_) => {
                        state.events.send(AppEvent::Positive("Created device !".to_owned()));
                        state.refresh_devices();
                    }
                    Err(e) => state.events.send(AppEvent::Negative(format!("Error: {e}"))),
                }  
            })
        );
        state.events.send(ev);
    }

}

impl StatefulWidget for &mut DevicesWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.devices.is_empty() {
            self.scroll_state = self.scroll_state.content_length((state.devices.len() - 1) * 3);
        }
        if self.state.selected().is_none() && !state.devices.is_empty() {
            self.state.select(Some(0));
        }
        let header_style = Style::default()
            .fg(Color::White)
            .bold();
        let selected_row_style = Style::default()
            .fg(Color::White)
            .bg(Color::LightMagenta)
            .bold();
        let header = [ "Name", "I/O", "Kind", "Connected", "Slot", "Address"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let mut longest_name = 5;
        let rows : Vec<Row> = state.devices.iter().map(|dev| {
            let name = Cell::from(format!("\n{}", dev.name));
            longest_name = std::cmp::max(dev.name.len() as u16, longest_name);
            let io = Cell::from(match dev.direction {
                DeviceDirection::Output => "\nO",
                DeviceDirection::Input => "\nI",
            });
            let kind = Cell::from(format!("\n{}", dev.kind));
            let connected = Cell::from(format!("\n{}", dev.is_connected));
            let slot = Cell::from(format!("\n{}", dev.slot_id.as_ref().map(ToString::to_string).unwrap_or_default()));
            let addr = Cell::from(format!("\n{}", dev.address.clone().unwrap_or_default()));
            Row::new([name, io, kind, connected, slot, addr]).height(3)
        }).collect();
        let bar = " > ";
        let t = Table::new(
            rows,
            [
                // + 1 is for padding.
                Constraint::Length(longest_name + 1),
                Constraint::Length(3),
                Constraint::Length(12),
                Constraint::Length(10),
                Constraint::Length(5),
                Constraint::Min(0),
            ],
        )
            .header(header)
            .row_highlight_style(selected_row_style)
            .highlight_symbol(Text::from(vec![
                "".into(),
                bar.into(),
                "".into(),
            ]))
            .highlight_spacing(HighlightSpacing::Always);
        t.render(area, buf, &mut self.state);
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .render(area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }), buf, &mut self.scroll_state);
    }
}
