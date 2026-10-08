use crate::{app::connection_view::ConnectionView, event::Event, notification::Notification, popup::Popup};

use ratatui::DefaultTerminal;

use sova_core::schedule::SovaNotification;
pub use state::*;

mod event_processing;
mod state;
mod page;
pub use page::*;
use tokio::sync::broadcast;

/// Application.
pub struct App {
    pub running: bool,
    pub state: AppState,
    pub popup: Popup,
    pub notification: Notification,

    pub log_tx: broadcast::Sender<SovaNotification>,
    pub log_rx: broadcast::Receiver<SovaNotification>,
    
    pub connection_view: ConnectionView,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(state: AppState) -> Self {
        let (log_tx, log_rx) = broadcast::channel(256);
        App { 
            running: true, 
            state, 
            popup: Popup::default(),
            notification: Notification::new(),
            log_tx,
            log_rx,

            connection_view: ConnectionView::new(),
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    pub async fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.state.events.next().await? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event)
                    if key_event.kind == crossterm::event::KeyEventKind::Press =>
                {
                    self.handle_key_event(key_event)?
                }
                _ => {}
            },
            Event::App(app_event) => self.handle_app_event(app_event).await,
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) { }

    /// Set running to false to quit the application.
    pub async fn quit(&mut self) {
        self.running = false;
        if let Some(task) = self.state.server_task.take() {
            let _ = task.stop().await;
        }
        if let Some(mut client) = self.state.client.take() {
            let _ = client.stop_task().await;
        }
    }
}
