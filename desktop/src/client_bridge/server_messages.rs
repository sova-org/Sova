use std::ops::ControlFlow;
use std::time::Instant;

use sova_core::compiler::CompilationState;
use sova_core::schedule::SovaNotification;
use sova_core::schedule::{ActionTiming, SchedulerMessage, playback::PlaybackState};
use sova_server::ServerMessage;

use crate::panels::log_panel::{LogEntry, LogSource};
use crate::widgets::syntax_highlight::CompiledSyntax;

use super::{
    BridgeEvent, ClientBridge, COMPILATION_FLASH_SECS, MUTATION_FLASH_SECS, SCENE_HISTORY_CAP,
    now_hhmm, scope_signature,
};

impl ClientBridge {
    pub fn poll(&mut self) {
        self.compilation_flashes
            .retain(|_, (_, t)| t.elapsed().as_secs_f32() < COMPILATION_FLASH_SECS);
        self.mutation_flashes
            .retain(|_, t| t.elapsed().as_secs_f32() < MUTATION_FLASH_SECS);

        let events: Vec<BridgeEvent> = {
            let Some(rx) = &self.event_rx else { return };
            std::iter::from_fn(|| rx.try_recv().ok()).collect()
        };

        for evt in events {
            match evt {
                BridgeEvent::LocalDisconnected => {
                    self.clear_state();
                    return;
                }
                BridgeEvent::Server(msg) => {
                    if self.handle_server_message(*msg).is_break() {
                        return;
                    }
                }
            }
        }
        self.cap_chat();

        if self.scene_dirty {
            self.scene_dirty = false;
            if self.skip_next_history_push {
                self.skip_next_history_push = false;
            } else if let Some(scene) = &self.scene {
                self.scene_history.truncate(self.history_index + 1);
                self.scene_history.push_back(scene.clone());
                if self.scene_history.len() > SCENE_HISTORY_CAP {
                    self.scene_history.pop_front();
                }
                self.history_index = self.scene_history.len() - 1;
            }
        }

        if let Some(engine) = &self.feedback_engine {
            self.devices = engine.devices().device_list();
            self.audio_state = engine.audio_state();
            engine.fill_peak_data(&mut self.peak_data);

            let prev_sig = scope_signature(&self.scope_data);
            engine.fill_scope_data(&mut self.scope_data);
            if scope_signature(&self.scope_data) != prev_sig {
                self.scope_generation += 1;
            }
        }
    }

