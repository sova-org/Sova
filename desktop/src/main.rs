#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "en");

mod audio_panel;
mod chat_panel;
mod client_bridge;
mod client_panel;
mod devices_panel;
mod doc_panel;
mod icons;
mod log_panel;
mod options_panel;
mod sample_browser;
mod sample_browser_panel;
mod scene_panel;
mod scope_bar_panel;
mod scope_panel;
mod server_panel;
mod settings;
mod spectrum_panel;
mod transport_bar;
mod visuals;
mod vu_meter_panel;
mod widgets;

use eframe::egui;
use server_panel::ServerAction;
use settings::{AppearanceSettings, VisualsSettings};
use sova_core::schedule::{ActionTiming, SchedulerMessage};
use sova_server::ClientMessage;

pub(crate) fn apply_appearance(ctx: &egui::Context, a: &AppearanceSettings) {
    ctx.set_theme(egui::ThemePreference::Dark);

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

        if style.visuals.dark_mode {
            style.visuals.extreme_bg_color = egui::Color32::from_gray(20);
        }

        style.spacing.button_padding = egui::vec2(5.0, 4.0);
        style.spacing.indent_ends_with_horizontal_line = true;
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

            ctx.add_font(egui::epaint::text::FontInsert::new(
                "nerd-font",
                egui::FontData::from_static(include_bytes!(
                    "../assets/SymbolsNerdFont-Regular.ttf"
                )),
                vec![
                    egui::epaint::text::InsertFontFamily {
                        family: egui::FontFamily::Proportional,
                        priority: egui::epaint::text::FontPriority::Lowest,
                    },
                    egui::epaint::text::InsertFontFamily {
                        family: egui::FontFamily::Monospace,
                        priority: egui::epaint::text::FontPriority::Lowest,
                    },
                ],
            ));

            let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio Runtime");
            let handle = runtime.handle().clone();

            let (log_tx, log_rx) = std::sync::mpsc::channel();

            let s = settings::load();
            rust_i18n::set_locale(&s.appearance.locale);
            apply_appearance(&ctx, &s.appearance);

            let server = server_panel::ServerPanel::new(
                handle.clone(),
                log_tx.clone(),
                ctx.clone(),
                s.server,
            );
            let client = client_panel::ClientPanel::new(s.client);
            let logs = log_panel::LogPanel::new(log_rx, s.windows.log_panel_height);
            let audio = audio_panel::AudioPanel::new(s.audio);
            let devices = devices_panel::DevicesPanel::new();
            let options = options_panel::OptionsPanel::new();

            let scope_panel = scope_panel::ScopePanel::new(s.scope);
            let spectrum_panel = spectrum_panel::SpectrumPanel::new(s.spectrum);
            let vu_meter_panel = vu_meter_panel::VuMeterPanel::new();
            let scope_bar_panel = scope_bar_panel::ScopeBarPanel::new(s.windows.scope_bar);
            let chat_panel = chat_panel::ChatPanel::new();
            let scene_panel = scene_panel::ScenePanel::new();

            let bridge = client_bridge::ClientBridge::new(handle, ctx, log_tx);

            let doc_panel = doc_panel::DocPanel::new();
            let visuals = visuals::VisualsEngine::new(cc.gl.clone(), &s.visuals);

            let mut app = SovaApp {
                server,
                client,
                logs,
                audio,
                devices,
                _runtime: runtime,
                debug_open: false,
                about_open: false,
                keybindings_open: false,
                confirm_exit: widgets::ConfirmDialog::new("confirm_exit"),
                command_palette: widgets::CommandPalette::new(),
                options,
                scope_panel,
                spectrum_panel,
                vu_meter_panel,
                scope_bar_panel,
                chat_panel,
                scene_panel,
                transport_bar: transport_bar::TransportBar::new(),
                editor_settings: s.editor,
                step_editors: widgets::StepEditorManager::new(),
                appearance: s.appearance,
                bridge,
                sample_browser_panel: sample_browser_panel::SampleBrowserPanel::new(),
                doc_panel,
                recent_scenes: s.recent_scenes,
                dismissed_tips: s.dismissed_tips,
                visuals,
                rename_input: None,
            };

            app.logs.collapsed = s.windows.logs_collapsed;
            app.chat_panel.detached = s.windows.chat_detached;
            app.sample_browser_panel.detached = s.windows.sample_browser_detached;

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
    keybindings_open: bool,
    confirm_exit: widgets::ConfirmDialog,
    command_palette: widgets::CommandPalette,
    options: options_panel::OptionsPanel,
    scope_panel: scope_panel::ScopePanel,
    spectrum_panel: spectrum_panel::SpectrumPanel,
    vu_meter_panel: vu_meter_panel::VuMeterPanel,
    scope_bar_panel: scope_bar_panel::ScopeBarPanel,
    chat_panel: chat_panel::ChatPanel,
    scene_panel: scene_panel::ScenePanel,
    transport_bar: transport_bar::TransportBar,
    editor_settings: widgets::EditorSettings,
    step_editors: widgets::StepEditorManager,
    appearance: AppearanceSettings,
    bridge: client_bridge::ClientBridge,
    sample_browser_panel: sample_browser_panel::SampleBrowserPanel,
    doc_panel: doc_panel::DocPanel,
    recent_scenes: Vec<std::path::PathBuf>,
    dismissed_tips: Vec<String>,
    visuals: visuals::VisualsEngine,
    rename_input: Option<String>,
}

