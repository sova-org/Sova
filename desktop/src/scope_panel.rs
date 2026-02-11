use eframe::egui;

use crate::widgets::Scope;

pub struct ScopePanel {
    pub open: bool,
}

impl ScopePanel {
    pub fn new() -> Self {
        Self { open: false }
    }

    pub fn show(&mut self, ctx: &egui::Context, scope_data: &[(f32, f32)]) {
        let mut open = self.open;
        egui::Window::new("Scope")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_size([400.0, 150.0])
            .show(ctx, |ui| {
                if scope_data.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "No audio data");
                } else {
                    let samples: Vec<f32> = scope_data.iter().map(|(l, _)| *l).collect();
                    let accent = ui.visuals().selection.bg_fill;
                    Scope::new(&samples, accent).show(ui);
                    ctx.request_repaint();
                }
            });
        self.open = open;
    }
}
