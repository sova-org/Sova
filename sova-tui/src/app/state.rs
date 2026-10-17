use std::{collections::BTreeMap, io::ErrorKind, iter};

use arboard::Clipboard;
use sova_core::{scene::{Frame, Line, script::Script}, vm::interpreter::Annotation};
use sova_server::{ClientMessage, SovaClient, client::ClientState};
use tokio::{io, sync::mpsc};

use crate::{app::page::AppPage, compiled_syntax::CompiledSyntax, event::{AppEvent, EventHandler}, network::server::ServerTask, widgets::popup::PopupValue, theme::Palette};

const MAX_LOGS : usize = 60;

pub struct AppState {
    /// Duplicate of internal client state, 
    /// however this does not require any async code to access, nor unwraping options
    pub client_state: ClientState,
    pub events: EventHandler,
    pub selected: (usize, usize),
    pub selected_prelude: usize, 
    pub clipboard: Option<Clipboard>,
    pub page: AppPage,
    pub palette: Palette,
    pub client: Option<SovaClient>,
    pub server_task: Option<ServerTask>,
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

    pub async fn send_message<T : Into<ClientMessage>>(&mut self, msg: T) -> io::Result<()> {
        let Some(client) = &mut self.client else {
            return Err(io::Error::new(ErrorKind::NotConnected, "No connected client to send message !"));
        };
        client.send(msg.into()).await
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

    pub fn selected_prelude(&self) -> Option<&Script> {
        self.client_state.scene.prelude.get(self.selected_prelude)
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

    pub fn go_to_next_prelude(&mut self) {
        let scene = &self.client_state.scene;
        if scene.prelude.len() > self.selected_prelude + 1 {
            self.selected_prelude += 1;
        }
    }

    pub fn go_to_previous_prelude(&mut self) {
        let scene = &self.client_state.scene;
        self.selected_prelude = self.selected_prelude
            .saturating_sub(1)
            .min(scene.prelude.len().saturating_sub(1))
    }

    pub fn popup<T, D, V, F>(&mut self, title: T, desc: D, value: V, callback: F) 
        where T: ToString, D: ToString, V: Into<PopupValue>, F: FnOnce(&mut AppState, PopupValue) + 'static + Send
    {
        self.events.send(AppEvent::Popup(
            title.to_string(), desc.to_string(), value.into(), Box::new(callback)
        ));
    }

    pub fn positive<T: ToString>(&mut self, msg: T) {
        self.events.send(AppEvent::Positive(msg.to_string()));
    }

    pub fn negative<T: ToString>(&mut self, msg: T) {
        self.events.send(AppEvent::Negative(msg.to_string()));
    }

    pub fn current_annotations<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Annotation> + 'a> {
        let annots = self.client_state.annotations
            .get(self.selected.0)
            .and_then(|line| line.get(self.selected.1));
        match annots {
            Some(a) => Box::new(a.iter()),
            None => Box::new(iter::empty()),
        }
    }

    pub fn go_to(&mut self, page: AppPage) {
        self.events.send(AppEvent::GoTo(page));
    }

    /// Shut downs the client AND the server tasks
    pub async fn disconnect(&mut self) {
        if let Some(mut c) = self.client.take() {
            let _ = c.stop_task().await;
        }
        if let Some(task) = self.server_task.take() {
            let _ = task.stop().await;
        }
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
                    Some(msg) => {
                        let mut guard = client_state.lock().await;
                        while guard.logs.len() > MAX_LOGS {
                            let delta = guard.logs.len() - MAX_LOGS;
                            let _ = guard.logs.drain(0..delta);
                        }
                        let new_state = ClientState::clone(&guard);
                        sender.send(AppEvent::UpdateClientState(new_state));
                        sender.send(msg);
                    },
                    None => break,
                }
            }
            sender.send(AppEvent::Disconnected);
        });
    }
}