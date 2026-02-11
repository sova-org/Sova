use eframe::egui;

pub struct Spectrum<'a> {
    bands: &'a [f32],
    color: egui::Color32,
}

impl<'a> Spectrum<'a> {
    pub fn new(bands: &'a [f32], color: egui::Color32) -> Self {
        Self { bands, color }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let avail = ui.available_size();
        let desired = egui::vec2(avail.x, avail.y.max(60.0));
        let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());

        if self.bands.is_empty() {
            return;
        }

        let painter = ui.painter_at(rect);
        let n = self.bands.len();
        let gap = 1.0;
        let bar_w = (rect.width() - gap * (n - 1) as f32) / n as f32;
        if bar_w <= 0.0 {
            return;
        }

        for (i, &mag) in self.bands.iter().enumerate() {
            let h = mag.clamp(0.0, 1.0) * rect.height();
            let x = rect.left() + i as f32 * (bar_w + gap);
            let bar = egui::Rect::from_min_size(
                egui::pos2(x, rect.bottom() - h),
                egui::vec2(bar_w, h),
            );
            painter.rect_filled(bar, 0.0, self.color);
        }
    }
}