    fn handle_server_message(&mut self, msg: ServerMessage) -> ControlFlow<()> {
        match msg {
            ServerMessage::Hello {
                username,
                scene,
                devices,
                peers,
                link_state,
                is_playing,
                languages,
                audio_engine_state,
                link_enabled,
            } => {
                self.confirmed_username = Some(username);
                self.scene = Some(scene);
                self.devices = devices;
                self.peers = peers;
                self.languages = languages;
                for lang_def in &self.languages {
                    if let Some(syn) = &lang_def.syntax
                        && let Some(compiled) = CompiledSyntax::new(syn)
                    {
                        self.syntax_map.insert(lang_def.name.to_owned(), compiled);
                    }
                }
                self.clock = super::ClockState {
                    tempo: link_state.0,
                    beat: link_state.1,
                    phase: 0.0,
                    quantum: link_state.2,
                    playing: is_playing,
                    num_peers: link_state.3,
                    start_stop_sync: link_state.4,
                    link_enabled,
                };
                self.audio_state = audio_engine_state;
                self.status = super::ConnectionStatus::Connected;
                self.just_connected = true;
            }
            ServerMessage::ConnectionRefused(reason) => {
                self.clear_state();
                if !reason.is_empty() {
                    self.status = super::ConnectionStatus::Error;
                    self.error_msg = Some(reason);
                }
                return ControlFlow::Break(());
            }
            ServerMessage::Notification(SovaNotification::UpdatedScene(s)) => {
                self.scene = Some(s);
                self.scene_dirty = true;
                self.errors.clear();
                self.annotations.clear();
                self.compilation_flashes.clear();
                self.mutation_flashes.clear();
            }
            ServerMessage::Notification(SovaNotification::AddedLine(idx, line)) => {
                if let Some(scene) = &mut self.scene
                    && idx <= scene.lines.len()
                {
                    let now = Instant::now();
                    for fi in 0..line.frames.len() {
                        self.mutation_flashes.insert((idx, fi), now);
                    }
                    scene.lines.insert(idx, line);
                    self.scene_dirty = true;
                }
            }
            ServerMessage::Notification(SovaNotification::RemovedLine(idx)) => {
                if let Some(scene) = &mut self.scene
                    && idx < scene.lines.len()
                {
                    scene.lines.remove(idx);
                    self.scene_dirty = true;
                }
                self.mutation_flashes.retain(|&(li, _), _| li != idx);
            }
            ServerMessage::Notification(SovaNotification::AddedFrame(li, fi, frame)) => {
                if let Some(line) = self.scene.as_mut().and_then(|s| s.lines.get_mut(li))
                    && fi <= line.frames.len()
                {
                    line.frames.insert(fi, frame);
                    self.mutation_flashes.insert((li, fi), Instant::now());
                    self.scene_dirty = true;
                }
            }
            ServerMessage::Notification(SovaNotification::RemovedFrame(li, fi)) => {
                if let Some(line) = self.scene.as_mut().and_then(|s| s.lines.get_mut(li))
                    && fi < line.frames.len()
                {
                    line.frames.remove(fi);
                    self.scene_dirty = true;
                }
                self.errors.remove(&(li, fi));
                self.mutation_flashes.insert((li, fi), Instant::now());
            }
            ServerMessage::Notification(SovaNotification::UpdatedFrames(items)) => {
                if let Some(scene) = &mut self.scene {
                    let now = Instant::now();
                    for (li, fi, frame) in items {
                        if let Some(f) = scene.lines.get_mut(li).and_then(|l| l.frames.get_mut(fi))
                        {
                            *f = frame;
                            self.mutation_flashes.insert((li, fi), now);
                        }
                    }
                    self.scene_dirty = true;
                }
            }
            ServerMessage::Notification(SovaNotification::UpdatedLines(items))
            | ServerMessage::Notification(SovaNotification::UpdatedLineConfigurations(items)) => {
                if let Some(scene) = &mut self.scene {
                    let now = Instant::now();
                    for (li, line) in items {
                        for fi in 0..line.frames.len() {
                            self.mutation_flashes.insert((li, fi), now);
                        }
                        if let Some(l) = scene.lines.get_mut(li) {
                            *l = line;
                        }
                    }
                    self.scene_dirty = true;
                }
            }
            ServerMessage::Notification(SovaNotification::UpdatedSceneMode(mode)) => {
                if let Some(scene) = &mut self.scene {
                    scene.mode = mode;
                }
            }
            ServerMessage::Notification(SovaNotification::UpdatedScenePrelude(scripts)) => {
                if let Some(scene) = &mut self.scene {
                    scene.prelude = scripts;
                }
            }
            ServerMessage::Notification(SovaNotification::FramePositionChanged(p)) => {
                let beat = self.clock.beat;
                self.position_start_beat.resize_with(p.len(), Vec::new);
                for (li, new_heads) in p.iter().enumerate() {
                    let old_heads = self.positions.get(li);
                    let starts = &mut self.position_start_beat[li];
                    starts.resize(new_heads.len(), beat);
                    for (hi, new_pos) in new_heads.iter().enumerate() {
                        if old_heads.and_then(|old| old.get(hi)) != Some(new_pos) {
                            starts[hi] = beat;
                        }
                    }
                }
                self.positions = p;
            }
            ServerMessage::ClockState(tempo, beat, _micros, quantum) => {
                self.clock.tempo = tempo;
                self.clock.beat = beat;
                self.clock.phase = if quantum > 0.0 { beat % quantum } else { 0.0 };
                self.clock.quantum = quantum;
            }
            ServerMessage::Notification(SovaNotification::PlaybackStateChanged(state)) => {
                self.clock.playing = !matches!(state, PlaybackState::Stopped);
                if !self.clock.playing {
                    self.positions.clear();
                    self.position_start_beat.clear();
                }
            }
            ServerMessage::Notification(SovaNotification::DeviceListChanged(devices)) => {
                self.devices = devices;
            }
            ServerMessage::AudioEngineState(state) => {
                self.audio_state = state;
            }
            ServerMessage::ScopeData(data) => {
                self.scope_data = data;
                self.scope_generation += 1;
                self.audio_state.running = true;
            }
            ServerMessage::PeakData(data) => {
                self.peak_data = data;
                self.audio_state.running = true;
            }
            ServerMessage::PeersUpdated(new_peers) => {
                let time = now_hhmm();
                for p in &new_peers {
                    if !self.peers.contains(p) {
                        self.chat_messages.push_back(super::ChatMessage {
                            time: time.clone(),
                            user: String::new(),
                            message: t!("chat.peer_joined", name = p).to_string(),
                            system: true,
                        });
                    }
                }
                for p in &self.peers {
                    if !new_peers.contains(p) {
                        self.chat_messages.push_back(super::ChatMessage {
                            time: time.clone(),
                            user: String::new(),
                            message: t!("chat.peer_left", name = p).to_string(),
                            system: true,
                        });
                        self.peer_editing.retain(|_, names| {
                            names.retain(|n| n != p);
                            !names.is_empty()
                        });
                        self.peer_cursors.remove(p);
                    }
                }
                self.peers = new_peers;
            }
            ServerMessage::Notification(SovaNotification::CompilationUpdated(
                li,
                fi,
                _id,
                state,
            )) => {
                self.errors.remove(&(li, fi));
                match &state {
                    CompilationState::Compiled(_) | CompilationState::Parsed(_) => {
                        self.compilation_flashes
                            .insert((li, fi), (true, Instant::now()));
                        self.last_error = None;
                    }
                    CompilationState::Error(e) => {
                        self.compilation_flashes
                            .insert((li, fi), (false, Instant::now()));
                        self.last_error =
                            Some((format!("L{}:F{} — {}", li, fi, e.info), Instant::now()));
                    }
                    _ => {}
                }
                if let Some(scene) = &mut self.scene {
                    *scene.frame_mut(li, fi).compilation_state_mut() = state;
                }
            }
            ServerMessage::Notification(SovaNotification::Log(msg)) => {
                let _ = self.log_tx.send(LogEntry {
                    source: LogSource::Client,
                    message: msg,
                });
            }
            ServerMessage::Chat(user, message) => {
                self.chat_messages.push_back(super::ChatMessage {
                    time: now_hhmm(),
                    user,
                    message,
                    system: false,
                });
            }
            ServerMessage::PeerStartedEditing(name, li, fi) => {
                self.peer_editing.entry((li, fi)).or_default().push(name);
            }
            ServerMessage::PeerStoppedEditing(name, li, fi) => {
                if let Some(names) = self.peer_editing.get_mut(&(li, fi)) {
                    names.retain(|n| n != &name);
                    if names.is_empty() {
                        self.peer_editing.remove(&(li, fi));
                    }
                }
            }
            ServerMessage::PeerCursorMoved(name, li, fi, tc) => {
                self.peer_cursors.insert(name, (li, fi, tc));
            }
            ServerMessage::Notification(SovaNotification::Annotations(a)) => {
                self.annotations = a;
            }
            ServerMessage::Notification(SovaNotification::Error(e)) => {
                self.last_error = Some((
                    format!("L{}:F{} — {}", e.line, e.frame, e.text),
                    Instant::now(),
                ));
                self.errors.insert((e.line, e.frame), e);
            }
            ServerMessage::FeedbackEnabled {
                scene,
                tempo,
                quantum,
                is_playing,
            } => {
                if let Some(engine) = &self.feedback_engine {
                    engine.send(SchedulerMessage::SetScene(scene, ActionTiming::Immediate));
                    engine.send(SchedulerMessage::SetTempo(tempo, ActionTiming::Immediate));
                    engine.send(SchedulerMessage::SetQuantum(
                        quantum,
                        ActionTiming::Immediate,
                    ));
                    if is_playing {
                        engine.send(SchedulerMessage::TransportStart(ActionTiming::Immediate));
                    }
                }
            }
            ServerMessage::Feedback(msg) => {
                if let Some(engine) = &self.feedback_engine {
                    engine.send(msg);
                }
            }
            ServerMessage::HydraCode(sender, code) => {
                if self.confirmed_username.as_deref() != Some(&sender) {
                    self.remote_hydra = Some((sender, code));
                }
            }
            ServerMessage::ScriptEdit {
                sender,
                li,
                fi,
                ops,
            } => {
                if self.confirmed_username.as_deref() != Some(&sender) {
                    self.pending_script_edits.push((li, fi, ops));
                }
            }
            ServerMessage::CoreRestarted => {
                self.errors.clear();
                self.annotations.clear();
                self.compilation_flashes.clear();
                self.mutation_flashes.clear();
                self.positions.clear();
                self.position_start_beat.clear();
            }
            ServerMessage::LinkState {
                enabled,
                start_stop_sync,
                num_peers,
            } => {
                self.clock.link_enabled = enabled;
                self.clock.start_stop_sync = start_stop_sync;
                self.clock.num_peers = num_peers;
            }
            ServerMessage::Snapshot(snapshot) => {
                self.scene = Some(snapshot.scene);
                self.clock.tempo = snapshot.tempo;
                self.clock.beat = snapshot.beat;
                self.clock.quantum = snapshot.quantum;
                self.devices = snapshot.devices;
                self.errors.clear();
                self.annotations.clear();
                self.compilation_flashes.clear();
                self.mutation_flashes.clear();
                self.positions.clear();
                self.position_start_beat.clear();
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}
