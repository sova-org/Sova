use std::collections::VecDeque;

use arboard::Clipboard;
use sova_core::{LogMessage, scene::{Frame, Line}};
use sova_server::{SovaClient, client::ClientState};
use tokio::sync::mpsc;

use crate::{app::page::AppPage, event::{AppEvent, EventHandler}, network::server::ServerTask, theme::Palette};

pub struct AppState {
    pub client_state: ClientState,
    pub events: EventHandler,
    pub selected: (usize, usize),
    pub clipboard: Option<Clipboard>,
    pub page: AppPage,
    pub palette: Palette,
    pub client: Option<SovaClient>,
    pub server_task: Option<ServerTask>,
    pub logs: VecDeque<LogMessage>
}

impl AppState {
    pub async fn update_client_state(&mut self) {
        match &self.client {
            Some(client) => {
                let guard = client.state().await;
                self.client_state = ClientState::clone(&guard);
            }
            None => (),
        }
    }

    pub async fn is_connected(&self) -> bool {
        match &self.client {
            Some(client) => client.state().await.connected,
            None => false
        }
    }

    pub fn selected_line(&self) -> Option<&Line> {
        self.client_state.scene.line(self.selected.0)
    }

    pub fn selected_frame(&self) -> Option<&Frame> {
        self.client_state.scene.frame(self.selected.0, self.selected.1)
    }

    pub fn run_client_update(&mut self) {
        let mut sender = self.events.get_sender();
        let Some(client) = &mut self.client else {
            sender.send(AppEvent::Negative("Unable to run update client task: no client !".to_string()));
            return;
        };
        let (tx, mut rx) = mpsc::unbounded_channel();
        let _ = client.run(Some(tx));
        let client_state = client.client_state.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Some(_) => {
                        let guard = client_state.lock().await;
                        let new_state = ClientState::clone(&guard);
                        sender.send(AppEvent::UpdateClientState(new_state));
                    },
                    None => break,
                }
            }
        });
    }
}