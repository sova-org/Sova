use crate::{app::scene_view::SceneView, event::{AppEvent, Event}, notification::Notification, popup::{Popup, PopupValue}};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;

mod state;
pub use state::*;

mod page;
pub use page::*;

/// Application.
pub struct App {
    pub running: bool,
    pub state: AppState,
    pub popup: Popup,
    pub notification: Notification
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(state: AppState) -> Self {
        App { 
            running: true, 
            state, 
            popup: Popup::default(),
            notification: Notification::new()
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.state.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event)
                    if key_event.kind == crossterm::event::KeyEventKind::Press =>
                {
                    self.handle_key_event(key_event)?
                }
                _ => {}
            },
            Event::App(app_event) => match app_event {
                AppEvent::Quit => self.quit(),
                AppEvent::Popup(title, content, value, callback) => {
                    self.popup.open(title, content, value, callback)
                }
                AppEvent::Info(text) => self.notification.info(text),
                AppEvent::Positive(text) => self.notification.positive(text),
                AppEvent::Negative(text) => self.notification.negative(text),
                _ => ()
            },
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if self.popup.showing {
            self.popup.process_event(&mut self.state, key_event);
            return Ok(());
        }

        match key_event.code {
            KeyCode::Esc => {
                self.state.events.send(AppEvent::Popup(
                    "Exit Sova ?".to_owned(),
                    "Are you sure you want to quit ?".to_owned(),
                    PopupValue::Bool(false),
                    Box::new(|state, x| {
                        if bool::from(x) {
                            state.events.send(AppEvent::Quit)
                        }
                    }),
                ));
            }
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.state.events.send(AppEvent::Quit)
            }
            _ => {
                match self.state.page {
                    AppPage::Scene => {
                        SceneView::on_key_event(key_event, &mut self.state);
                    }
                    _ => ()
                }
            }
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
