use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::{app::AppState, event::AppEvent, popup::PopupValue};

#[derive(Default)]
pub struct TimeWidget;

impl TimeWidget {
    pub fn get_help() -> &'static str {
        "\
        T: Configure tempo     Up: Increase tempo     Space: Play/Pause \n\
        Q: Configure quantum   Down: decrease tempo                     \n\
        R: Reset beat          S: Start/Stop sync                       \n\
        "
    }

    pub fn process_event(state: &mut AppState, event: KeyEvent) {
        match event.code {
            KeyCode::Char('t') => {
                let tempo = state.clock.tempo();
                state.events.send(AppEvent::Popup(
                    "Tempo".to_owned(),
                    "Configure tempo value".to_owned(),
                    PopupValue::Float(tempo),
                    Box::new(|state, x| {
                        state.clock.set_tempo(x.into());
                    }),
                ));
            }
            KeyCode::Char('q') => {
                let quantum = state.clock.quantum();
                state.events.send(AppEvent::Popup(
                    "Quantum".to_owned(),
                    "Configure quantum value".to_owned(),
                    PopupValue::Float(quantum),
                    Box::new(|state, x| {
                        state.clock.set_quantum(x.into());
                    }),
                ));
            }
            KeyCode::Up => {
                state.clock.set_tempo(state.clock.tempo() + 1.0);
            }
            KeyCode::Down => {
                state.clock.set_tempo(state.clock.tempo() - 1.0);
            }
            KeyCode::Char('s') => {
                state.clock.set_start_stop_sync();
                state
                    .events
                    .send(AppEvent::Positive("Start/Stop sync".to_owned()));
            }
            KeyCode::Char('r') => {
                state.clock.reset_beat();
            }
            KeyCode::Char(' ') => {
                state.clock.play_pause();
                state
                    .events
                    .send(AppEvent::Positive("Play/Pause".to_owned()));
            }
            _ => (),
        }
    }
}

impl StatefulWidget for TimeWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        use Constraint::*;
        let layout = Layout::vertical([Length(3), Length(3), Length(3), Length(3), Length(3)])
            .flex(Flex::Center);
        let [tempo_area, quantum_area, sync_area, playing_area, date_area] =
            layout.areas(area.inner(Margin {
                horizontal: 3,
                vertical: 0,
            }));
        let sync = if state.clock.is_sync_enabled() {
            "Enabled".light_green().bold()
        } else {
            "Disabled".light_red().bold()
        };
        let playing = if state.clock.is_playing() {
            "Playing".light_green().bold()
        } else {
            "Paused".light_red().bold()
        };
        Paragraph::new(Line::from(vec![
            Span::from("Tempo : "),
            state.clock.tempo().to_string().white().bold(),
        ]))
        .render(tempo_area, buf);
        Paragraph::new(Line::from(vec![
            Span::from("Quantum : "),
            state.clock.quantum().to_string().white().bold(),
        ]))
        .render(quantum_area, buf);
        Paragraph::new(Line::from(vec![Span::from("Sync : "), sync])).render(sync_area, buf);
        Paragraph::new(Line::from(vec![Span::from("Playing : "), playing]))
            .render(playing_area, buf);
        Paragraph::new(format!("Date : {}", state.clock.micros())).render(date_area, buf);
    }
}
