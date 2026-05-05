use eframe::egui;
use egui_file_dialog::FileDialog;

use crate::{
    panels::{
        audio_panel, chat_panel, client_panel, devices_panel, doc_panel, log_panel, options_panel,
        sample_browser_panel, scope_bar_panel, scope_panel, server_panel, spectrum_panel,
        tools_panel, transport_bar, vu_meter_panel,
    },
    scene_panel,
    settings::AppearanceSettings,
    visuals, widgets,
};

pub(crate) enum PendingDialog {
    None,
    SaveScene {
        snapshot: Box<sova_server::Snapshot>,
    },
    LoadScene {
        timing: sova_core::schedule::ActionTiming,
    },
    PickSampleFolder,
}

pub(crate) struct Session {
    pub(crate) recent_scenes: Vec<std::path::PathBuf>,
    pub(crate) toasts: widgets::ToastStack,
    pub(crate) rename_input: Option<String>,
}

impl Session {
    pub(crate) fn new(recent_scenes: Vec<std::path::PathBuf>) -> Self {
        Self {
            recent_scenes,
            toasts: widgets::ToastStack::new(),
            rename_input: None,
        }
    }
}

pub(crate) struct AudioControl {
    pub(crate) master_volume: f32,
    pub(crate) muted: bool,
    pub(crate) was_running: bool,
}

impl AudioControl {
    pub(crate) fn new(master_volume: f32) -> Self {
        Self {
            master_volume,
            muted: false,
            was_running: false,
        }
    }

    pub(crate) fn effective_gain(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.master_volume * self.master_volume
        }
    }
}

pub(crate) struct Dialogs {
    pub(crate) file: FileDialog,
    pub(crate) pending: PendingDialog,
    pub(crate) confirm_exit: widgets::ConfirmDialog,
    pub(crate) confirm_reset_scene: widgets::ConfirmDialog,
    pub(crate) confirm_load_demo: widgets::ConfirmDialog,
    pub(crate) pending_demo: Option<(&'static str, &'static [u8])>,
}

impl Dialogs {
    pub(crate) fn new() -> Self {
        Self {
            file: FileDialog::new().anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]),
            pending: PendingDialog::None,
            confirm_exit: widgets::ConfirmDialog::new("confirm_exit"),
            confirm_reset_scene: widgets::ConfirmDialog::new("confirm_reset_scene"),
            confirm_load_demo: widgets::ConfirmDialog::new("confirm_load_demo"),
            pending_demo: None,
        }
    }
}

pub(crate) struct VizScratch {
    pub(crate) spectrum_analyzer: Option<widgets::SpectrumAnalyzer>,
    pub(crate) raw_bands: Vec<f32>,
    pub(crate) aligned_scope: Vec<f32>,
    pub(crate) last_scope_gen: u64,
}

impl VizScratch {
    pub(crate) fn new() -> Self {
        Self {
            spectrum_analyzer: None,
            raw_bands: Vec::new(),
            aligned_scope: Vec::new(),
            last_scope_gen: 0,
        }
    }
}

pub(crate) struct AppPrefs {
    pub(crate) appearance: AppearanceSettings,
    pub(crate) editor: widgets::EditorSettings,
}

impl AppPrefs {
    pub(crate) fn new(appearance: AppearanceSettings, editor: widgets::EditorSettings) -> Self {
        Self { appearance, editor }
    }
}

pub(crate) struct Panels {
    pub(crate) server: server_panel::ServerPanel,
    pub(crate) client: client_panel::ClientPanel,
    pub(crate) logs: log_panel::LogPanel,
    pub(crate) audio: audio_panel::AudioPanel,
    pub(crate) devices: devices_panel::DevicesPanel,
    pub(crate) options: options_panel::OptionsPanel,
    pub(crate) scope: scope_panel::ScopePanel,
    pub(crate) spectrum: spectrum_panel::SpectrumPanel,
    pub(crate) vu_meter: vu_meter_panel::VuMeterPanel,
    pub(crate) scope_bar: scope_bar_panel::ScopeBarPanel,
    pub(crate) chat: chat_panel::ChatPanel,
    pub(crate) scene: scene_panel::ScenePanel,
    pub(crate) transport_bar: transport_bar::TransportBar,
    pub(crate) sample_browser: sample_browser_panel::SampleBrowserPanel,
    pub(crate) doc: doc_panel::DocPanel,
    pub(crate) tools: tools_panel::ToolsPanel,
    pub(crate) visuals: visuals::VisualsEngine,
    pub(crate) command_palette: widgets::CommandPalette,
    pub(crate) debug_open: bool,
    pub(crate) about_open: bool,
    pub(crate) keybindings_open: bool,
}

impl Panels {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        server: server_panel::ServerPanel,
        client: client_panel::ClientPanel,
        logs: log_panel::LogPanel,
        audio: audio_panel::AudioPanel,
        devices: devices_panel::DevicesPanel,
        options: options_panel::OptionsPanel,
        scope: scope_panel::ScopePanel,
        spectrum: spectrum_panel::SpectrumPanel,
        vu_meter: vu_meter_panel::VuMeterPanel,
        scope_bar: scope_bar_panel::ScopeBarPanel,
        chat: chat_panel::ChatPanel,
        scene: scene_panel::ScenePanel,
        transport_bar: transport_bar::TransportBar,
        sample_browser: sample_browser_panel::SampleBrowserPanel,
        doc: doc_panel::DocPanel,
        tools: tools_panel::ToolsPanel,
        visuals: visuals::VisualsEngine,
    ) -> Self {
        Self {
            server,
            client,
            logs,
            audio,
            devices,
            options,
            scope,
            spectrum,
            vu_meter,
            scope_bar,
            chat,
            scene,
            transport_bar,
            sample_browser,
            doc,
            tools,
            visuals,
            command_palette: widgets::CommandPalette::new(),
            debug_open: false,
            about_open: false,
            keybindings_open: false,
        }
    }
}

/// Which component owns unmodified (bare) keyboard input this frame.
/// Determined once at the start of each update before any panel renders.
/// Panels check this value to decide whether to read bare keys like J/K/arrows.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputOwner {
    MenuBar,
    Palette,
    Scene,
    SampleBrowser,
}
