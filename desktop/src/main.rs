//#![windows_subsystem = "windows"]

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "en");

mod audio_panel;
mod chat_panel;
mod client_bridge;
mod client_panel;
mod devices_panel;
mod feedback_engine;
mod fonts;
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
use settings::{AppearanceSettings, DocSide, VisualsSettings};
use sova_core::schedule::{ActionTiming, SchedulerMessage};
use sova_server::ClientMessage;

include!(concat!(env!("OUT_DIR"), "/demos_generated.rs"));

const DEMOS_GENERAL: &[(&str, &[u8])] = &[
    ("Aliens near us", include_bytes!("../assets/demos/demos/aliens_near_us.sova")),
    ("2005 algorave", include_bytes!("../assets/demos/demos/2005_algorave.sova")),
    ("By the pond", include_bytes!("../assets/demos/demos/by_the_pond.sova")),
    ("Classic move", include_bytes!("../assets/demos/demos/classic_move.sova")),
    ("Lush elegiac stuff", include_bytes!("../assets/demos/demos/lush_elegiac_stuff.sova")),
    ("Intense boots and cats", include_bytes!("../assets/demos/demos/intense_boots_and_cats.sova")),
    ("Infinite gongs", include_bytes!("../assets/demos/demos/infinite_gongs.sova")),
    ("Chill 808", include_bytes!("../assets/demos/demos/chill_808.sova")),
    ("Mayo sandwich", include_bytes!("../assets/demos/demos/mayo_sandwich.sova")),
    ("First day with my modular", include_bytes!("../assets/demos/demos/first_day_with_my_modular.sova")),
    ("Bit after bit", include_bytes!("../assets/demos/demos/bit_after_bit.sova")),
    ("Some soup ?", include_bytes!("../assets/demos/demos/some_soup.sova")),
    ("Storm of sand", include_bytes!("../assets/demos/demos/darude.sova")),
];

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

        let bg = a.bg_brightness;
        style.visuals.extreme_bg_color = egui::Color32::from_gray(bg);
        style.visuals.panel_fill = egui::Color32::from_gray(bg.saturating_add(8));
        style.visuals.window_fill = egui::Color32::from_gray(bg.saturating_add(12));

        style.spacing.button_padding = egui::vec2(5.0, 4.0);
        style.spacing.indent_ends_with_horizontal_line = true;

        let ui_size = a.ui_font_size;
        style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(ui_size));
        style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(ui_size));
        style.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional((ui_size - 2.0).max(8.0)));
        style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(ui_size + 7.0));

        style.animation_time = a.animation_time;
    });
}

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
            fonts::apply_custom_fonts(&ctx, &s.appearance.ui_font, &s.appearance.editor_font);

            let server = server_panel::ServerPanel::new(
                handle.clone(),
                log_tx.clone(),
                ctx.clone(),
                s.server,
            );
            let client = client_panel::ClientPanel::new(s.client);
            let logs = log_panel::LogPanel::new(log_rx);
            let saved_master_volume = s.audio.master_volume;
            let audio = audio_panel::AudioPanel::new(s.audio);
            let devices = devices_panel::DevicesPanel::new();
            let options = options_panel::OptionsPanel::new();

            let scope_panel = scope_panel::ScopePanel::new(s.scope);
            let spectrum_panel = spectrum_panel::SpectrumPanel::new(s.spectrum);
            let vu_meter_panel = vu_meter_panel::VuMeterPanel::new();
            let scope_bar_panel = scope_bar_panel::ScopeBarPanel::new(s.windows.scope_bar_height);
            let chat_panel = chat_panel::ChatPanel::new();
            let mut scene_panel = scene_panel::ScenePanel::new();
            scene_panel.prelude_collapsed = s.scene.prelude_collapsed;
            scene_panel.prelude_col_width = s.scene.prelude_col_width;

            let bridge = client_bridge::ClientBridge::new(handle, ctx, log_tx);

            let doc_panel = doc_panel::DocPanel::new(s.doc);
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
                confirm_reset_scene: widgets::ConfirmDialog::new("confirm_reset_scene"),
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
                appearance: s.appearance,
                bridge,
                sample_browser_panel: sample_browser_panel::SampleBrowserPanel::new(),
                doc_panel,
                recent_scenes: s.recent_scenes,
                dismissed_tips: s.dismissed_tips,
                visuals,
                toasts: widgets::ToastStack::new(),
                rename_input: None,
                master_volume: saved_master_volume,
                muted: false,
                was_audio_running: false,
            };

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
    confirm_reset_scene: widgets::ConfirmDialog,
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
    appearance: AppearanceSettings,
    bridge: client_bridge::ClientBridge,
    sample_browser_panel: sample_browser_panel::SampleBrowserPanel,
    doc_panel: doc_panel::DocPanel,
    recent_scenes: Vec<std::path::PathBuf>,
    dismissed_tips: Vec<String>,
    visuals: visuals::VisualsEngine,
    toasts: widgets::ToastStack,
    rename_input: Option<String>,
    master_volume: f32,
    muted: bool,
    was_audio_running: bool,
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
                self.doc_panel.toggle_settings_tab(doc_panel::SettingsTab::Options);
            }
            if i.modifiers.command && i.modifiers.shift {
                if i.key_pressed(egui::Key::S) {
                    self.doc_panel.toggle_settings_tab(doc_panel::SettingsTab::Config);
                }
                if i.key_pressed(egui::Key::A) {
                    self.doc_panel.toggle_settings_tab(doc_panel::SettingsTab::Config);
                }
                if i.key_pressed(egui::Key::I) {
                    self.doc_panel.toggle_settings_tab(doc_panel::SettingsTab::Devices);
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
                    self.doc_panel.toggle_settings_tab(doc_panel::SettingsTab::Logs);
                }
                if i.key_pressed(egui::Key::B) {
                    self.debug_open = !self.debug_open;
                }
                if i.key_pressed(egui::Key::C) {
                    self.chat_panel.open = !self.chat_panel.open;
                }
                if i.key_pressed(egui::Key::E) {
                    let sample_browser_available =
                        !self.bridge.is_connected() || self.server.is_running();
                    if sample_browser_available {
                        self.sample_browser_panel.open = !self.sample_browser_panel.open;
                    }
                }
                if i.key_pressed(egui::Key::H) {
                    self.doc_panel.settings.collapsed = !self.doc_panel.settings.collapsed;
                    if !self.doc_panel.settings.collapsed {
                        self.doc_panel.settings.pinned = true;
                    }
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
                    SchedulerMessage::TransportStop(ActionTiming::Immediate)
                } else {
                    SchedulerMessage::TransportStart(ActionTiming::Immediate)
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
            .send(SchedulerMessage::SetScene(snapshot.scene, timing));
        self.bridge
            .send(SchedulerMessage::SetTempo(snapshot.tempo, timing));
        self.bridge.send(SchedulerMessage::SetQuantum(snapshot.quantum, timing));
        self.push_recent_scene(path.to_path_buf());
    }

    fn load_scene_from_bytes(&mut self, bytes: &[u8], timing: ActionTiming) {
        let Ok(snapshot) = serde_json::from_slice::<sova_server::Snapshot>(bytes) else {
            return;
        };
        self.bridge
            .send(SchedulerMessage::SetScene(snapshot.scene, timing));
        self.bridge
            .send(SchedulerMessage::SetTempo(snapshot.tempo, timing));
        self.bridge.send(SchedulerMessage::SetQuantum(snapshot.quantum, timing));
    }

    fn push_recent_scene(&mut self, path: std::path::PathBuf) {
        self.recent_scenes.retain(|p| p != &path);
        self.recent_scenes.insert(0, path);
        self.recent_scenes.truncate(10);
    }

    fn save_settings(&self) {
        let s = settings::AppSettings {
            windows: settings::WindowSettings {
                chat_detached: self.chat_panel.detached,
                sample_browser_detached: self.sample_browser_panel.detached,
                scope_bar_height: self.scope_bar_panel.height(),
            },
            editor: self.editor_settings.clone(),
            server: self.server.settings(),
            client: self.client.settings(),
            audio: {
                let mut audio = self.audio.settings();
                audio.master_volume = self.master_volume;
                audio
            },
            appearance: self.appearance.clone(),
            scope: self.scope_panel.settings.clone(),
            spectrum: self.spectrum_panel.settings.clone(),
            visuals: VisualsSettings {
                code: self.visuals.code().to_owned(),
                shared: self.visuals.shared,
            },
            doc: self.doc_panel.settings.clone(),
            scene: settings::SceneSettings {
                prelude_collapsed: self.scene_panel.prelude_collapsed,
                prelude_col_width: self.scene_panel.prelude_col_width,
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
                self.bridge.disconnect();
                self.server.stop();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            widgets::ConfirmAction::Cancelled | widgets::ConfirmAction::None => {}
        }

        if let widgets::ConfirmAction::Confirmed = self.confirm_reset_scene.show(ctx) {
            self.bridge
                .send(ClientMessage::ResetScene(ActionTiming::Immediate));
        }

        self.server.poll();
        self.bridge.poll();
        if self.bridge.just_connected {
            self.bridge.just_connected = false;
            self.bridge.send(ClientMessage::SetMasterVolume(self.effective_gain()));
        }
        let audio_running = self.bridge.audio_state().running || self.bridge.has_feedback();
        if audio_running && !self.was_audio_running {
            self.scope_bar_panel.open = true;
            self.vu_meter_panel.open = true;
        }
        self.was_audio_running = audio_running;
        self.logs.poll();

        egui::TopBottomPanel::top("menu_bar")
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .inner_margin(egui::Margin::symmetric(8, 4)),
            )
            .show(ctx, |ui| {
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
                    if ui
                        .add_enabled(connected, egui::Button::new(t!("menu.reset_scene")))
                        .clicked()
                    {
                        ui.close();
                        self.confirm_reset_scene.open(
                            t!("reset_scene.title"),
                            t!("reset_scene.message"),
                        );
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
                    ui.add_enabled_ui(connected, |ui| {
                        ui.menu_button(t!("menu.demos"), |ui| {
                            let mut demo_submenu = |label: &str, demos: &[(&str, &[u8])]| {
                                ui.add_enabled_ui(!demos.is_empty(), |ui| {
                                    ui.menu_button(label, |ui| {
                                        // Split demos into groups at separator boundaries
                                        let mut groups: Vec<Vec<(&str, &[u8])>> = vec![vec![]];
                                        for (name, bytes) in demos {
                                            if *name == "\x00" {
                                                groups.push(vec![]);
                                            } else {
                                                groups.last_mut().unwrap().push((name, bytes));
                                            }
                                        }
                                        groups.retain(|g| !g.is_empty());

                                        let total_items: usize = groups.iter().map(|g| g.len()).sum();
                                        let n_cols = if total_items > 30 {
                                            3
                                        } else if total_items > 15 {
                                            2
                                        } else {
                                            1
                                        };

                                        if n_cols == 1 {
                                            for (gi, group) in groups.iter().enumerate() {
                                                if gi > 0 {
                                                    ui.separator();
                                                }
                                                for (name, bytes) in group {
                                                    if ui.button(*name).clicked() {
                                                        self.load_scene_from_bytes(bytes, ActionTiming::Immediate);
                                                        ui.close();
                                                    }
                                                }
                                            }
                                        } else {
                                            // Distribute groups across columns, balancing item count
                                            let target = total_items.div_ceil(n_cols);
                                            let mut columns: Vec<Vec<usize>> = vec![vec![]; n_cols];
                                            let mut col = 0;
                                            let mut col_count = 0;
                                            for (gi, group) in groups.iter().enumerate() {
                                                columns[col].push(gi);
                                                col_count += group.len();
                                                if col_count >= target && col + 1 < n_cols {
                                                    col += 1;
                                                    col_count = 0;
                                                }
                                            }

                                            ui.columns(n_cols, |cols| {
                                                for (ci, col_groups) in columns.iter().enumerate() {
                                                    for (i, &gi) in col_groups.iter().enumerate() {
                                                        if i > 0 {
                                                            cols[ci].separator();
                                                        }
                                                        for (name, bytes) in &groups[gi] {
                                                            if cols[ci].button(*name).clicked() {
                                                                self.load_scene_from_bytes(bytes, ActionTiming::Immediate);
                                                                cols[ci].close();
                                                            }
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    });
                                });
                            };
                            demo_submenu("Cagire", DEMOS_CAGIRE);
                            demo_submenu("Boinx", DEMOS_BOINX);
                            demo_submenu("Demos", DEMOS_GENERAL);
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
                let is_mac = ctx.os() == egui::os::OperatingSystem::Mac;
                let (mod_sym, shift_sym) = if is_mac {
                    ("⌘", "⇧")
                } else {
                    ("Ctrl+", "Shift+")
                };
                let r = ui.menu_button(t!("menu.server"), |ui| {
                    if self.server.is_running() {
                        if ui.button(t!("menu.stop_server")).clicked() {
                            ui.close();
                            self.bridge.disconnect();
                            self.server.stop();
                        }
                    } else if ui.button(t!("menu.start_server")).clicked() {
                        ui.close();
                        self.server.start(self.audio.generate_audio_config());
                    }
                    if self.bridge.is_connected() {
                        ui.separator();
                        if ui.button(t!("common.disconnect")).clicked() {
                            ui.close();
                            self.bridge.disconnect();
                        }
                        if let Some(input) = &mut self.rename_input {
                            let r = ui.text_edit_singleline(input);
                            if r.lost_focus() {
                                if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !input.trim().is_empty() {
                                    let new_name = input.trim().to_owned();
                                    self.bridge.send(ClientMessage::SetName { name: new_name.clone(), password: None });
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
                        self.bridge.restart_audio(self.audio.generate_audio_config());
                    }
                    if ui
                        .add_enabled(enabled, egui::Button::new(t!("menu.restart_core")))
                        .clicked()
                    {
                        ui.close();
                        self.bridge.send(ClientMessage::RestartCore);
                    }
                });
                if r.response.hovered() {
                    widgets::hint::set(ctx, t!("hint.engine_menu"));
                }

                let r = ui.menu_button(t!("menu.audio"), |ui| {
                    let menu_checkbox = |ui: &mut egui::Ui,
                                         checked: &mut bool,
                                         label: std::borrow::Cow<'_, str>,
                                         shortcut: &str| {
                        let text = egui::RichText::new(shortcut).weak();
                        ui.checkbox(checked, label).on_hover_text(text);
                    };

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
                });
                if r.response.hovered() {
                    widgets::hint::set(ctx, t!("hint.audio_menu"));
                }
                let r = ui.menu_button(t!("menu.view"), |ui| {
                    let menu_checkbox = |ui: &mut egui::Ui,
                                         checked: &mut bool,
                                         label: std::borrow::Cow<'_, str>,
                                         shortcut: &str| {
                        let text = egui::RichText::new(shortcut).weak();
                        ui.checkbox(checked, label).on_hover_text(text);
                    };

                    menu_checkbox(
                        ui,
                        &mut self.chat_panel.open,
                        t!("chat.title"),
                        &format!("{mod_sym}{shift_sym}C"),
                    );
                    {
                        let sample_browser_available =
                            !self.bridge.is_connected() || self.server.is_running();
                        ui.add_enabled_ui(sample_browser_available, |ui| {
                            menu_checkbox(
                                ui,
                                &mut self.sample_browser_panel.open,
                                t!("sample_browser.title"),
                                &format!("{mod_sym}{shift_sym}E"),
                            );
                        });
                    }
                    menu_checkbox(
                        ui,
                        &mut self.visuals.open,
                        t!("visuals.title"),
                        &format!("{mod_sym}{shift_sym}V"),
                    );
                    ui.separator();
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

                if self.bridge.is_connected() {
                    ui.separator();
                    if let Some(transport_bar::TransportAction::Panic) =
                        self.transport_bar.show_inline(ui, ctx, &self.bridge)
                    {
                        self.muted = true;
                        self.bridge.send(ClientMessage::SetMasterVolume(0.0));
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let audio = self.bridge.audio_state();
                    if audio.running {
                        let cpu_pct = audio.cpu_load * 100.0;
                        let cpu_color = if cpu_pct >= 80.0 {
                            egui::Color32::from_rgb(255, 80, 80)
                        } else if cpu_pct >= 50.0 {
                            egui::Color32::from_rgb(255, 180, 50)
                        } else {
                            ui.visuals().text_color()
                        };
                        let text = format!("V:{}  CPU {:.0}%", audio.active_voices, cpu_pct);
                        ui.colored_label(cpu_color, text);
                    }

                    let mut vol = if self.muted { 0.0 } else { self.master_volume };
                    let slider = egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false);
                    let r = ui.add_sized([100.0, ui.available_height()], slider);
                    if r.changed() {
                        self.master_volume = vol;
                        self.muted = false;
                        self.bridge
                            .send(ClientMessage::SetMasterVolume(self.effective_gain()));
                    }
                    if r.hovered() {
                        widgets::hint::set(ctx, t!("hint.master_volume"));
                    }

                    let icon = if self.muted || self.master_volume == 0.0 {
                        icons::MUTE
                    } else {
                        icons::UNMUTE
                    };
                    let btn = ui.button(icon);
                    if btn.clicked() {
                        self.muted = !self.muted;
                        self.bridge.send(ClientMessage::SetMasterVolume(self.effective_gain()));
                    }
                    if btn.hovered() {
                        let hint = if self.muted {
                            t!("hint.unmute")
                        } else {
                            t!("hint.mute")
                        };
                        widgets::hint::set(ctx, hint);
                    }
                });
            });
        });

        if let Some((msg, _)) = self.bridge.last_error.take() {
            self.toasts.push(widgets::ToastLevel::Error, msg);
        }

        let bar = egui::TopBottomPanel::bottom("bottom_bar")
            .show(ctx, |ui| {
                widgets::bottom_bar(ui, &self.server.info(), &self.client.info(&self.bridge))
            })
            .inner;
        if bar.open_palette {
            self.command_palette.open();
        }
        if bar.disconnect {
            self.bridge.disconnect();
        }

        // Scope bar as bottom panel (must be before VU meter and CentralPanel)
        if self.scope_bar_panel.open && (self.bridge.audio_state().running || self.bridge.has_feedback()) {
            self.scope_bar_panel
                .show_bottom_panel(ctx, self.bridge.scope_data(), &self.scope_panel.settings);
        }

        // Sidebar (docs + settings + logs, must be before VU meter and CentralPanel)
        let settings_ctx = doc_panel::SettingsContext {
            server: &mut self.server,
            audio: &mut self.audio,
            options: &mut self.options,
            devices: &mut self.devices,
            logs: &mut self.logs,
            editor_settings: &mut self.editor_settings,
            appearance: &mut self.appearance,
            dismissed_tips: &mut self.dismissed_tips,
        };
        let (sidebar_server_action, sidebar_appearance_changed) = self.doc_panel.show_side_panel(
            ctx,
            &self.bridge,
            settings_ctx,
        );
        match sidebar_server_action {
            ServerAction::Start => {
                self.server.start(self.audio.generate_audio_config());
            }
            ServerAction::Stop => {
                self.bridge.disconnect();
                self.server.stop();
            }
            ServerAction::None => {}
        }
        if sidebar_appearance_changed {
            apply_appearance(ctx, &self.appearance);
            fonts::apply_custom_fonts(ctx, &self.appearance.ui_font, &self.appearance.editor_font);
        }

        // VU meter on opposite side of doc panel (must be before CentralPanel)
        if self.vu_meter_panel.open && (self.bridge.audio_state().running || self.bridge.has_feedback()) {
            let vu_side = match self.doc_panel.settings.side {
                DocSide::Left => egui::containers::panel::Side::Right,
                DocSide::Right => egui::containers::panel::Side::Left,
            };
            self.vu_meter_panel
                .show_side_panel(ctx, self.bridge.peak_data(), vu_side);
        }

        // Render visuals shader as background layer
        let clock = self.bridge.clock();
        self.visuals.paint_background_central(
            ctx,
            self.appearance.visuals_enabled,
            clock.beat as f32,
            clock.tempo as f32,
            clock.phase as f32,
        );

        let central_frame = if self.appearance.visuals_enabled {
            egui::Frame::central_panel(&ctx.style()).fill(egui::Color32::TRANSPARENT)
        } else {
            egui::Frame::central_panel(&ctx.style())
        };

        if self.bridge.is_connected() {
            let sidebar_open = self.doc_panel.is_expanded()
                && self.doc_panel.mode() == doc_panel::SidebarMode::Settings;
            let panels = scene_panel::PanelVisibility {
                sidebar: sidebar_open,
                devices: self.devices.open,
                scope: self.scope_panel.open,
                spectrum: self.spectrum_panel.open,
                vu_meter: self.vu_meter_panel.open,
                scope_bar: self.scope_bar_panel.open,
                logs: self.doc_panel.is_logs_open(),
                debug: self.debug_open,
            };
            egui::CentralPanel::default()
                .frame(central_frame)
                .show(ctx, |ui| {
                    let pending_edits: Vec<_> = self.bridge.pending_script_edits.drain(..).collect();
                    self.scene_panel.show(ui, &self.bridge, self.appearance.visuals_enabled, self.appearance.scene_opacity, &self.editor_settings, pending_edits);
                });
            if panels.sidebar != sidebar_open {
                if panels.sidebar {
                    self.doc_panel.open_settings_tab(doc_panel::SettingsTab::Config);
                } else {
                    self.doc_panel.settings.collapsed = true;
                }
            }
            self.devices.open = panels.devices;
            self.scope_panel.open = panels.scope;
            self.spectrum_panel.open = panels.spectrum;
            self.vu_meter_panel.open = panels.vu_meter;
            self.scope_bar_panel.open = panels.scope_bar;
            if panels.logs != self.doc_panel.is_logs_open() {
                self.doc_panel.toggle_settings_tab(doc_panel::SettingsTab::Logs);
            }
            self.debug_open = panels.debug;
        } else {
            let action = egui::CentralPanel::default()
                .frame(central_frame)
                .show(ctx, |ui| {
                    self.client
                        .show_centered(ui, &mut self.bridge, self.server.is_running())
                })
                .inner;
            if action.start_server {
                self.server.start(self.audio.generate_audio_config());
            }
            if action.stop_server {
                self.server.stop();
            }
            if action.open_server_config {
                self.doc_panel.open_settings_tab(doc_panel::SettingsTab::Config);
            }
            if action.start_feedback && !self.bridge.has_feedback() {
                self.bridge.start_feedback(self.audio.generate_audio_config());
            }
        }

        // Floating windows
        self.chat_panel
            .show(ctx, &mut self.bridge, &self.appearance);
        self.toasts.poll_chat(self.bridge.chat_messages());
        self.toasts.show(ctx);
        self.devices.show(ctx, &self.bridge);

        let sample_paths = self.audio.sample_paths();
        #[cfg(feature = "default-samples")]
        let default_sample_path = Some(self.audio.default_samples_path());
        #[cfg(not(feature = "default-samples"))]
        let default_sample_path: Option<&std::path::Path> = None;
        let is_hosting = self.server.is_running();
        self.sample_browser_panel
            .show(ctx, &self.bridge, default_sample_path, sample_paths, &self.appearance, is_hosting);

        let scope_data = self.bridge.scope_data();
        self.scope_panel.show(ctx, scope_data, &self.appearance);
        self.spectrum_panel.show(ctx, scope_data, &self.appearance);

        self.visuals.show_editor(ctx, &self.editor_settings);

        if self.visuals.take_pending_broadcast() {
            self.bridge.send_hydra_code(self.visuals.code());
        }
        if self.visuals.shared
            && let Some((sender, code)) = self.bridge.take_remote_hydra()
        {
            self.visuals.remote_sender = Some(sender);
            self.visuals.apply_remote_code(&code);
        }

        show_debug_window(ctx, &mut self.debug_open);
        show_keybindings_window(ctx, &mut self.keybindings_open);
        widgets::about_dialog(ctx, &mut self.about_open);

        self.command_palette.update_states(&widgets::PanelStates {
            sidebar: self.doc_panel.is_expanded() && self.doc_panel.mode() == doc_panel::SidebarMode::Settings,
            devices: self.devices.open,
            scope: self.scope_panel.open,
            spectrum: self.spectrum_panel.open,
            vu_meter: self.vu_meter_panel.open,
            scope_bar: self.scope_bar_panel.open,
            chat: self.chat_panel.open,
            logs: self.doc_panel.is_logs_open(),
            debug: self.debug_open,
            keybindings: self.keybindings_open,
            about: self.about_open,
            sample_browser: self.sample_browser_panel.open,
            documentation: !self.doc_panel.settings.collapsed,
            visuals: self.visuals.open,
        });
        match self.command_palette.show(ctx) {
            widgets::PaletteAction::Execute(cmd) => self.execute_command(cmd),
            widgets::PaletteAction::None => {}
        }

        // Contextual tips (first match wins)
        let tip_id = if self.doc_panel.is_expanded() && self.doc_panel.mode() == doc_panel::SidebarMode::Settings {
            Some("settings")
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
        } else if !self.doc_panel.settings.collapsed {
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
    fn effective_gain(&self) -> f32 {
        if self.muted { 0.0 } else { self.master_volume * self.master_volume }
    }

    fn execute_command(&mut self, cmd: widgets::CommandId) {
        use widgets::CommandId::*;
        match cmd {
            Server => self.doc_panel.toggle_settings_tab(doc_panel::SettingsTab::Config),
            Audio => self.doc_panel.toggle_settings_tab(doc_panel::SettingsTab::Config),
            Devices => self.doc_panel.toggle_settings_tab(doc_panel::SettingsTab::Devices),
            Scope => self.scope_panel.open = !self.scope_panel.open,
            Spectrum => self.spectrum_panel.open = !self.spectrum_panel.open,
            VuMeter => self.vu_meter_panel.open = !self.vu_meter_panel.open,
            ScopeBar => self.scope_bar_panel.open = !self.scope_bar_panel.open,
            Chat => self.chat_panel.open = !self.chat_panel.open,
            Logs => self.doc_panel.toggle_settings_tab(doc_panel::SettingsTab::Logs),
            Options => self.doc_panel.toggle_settings_tab(doc_panel::SettingsTab::Options),
            Debug => self.debug_open = !self.debug_open,
            Keybindings => self.keybindings_open = !self.keybindings_open,
            About => self.about_open = !self.about_open,
            SampleBrowser => {
                let sample_browser_available =
                    !self.bridge.is_connected() || self.server.is_running();
                if sample_browser_available {
                    self.sample_browser_panel.open = !self.sample_browser_panel.open;
                }
            }
            Documentation => {
                self.doc_panel.settings.collapsed = !self.doc_panel.settings.collapsed;
                if !self.doc_panel.settings.collapsed {
                    self.doc_panel.settings.pinned = true;
                }
            }
            Visuals => self.visuals.open = !self.visuals.open,
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
                    self.confirm_reset_scene.open(
                        t!("reset_scene.title"),
                        t!("reset_scene.message"),
                    );
                }
            }
            ZoomIn => {
                self.appearance.zoom = (self.appearance.zoom + 0.1).min(3.0);
            }
            ZoomOut => {
                self.appearance.zoom = (self.appearance.zoom - 0.1).max(0.5);
            }
            ZoomReset => {
                self.appearance.zoom = 1.0;
            }
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
                ui.heading(t!("kb.scene_nav"));
                egui::Grid::new("kb_scene_nav")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        row(ui, t!("kb.navigate"), "Arrow keys".into());
                        row(ui, t!("kb.navigate_vim"), "h / j / k / l".into());
                        row(ui, t!("kb.enter_edit"), "Enter / i".into());
                        row(ui, t!("kb.exit_edit"), "Escape".into());
                        row(ui, t!("kb.extend_selection"), "Shift+Up/Down".into());
                        row(ui, t!("kb.duplicate_after"), format!("{m}+D"));
                        row(ui, t!("kb.duplicate_before"), format!("{m}+Shift+D"));
                        row(ui, t!("kb.insert_after"), "Shift+I".into());
                        row(ui, t!("kb.insert_before"), format!("{m}+Shift+I"));
                        row(ui, t!("kb.delete_frame"), "Delete".into());
                        row(ui, t!("kb.move_frame_down"), "Shift+J".into());
                        row(ui, t!("kb.move_frame_up"), "Shift+K".into());
                        row(ui, t!("kb.toggle_enabled"), "e".into());
                        row(ui, t!("kb.toggle_looping"), ".".into());
                        row(ui, t!("kb.toggle_trailing"), ",".into());
                        row(ui, t!("kb.move_line_left"), "Alt+H".into());
                        row(ui, t!("kb.move_line_right"), "Alt+L".into());
                        row(ui, t!("kb.select_all"), format!("{m}+A"));
                        row(ui, t!("kb.copy"), format!("{m}+C"));
                        row(ui, t!("kb.cut"), format!("{m}+X"));
                        row(ui, t!("kb.paste"), format!("{m}+V"));
                        row(ui, t!("kb.delete_line"), format!("{m}+Delete"));
                    });

                ui.add_space(8.0);
                ui.heading(t!("kb.scene_edit"));
                egui::Grid::new("kb_scene_edit")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        row(ui, t!("kb.exit_edit"), "Escape".into());
                        row(ui, t!("kb.evaluate"), format!("{m}+Enter"));
                        row(ui, t!("kb.lang_selector"), format!("{m}+L"));
                        row(ui, t!("kb.search"), format!("{m}+F"));
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
