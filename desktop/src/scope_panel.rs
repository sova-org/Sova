use std::sync::Arc;

use eframe::egui;
use sova_server::audio::ScopeCapture;

use crate::widgets::Scope;

pub struct ScopePanel {
    pub open: bool,
}

impl ScopePanel {
    pub fn new() -> Self {
        Self { open: false }
    }

    pub fn show(&mut self, ctx: &egui::Context, scope: Option<Arc<ScopeCapture>>) {
        let mut open = self.open;
        egui::Window::new("Scope")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_size([400.0, 150.0])
            .show(ctx, |ui| match scope {
                None => {
                    ui.colored_label(egui::Color32::GRAY, "Audio not running");
                }
                Some(scope) => {
                    let samples = scope.read_samples();
                    let accent = ui.visuals().selection.bg_fill;
                    Scope::new(&samples, accent).show(ui);
                    ctx.request_repaint();
                }
            });
        self.open = open;
    }
}
