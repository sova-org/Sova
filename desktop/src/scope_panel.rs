use eframe::egui;

use crate::settings::{AppearanceSettings, ScopeSettings};
use crate::widgets::{self, Waveform};

pub struct ScopePanel {
    pub open: bool,
    pub settings: ScopeSettings,
    smoothed: Vec<f32>,
}

impl ScopePanel {
    pub fn new(settings: ScopeSettings) -> Self {
        Self {
            open: false,
            settings,
            smoothed: Vec::new(),
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        scope_data: &[f32],
        appearance: &AppearanceSettings,
    ) {
        if !self.open {
            return;
        }
        if self.settings.detached {
            self.show_detached(ctx, scope_data, appearance);
        } else {
            self.show_embedded(ctx, scope_data);
        }
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        use crate::widgets::hint;
        let r = ui.add(
            egui::Slider::new(&mut self.settings.smoothing, 0.0..=0.99).text(t!("scope.smoothing").as_ref()),
        );
        hint::on_hover(ui.ctx(), &r, t!("scope.hint.smoothing"));
        let r = ui.add(
            egui::Slider::new(&mut self.settings.stroke_width, 0.5..=4.0).text(t!("scope.stroke").as_ref()),
        );
        hint::on_hover(ui.ctx(), &r, t!("scope.hint.stroke"));
        let r = ui.add(
            egui::Slider::new(&mut self.settings.fill_alpha, 0.0..=1.0).text(t!("scope.fill").as_ref()),
        );
        hint::on_hover(ui.ctx(), &r, t!("scope.hint.fill"));
    }

    fn content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, scope_data: &[f32]) {
        if scope_data.is_empty() {
            ui.colored_label(egui::Color32::GRAY, t!("scope.no_data"));
            return;
        }

        let accent = ui.visuals().selection.bg_fill;
        let stroke_width = self.settings.stroke_width;
        let fill_alpha = self.settings.fill_alpha;
        let a = self.settings.smoothing;

        let data: &[f32] = if a > 0.0 {
            self.smoothed.resize(scope_data.len(), 0.0);
            for (i, &s) in scope_data.iter().enumerate() {
                self.smoothed[i] = self.smoothed[i] * a + s * (1.0 - a);
            }
            &self.smoothed
        } else {
            scope_data
        };

        Waveform::new(data, accent)
            .stroke_width(stroke_width)
            .fill_alpha(fill_alpha)
            .show(ui);
        ctx.request_repaint();
    }

    fn show_embedded(&mut self, ctx: &egui::Context, scope_data: &[f32]) {
        let mut open = self.open;
        egui::Window::new(t!("scope.title"))
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_size([400.0, 150.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let r = ui.button(crate::icons::POPOUT).on_hover_text(t!("common.pop_out"));
                    if r.hovered() {
                        crate::widgets::hint::set(ctx, t!("scope.hint.detach"));
                    }
                    if r.clicked() {
                        self.settings.detached = true;
                    }
                    egui::CollapsingHeader::new(t!("common.settings"))
                        .default_open(false)
                        .show(ui, |ui| self.settings_ui(ui));
                });
                self.content(ui, ctx, scope_data);
            });
        self.open = open;
    }

    fn show_detached(
        &mut self,
        ctx: &egui::Context,
        scope_data: &[f32],
        appearance: &AppearanceSettings,
    ) {
        let mut open = self.open;
        let mut detached = self.settings.detached;
        widgets::show_detached_viewport(
            ctx,
            &mut open,
            &mut detached,
            "scope_viewport",
            &t!("scope.detached_title"),
            [400.0, 200.0],
            appearance,
            |ui| {
                egui::CollapsingHeader::new(t!("common.settings"))
                    .default_open(false)
                    .show(ui, |ui| self.settings_ui(ui));
                self.content(ui, ctx, scope_data);
            },
        );
        self.open = open;
        self.settings.detached = detached;
    }
}
