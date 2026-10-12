use std::collections::{BTreeMap, VecDeque};

use arboard::Clipboard;
use sova_core::{LogMessage, scene::{Frame, Line}, vm::interpreter::Annotation};
use sova_server::{SovaClient, client::ClientState};
use tokio::sync::mpsc;

use crate::{app::page::AppPage, compiled_syntax::CompiledSyntax, event::{AppEvent, EventHandler}, network::server::ServerTask, theme::Palette};

pub struct AppState {
    pub client_state: ClientState,
    pub events: EventHandler,
    pub selected: (usize, usize),
    pub clipboard: Option<Clipboard>,
    pub page: AppPage,
    pub palette: Palette,
    pub client: Option<SovaClient>,
    pub server_task: Option<ServerTask>,
    pub logs: VecDeque<LogMessage>,
    pub compiled_syntax: BTreeMap<String, CompiledSyntax>
}

impl AppState {
    pub async fn update_client_state(&mut self) {
        match &self.client {
            Some(client) => {
                let guard = client.state().await;
                self.client_state = ClientState::clone(&guard);
            }
            None => {
                self.events.send(AppEvent::Negative("Unable to update client state !".to_string()));
            }
        }
    }

    pub async fn is_connected(&self) -> bool {
        match &self.client {
            Some(client) => client.state().await.connected,
            None => false
        }
    }

    pub fn is_server_running(&self) -> bool {
        match &self.server_task {
            Some(task) => task.is_running(),
            None => false,
        }
    }

    pub fn selected_line(&self) -> Option<&Line> {
        self.client_state.scene.line(self.selected.0)
    }

    pub fn selected_frame(&self) -> Option<&Frame> {
        self.client_state.scene.frame(self.selected.0, self.selected.1)
    }

    pub fn go_to_next_frame(&mut self) {
        let scene = &self.client_state.scene;
        if scene.has_frame(self.selected.0, self.selected.1 + 1) {
            self.selected.1 += 1;
            self.events.send(AppEvent::RefreshScript);
        } else if scene.has_frame(self.selected.0 + 1, 0) {
            self.selected = (self.selected.0 + 1, 0);
            self.events.send(AppEvent::RefreshScript);
        }
    }

    pub fn go_to_previous_frame(&mut self) {
        let scene = &self.client_state.scene;
        if self.selected.1 > 0 {
            self.selected.1 -= 1;
            self.events.send(AppEvent::RefreshScript);
        } else {
            let mut i = self.selected.0;
            while i > 0 {
                i -= 1;
                if scene.line(i).map(|l| !l.is_empty()).unwrap_or_default() {
                    let j = scene.line(i).unwrap().n_frames() - 1;
                    self.selected = (i, j);
                    self.events.send(AppEvent::RefreshScript);
                    break;
                }
            }
            
        }
    }

    pub fn current_annotations(&self) -> Vec<Annotation> {
        self.client_state.annotations
            .get(self.selected.0)
            .and_then(|line| line.get(self.selected.1))
            .cloned()
            .unwrap_or_default()
    }

    pub fn go_to(&mut self, page: AppPage) {
        self.events.send(AppEvent::GoTo(page));
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
            sender.send(AppEvent::Disconnected);
        });
    }
}