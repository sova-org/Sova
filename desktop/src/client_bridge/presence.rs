use std::collections::VecDeque;

use sova_server::{ClientMessage, FrameTextId};

use super::{ChatMessage, ClientBridge, MAX_CHAT_MESSAGES, now_hhmm};

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

    /// Decoded `(peer_name, line, col)` triples for every peer whose ephemeral
    /// cursor presence anchors into the given `(li, fi)` frame's Loro doc.
    pub fn text_cursors_for_frame(&self, li: usize, fi: usize) -> Vec<(String, usize, usize)> {
        let Some(target_id) = self.frame_text_id_at(li, fi) else {
            return Vec::new();
        };
        let Some(doc) = self.frame_doc(target_id) else {
            return Vec::new();
        };
        // Resolve our own username up front; treat None as the empty string so
        // a brief None state can't leak our own cursor through the filter (we
        // never publish under "" so it can't match anyone real either).
        let me = self.confirmed_username().unwrap_or("");
        let states = self.presence.get_all_states();

        let mut out = Vec::new();
        for (key, value) in &states {
            let Some(rest) = key.strip_prefix("peer/") else {
                continue;
            };
            let Some((name, suffix)) = rest.split_once('/') else {
                continue;
            };
            if suffix != "cursor_frame" {
                continue;
            }
            if name == me {
                continue;
            }
            let frame_id = match value {
                loro::LoroValue::I64(v) => FrameTextId(*v as u64),
                _ => continue,
            };
            if frame_id != target_id {
                continue;
            }
            let pos_key = format!("peer/{}/cursor_pos", name);
            let Some(loro::LoroValue::Binary(bytes)) = states.get(&pos_key) else {
                continue;
            };
            let Ok(cursor) = loro::cursor::Cursor::decode(bytes.as_ref()) else {
                continue;
            };
            let Ok(pos_query) = doc.get_cursor_pos(&cursor) else {
                continue;
            };
            let codepoint = pos_query.current.pos;
            let live = doc
                .get_text(sova_server::FrameTextStore::CONTENT_CONTAINER)
                .to_string();
            let (line, col) = codepoint_to_line_col(&live, codepoint);
            out.push((name.to_owned(), line, col));
        }
        out
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

fn codepoint_to_line_col(s: &str, codepoint: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    for (i, ch) in s.chars().enumerate() {
        if i >= codepoint {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}
