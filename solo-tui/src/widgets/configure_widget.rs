use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{buffer::Buffer, layout::{Constraint, Flex, Layout, Margin, Rect}, style::Stylize, text::{self, Span}, widgets::{Paragraph, StatefulWidget, Widget}};
use sova_core::{scene::ExecutionMode, schedule::{ActionTiming, SchedulerMessage}};
use sova_server::Snapshot;

use crate::{app::AppState, event::AppEvent, popup::PopupValue};

pub struct ConfigureWidget;

impl ConfigureWidget {
    pub fn get_help() -> &'static str {
        "\
        C-S: Save    T: Toggle line trailing \n\
        C-L: Load    L: Toggle line looping  \n\
        M: Change scene mode
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
                        state.events.send(AppEvent::ChangeScript);
                        state.events.send(AppEvent::Positive("Loaded scene !".to_owned()));
                    })
                ));
            } 
            KeyCode::Char('m') => {
                let modes = vec![
                    ExecutionMode::Free.to_string(), 
                    ExecutionMode::AtQuantum.to_string(), 
                    ExecutionMode::LongestLine.to_string()
                ];
                let mode = state.scene_image.mode.to_string();
                let index = modes.iter().position(|m| *m == mode).unwrap_or_default();
                state.events.send(AppEvent::Popup(
                    "Scene mode".to_owned(), 
                    "Execution mode of the scene".to_owned(), 
                    PopupValue::Choice(index, modes), 
                    Box::new(|state, x| {
                        let chosen = String::from(x);
                        let mode = ExecutionMode::from(chosen);
                        state.events.send(
                            AppEvent::SchedulerControl(SchedulerMessage::SetSceneMode(mode, ActionTiming::Immediate))
                        );
                        state.events.send(AppEvent::Positive(format!("Set scene mode to {mode}")));
                    })
                ));
            } 
            KeyCode::Char('l') => {
                let Some(line) = state.selected_line() else {
                    return;
                };
                let mut config = line.configuration();
                config.looping = !config.looping;
                let config = vec![(state.selected.0, config)];
                state.events.send(
                    AppEvent::SchedulerControl(SchedulerMessage::ConfigureLines(config, ActionTiming::Immediate))
                );
                state.events.send(AppEvent::Positive(format!("Toggled line looping")));
            } 
            KeyCode::Char('t') => {
                let Some(line) = state.selected_line() else {
                    return;
                };
                let mut config = line.configuration();
                config.trailing = !config.trailing;
                let config = vec![(state.selected.0, config)];
                state.events.send(
                    AppEvent::SchedulerControl(SchedulerMessage::ConfigureLines(config, ActionTiming::Immediate))
                );
                state.events.send(AppEvent::Positive(format!("Toggled line trailing")));
            } 
            _ => ()
        }
    }
}

impl StatefulWidget for ConfigureWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        use Constraint::*;
        let layout = Layout::vertical([Length(3), Length(3), Length(3), Length(3)]).flex(Flex::Center);
        let [load_area, mode_area, looping_area, trailing_area] = layout.areas(area.inner(Margin {
            horizontal: 3,
            vertical: 0
        }));
        
        let mode = state.scene_image.mode.to_string().light_green().bold();
        let mut looping = "No line".gray().bold();
        let mut trailing = looping.clone();
        
        if let Some(line) = state.selected_line() {
            looping = if line.looping {
                "Enabled".light_green().bold()
            } else {
                "Disabled".light_red().bold()
            };
            trailing = if line.trailing {
                "Enabled".light_green().bold()
            } else {
                "Disabled".light_red().bold()
            };
        }
        
        Paragraph::new("C-S/C-L to Save/Load scene".bold()).centered().render(load_area, buf);
        Paragraph::new(text::Line::from(vec![Span::from("(Scene) Mode : "), mode]))
            .render(mode_area, buf);
        Paragraph::new(text::Line::from(vec![Span::from("(Line) Looping : "), looping]))
            .render(looping_area, buf);
        Paragraph::new(text::Line::from(vec![Span::from("(Line) Trailing : "), trailing]))
            .render(trailing_area, buf);
    }
}