use std::{io::ErrorKind, time::Duration};

use crate::{app::{connecting_view::ConnectingView, connection_view::ConnectionView, scene_view::SceneView}, event::{AppEvent, Event}, network::server::start_server, notification::Notification, popup::{Popup, PopupValue}};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{DefaultTerminal, widgets::Widget};

mod state;
use sova_core::Scene;
use sova_server::{SovaClient, SovaCoreServer};
pub use state::*;

mod page;
pub use page::*;
use tokio::{io, task::JoinHandle, time::timeout};

/// Application.
pub struct App {
    pub running: bool,
    pub state: AppState,
    pub popup: Popup,
    pub notification: Notification,
    
    pub connection_view: ConnectionView,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(state: AppState) -> Self {
        App { 
            running: true, 
            state, 
            popup: Popup::default(),
            notification: Notification::new(),

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
            Event::Tick => self.tick().await,
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
                    self.popup.open(title, content, value, callback, self.state.palette)
                }
                AppEvent::Info(text) => self.notification.info(text, self.state.palette),
                AppEvent::Positive(text) => self.notification.positive(text, self.state.palette),
                AppEvent::Negative(text) => self.notification.negative(text, self.state.palette),
                AppEvent::Connect(ip, port, username, pass) => {
                    self.state.page = AppPage::Connecting;
                    let mut client = SovaClient::new(ip, port);
                    client.name = username;
                    if !pass.is_empty() {
                        client.password = Some(pass)
                    } else {
                        client.password = None;
                    }
                    self.state.connection_task = Some(tokio::spawn(async move {
                        let res = client.connect_with_timeout(Duration::from_secs(5)).await;
                        (client, res)
                    }));
                }
                AppEvent::Server(port, username, pass) => {
                    self.state.page = AppPage::Scene;
                    let pass = if pass.is_empty() { None } else { Some(pass) };
                    let serv = start_server(port, pass.clone());
                    let mut client = SovaClient::new("127.0.0.1".to_string(), port);
                    client.name = username;
                    client.password = pass;
                    self.state.connection_task = Some(tokio::spawn(async move {
                        let res = client.connect_with_timeout(Duration::from_secs(5)).await;
                        (client, res)
                    }));
                }
                AppEvent::Connected => {
                    self.state.page = AppPage::Scene;
                }
                AppEvent::ConnectionFailed(msg) => {
                    self.notification.negative(msg, self.state.palette);
                    self.state.page = AppPage::Connection;
                }
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
            _ => {
                match self.state.page {
                    AppPage::Connection => {
                        self.connection_view.on_key_event(key_event, &mut self.state);
                    }
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
    pub async fn tick(&mut self) {
        match self.state.page {
            AppPage::Connecting => {
                ConnectingView::update_state(&mut self.state).await;
            }
            _ => ()
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
