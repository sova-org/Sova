use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent};
use sova_server::SovaClient;

use crate::{app::{App, AppPage, scene_view::SceneView}, event::AppEvent, network::{client::run_connection_task, server::start_server}, popup::PopupValue};

impl App {
    pub async fn handle_app_event(&mut self, app_event: AppEvent) {
        match app_event {
            AppEvent::Quit => self.quit().await,
            AppEvent::GoTo(app_page) => {
                self.state.page = app_page;
            }
            AppEvent::Popup(title, content, value, callback) => {
                self.popup.open(title, content, value, callback, &self.state.palette)
            }
            AppEvent::Info(text) => self.notification.info(text, &self.state.palette),
            AppEvent::Positive(text) => self.notification.positive(text, &self.state.palette),
            AppEvent::Negative(text) => self.notification.negative(text, &self.state.palette),
            AppEvent::Connect(ip, port, username, pass) => {
                self.state.page = AppPage::Connecting;
                let mut client = SovaClient::new(ip, port);
                client.name = username;
                if !pass.is_empty() {
                    client.password = Some(pass)
                } else {
                    client.password = None;
                }
                run_connection_task(client, Duration::from_secs(5), self.state.events.get_sender());
            }
            AppEvent::Server(port, username, pass) => {
                if self.state.server_task.is_some() {
                    self.notification.negative("Unable to start server: already running!", &self.state.palette);
                }
                let pass = if pass.is_empty() { None } else { Some(pass) };
                self.state.server_task = Some(start_server(port, pass.clone(), self.log_tx.clone()));
                self.state.page = AppPage::Connecting;
                let sender = self.state.events.get_sender();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    let mut client = SovaClient::new("127.0.0.1".to_string(), port);
                    client.name = username;
                    run_connection_task(client, Duration::from_secs(5), sender);
                });
            }
            AppEvent::Connected(client) => {
                // Lock should be immediate as no update task running
                self.state.client = Some(client);
                self.state.update_client_state().await;
                self.state.run_client_update();
                self.state.page = AppPage::Scene;
                self.notification.positive("Connected !", &self.state.palette);
            }
            AppEvent::ConnectionFailed(msg) => {
                self.notification.negative(msg, &self.state.palette);
                self.state.page = AppPage::Connection;
            }
            AppEvent::UpdateClientState(client_state) => {
                self.state.client_state = client_state;
            }
            AppEvent::Disconnected => {
                self.state.page = AppPage::Connection;
                let err = self.state.client_state.server_errors
                    .pop_back()
                    .unwrap_or("Disconnected !".to_string());
                if let Some(mut c) = self.state.client.take() {
                    let _ = c.stop_task().await;
                }
                self.notification.negative(err, &self.state.palette);
            }
            AppEvent::RefreshScript => (),
        }
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
}