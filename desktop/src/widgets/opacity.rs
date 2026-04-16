use eframe::egui;

#[derive(Clone, Copy)]
pub struct SceneOpacity {
    base: f32,
    active: bool,
}

impl SceneOpacity {
    pub fn new(visuals_enabled: bool, opacity: f32) -> Self {
        Self {
            base: opacity,
            active: visuals_enabled,
        }
    }

    pub fn visuals_enabled(&self) -> bool {
        self.active
    }

    pub fn panel_fill(&self, ctx: &egui::Context) -> egui::Color32 {
        let base = ctx.style().visuals.panel_fill;
        if !self.active {
            return base;
        }
        egui::Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), self.alpha(1.0))
    }

    pub fn alpha(&self, scale: f32) -> u8 {
        if !self.active {
            return 255;
        }
        ((self.base * scale).clamp(0.0, 1.0) * 255.0) as u8
    }

    pub fn fill(&self, c: egui::Color32, scale: f32) -> egui::Color32 {
        if !self.active {
            return c;
        }
        egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), self.alpha(scale))
    }

    pub fn override_widget_visuals(&self, ui: &mut egui::Ui) {
        if !self.active {
            return;
        }
        let v = ui.visuals_mut();
        v.extreme_bg_color = egui::Color32::TRANSPARENT;
        v.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        v.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        v.widgets.hovered.bg_fill = egui::Color32::from_white_alpha(self.alpha(0.3));
        v.widgets.hovered.weak_bg_fill = egui::Color32::from_white_alpha(self.alpha(0.3));
        v.widgets.active.bg_fill = egui::Color32::from_white_alpha(self.alpha(0.4));
        v.widgets.active.weak_bg_fill = egui::Color32::from_white_alpha(self.alpha(0.4));
    }
}
