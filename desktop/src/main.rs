#![windows_subsystem = "windows"]

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "en");

mod app_types;
mod client_bridge;
mod commands;
mod feedback_engine;
mod fonts;
mod icons;
mod input;
mod menu_bar;
mod panels;
mod render;
mod sample_browser;
mod scene_io;
mod scene_panel;
mod settings;
mod theme;
mod visuals;
mod widgets;

use app_types::{AppPrefs, AudioControl, Dialogs, InputOwner, Panels, Session, VizScratch};
use eframe::egui;


fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("failed to load icon");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("sova")
            .with_title("Sova")
            .with_icon(icon)
            .with_maximized(true)
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Sova",
        options,
        Box::new(|cc| {
            let ctx = cc.egui_ctx.clone();
            egui_extras::install_image_loaders(&ctx);

            let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio Runtime");
            let handle = runtime.handle().clone();

            let (log_tx, log_rx) = std::sync::mpsc::channel();

            let s = settings::load();
            rust_i18n::set_locale(&s.appearance.locale);
            theme::apply_appearance(&ctx, &s.appearance);
            fonts::apply_fonts(&ctx, &s.appearance.ui_font, &s.appearance.editor_font);

            let server = panels::server_panel::ServerPanel::new(
                handle.clone(),
                log_tx.clone(),
                ctx.clone(),
                s.server,
            );
            let client = panels::client_panel::ClientPanel::new(s.client);
            let logs = panels::log_panel::LogPanel::new(log_rx);
            let saved_master_volume = s.audio.master_volume;
            let audio = panels::audio_panel::AudioPanel::new(s.audio);
            let devices = panels::devices_panel::DevicesPanel::new();
            let options = panels::options_panel::OptionsPanel::new();

            let scope_panel = panels::scope_panel::ScopePanel::new(s.scope);
            let spectrum_panel = panels::spectrum_panel::SpectrumPanel::new(s.spectrum);
            let vu_meter_panel = panels::vu_meter_panel::VuMeterPanel::new();
            let scope_bar_panel = panels::scope_bar_panel::ScopeBarPanel::new(
                s.windows.scope_bar_height,
                s.windows.scope_bar_mode,
            );
            let chat_panel = panels::chat_panel::ChatPanel::new();
            let mut scene_panel = scene_panel::ScenePanel::new();
            scene_panel.prelude_collapsed = s.scene.prelude_collapsed;
            scene_panel.prelude_col_width = s.scene.prelude_col_width;
            scene_panel.view_mode = if s.scene.view_mode == 1 {
                scene_panel::ViewMode::Stack
            } else {
                scene_panel::ViewMode::Sequencer
            };

            let mut transport_bar = panels::transport_bar::TransportBar::new();
            transport_bar.show_phase_bar = s.scene.show_phase_bar;

            let bridge = client_bridge::ClientBridge::new(handle, ctx, log_tx);

            let doc_panel = panels::doc_panel::DocPanel::new(s.doc);
            let tools_panel = panels::tools_panel::ToolsPanel::new(s.tools);
            let visuals = visuals::VisualsEngine::new(cc.gl.clone());
            let sample_browser_panel = panels::sample_browser_panel::SampleBrowserPanel::new();

            let panels = Panels::new(
                server,
                client,
                logs,
                audio,
                devices,
                options,
                scope_panel,
                spectrum_panel,
                vu_meter_panel,
                scope_bar_panel,
                chat_panel,
                scene_panel,
                transport_bar,
                sample_browser_panel,
                doc_panel,
                tools_panel,
                visuals,
            );

            let mut app = SovaApp {
                _runtime: runtime,
                bridge,
                prefs: AppPrefs::new(s.appearance, s.editor),
                panels,
                dialogs: Dialogs::new(),
                audio_ctl: AudioControl::new(saved_master_volume),
                viz: VizScratch::new(),
                session: Session::new(s.recent_scenes),
                menu_bar: menu_bar::MenuBarState::new(),
                last_saved_settings: settings::AppSettings::default(),
                settings_dirty_since: None,
                input_owner: InputOwner::Scene,
                sample_browser_rect: None,
            };

            app.panels.chat.detached = s.windows.chat_detached;
            app.panels.sample_browser.detached = s.windows.sample_browser_detached;
            app.last_saved_settings = app.build_settings();

            Ok(Box::new(app))
        }),
    )
}

pub(crate) struct SovaApp {
    pub(crate) _runtime: tokio::runtime::Runtime,
    pub(crate) bridge: client_bridge::ClientBridge,
    pub(crate) prefs: AppPrefs,
    pub(crate) panels: Panels,
    pub(crate) dialogs: Dialogs,
    pub(crate) audio_ctl: AudioControl,
    pub(crate) viz: VizScratch,
    pub(crate) session: Session,
    pub(crate) menu_bar: menu_bar::MenuBarState,
    pub(crate) last_saved_settings: settings::AppSettings,
    pub(crate) settings_dirty_since: Option<std::time::Instant>,
    pub(crate) input_owner: InputOwner,
    /// Rect of the embedded sample browser from the previous frame.
    /// Only the browser sub-area, not the whole tools panel (which may also have chat).
    pub(crate) sample_browser_rect: Option<egui::Rect>,
}

impl SovaApp {
    fn build_settings(&self) -> settings::AppSettings {
        settings::AppSettings {
            windows: settings::WindowSettings {
                chat_detached: self.panels.chat.detached,
                sample_browser_detached: self.panels.sample_browser.detached,
                scope_bar_height: self.panels.scope_bar.height(),
                scope_bar_mode: self.panels.scope_bar.mode,
            },
            editor: self.prefs.editor.clone(),
            server: self.panels.server.settings(),
            client: self.panels.client.settings(),
            audio: {
                let mut audio = self.panels.audio.settings();
                audio.master_volume = self.audio_ctl.master_volume;
                audio
            },
            appearance: self.prefs.appearance.clone(),
            scope: self.panels.scope.settings.clone(),
            spectrum: self.panels.spectrum.settings.clone(),
            doc: self.panels.doc.settings.clone(),
            scene: settings::SceneSettings {
                prelude_collapsed: self.panels.scene.prelude_collapsed,
                prelude_col_width: self.panels.scene.prelude_col_width,
                view_mode: match self.panels.scene.view_mode {
                    scene_panel::ViewMode::Stack => 1,
                    scene_panel::ViewMode::Sequencer => 0,
                },
                show_phase_bar: self.panels.transport_bar.show_phase_bar,
            },
            tools: self.panels.tools.settings.clone(),
            recent_scenes: self.session.recent_scenes.clone(),
        }
    }

    fn save_settings(&mut self) {
        let current = self.build_settings();
        settings::save(&current);
        self.last_saved_settings = current;
        self.settings_dirty_since = None;
    }

}

impl eframe::App for SovaApp {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.save_settings();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.prefs.appearance.zoom = ctx.zoom_factor();
        self.handle_global_shortcuts(ctx);
        self.handle_close_request(ctx);
        self.handle_dialogs(ctx);
        self.poll_updates();
        self.render_top_bar(ctx);
        self.render_sidebar_and_panels(ctx);
        self.render_central_panel(ctx);
        self.render_floating_windows(ctx);
        self.render_overlays(ctx);
    }
}


