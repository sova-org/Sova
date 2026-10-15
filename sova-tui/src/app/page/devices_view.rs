use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Margin, Rect},
    style::Style,
    symbols::scrollbar,
    widgets::{
        Cell, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Table, TableState,
    },
};
use sova_core::{protocol::DeviceDirection};
use sova_server::ClientMessage;

use crate::{app::{AppPage, AppState}};

#[derive(Debug, Default)]
pub struct DevicesView {
    state: TableState,
    scroll_state: ScrollbarState,
}

impl DevicesView {
    pub fn on_key_event(&mut self, event: KeyEvent, state: &mut AppState) {
        match event.code {
            KeyCode::Esc => {
                state.go_to(AppPage::Scene);
            }
            KeyCode::Up => {
                self.state.select_previous();
                if let Some(i) = self.state.selected() {
                    self.scroll_state = self.scroll_state.position(i);
                }
            }
            KeyCode::Down => {
                self.state.select_next();
                if let Some(i) = self.state.selected() {
                    self.scroll_state = self.scroll_state.position(i);
                }
            }
            KeyCode::Enter => {
                let Some(selected) = self.state.selected() else {
                    return;
                };
                let dev = &state.client_state.devices[selected];
                let name = dev.name.clone();
                state.popup(
                    "Assign device",
                    format!("Which slot to assign device {} ?", dev.name),
                    1,
                    move |state, x| {
                        let i = i64::from(x);
                        if i < 0 {
                            return;
                        }
                        let _ = state.events.send(
                            ClientMessage::AssignDeviceToSlot(i as usize, name)
                        );
                    },
                );
            }
            KeyCode::Char('l') => {
                let Some(selected) = self.state.selected() else {
                    return;
                };
                let dev = &state.client_state.devices[selected];
                let name = dev.name.clone();
                let latency = dev.latency;
                state.popup(
                    "Setup latency",
                    format!("Latency value (in seconds) for device {}", dev.name),
                    latency,
                    move |state, x| {
                        let _ = state.events.send(
                            ClientMessage::SetDeviceLatency(name, x.into())
                        );
                    },
                );
            }
            KeyCode::Char('u') => {
                let Some(selected) = self.state.selected() else {
                    return;
                };
                let dev = &state.client_state.devices[selected];
                if let Some(id) = dev.slot_id {
                    let _ = state.events.send(
                        ClientMessage::UnassignDeviceFromSlot(id)
                    );
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
            _ => (),
        }
    }

    pub fn get_help() -> &'static str {
        "\
        A: Assign      O: Create OSC Out      L: Setup latency\n\
        U: Unassign    M: Connect Midi Out    \n\
        "
    }

    pub fn connect_midi(selected: usize, state: &mut AppState) {
        let dev = &state.client_state.devices[selected];
        state.events.send(
            ClientMessage::ConnectMidiDeviceByName(dev.name.clone())
        );
    }

    pub fn create_osc_out(state: &mut AppState) {
        state.popup(
            "Create OSC Out".to_owned(),
            "Configure a new OSC Output (name:ip:port)".to_owned(),
            String::default(),
            |state, x| {
                let input = String::from(x);
                let vec: Vec<&str> = input.split(":").collect();
                if vec.len() != 3 {
                    state.negative("Wrong address format !");
                    return;
                }
                state.events.send(ClientMessage::CreateOscDevice(
                    vec[0].to_string(), 
                    vec[1].to_string(), 
                    vec[2].parse().unwrap_or_default()
                ));
            },
        );
    }
}

impl StatefulWidget for &mut DevicesView {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let devices = &state.client_state.devices;
        if !devices.is_empty() {
            self.scroll_state = self
                .scroll_state
                .content_length(devices.len() - 1);
        }
        if self.state.selected().is_none() && !devices.is_empty() {
            self.state.select(Some(0));
        }
        let header_style = state.palette.text().bold().bg(state.palette.surface);
        let selected_row_style = Style::default()
            .fg(state.palette.accent)
            .bg(state.palette.selection)
            .bold();
        let header = [
            "Name",
            "I/O",
            "Kind",
            "Connected",
            "Slot",
            "Lat.",
            "Address",
        ]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);
        let mut longest_name = 5;
        let rows: Vec<Row> = devices
            .iter()
            .map(|dev| {
                let name = Cell::from(format!("{}", dev.name));
                longest_name = std::cmp::max(dev.name.len() as u16, longest_name);
                let io = Cell::from(match dev.direction {
                    DeviceDirection::Output => "O",
                    DeviceDirection::Input => "I",
                });
                let kind = Cell::from(format!("{}", dev.kind));
                let connected = Cell::from(format!("{}", dev.is_connected));
                let slot = Cell::from(format!(
                    "{}",
                    dev.slot_id
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default()
                ));
                let latency = Cell::from(format!("{}", dev.latency));
                let addr = Cell::from(format!("{}", dev.address.clone().unwrap_or_default()));
                Row::new([name, io, kind, connected, slot, latency, addr]).height(1)
            })
            .collect();
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
                Constraint::Length(5),
                Constraint::Min(0),
            ],
        )
        .header(header)
        .style(state.palette.text())
        .row_highlight_style(selected_row_style)
        .highlight_symbol(bar)
        .highlight_spacing(HighlightSpacing::Always);
        t.render(area, buf, &mut self.state);
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .render(
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                buf,
                &mut self.scroll_state,
            );
    }
}
