use std::collections::HashMap;
use std::time::Instant;

use sova_core::device_map::DeviceMap;
use sova_core::protocol::DeviceInfo;
use sova_core::schedule::ActionTiming;
use sova_core::vm::interpreter::Annotation;
use sova_core::{compiler::CompilationState, vm::language::LanguageDefinition};
use sova_server::{AudioEngineState, AudioRestartConfig, ClientMessage, Snapshot};

use crate::feedback_engine::FeedbackEngine;
use crate::panels::log_panel::{LogEntry, LogSource};

use super::{ClientBridge, ClockState, ConnectionStatus};

impl ClientBridge {
    pub(super) fn clear_state(&mut self) {
        *self = Self::new(self.runtime.clone(), self.ctx.clone(), self.log_tx.clone());
    }

    pub fn status(&self) -> ConnectionStatus {
        self.status
    }

    pub fn error_msg(&self) -> Option<&str> {
        self.error_msg.as_deref()
    }

    pub fn is_connected(&self) -> bool {
        self.status == ConnectionStatus::Connected
    }

    pub fn scene(&self) -> Option<&sova_core::scene::Scene> {
        self.scene.as_ref()
    }

    pub fn positions(&self) -> &[Vec<(usize, usize)>] {
        &self.positions
    }

    pub fn position_start_beat(&self) -> &[Vec<f64>] {
        &self.position_start_beat
    }

    pub fn devices(&self) -> &[DeviceInfo] {
        &self.devices
    }

    pub fn clock(&self) -> &ClockState {
        &self.clock
    }

    pub fn audio_state(&self) -> &AudioEngineState {
        &self.audio_state
    }

    pub fn scope_data(&self) -> &[f32] {
        &self.scope_data
    }

    pub fn scope_generation(&self) -> u64 {
        self.scope_generation
    }

    pub fn peak_data(&self) -> &[f32] {
        &self.peak_data
    }

    pub fn start_feedback(&mut self, audio_config: AudioRestartConfig) {
        let host_tx = self.install_host_channel();
        match FeedbackEngine::start(audio_config, host_tx) {
            Ok(engine) => self.feedback_engine = Some(engine),
            Err(e) => {
                let _ = self.log_tx.send(LogEntry {
                    source: LogSource::Client,
                    message: sova_core::protocol::log::LogMessage::error(format!(
                        "Failed to start feedback engine: {e}"
                    )),
                });
            }
        }
    }

    pub fn has_feedback(&self) -> bool {
        self.feedback_engine.is_some()
    }

    pub fn restart_audio(&self, config: AudioRestartConfig) {
        if let Some(engine) = &self.feedback_engine {
            engine.restart_audio(config);
        } else {
            self.send(ClientMessage::RestartAudioEngine(config));
        }
    }

    pub fn languages(&self) -> &[LanguageDefinition] {
        &self.languages
    }

    pub fn compilation_flashes(&self) -> &HashMap<(usize, usize), (bool, Instant)> {
        &self.compilation_flashes
    }

    pub fn mutation_flashes(&self) -> &HashMap<(usize, usize), Instant> {
        &self.mutation_flashes
    }

    pub fn frame_annotations(&self, li: usize, fi: usize) -> &[Annotation] {
        self.annotations
            .get(li)
            .and_then(|l| l.get(fi))
            .map_or(&[], |v| v.as_slice())
    }

    pub fn compilation_state(&self, li: usize, fi: usize) -> Option<&CompilationState> {
        self.scene()
            .and_then(|s| s.frame(li, fi))
            .map(|f| f.script().compilation_state())
    }

    pub fn undo(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            let scene = self.scene_history[self.history_index].clone();
            self.skip_next_history_push = true;
            self.send(sova_core::schedule::SchedulerMessage::SetScene(
                scene,
                ActionTiming::Immediate,
            ));
        }
    }

    pub fn redo(&mut self) {
        if self.history_index + 1 < self.scene_history.len() {
            self.history_index += 1;
            let scene = self.scene_history[self.history_index].clone();
            self.skip_next_history_push = true;
            self.send(sova_core::schedule::SchedulerMessage::SetScene(
                scene,
                ActionTiming::Immediate,
            ));
        }
    }

    fn route_device_op<R>(&self, local: impl FnOnce(&DeviceMap) -> R, wire: ClientMessage) {
        if let Some(engine) = &self.feedback_engine {
            let _ = local(engine.devices());
        } else {
            self.send(wire);
        }
    }

    pub fn connect_midi(&self, name: &str) {
        self.route_device_op(
            |d| d.connect_midi_by_name(name),
            ClientMessage::ConnectMidiDeviceByName(name.to_owned()),
        );
    }

    pub fn disconnect_midi(&self, name: &str) {
        self.route_device_op(
            |d| d.disconnect_midi_by_name(name),
            ClientMessage::DisconnectMidiDeviceByName(name.to_owned()),
        );
    }

    pub fn create_virtual_midi(&self, name: &str) {
        self.route_device_op(
            |d| d.create_virtual_midi_port(name),
            ClientMessage::CreateVirtualMidiOutput(name.to_owned()),
        );
    }

    pub fn assign_slot(&self, slot: usize, name: &str) {
        self.route_device_op(
            |d| d.assign_slot(slot, name),
            ClientMessage::AssignDeviceToSlot(slot, name.to_owned()),
        );
    }

    pub fn unassign_slot(&self, slot: usize) {
        self.route_device_op(
            |d| d.unassign_slot(slot),
            ClientMessage::UnassignDeviceFromSlot(slot),
        );
    }

    pub fn create_osc(&self, name: &str, ip: &str, port: u16) {
        self.route_device_op(
            |d| d.create_osc_output_device(name, ip, port),
            ClientMessage::CreateOscDevice(name.to_owned(), ip.to_owned(), port),
        );
    }

    pub fn create_osc_input(&self, name: &str, port: u16) {
        self.route_device_op(
            |d| d.create_osc_input_device(name, port),
            ClientMessage::CreateOscInputDevice(name.to_owned(), port),
        );
    }

    pub fn remove_osc(&self, name: &str) {
        self.route_device_op(
            |d| d.remove_output_device(name),
            ClientMessage::RemoveOscDevice(name.to_owned()),
        );
    }

    pub fn set_latency(&self, name: &str, latency: f64) {
        self.route_device_op(
            |d| d.set_latency(name.to_owned(), latency),
            ClientMessage::SetDeviceLatency(name.to_owned(), latency),
        );
    }

    pub fn build_snapshot(&self) -> Option<Snapshot> {
        let scene = self.scene.as_ref()?.clone();
        Some(Snapshot {
            scene,
            tempo: self.clock.tempo,
            beat: self.clock.beat,
            micros: 0,
            quantum: self.clock.quantum,
            devices: self.devices.clone(),
            frame_text_layout: Vec::new(),
            frame_doc_snapshots: Vec::new(),
            presence: Vec::new(),
        })
    }
}
