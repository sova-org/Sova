use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{buffer::Buffer, layout::{Constraint, Flex, Layout, Rect}, widgets::{Paragraph, StatefulWidget, Widget}};
use sova_core::schedule::{ActionTiming, SchedulerMessage};
use sova_server::Snapshot;

use crate::{app::AppState, event::AppEvent, popup::PopupValue};

pub struct ConfigureWidget;

impl ConfigureWidget {
    pub fn get_help() -> &'static str {
        "\
        C-S: Save \n\
        C-L: Load \n\
        "
    }

    pub fn process_event(state: &mut AppState, event: KeyEvent) { 
        match event.code {
            KeyCode::Char('s') if event.modifiers == KeyModifiers::CONTROL => {
                state.events.send(AppEvent::Popup(
                    "Save scene".to_owned(), 
                    "Input path to save scene".to_owned(), 
                    PopupValue::Text("scene.sova".to_owned()), 
                    Box::new(|state, x| {
                        let micros = state.clock.micros();
                        let beat = state.clock.beat_at_date(micros);
                        let path = String::from(x);
                        let snapshot = Snapshot {
                            scene: state.scene_image.clone(),
                            tempo: state.clock.tempo(),
                            beat,
                            micros,
                            quantum: state.clock.quantum(),
                            devices: None
                        };
                        let Ok(snapshot) = serde_json::to_vec(&snapshot) else {
                            state.events.send(AppEvent::Negative("Failed to save scene !".to_owned()));
                            return;
                        };
                        let res = std::fs::write(path, snapshot);
                        if res.is_ok() {
                            state.events.send(AppEvent::Positive("Saved scene !".to_owned()));
                        } else {
                            state.events.send(AppEvent::Negative("Failed to save scene !".to_owned()));
                        }
                    })
                ));
            } 
            KeyCode::Char('l') if event.modifiers == KeyModifiers::CONTROL => {
                state.events.send(AppEvent::Popup(
                    "Load scene".to_owned(), 
                    "Path of the file to load".to_owned(), 
                    PopupValue::Text("scene.sova".to_owned()), 
                    Box::new(|state, x| {
                        let path = String::from(x);
                        let Ok(bytes) = std::fs::read(path) else {
                            state.events.send(AppEvent::Negative("Failed to read file !".to_owned()));
                            return;
                        };
                        let Ok(snapshot) = serde_json::from_slice::<Snapshot>(&bytes) else {
                            state.events.send(AppEvent::Negative("Failed to load scene !".to_owned()));
                            return;
                        };
                        state.events.send(
                            AppEvent::SchedulerControl(SchedulerMessage::SetScene(snapshot.scene, ActionTiming::Immediate))
                        );
                        state.events.send(
                            AppEvent::SchedulerControl(SchedulerMessage::SetTempo(snapshot.tempo, ActionTiming::Immediate))
                        );
                        state.events.send(
                            AppEvent::SchedulerControl(SchedulerMessage::SetQuantum(snapshot.quantum, ActionTiming::Immediate))
                        );
                        state.events.send(AppEvent::Positive("Loaded scene !".to_owned()));
                    })
                ));
            } 
            _ => ()
        }
    }
}

impl StatefulWidget for ConfigureWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        use Constraint::*;
        let layout = Layout::vertical([Length(1)]).flex(Flex::Center);
        let [center] = layout.areas(area);
        Paragraph::new("C-S/C-L to Save/Load scene").centered().render(center, buf);
    }
}