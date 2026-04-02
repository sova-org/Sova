use eframe::egui;

use crate::settings::{AppearanceSettings, ScopeSettings};
use crate::widgets::{self, Waveform};

pub struct ScopePanel {
    pub open: bool,
    pub settings: ScopeSettings,
    aligned: Vec<f32>,
    line_buffer: Vec<f32>,
    trace: Vec<(f32, f32)>,
}

impl ScopePanel {
    pub fn new(settings: ScopeSettings) -> Self {
        Self {
            open: false,
            settings,
            aligned: Vec::new(),
            line_buffer: Vec::new(),
            trace: Vec::new(),
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
            egui::Slider::new(&mut self.settings.persistence, 0.0..=0.95)
                .text(t!("scope.trace").as_ref()),
        );
        hint::on_hover(ui.ctx(), &r, t!("scope.hint.trace"));
    }

    fn content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, scope_data: &[f32]) {
        if scope_data.is_empty() {
            ui.colored_label(egui::Color32::GRAY, t!("scope.no_data"));
            return;
        }

        let accent = ui.visuals().selection.bg_fill;
        widgets::align_trigger(&mut self.aligned, scope_data);
        let target = (ui.available_width() as usize).clamp(128, 800);
        widgets::downsample_lttb(&mut self.line_buffer, &self.aligned, target);

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
        ctx.request_repaint_after(std::time::Duration::from_millis(33));
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
        scope_data: &[f32],
        detached_toggle: bool,
    ) {
        let previous_spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = egui::vec2(previous_spacing.x, 0.0);

        self.toolbar(ui, ctx, detached_toggle);

        let available = ui.available_size();
        ui.allocate_ui_with_layout(available, egui::Layout::top_down(egui::Align::Min), |ui| {
            self.content(ui, ctx, scope_data);
        });

        ui.spacing_mut().item_spacing = previous_spacing;
    }

    fn show_embedded(&mut self, ctx: &egui::Context, scope_data: &[f32]) {
        let mut open = self.open;
        egui::Window::new(t!("scope.title"))
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_size([400.0, 150.0])
            .frame(egui::Frame::window(ctx.style().as_ref()).inner_margin(egui::Margin::ZERO))
            .show(ctx, |ui| {
                self.window_content(ui, ctx, scope_data, true);
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
                self.window_content(ui, ctx, scope_data, false);
            },
        );
        self.open = open;
        self.settings.detached = detached;
    }
}
