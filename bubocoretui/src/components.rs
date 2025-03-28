use crate::{App, event::AppEvent};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::Rect;
use std::error::Error;

pub mod editor;
pub mod grid;
pub mod help;
pub mod options;
pub mod splash;

pub trait Component {
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> Result<bool, Box<dyn Error>>;
    fn draw(&self, app: &App, frame: &mut ratatui::Frame, area: Rect);
}

pub fn handle_common_keys(
    app: &mut App,
    key_event: KeyEvent,
) -> Result<bool, Box<dyn Error + 'static>> {
    match key_event.code {
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.events.send(AppEvent::Quit);
            Ok(true)
        }
        KeyCode::F(1) => {
            app.events.send(AppEvent::SwitchToEditor);
            Ok(true)
        }
        KeyCode::F(2) => {
            app.events.send(AppEvent::SwitchToGrid);
            Ok(true)
        }
        KeyCode::F(3) => {
            app.events.send(AppEvent::SwitchToOptions);
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn inner_area(area: Rect) -> Rect {
    let inner = area;
    Rect {
        x: inner.x + 1,
        y: inner.y + 1,
        width: inner.width.saturating_sub(2),
        height: inner.height.saturating_sub(2),
    }
}
