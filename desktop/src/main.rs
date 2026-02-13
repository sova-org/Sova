mod audio_panel;
mod client_bridge;
mod client_panel;
mod devices_panel;
mod log_panel;
mod options_panel;
mod scene_panel;
mod scope_panel;
mod server_panel;
mod settings;
mod spectrum_panel;
mod transport_bar;
mod widgets;

use eframe::egui;
use server_panel::ServerAction;
use settings::{AppearanceSettings, SpacingPref, ThemePref};

fn apply_appearance(ctx: &egui::Context, a: &AppearanceSettings) {
    ctx.set_theme(match a.theme {
        ThemePref::Dark => egui::ThemePreference::Dark,
        ThemePref::Light => egui::ThemePreference::Light,
        ThemePref::System => egui::ThemePreference::System,
    });

    ctx.set_zoom_factor(a.zoom);

    ctx.all_styles_mut(|style| {
        let zero = egui::CornerRadius::ZERO;
        style.visuals.window_corner_radius = zero;
        style.visuals.menu_corner_radius = zero;
        style.visuals.widgets.noninteractive.corner_radius = zero;
        style.visuals.widgets.inactive.corner_radius = zero;
        style.visuals.widgets.hovered.corner_radius = zero;
        style.visuals.widgets.active.corner_radius = zero;
        style.visuals.widgets.open.corner_radius = zero;

        let accent =
            egui::Color32::from_rgb(a.accent_color[0], a.accent_color[1], a.accent_color[2]);
        style.visuals.selection.bg_fill = accent;

        if a.window_shadows {
            let defaults = if style.visuals.dark_mode {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            };
            style.visuals.window_shadow = defaults.window_shadow;
            style.visuals.popup_shadow = defaults.popup_shadow;
        } else {
            style.visuals.window_shadow = egui::Shadow::NONE;
            style.visuals.popup_shadow = egui::Shadow::NONE;
        }

        match a.spacing {
            SpacingPref::Compact => {
                style.spacing.item_spacing = egui::vec2(4.0, 2.0);
                style.spacing.button_padding = egui::vec2(2.0, 1.0);
            }
            SpacingPref::Normal => {}
            SpacingPref::Comfortable => {
                style.spacing.item_spacing = egui::vec2(12.0, 8.0);
                style.spacing.button_padding = egui::vec2(8.0, 4.0);
            }
        }
    });
}

fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("failed to load icon");

    let options = eframe::NativeOptions {
        centered: true,
        viewport: egui::ViewportBuilder::default()
            .with_app_id("sova")
            .with_title("Sova")
            .with_icon(icon)
            .with_inner_size([800.0, 600.0])
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
            apply_appearance(&ctx, &s.appearance);

            let server = server_panel::ServerPanel::new(
                handle.clone(),
                log_tx.clone(),
                ctx.clone(),
                s.server,
            );
            let client = client_panel::ClientPanel::new(s.client);
            let logs = log_panel::LogPanel::new(log_rx);
            let audio = audio_panel::AudioPanel::new(s.audio);
            let devices = devices_panel::DevicesPanel::new();
            let options = options_panel::OptionsPanel::new();

            let scope_panel = scope_panel::ScopePanel::new();
            let spectrum_panel = spectrum_panel::SpectrumPanel::new();
            let scene_panel = scene_panel::ScenePanel::new();

            let bridge = client_bridge::ClientBridge::new(handle, ctx, log_tx);

            let mut app = SovaApp {
                server,
                client,
                logs,
                audio,
                devices,
                _runtime: runtime,
                debug_open: s.windows.debug,
                about_open: false,
                confirm_exit: widgets::ConfirmDialog::new(),
                options,
                scope_panel,
                spectrum_panel,
                scene_panel,
                transport_bar: transport_bar::TransportBar::new(),
                editor_settings: s.editor,
                step_editors: widgets::StepEditorManager::new(),
                appearance: s.appearance,
                bridge,
            };

            app.server.open = s.windows.server;
            app.logs.collapsed = s.windows.logs_collapsed;
            app.audio.open = s.windows.audio;
            app.devices.open = s.windows.devices;
            app.options.open = s.windows.options;
            app.scope_panel.open = s.windows.scope;
            app.spectrum_panel.open = s.windows.spectrum;

            Ok(Box::new(app))
        }),
    )
}

struct SovaApp {
    server: server_panel::ServerPanel,
    client: client_panel::ClientPanel,
    logs: log_panel::LogPanel,
    audio: audio_panel::AudioPanel,
    devices: devices_panel::DevicesPanel,
    _runtime: tokio::runtime::Runtime,
    debug_open: bool,
    about_open: bool,
    confirm_exit: widgets::ConfirmDialog,
    options: options_panel::OptionsPanel,
    scope_panel: scope_panel::ScopePanel,
    spectrum_panel: spectrum_panel::SpectrumPanel,
    scene_panel: scene_panel::ScenePanel,
    transport_bar: transport_bar::TransportBar,
    editor_settings: widgets::EditorSettings,
    step_editors: widgets::StepEditorManager,
    appearance: AppearanceSettings,
    bridge: client_bridge::ClientBridge,
}

