use eframe::egui;

use crate::settings::{AppearanceSettings, ScopeSettings};
use crate::widgets::{self, Waveform};

pub struct ScopePanel {
    pub open: bool,
    pub settings: ScopeSettings,
    line_buffer: Vec<f32>,
    trace: Vec<(f32, f32)>,
}

impl ScopePanel {
    pub fn new(settings: ScopeSettings) -> Self {
        Self {
            open: false,
            settings,
            line_buffer: Vec::new(),
            trace: Vec::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, aligned: &[f32], appearance: &AppearanceSettings) {
        if !self.open {
            return;
        }
        if self.settings.detached {
            self.show_detached(ctx, aligned, appearance);
        } else {
            self.show_embedded(ctx, aligned);
        }
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        use crate::widgets::hint;
        let r = ui.add(
            egui::Slider::new(&mut self.settings.persistence, 0.0..=0.95)
                .text(t!("scope.trace").as_ref()),
        );
        hint::on_hover(ui.ctx(), &r, t!("scope.hint.trace"));
    }

    fn content(&mut self, ui: &mut egui::Ui, aligned: &[f32]) {
        if aligned.is_empty() {
            ui.colored_label(egui::Color32::GRAY, t!("scope.no_data"));
            return;
        }

        let accent = ui.visuals().selection.bg_fill;
        let target = (ui.available_width() as usize).clamp(128, 800);
        widgets::downsample_lttb(&mut self.line_buffer, aligned, target);

        let mut waveform = Waveform::from_line(&self.line_buffer, accent)
            .stroke_width(2.2)
            .fill_alpha(0.46);

        if self.settings.persistence > 0.0 {
            widgets::apply_trace(
                &mut self.trace,
                &self.line_buffer,
                self.settings.persistence,
            );
            waveform = waveform.with_trace(&self.trace);
        }

        waveform.show(ui);
    }

    fn toolbar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, detached_toggle: bool) {
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if detached_toggle {
                        let r = ui
                            .button(crate::icons::rich(crate::icons::POPOUT))
                            .on_hover_text(t!("common.pop_out"));
                        if r.hovered() {
                            crate::widgets::hint::set(ctx, t!("scope.hint.detach"));
                        }
                        if r.clicked() {
                            self.settings.detached = true;
                        }
                    }
                    egui::CollapsingHeader::new(t!("common.settings"))
                        .default_open(false)
                        .show(ui, |ui| self.settings_ui(ui));
                });
            });
    }

    fn window_content(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        aligned: &[f32],
        detached_toggle: bool,
    ) {
        let previous_spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = egui::vec2(previous_spacing.x, 0.0);

        self.toolbar(ui, ctx, detached_toggle);

        let available = ui.available_size();
        ui.allocate_ui_with_layout(available, egui::Layout::top_down(egui::Align::Min), |ui| {
            self.content(ui, aligned);
        });

        ui.spacing_mut().item_spacing = previous_spacing;
    }

    fn show_embedded(&mut self, ctx: &egui::Context, aligned: &[f32]) {
        let mut open = self.open;
        let frame = egui::Frame::window(ctx.style().as_ref()).inner_margin(egui::Margin::ZERO);
        widgets::embedded_window(
            ctx,
            t!("scope.title"),
            &mut open,
            [400.0, 150.0],
            Some(frame),
            |ui| self.window_content(ui, ctx, aligned, true),
        );
        self.open = open;
    }

    fn show_detached(
        &mut self,
        ctx: &egui::Context,
        aligned: &[f32],
        appearance: &AppearanceSettings,
    ) {
        let mut open = self.open;
        let mut detached = self.settings.detached;
        widgets::show_detached_viewport(
            ctx,
            &mut open,
            &mut detached,
            &t!("scope.detached_title"),
            [400.0, 200.0],
            appearance,
            |ui| {
                self.window_content(ui, ctx, aligned, false);
            },
        );
        self.open = open;
        self.settings.detached = detached;
    }
}
