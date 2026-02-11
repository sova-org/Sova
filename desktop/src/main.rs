mod client_panel;
mod log_panel;
mod server_panel;
mod widgets;

use eframe::egui;

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
    let options = eframe::NativeOptions {
        centered: true,
        viewport: egui::ViewportBuilder::default()
            .with_app_id("sova")
            .with_title("Sova")
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
                _runtime: runtime,
                debug_open: true,
                about_open: false,
            }))
        }),
    )
}

struct SovaApp {
    server: server_panel::ServerPanel,
    client: client_panel::ClientPanel,
    logs: log_panel::LogPanel,
    _runtime: tokio::runtime::Runtime,
    debug_open: bool,
    about_open: bool,
}

impl eframe::App for SovaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.server.poll();
        self.client.poll();
        self.logs.poll();

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if ui.selectable_label(self.about_open, "Sova").clicked() {
                    self.about_open = !self.about_open;
                }
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.server.open, "Server");
                    ui.checkbox(&mut self.client.open, "Client");
                    ui.checkbox(&mut self.logs.open, "Logs");
                    ui.checkbox(&mut self.debug_open, "Debug");
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            widgets::bottom_bar(ui, &self.server.info(), &self.client.info());
        });

        egui::CentralPanel::default().show(ctx, |_ui| {});

        self.server.show(ctx);
        self.client.show(ctx);
        self.logs.show(ctx);
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
