use eframe::egui;
use sova_core::schedule::{ActionTiming, SchedulerMessage};
use sova_server::ClientMessage;

use crate::{panels::doc_panel, menu_bar, widgets, SovaApp};

impl SovaApp {
    pub(crate) fn build_menus(&self, ctx: &egui::Context) -> Vec<menu_bar::MenuDef> {
        menu_bar::build_menus(&menu_bar::MenuContext {
            connected: self.bridge.is_connected(),
            server_running: self.panels.server.is_running(),
            scope_open: self.panels.scope.open,
            spectrum_open: self.panels.spectrum.open,
            vu_meter_open: self.panels.vu_meter.open,
            scope_bar_open: self.panels.scope_bar.open,
            chat_open: self.panels.tools.settings.show_chat,
            sample_browser_open: self.panels.tools.settings.show_sample_browser,
            sample_browser_available: !self.bridge.is_connected()
                || self.panels.server.is_running(),
            visuals_open: self.panels.visuals.open,
            debug_open: self.panels.debug_open,
            scene_view_mode: self.panels.scene.view_mode,
            recent_scenes: &self.session.recent_scenes,
            egui_ctx: ctx,
        })
    }

    pub(crate) fn dispatch_menu_action(
        &mut self,
        action: menu_bar::MenuAction,
        ctx: &egui::Context,
    ) {
        use menu_bar::MenuAction;
        match action {
            MenuAction::Command(cmd) => self.dispatch(cmd),
            MenuAction::LoadSceneAtEnd => {
                if self.bridge.is_connected() {
                    self.load_scene(ActionTiming::AtNextPhase);
                }
            }
            MenuAction::LoadRecentScene(path) => {
                self.load_scene_from_path(&path, ActionTiming::Immediate);
            }
            MenuAction::ClearRecentScenes => {
                self.session.recent_scenes.clear();
            }
            MenuAction::LoadDemo(_name, bytes) => {
                self.load_scene_from_bytes(bytes, ActionTiming::Immediate);
            }
            MenuAction::StartServer => {
                self.panels
                    .server
                    .start(self.panels.audio.generate_audio_config());
            }
            MenuAction::StopServer => {
                self.bridge.disconnect();
                self.panels.server.stop();
            }
            MenuAction::Disconnect => {
                self.bridge.disconnect();
            }
            MenuAction::BeginRename => {
                let current = self.bridge.confirmed_username().unwrap_or("").to_owned();
                self.session.rename_input = Some(current);
            }
            MenuAction::RestartAudio => {
                if self.bridge.is_connected() {
                    self.bridge
                        .restart_audio(self.panels.audio.generate_audio_config());
                }
            }
            MenuAction::SetSceneViewMode(mode) => {
                if self.panels.scene.view_mode != mode {
                    self.panels.scene.view_mode = mode;
                    self.panels.scene.scroll_to_cursor = true;
                }
            }
            MenuAction::Exit => {
                self.save_settings();
                if self.panels.server.is_running() {
                    self.dialogs
                        .confirm_exit
                        .open(t!("exit.title"), t!("exit.message"));
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    pub(crate) fn dispatch(&mut self, cmd: widgets::CommandId) {
        use widgets::CommandId::*;
        match cmd {
            Server | Audio => self
                .panels
                .doc
                .toggle_settings_tab(doc_panel::SettingsTab::Server),
            Devices => self
                .panels
                .doc
                .toggle_settings_tab(doc_panel::SettingsTab::Devices),
            Scope => self.panels.scope.open = !self.panels.scope.open,
            Spectrum => self.panels.spectrum.open = !self.panels.spectrum.open,
            VuMeter => self.panels.vu_meter.open = !self.panels.vu_meter.open,
            ScopeBar => self.panels.scope_bar.open = !self.panels.scope_bar.open,
            Chat => self.panels.tools.toggle_chat(),
            Logs => self
                .panels
                .doc
                .toggle_settings_tab(doc_panel::SettingsTab::Logs),
            Options => self
                .panels
                .doc
                .toggle_settings_tab(doc_panel::SettingsTab::Appearance),
            Debug => self.panels.debug_open = !self.panels.debug_open,
            Keybindings => self.panels.keybindings_open = !self.panels.keybindings_open,
            About => self.panels.about_open = !self.panels.about_open,
            SampleBrowser => {
                let available = !self.bridge.is_connected() || self.panels.server.is_running();
                if available {
                    self.panels.tools.toggle_sample_browser();
                }
            }
            Documentation => {
                self.panels.doc.settings.collapsed = !self.panels.doc.settings.collapsed;
                if !self.panels.doc.settings.collapsed {
                    self.panels.doc.settings.pinned = true;
                }
            }
            Visuals => self.panels.visuals.open = !self.panels.visuals.open,
            RestartCore => {
                if self.bridge.is_connected() {
                    self.bridge.send(ClientMessage::RestartCore);
                }
            }
            PlayPause => {
                if self.bridge.is_connected() {
                    let clock = self.bridge.clock();
                    let msg = if clock.playing {
                        SchedulerMessage::TransportStop(ActionTiming::Immediate)
                    } else {
                        SchedulerMessage::TransportStart(ActionTiming::Immediate)
                    };
                    self.bridge.send(msg);
                }
            }
            SaveScene => {
                if self.bridge.is_connected() {
                    self.save_scene();
                }
            }
            LoadScene => {
                if self.bridge.is_connected() {
                    self.load_scene(ActionTiming::Immediate);
                }
            }
            ResetScene => {
                if self.bridge.is_connected() {
                    self.dialogs
                        .confirm_reset_scene
                        .open(t!("reset_scene.title"), t!("reset_scene.message"));
                }
            }
            ZoomIn => {
                self.prefs.appearance.zoom = (self.prefs.appearance.zoom + 0.1).min(3.0);
            }
            ZoomOut => {
                self.prefs.appearance.zoom = (self.prefs.appearance.zoom - 0.1).max(0.5);
            }
            ZoomReset => {
                self.prefs.appearance.zoom = 1.0;
            }
            ToggleViewMode => {
                use crate::scene_panel::ViewMode;
                self.panels.scene.view_mode = match self.panels.scene.view_mode {
                    ViewMode::Stack => ViewMode::Sequencer,
                    ViewMode::Sequencer => ViewMode::Stack,
                };
                self.panels.scene.scroll_to_cursor = true;
            }
        }
    }
}
