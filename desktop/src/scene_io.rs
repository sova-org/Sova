use egui_file_dialog::FileDialog;
use eframe::egui;
use sova_core::schedule::{ActionTiming, SchedulerMessage};
use sova_server::ClientMessage;

use crate::{
    app_types::PendingDialog,
    SovaApp,
};

impl SovaApp {
    pub(crate) fn save_scene(&mut self) {
        let Some(snapshot) = self.bridge.build_snapshot() else {
            return;
        };
        self.dialogs.file = FileDialog::new()
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .add_file_filter_extensions("Sova Scene", vec!["sova"]);
        self.dialogs.file.save_file();
        self.dialogs.pending = PendingDialog::SaveScene {
            snapshot: Box::new(snapshot),
        };
    }

    pub(crate) fn load_scene(&mut self, timing: ActionTiming) {
        self.dialogs.file = FileDialog::new()
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .add_file_filter_extensions("Sova Scene", vec!["sova"]);
        self.dialogs.file.pick_file();
        self.dialogs.pending = PendingDialog::LoadScene { timing };
    }

    pub(crate) fn load_scene_from_path(&mut self, path: &std::path::Path, timing: ActionTiming) {
        let Ok(bytes) = std::fs::read(path) else {
            return;
        };
        let Ok(snapshot) = serde_json::from_slice::<sova_server::Snapshot>(&bytes) else {
            return;
        };
        self.panels.scene.clear_frame_states();
        self.bridge
            .send(SchedulerMessage::SetScene(snapshot.scene, timing));
        self.bridge
            .send(SchedulerMessage::SetTempo(snapshot.tempo, timing));
        self.bridge
            .send(SchedulerMessage::SetQuantum(snapshot.quantum, timing));
        self.bridge
            .send(ClientMessage::RestoreDevices(snapshot.devices));
        self.push_recent_scene(path.to_path_buf());
    }

    pub(crate) fn load_scene_from_bytes(&mut self, bytes: &[u8], timing: ActionTiming) {
        let Ok(snapshot) = serde_json::from_slice::<sova_server::Snapshot>(bytes) else {
            return;
        };
        self.panels.scene.clear_frame_states();
        self.bridge
            .send(SchedulerMessage::SetScene(snapshot.scene, timing));
        self.bridge
            .send(SchedulerMessage::SetTempo(snapshot.tempo, timing));
        self.bridge
            .send(SchedulerMessage::SetQuantum(snapshot.quantum, timing));
        self.bridge
            .send(ClientMessage::RestoreDevices(snapshot.devices));
    }

    pub(crate) fn push_recent_scene(&mut self, path: std::path::PathBuf) {
        self.session.recent_scenes.retain(|p| p != &path);
        self.session.recent_scenes.insert(0, path);
        self.session.recent_scenes.truncate(10);
    }
}
