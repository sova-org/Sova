use eframe::egui;

pub struct Scope<'a> {
    peaks: &'a [(f32, f32)],
    color: egui::Color32,
}

impl<'a> Scope<'a> {
    pub fn new(peaks: &'a [(f32, f32)], color: egui::Color32) -> Self {
        Self { peaks, color }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let avail = ui.available_size();
        let desired = egui::vec2(avail.x, avail.y.max(60.0));
        let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
        let painter = ui.painter_at(rect);

        let center_y = rect.center().y;
        painter.line_segment(
            [
                egui::pos2(rect.left(), center_y),
                egui::pos2(rect.right(), center_y),
            ],
            egui::Stroke::new(0.5, self.color.gamma_multiply(0.2)),
        );

        if self.peaks.is_empty() {
            return;
        }

        let width = rect.width() as usize;
        if width == 0 {
            return;
        }

        let half_h = rect.height() * 0.5;
        let peaks_per_pixel = self.peaks.len() as f32 / width as f32;

        for x in 0..width {
            let start = (x as f32 * peaks_per_pixel) as usize;
            let end = (((x + 1) as f32 * peaks_per_pixel) as usize).min(self.peaks.len());

            let mut col_min = f32::MAX;
            let mut col_max = f32::MIN;
            for &(lo, hi) in &self.peaks[start..end] {
                col_min = col_min.min(lo);
                col_max = col_max.max(hi);
            }

            let px = rect.left() + x as f32;
            let y_top = center_y - col_max.clamp(-1.0, 1.0) * half_h;
            let y_bot = center_y - col_min.clamp(-1.0, 1.0) * half_h;

            painter.line_segment(
                [egui::pos2(px, y_top), egui::pos2(px, y_bot)],
                egui::Stroke::new(1.0, self.color),
            );
        }
    }
}