impl SovaApp {
    fn save_settings(&self) {
        let s = settings::AppSettings {
            windows: settings::WindowSettings {
                server: self.server.open,
                audio: self.audio.open,
                devices: self.devices.open,
                logs_collapsed: self.logs.collapsed,
                debug: self.debug_open,
                options: self.options.open,
                scope: self.scope_panel.open,
                spectrum: self.spectrum_panel.open,
            },
            editor: self.editor_settings.clone(),
            server: self.server.settings(),
            client: self.client.settings(),
            audio: self.audio.settings(),
            appearance: self.appearance.clone(),
        };
        settings::save(&s);
    }
}

impl eframe::App for SovaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            self.save_settings();
            if self.server.is_running() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                if !self.confirm_exit.is_open() {
                    self.confirm_exit.open(
                        "Exit Sova",
                        "The server is still running. Are you sure you want to exit?",
                    );
                }
            }
        }

        match self.confirm_exit.show(ctx) {
            widgets::ConfirmAction::Confirmed => {
                self.save_settings();
                self.step_editors.close_all();
                self.bridge.disconnect();
                self.server.stop();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            widgets::ConfirmAction::Cancelled | widgets::ConfirmAction::None => {}
        }

        self.server.poll();
        self.bridge.poll();
        self.logs.poll();

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let icon = egui::Image::new(egui::include_image!("../assets/icon.png"))
                    .fit_to_exact_size(egui::vec2(16.0, 16.0));
                if ui.add(egui::Button::image(icon).frame(false)).clicked() {
                    self.about_open = !self.about_open;
                }
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        ui.close();
                        if self.server.is_running() {
                            self.confirm_exit.open(
                                "Exit Sova",
                                "The server is still running. Are you sure you want to exit?",
                            );
                        } else {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.server.open, "Server");
                    ui.separator();
                    ui.checkbox(&mut self.audio.open, "Audio");
                    ui.checkbox(&mut self.devices.open, "Devices");
                    ui.checkbox(&mut self.scope_panel.open, "Scope");
                    ui.checkbox(&mut self.spectrum_panel.open, "Spectrum");
                    ui.separator();
                    let mut logs_expanded = !self.logs.collapsed;
                    if ui.checkbox(&mut logs_expanded, "Logs").changed() {
                        self.logs.collapsed = !logs_expanded;
                    }
                    ui.separator();
                    ui.checkbox(&mut self.options.open, "Options");
                    ui.checkbox(&mut self.debug_open, "Debug");
                });
            });
        });

        // Transport bar
        self.transport_bar.show(ctx, &self.bridge);

        let disconnect = egui::TopBottomPanel::bottom("bottom_bar")
            .show(ctx, |ui| {
                widgets::bottom_bar(ui, &self.server.info(), &self.client.info(&self.bridge))
            })
            .inner;
        if disconnect {
            self.bridge.disconnect();
            self.step_editors.close_all();
        }

        self.logs.show(ctx);

        if self.bridge.is_connected() {
            let mut panels = scene_panel::PanelVisibility {
                server: self.server.open,
                audio: self.audio.open,
                devices: self.devices.open,
                scope: self.scope_panel.open,
                spectrum: self.spectrum_panel.open,
                logs: !self.logs.collapsed,
                options: self.options.open,
                debug: self.debug_open,
            };
            let open_step = egui::CentralPanel::default()
                .show(ctx, |ui| {
                    self.scene_panel.show(ui, &self.bridge, &mut panels)
                })
                .inner;
            self.server.open = panels.server;
            self.audio.open = panels.audio;
            self.devices.open = panels.devices;
            self.scope_panel.open = panels.scope;
            self.spectrum_panel.open = panels.spectrum;
            self.logs.collapsed = !panels.logs;
            self.options.open = panels.options;
            self.debug_open = panels.debug;
            if let Some((li, fi)) = open_step
                && let Some(frame) = self
                    .bridge
                    .scene()
                    .and_then(|s| s.lines.get(li))
                    .and_then(|l| l.frames.get(fi))
            {
                self.step_editors.open(li, fi, frame);
                self.bridge
                    .send(sova_server::ClientMessage::StartedEditingFrame(li, fi));
            }

            self.step_editors
                .show(ctx, &self.bridge, &self.editor_settings);
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.client.show_centered(ui, &mut self.bridge);
            });
        }

        // Floating windows
        let server_action = self.server.show(ctx);
        match server_action {
            ServerAction::Start => {
                self.server.start();
            }
            ServerAction::Stop => {
                self.bridge.disconnect();
                self.step_editors.close_all();
                self.server.stop();
            }
            ServerAction::None => {}
        }

        self.audio.show(ctx, &self.bridge);
        self.devices.show(ctx, &self.bridge);

        let scope_data = self.bridge.scope_data();
        self.scope_panel.show(ctx, scope_data);
        self.spectrum_panel.show(ctx, scope_data);

        if self
            .options
            .show(ctx, &mut self.editor_settings, &mut self.appearance)
        {
            apply_appearance(ctx, &self.appearance);
        }
        show_debug_window(ctx, &mut self.debug_open);
        widgets::about_dialog(ctx, &mut self.about_open);
    }
}

fn show_debug_window(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Debug")
        .open(open)
        .resizable(true)
        .collapsible(true)
        .default_width(320.0)
        .vscroll(true)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new("Settings").show(ui, |ui| ctx.settings_ui(ui));
            egui::CollapsingHeader::new("Inspection").show(ui, |ui| ctx.inspection_ui(ui));
            egui::CollapsingHeader::new("Textures").show(ui, |ui| ctx.texture_ui(ui));
            egui::CollapsingHeader::new("Memory").show(ui, |ui| ctx.memory_ui(ui));
        });
}