impl SovaApp {
    fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        let cmd_k = ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::K));
        if cmd_k {
            if !self.command_palette.is_open() {
                self.command_palette.open();
            }
            return;
        }
        if self.command_palette.is_open() {
            return;
        }
        if ctx.memory(|m| m.focused().is_some()) {
            return;
        }
        ctx.input(|i| {
            if i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::Comma) {
                self.options.open = !self.options.open;
            }
            if i.modifiers.command && i.modifiers.shift {
                if i.key_pressed(egui::Key::S) {
                    self.server.open = !self.server.open;
                }
                if i.key_pressed(egui::Key::A) {
                    self.audio.open = !self.audio.open;
                }
                if i.key_pressed(egui::Key::I) {
                    self.devices.open = !self.devices.open;
                }
                if i.key_pressed(egui::Key::O) {
                    self.scope_panel.open = !self.scope_panel.open;
                }
                if i.key_pressed(egui::Key::P) {
                    self.spectrum_panel.open = !self.spectrum_panel.open;
                }
                if i.key_pressed(egui::Key::U) {
                    self.vu_meter_panel.open = !self.vu_meter_panel.open;
                }
                if i.key_pressed(egui::Key::W) {
                    self.scope_bar_panel.open = !self.scope_bar_panel.open;
                }
                if i.key_pressed(egui::Key::L) {
                    self.logs.collapsed = !self.logs.collapsed;
                }
                if i.key_pressed(egui::Key::B) {
                    self.debug_open = !self.debug_open;
                }
                if i.key_pressed(egui::Key::C) {
                    self.chat_panel.open = !self.chat_panel.open;
                }
                if i.key_pressed(egui::Key::E) {
                    self.sample_browser_panel.open = !self.sample_browser_panel.open;
                }
                if i.key_pressed(egui::Key::H) {
                    self.doc_panel.open = !self.doc_panel.open;
                }
                if i.key_pressed(egui::Key::V) {
                    self.visuals.open = !self.visuals.open;
                }
            }
            if i.modifiers.command
                && i.modifiers.shift
                && i.key_pressed(egui::Key::Space)
                && self.bridge.is_connected()
            {
                let clock = self.bridge.clock();
                let msg = if clock.playing {
                    ClientMessage::TransportStop(ActionTiming::Immediate)
                } else {
                    ClientMessage::TransportStart(ActionTiming::Immediate)
                };
                self.bridge.send(msg);
            }
            if i.modifiers.command
                && !i.modifiers.shift
                && i.key_pressed(egui::Key::S)
                && self.bridge.is_connected()
            {
                self.save_scene();
            }
            if i.modifiers.command
                && !i.modifiers.shift
                && i.key_pressed(egui::Key::O)
                && self.bridge.is_connected()
            {
                self.load_scene(ActionTiming::Immediate);
            }
            if i.key_pressed(egui::Key::F1) && !i.modifiers.command && !i.modifiers.shift {
                self.keybindings_open = !self.keybindings_open;
            }
        });
    }

    fn save_scene(&mut self) {
        let Some(snapshot) = self.bridge.build_snapshot() else {
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Sova Scene", &["sova"])
            .save_file()
        else {
            return;
        };
        let Ok(bytes) = serde_json::to_vec(&snapshot) else {
            return;
        };
        if std::fs::write(&path, bytes).is_ok() {
            self.push_recent_scene(path);
        }
    }

    fn load_scene(&mut self, timing: ActionTiming) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Sova Scene", &["sova"])
            .pick_file()
        else {
            return;
        };
        self.load_scene_from_path(&path, timing);
    }

    fn load_scene_from_path(&mut self, path: &std::path::Path, timing: ActionTiming) {
        let Ok(bytes) = std::fs::read(path) else {
            return;
        };
        let Ok(snapshot) = serde_json::from_slice::<sova_server::Snapshot>(&bytes) else {
            return;
        };
        self.bridge
            .send(ClientMessage::SetScene(snapshot.scene, timing));
        self.bridge
            .send(ClientMessage::SetTempo(snapshot.tempo, timing));
        self.bridge.send(ClientMessage::SchedulerControl(
            SchedulerMessage::SetQuantum(snapshot.quantum, timing),
        ));
        self.push_recent_scene(path.to_path_buf());
    }

    fn push_recent_scene(&mut self, path: std::path::PathBuf) {
        self.recent_scenes.retain(|p| p != &path);
        self.recent_scenes.insert(0, path);
        self.recent_scenes.truncate(10);
    }

    fn save_settings(&self) {
        let s = settings::AppSettings {
            windows: settings::WindowSettings {
                logs_collapsed: self.logs.collapsed,
                log_panel_height: self.logs.height(),
                chat_detached: self.chat_panel.detached,
                sample_browser_detached: self.sample_browser_panel.detached,
                scope_bar: settings::ScopeBarSettings {
                    height: self.scope_bar_panel.height(),
                    smoothing: self.scope_bar_panel.smoothing(),
                },
            },
            editor: self.editor_settings.clone(),
            server: self.server.settings(),
            client: self.client.settings(),
            audio: self.audio.settings(),
            appearance: self.appearance.clone(),
            scope: self.scope_panel.settings.clone(),
            spectrum: self.spectrum_panel.settings.clone(),
            visuals: VisualsSettings {
                code: self.visuals.code().to_owned(),
            },
            recent_scenes: self.recent_scenes.clone(),
            dismissed_tips: self.dismissed_tips.clone(),
        };
        settings::save(&s);
    }
}

