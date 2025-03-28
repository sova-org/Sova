use crate::App;
use crate::components::Component;
use crate::event::AppEvent;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, prelude::Rect, style::Style};
use std::error::Error;
use tui_big_text::{BigText, PixelSize};

pub struct SplashComponent;

impl SplashComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for SplashComponent {
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> Result<bool, Box<dyn Error + 'static>> {
        match key_event.code {
            KeyCode::Enter => {
                app.status_message = String::from("Ctrl+P for prompt");
                app.events.send(AppEvent::SwitchToEditor);
                Ok(true)
            }
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.events.send(AppEvent::Quit);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn draw(&self, _app: &App, frame: &mut Frame, _area: Rect) {
        let big_text = BigText::builder()
            .centered()
            .pixel_size(PixelSize::Full)
            .style(Style::new())
            .lines(vec!["".into(), "BuboCore".into()])
            .build();
        frame.render_widget(big_text, frame.area());
    }
}
