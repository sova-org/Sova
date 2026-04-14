use std::collections::{HashMap, VecDeque};

use sova_server::ClientMessage;

use super::{ChatMessage, ClientBridge, PeerCursorState, now_hhmm, MAX_CHAT_MESSAGES};

impl ClientBridge {
    pub fn peers(&self) -> &[String] {
        &self.peers
    }

    pub fn confirmed_username(&self) -> Option<&str> {
        self.confirmed_username.as_deref()
    }

    pub fn set_confirmed_username(&mut self, name: String) {
        self.confirmed_username = Some(name);
    }

    pub fn peer_cursors(&self) -> &HashMap<String, PeerCursorState> {
        &self.peer_cursors
    }

    pub fn text_cursors_for_frame(&self, li: usize, fi: usize) -> Vec<(&str, usize, usize)> {
        self.peer_cursors
            .iter()
            .filter_map(|(name, &(pli, pfi, ref tc))| {
                if pli == li && pfi == fi {
                    tc.map(|(line, col)| (name.as_str(), line, col))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn editing_peers_for_frame(&self, li: usize, fi: usize) -> &[String] {
        self.peer_editing
            .get(&(li, fi))
            .map_or(&[], |names| names.as_slice())
    }

    pub fn chat_messages(&self) -> &VecDeque<ChatMessage> {
        &self.chat_messages
    }

    pub fn push_chat(&mut self, user: String, message: String) {
        self.chat_messages.push_back(ChatMessage {
            time: now_hhmm(),
            user,
            message,
            system: false,
        });
        self.cap_chat();
    }

    pub fn send_chat(&self, msg: &str) {
        self.send(ClientMessage::Chat(msg.to_owned()));
    }

    pub(super) fn cap_chat(&mut self) {
        while self.chat_messages.len() > MAX_CHAT_MESSAGES {
            self.chat_messages.pop_front();
        }
    }

    pub fn take_remote_hydra(&mut self) -> Option<(String, String)> {
        self.remote_hydra.take()
    }

    pub fn send_hydra_code(&self, code: &str) {
        self.send(ClientMessage::HydraCode(code.to_owned()));
    }
}
