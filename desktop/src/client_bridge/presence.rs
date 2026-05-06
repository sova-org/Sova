use std::collections::VecDeque;

use crossbeam_channel::{Sender, unbounded};
use sova_core::HostMessage;
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

        // First pass: collect (name, codepoint) pairs anchored to this frame.
        let mut candidates: Vec<(String, usize)> = Vec::new();
        for (key, value) in &states {
            let Some(rest) = key.strip_prefix("peer/") else {
                continue;
            };
            let Some((name, suffix)) = rest.split_once('/') else {
                continue;
            };
            if suffix != "cursor_frame" || name == me {
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
            candidates.push((name.to_owned(), pos_query.current.pos));
        }

        if candidates.is_empty() {
            return Vec::new();
        }

        // Hoist the live text fetch out of the per-peer loop and build a
        // codepoint index of newlines once. Each peer's (line, col) is then a
        // partition_point binary search instead of a full string rescan.
        let live = doc
            .get_text(sova_server::FrameTextStore::CONTENT_CONTAINER)
            .to_string();
        let newlines: Vec<usize> = live
            .chars()
            .enumerate()
            .filter_map(|(i, ch)| (ch == '\n').then_some(i))
            .collect();

        candidates
            .into_iter()
            .map(|(name, codepoint)| {
                let line = newlines.partition_point(|&n| n < codepoint);
                let col = if line == 0 {
                    codepoint
                } else {
                    codepoint - (newlines[line - 1] + 1)
                };
                (name, line, col)
            })
            .collect()
    }

    /// Single point of cleanup when a peer disconnects. Future presence-related
    /// per-peer state (selections, follow targets, ...) gets one place to add to.
    pub(super) fn forget_peer(&mut self, name: &str) {
        self.peer_editing.retain(|_, names| {
            names.retain(|n| n != name);
            !names.is_empty()
        });
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

    /// Creates a fresh host-message channel and stores the receiver. Returns
    /// the sender so the caller can register it on a [DeviceMap]. Replaces
    /// any previously installed receiver — the previous channel's sender
    /// will start erroring on the next send, which is the correct shutdown
    /// signal for an outgoing embedded-server / feedback-engine instance.
    pub fn install_host_channel(&mut self) -> Sender<HostMessage> {
        let (tx, rx) = unbounded();
        self.host_rx = Some(rx);
        tx
    }

    /// Drains any host messages emitted by the in-process scheduler. Returns
    /// an empty `Vec` when no host channel is installed (remote-only mode).
    pub fn drain_host_messages(&mut self) -> Vec<HostMessage> {
        let Some(rx) = &self.host_rx else {
            return Vec::new();
        };
        rx.try_iter().collect()
    }
}