impl eframe::App for SovaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.appearance.zoom = ctx.zoom_factor();
        self.handle_global_shortcuts(ctx);

        if ctx.input(|i| i.viewport().close_requested()) {
            self.save_settings();
            if self.server.is_running() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                if !self.confirm_exit.is_open() {
                    self.confirm_exit.open(t!("exit.title"), t!("exit.message"));
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
                    .fit_to_exact_size(egui::vec2(20.0, 20.0));
                let r = ui.add(egui::Button::image(icon).frame(false));
                if r.hovered() {
                    widgets::hint::set(ctx, t!("hint.about_sova"));
                }
                if r.clicked() {
                    self.about_open = !self.about_open;
                }
                let r = ui.menu_button(t!("menu.file"), |ui| {
                    let connected = self.bridge.is_connected();
                    if ui
                        .add_enabled(connected, egui::Button::new(t!("menu.save_scene")))
                        .clicked()
                    {
                        ui.close();
                        self.save_scene();
                    }
                    if ui
                        .add_enabled(connected, egui::Button::new(t!("menu.load_scene")))
                        .clicked()
                    {
                        ui.close();
                        self.load_scene(ActionTiming::Immediate);
                    }
                    if ui
                        .add_enabled(connected, egui::Button::new(t!("menu.load_scene_at_end")))
                        .clicked()
                    {
                        ui.close();
                        self.load_scene(ActionTiming::AtNextPhase);
                    }
                    let has_recent = !self.recent_scenes.is_empty();
                    ui.add_enabled_ui(connected && has_recent, |ui| {
                        ui.menu_button(t!("menu.recent"), |ui| {
                            let mut load_path = None;
                            let mut clear = false;
                            for path in &self.recent_scenes {
                                if !path.exists() {
                                    continue;
                                }
                                let label = path
                                    .file_name()
                                    .map(|n| n.to_string_lossy().into_owned())
                                    .unwrap_or_else(|| path.display().to_string());
                                let btn =
                                    ui.button(&label).on_hover_text(path.display().to_string());
                                if btn.clicked() {
                                    load_path = Some(path.clone());
                                    ui.close();
                                }
                            }
                            ui.separator();
                            if ui.button(t!("menu.clear")).clicked() {
                                clear = true;
                                ui.close();
                            }
                            if let Some(p) = load_path {
                                self.load_scene_from_path(&p, ActionTiming::Immediate);
                            }
                            if clear {
                                self.recent_scenes.clear();
                            }
                        });
                    });
                    ui.separator();
                    if ui.button(t!("menu.exit")).clicked() {
                        ui.close();
                        if self.server.is_running() {
                            self.confirm_exit.open(t!("exit.title"), t!("exit.message"));
                        } else {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                if r.response.hovered() {
                    widgets::hint::set(ctx, t!("hint.file_operations"));
                }
                let r = ui.menu_button(t!("menu.server"), |ui| {
                    if self.server.is_running() {
                        if ui.button(t!("menu.stop_server")).clicked() {
                            ui.close();
                            self.bridge.disconnect();
                            self.step_editors.close_all();
                            self.server.stop();
                        }
                    } else if ui.button(t!("menu.start_server")).clicked() {
                        ui.close();
                        self.server.start(self.audio.initial_audio_config());
                    }
                    if self.bridge.is_connected() {
                        ui.separator();
                        if let Some(input) = &mut self.rename_input {
                            let r = ui.text_edit_singleline(input);
                            if r.lost_focus() {
                                if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !input.trim().is_empty() {
                                    let new_name = input.trim().to_owned();
                                    self.bridge.send(ClientMessage::SetName(new_name.clone()));
                                    self.bridge.set_confirmed_username(new_name);
                                    self.rename_input = None;
                                    ui.close();
                                } else {
                                    self.rename_input = None;
                                }
                            } else {
                                r.request_focus();
                            }
                        } else if ui.button(t!("menu.rename")).clicked() {
                            let current = self.bridge.confirmed_username().unwrap_or("").to_owned();
                            self.rename_input = Some(current);
                        }
                    }
                });
                if r.response.hovered() {
                    widgets::hint::set(ctx, t!("hint.server_menu"));
                }
                let r = ui.menu_button(t!("menu.engine"), |ui| {
                    let enabled = self.bridge.is_connected();
                    if ui
                        .add_enabled(enabled, egui::Button::new(t!("menu.restart_audio")))
                        .clicked()
                    {
                        ui.close();
                        self.bridge.send(self.audio.restart_message());
                    }
                });
                if r.response.hovered() {
                    widgets::hint::set(ctx, t!("hint.engine_menu"));
                }
                let r = ui.menu_button(t!("menu.view"), |ui| {
                    let is_mac = ctx.os() == egui::os::OperatingSystem::Mac;
                    let (mod_sym, shift_sym) = if is_mac {
                        ("⌘", "⇧")
                    } else {
                        ("Ctrl+", "Shift+")
                    };

                    let menu_checkbox = |ui: &mut egui::Ui,
                                         checked: &mut bool,
                                         label: std::borrow::Cow<'_, str>,
                                         shortcut: &str| {
                        let text = egui::RichText::new(shortcut).weak();
                        ui.checkbox(checked, label).on_hover_text(text);
                    };

                    menu_checkbox(
                        ui,
                        &mut self.server.open,
                        t!("server.title"),
                        &format!("{mod_sym}{shift_sym}S"),
                    );
                    ui.separator();
                    menu_checkbox(
                        ui,
                        &mut self.audio.open,
                        t!("audio.title"),
                        &format!("{mod_sym}{shift_sym}A"),
                    );
                    menu_checkbox(
                        ui,
                        &mut self.devices.open,
                        t!("devices.title"),
                        &format!("{mod_sym}{shift_sym}I"),
                    );
                    menu_checkbox(
                        ui,
                        &mut self.scope_panel.open,
                        t!("scope.title"),
                        &format!("{mod_sym}{shift_sym}O"),
                    );
                    menu_checkbox(
                        ui,
                        &mut self.spectrum_panel.open,
                        t!("spectrum.title"),
                        &format!("{mod_sym}{shift_sym}P"),
                    );
                    menu_checkbox(
                        ui,
                        &mut self.vu_meter_panel.open,
                        t!("cmd.vu_meter"),
                        &format!("{mod_sym}{shift_sym}U"),
                    );
                    menu_checkbox(
                        ui,
                        &mut self.scope_bar_panel.open,
                        t!("cmd.scope_bar"),
                        &format!("{mod_sym}{shift_sym}W"),
                    );
                    menu_checkbox(
                        ui,
                        &mut self.chat_panel.open,
                        t!("chat.title"),
                        &format!("{mod_sym}{shift_sym}C"),
                    );
                    menu_checkbox(
                        ui,
                        &mut self.sample_browser_panel.open,
                        t!("sample_browser.title"),
                        &format!("{mod_sym}{shift_sym}E"),
                    );
                    menu_checkbox(
                        ui,
                        &mut self.doc_panel.open,
                        t!("doc.title"),
                        &format!("{mod_sym}{shift_sym}H"),
                    );
                    menu_checkbox(
                        ui,
                        &mut self.visuals.open,
                        t!("visuals.title"),
                        &format!("{mod_sym}{shift_sym}V"),
                    );
                    ui.separator();
                    let mut logs_expanded = !self.logs.collapsed;
                    let changed = ui.checkbox(&mut logs_expanded, t!("cmd.logs")).changed();
                    if changed {
                        self.logs.collapsed = !logs_expanded;
                    }
                    ui.separator();
                    menu_checkbox(
                        ui,
                        &mut self.options.open,
                        t!("options.title"),
                        &format!("{mod_sym},"),
                    );
                    menu_checkbox(
                        ui,
                        &mut self.debug_open,
                        t!("debug.title"),
                        &format!("{mod_sym}{shift_sym}B"),
                    );
                    ui.separator();
                    if ui.button(t!("menu.keybindings")).clicked() {
                        self.keybindings_open = !self.keybindings_open;
                        ui.close();
                    }
                });
                if r.response.hovered() {
                    widgets::hint::set(ctx, t!("hint.view_menu"));
                }
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

        // Scope bar as bottom panel (must be before VU meter and CentralPanel)
        if self.scope_bar_panel.open && self.bridge.audio_state().running {
            self.scope_bar_panel
                .show_bottom_panel(ctx, self.bridge.scope_data());
        }

        // VU meter as right side panel (must be before CentralPanel)
        if self.vu_meter_panel.open && self.bridge.audio_state().running {
            self.vu_meter_panel
                .show_side_panel(ctx, self.bridge.scope_data());
        }

        // Render visuals shader as background layer
        self.visuals
            .paint_background_central(ctx, self.appearance.visuals_enabled);

        let central_frame = if self.appearance.visuals_enabled {
            egui::Frame::central_panel(&ctx.style()).fill(egui::Color32::from_black_alpha(100))
        } else {
            egui::Frame::central_panel(&ctx.style())
        };

        if self.bridge.is_connected() {
            let mut panels = scene_panel::PanelVisibility {
                server: self.server.open,
                audio: self.audio.open,
                devices: self.devices.open,
                scope: self.scope_panel.open,
                spectrum: self.spectrum_panel.open,
                vu_meter: self.vu_meter_panel.open,
                scope_bar: self.scope_bar_panel.open,
                logs: !self.logs.collapsed,
                options: self.options.open,
                debug: self.debug_open,
            };
            let open_step = egui::CentralPanel::default()
                .frame(central_frame)
                .show(ctx, |ui| {
                    self.scene_panel.show(ui, &self.bridge, &mut panels, self.appearance.visuals_enabled, &self.editor_settings)
                })
                .inner;
            self.server.open = panels.server;
            self.audio.open = panels.audio;
            self.devices.open = panels.devices;
            self.scope_panel.open = panels.scope;
            self.spectrum_panel.open = panels.spectrum;
            self.vu_meter_panel.open = panels.vu_meter;
            self.scope_bar_panel.open = panels.scope_bar;
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
            let action = egui::CentralPanel::default()
                .frame(central_frame)
                .show(ctx, |ui| {
                    self.client
                        .show_centered(ui, &mut self.bridge, self.server.is_running())
                })
                .inner;
            if action.start_server {
                self.server.start(self.audio.initial_audio_config());
            }
            if action.open_server_config {
                self.server.open = true;
            }
        }

        // Floating windows
        let server_action = self.server.show(ctx);
        match server_action {
            ServerAction::Start => {
                self.server.start(self.audio.initial_audio_config());
            }
            ServerAction::Stop => {
                self.bridge.disconnect();
                self.step_editors.close_all();
                self.server.stop();
            }
            ServerAction::None => {}
        }

        self.chat_panel
            .show(ctx, &mut self.bridge, &self.appearance);
        self.audio.show(ctx, &self.bridge);
        self.devices.show(ctx, &self.bridge);

        let sample_paths = self.audio.sample_paths();
        self.sample_browser_panel
            .show(ctx, &self.bridge, sample_paths, &self.appearance);

        let scope_data = self.bridge.scope_data();
        self.scope_panel.show(ctx, scope_data, &self.appearance);
        self.spectrum_panel.show(ctx, scope_data, &self.appearance);

        if self.options.show(
            ctx,
            &mut self.editor_settings,
            &mut self.appearance,
            &mut self.dismissed_tips,
        ) {
            apply_appearance(ctx, &self.appearance);
        }
        self.doc_panel.show(ctx, &self.bridge);
        self.visuals.show_editor(ctx, &self.editor_settings);
        show_debug_window(ctx, &mut self.debug_open);
        show_keybindings_window(ctx, &mut self.keybindings_open);
        widgets::about_dialog(ctx, &mut self.about_open);

        match self.command_palette.show(ctx) {
            widgets::PaletteAction::Execute(cmd) => self.execute_command(cmd),
            widgets::PaletteAction::None => {}
        }

        // Contextual tips (first match wins)
        let tip_id = if self.step_editors.has_open() {
            Some("step_editor")
        } else if self.server.open {
            Some("server")
        } else if self.audio.open {
            Some("audio")
        } else if self.devices.open {
            Some("devices")
        } else if self.scope_panel.open {
            Some("scope")
        } else if self.spectrum_panel.open {
            Some("spectrum")
        } else if self.chat_panel.open {
            Some("chat")
        } else if self.sample_browser_panel.open {
            Some("sample_browser")
        } else if self.doc_panel.open {
            Some("docs")
        } else if self.bridge.is_connected() {
            Some("scene_grid")
        } else {
            Some("welcome")
        };
        if let Some(id) = tip_id {
            widgets::tip_popup::show(ctx, id, &mut self.dismissed_tips);
        }
    }
}

impl SovaApp {
    fn execute_command(&mut self, cmd: widgets::CommandId) {
        use widgets::CommandId::*;
        match cmd {
            Server => self.server.open = !self.server.open,
            Audio => self.audio.open = !self.audio.open,
            Devices => self.devices.open = !self.devices.open,
            Scope => self.scope_panel.open = !self.scope_panel.open,
            Spectrum => self.spectrum_panel.open = !self.spectrum_panel.open,
            VuMeter => self.vu_meter_panel.open = !self.vu_meter_panel.open,
            ScopeBar => self.scope_bar_panel.open = !self.scope_bar_panel.open,
            Chat => self.chat_panel.open = !self.chat_panel.open,
            Logs => self.logs.collapsed = !self.logs.collapsed,
            Options => self.options.open = !self.options.open,
            Debug => self.debug_open = !self.debug_open,
            Keybindings => self.keybindings_open = !self.keybindings_open,
            About => self.about_open = !self.about_open,
            SampleBrowser => self.sample_browser_panel.open = !self.sample_browser_panel.open,
            Documentation => self.doc_panel.open = !self.doc_panel.open,
            Visuals => self.visuals.open = !self.visuals.open,
        }
    }
}

fn show_keybindings_window(ctx: &egui::Context, open: &mut bool) {
    let screen = ctx.content_rect();
    let max_h = screen.height() * 0.8;
    let wide = screen.width() > 700.0;

    egui::Window::new(t!("kb.title"))
        .open(open)
        .resizable(true)
        .collapsible(false)
        .default_width(if wide { 640.0 } else { 340.0 })
        .max_height(max_h)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(screen.center())
        .vscroll(true)
        .show(ctx, |ui| {
            let m = if ctx.os() == egui::os::OperatingSystem::Mac {
                "Cmd"
            } else {
                "Ctrl"
            };

            let row = |ui: &mut egui::Ui, action: std::borrow::Cow<'_, str>, shortcut: String| {
                ui.label(action);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.monospace(shortcut);
                });
                ui.end_row();
            };

            let left = |ui: &mut egui::Ui| {
                ui.heading(t!("kb.general"));
                egui::Grid::new("kb_general")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        row(ui, t!("kb.command_palette"), format!("{m}+K"));
                    });

                ui.add_space(8.0);
                ui.heading(t!("kb.file"));
                egui::Grid::new("kb_file")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        row(ui, t!("kb.save_scene"), format!("{m}+S"));
                        row(ui, t!("kb.load_scene"), format!("{m}+O"));
                    });

                ui.add_space(8.0);
                ui.heading(t!("kb.transport"));
                egui::Grid::new("kb_transport")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        row(ui, t!("kb.play_pause"), format!("{m}+Shift+Space"));
                    });

                ui.add_space(8.0);
                ui.heading(t!("kb.panels"));
                egui::Grid::new("kb_panels")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        row(ui, t!("options.title"), format!("{m}+,"));
                        row(ui, t!("server.title"), format!("{m}+Shift+S"));
                        row(ui, t!("audio.title"), format!("{m}+Shift+A"));
                        row(ui, t!("devices.title"), format!("{m}+Shift+I"));
                        row(ui, t!("scope.title"), format!("{m}+Shift+O"));
                        row(ui, t!("spectrum.title"), format!("{m}+Shift+P"));
                        row(ui, t!("cmd.vu_meter"), format!("{m}+Shift+U"));
                        row(ui, t!("cmd.scope_bar"), format!("{m}+Shift+W"));
                        row(ui, t!("chat.title"), format!("{m}+Shift+C"));
                        row(ui, t!("cmd.logs"), format!("{m}+Shift+L"));
                        row(ui, t!("sample_browser.title"), format!("{m}+Shift+E"));
                        row(ui, t!("doc.title"), format!("{m}+Shift+H"));
                        row(ui, t!("visuals.title"), format!("{m}+Shift+V"));
                        row(ui, t!("debug.title"), format!("{m}+Shift+B"));
                        row(ui, t!("kb.title"), "F1".into());
                    });
            };

            let right = |ui: &mut egui::Ui| {
                ui.heading(t!("kb.scene_grid"));
                egui::Grid::new("kb_scene")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        row(ui, t!("kb.navigate"), "Arrow keys".into());
                        row(ui, t!("kb.edit_step"), "Enter".into());
                        row(ui, t!("kb.edit_duration"), "D".into());
                        row(ui, t!("kb.edit_repetitions"), "R".into());
                        row(ui, t!("kb.rename_frame"), "N".into());
                        row(ui, t!("kb.edit_speed"), "S".into());
                        row(ui, t!("kb.toggle_looping"), "L".into());
                        row(ui, t!("kb.toggle_trailing"), "T".into());
                        row(ui, t!("kb.cancel"), "Escape".into());
                        row(ui, t!("kb.delete_step"), "Delete".into());
                        row(ui, t!("kb.select_all"), format!("{m}+A"));
                        row(ui, t!("kb.copy"), format!("{m}+C"));
                        row(ui, t!("kb.cut"), format!("{m}+X"));
                        row(ui, t!("kb.paste"), format!("{m}+V"));
                        row(ui, t!("kb.duplicate"), format!("{m}+D"));
                        row(ui, t!("kb.duplicate_line"), format!("{m}+Shift+D"));
                        row(ui, t!("kb.extend_selection"), "Shift+Up/Down".into());
                        row(ui, t!("kb.move_frame"), "Alt+Up/Down".into());
                        row(ui, t!("kb.move_line"), "Alt+Left/Right".into());
                        row(ui, t!("kb.delete_line"), format!("{m}+Delete"));
                    });

                ui.add_space(8.0);
                ui.heading(t!("kb.code_editor"));
                egui::Grid::new("kb_editor")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        row(ui, t!("kb.search"), format!("{m}+F"));
                        row(ui, t!("kb.evaluate"), format!("{m}+Enter"));
                    });
            };

            if wide {
                ui.columns(2, |cols| {
                    left(&mut cols[0]);
                    right(&mut cols[1]);
                });
            } else {
                left(ui);
                right(ui);
            }
        });
}

fn show_debug_window(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new(t!("debug.title"))
        .open(open)
        .resizable(true)
        .collapsible(true)
        .default_width(320.0)
        .vscroll(true)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new(t!("debug.settings")).show(ui, |ui| ctx.settings_ui(ui));
            egui::CollapsingHeader::new(t!("debug.inspection"))
                .show(ui, |ui| ctx.inspection_ui(ui));
            egui::CollapsingHeader::new(t!("debug.textures")).show(ui, |ui| ctx.texture_ui(ui));
            egui::CollapsingHeader::new(t!("debug.memory")).show(ui, |ui| ctx.memory_ui(ui));
        });
}
