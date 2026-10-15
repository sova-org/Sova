use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sova_core::schedule::{ActionTiming, SchedulerMessage, playback::PlaybackState};
use sova_server::SovaClient;

use crate::{app::{App, AppPage, scene_view::SceneView}, compiled_syntax::start_syntax_compilation, event::AppEvent, network::{client::run_connection_task, server::start_server}, widgets::{Help, log_widget::LogWidget, popup::PopupValue}};

impl App {
    pub async fn handle_app_event(&mut self, app_event: AppEvent) {
        match app_event {
            AppEvent::Quit => self.quit().await,
            AppEvent::GoTo(app_page) => {
                match app_page {
                    AppPage::Logs => self.floating.open("Logs", LogWidget::default()),
                    AppPage::Help => self.floating.open("Help", Help),
                    AppPage::Edit => {
                        self.editor.update_content(&self.state);
                        self.state.page = AppPage::Edit;
                    }
                    AppPage::Prelude => {
                        self.prelude_view.update_content(&self.state);
                        self.state.page = AppPage::Prelude;
                    }
                    _ => self.state.page = app_page
                }
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
                if self.state.is_server_running() {
                    self.notification.negative("Unable to start server: already running!", &self.state.palette);
                    return;
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
                start_syntax_compilation(&self.state.client_state.languages, self.state.events.get_sender());
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
                self.state.disconnect().await;
                self.notification.negative(err, &self.state.palette);
            }
            AppEvent::RefreshScript => {
                self.editor.update_content(&self.state);
            }
            AppEvent::UpdateSyntax(lang, syn) => {
                self.state.compiled_syntax.insert(lang, syn);
            }
            AppEvent::TogglePlayPause => {
                let msg = match self.state.client_state.playback_state {
                    PlaybackState::Stopped => SchedulerMessage::TransportStart(ActionTiming::Immediate),
                    PlaybackState::Starting(_) 
                    | PlaybackState::Playing => SchedulerMessage::TransportStop(ActionTiming::Immediate),
                };
                let _ = self.state.send_message(msg).await;
            }
            AppEvent::SendClientMessage(msg) => {
                let _ = self.state.send_message(msg).await;
            }
            AppEvent::ShowHelp => {
                self.floating.open("Help", Help);
            }
        }
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if self.popup.showing {
            self.popup.process_event(&mut self.state, key_event);
            return Ok(());
        }
        if self.floating.is_showing() {
            self.floating.process_event(&mut self.state, key_event);
            return Ok(());
        }

        match key_event.code {
            KeyCode::Char(' ') if 
                key_event.modifiers.contains(KeyModifiers::CONTROL) && 
                self.state.page.is_in_session() => 
            {
                self.state.events.send(AppEvent::TogglePlayPause);
            }
            KeyCode::Char('h') if 
                key_event.modifiers.contains(KeyModifiers::CONTROL) && 
                self.state.page.is_in_session() => 
            {
                self.state.events.send(AppEvent::ShowHelp);
            }
            KeyCode::Char('g') if 
                key_event.modifiers.contains(KeyModifiers::CONTROL) && 
                self.state.page.is_in_session() => 
            {
                let choices = vec![
                    AppPage::Scene.to_string(),
                    AppPage::Prelude.to_string(),
                    AppPage::Devices.to_string(),
                    AppPage::Audio.to_string(),
                    AppPage::Chat.to_string(),
                    AppPage::Help.to_string(),
                    AppPage::Logs.to_string(),
                ];
                self.state.popup("Go to", "Select the page to go to", PopupValue::Choice(0, choices), |state, value| {
                    let page = AppPage::from(String::from(value).as_str());
                    state.go_to(page);
                });
            }
            _ => {
                match self.state.page {
                    AppPage::Connection => {
                        self.connection_view.on_key_event(key_event, &mut self.state);
                    }
                    AppPage::Scene => {
                        SceneView::on_key_event(key_event, &mut self.state);
                    }
                    AppPage::Edit => {
                        self.editor.on_key_event(key_event, &mut self.state);
                    }
                    AppPage::Prelude => {
                        self.prelude_view.on_key_event(key_event, &mut self.state);
                    }
                    _ => ()
                }
            }
        }
        Ok(())
    }
}