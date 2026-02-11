mod audio_panel;
mod client_panel;
mod devices_panel;
mod log_panel;
mod options_panel;
mod server_panel;
mod widgets;

use eframe::egui;
use server_panel::ServerAction;
use std::sync::Arc;

fn apply_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let zero = egui::CornerRadius::ZERO;
    style.visuals.window_corner_radius = zero;
    style.visuals.menu_corner_radius = zero;
    style.visuals.widgets.noninteractive.corner_radius = zero;
    style.visuals.widgets.inactive.corner_radius = zero;
    style.visuals.widgets.hovered.corner_radius = zero;
    style.visuals.widgets.active.corner_radius = zero;
    style.visuals.widgets.open.corner_radius = zero;
    ctx.set_style(style);
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
            apply_style(&ctx);
            egui_extras::install_image_loaders(&ctx);

            let runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            let handle = runtime.handle().clone();

            let (log_tx, log_rx) = std::sync::mpsc::channel();

            Ok(Box::new(SovaApp {
                server: server_panel::ServerPanel::new(handle.clone(), log_tx.clone(), ctx.clone()),
                client: client_panel::ClientPanel::new(ctx, handle, log_tx),
                logs: log_panel::LogPanel::new(log_rx),
                audio: audio_panel::AudioPanel::new(),
                devices: devices_panel::DevicesPanel::new(),
                _runtime: runtime,
                debug_open: true,
                about_open: false,
                confirm_exit: widgets::ConfirmDialog::new(),
                options: options_panel::OptionsPanel::new(),
                editor_settings: widgets::EditorSettings::default(),
                editor_open: true,
                editor: widgets::CodeEditor::new(),
                editor_text: "fn main() {\n    println!(\"Hello, world!\");\n    let x = 42;\n    let y = x + 1;\n}\n".to_owned(),
            }))
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
    editor_settings: widgets::EditorSettings,
    editor_open: bool,
    editor: widgets::CodeEditor,
    editor_text: String,
}

impl eframe::App for SovaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) && self.server.is_running() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            if !self.confirm_exit.is_open() {
                self.confirm_exit.open(
                    "Exit Sova",
                    "The server is still running. Are you sure you want to exit?",
                );
            }
        }

        match self.confirm_exit.show(ctx) {
            widgets::ConfirmAction::Confirmed => {
                self.audio.stop();
                self.server.stop();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            widgets::ConfirmAction::Cancelled | widgets::ConfirmAction::None => {}
        }

        self.server.poll();
        self.client.poll();
        self.logs.poll();

        if !self.server.is_running() && self.audio.is_running() {
            self.audio.stop();
        }

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
                    ui.checkbox(&mut self.audio.open, "Audio");
                    ui.checkbox(&mut self.devices.open, "Devices");
                    ui.checkbox(&mut self.client.open, "Client");
                    ui.checkbox(&mut self.logs.open, "Logs");
                    ui.checkbox(&mut self.editor_open, "Editor");
                    ui.checkbox(&mut self.debug_open, "Debug");
                    ui.separator();
                    ui.checkbox(&mut self.options.open, "Options");
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            widgets::bottom_bar(ui, &self.server.info(), &self.client.info());
        });

        egui::CentralPanel::default().show(ctx, |_ui| {});

        let server_action = self.server.show(ctx);
        match server_action {
            ServerAction::Start => {
                self.server
                    .start(Arc::clone(&self.audio.audio_engine_state));
            }
            ServerAction::Stop => {
                self.audio.stop();
                self.server.stop();
            }
            ServerAction::None => {}
        }

        self.client.show(ctx);
        self.logs.show(ctx);

        let resources = self.server.server_resources();
        self.audio.show(ctx, resources.as_ref());
        self.devices.show(ctx, resources.as_ref());

        show_editor_window(
            ctx,
            &mut self.editor_open,
            &mut self.editor,
            &mut self.editor_text,
            &self.editor_settings,
        );
        self.options.show(ctx, &mut self.editor_settings);
        show_debug_window(ctx, &mut self.debug_open);
        widgets::about_dialog(ctx, &mut self.about_open);
    }
}

fn show_editor_window(
    ctx: &egui::Context,
    open: &mut bool,
    editor: &mut widgets::CodeEditor,
    text: &mut String,
    settings: &widgets::EditorSettings,
) {
    egui::Window::new("Editor")
        .open(open)
        .resizable(true)
        .default_width(500.0)
        .default_height(400.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                editor.show(ui, egui::Id::new("test_editor"), text, settings);
            });
        });
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
