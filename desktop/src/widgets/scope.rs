use eframe::egui;

pub struct Scope<'a> {
    samples: &'a [f32],
    color: egui::Color32,
}

impl<'a> Scope<'a> {
    pub fn new(samples: &'a [f32], color: egui::Color32) -> Self {
        Self { samples, color }
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

        if self.samples.is_empty() {
            return;
        }

        let width = rect.width() as usize;
        if width == 0 {
            return;
        }

        let half_h = rect.height() * 0.5;
        let samples_per_pixel = self.samples.len() as f32 / width as f32;

        let points: Vec<egui::Pos2> = (0..width)
            .map(|x| {
                let start = (x as f32 * samples_per_pixel) as usize;
                let end = ((x + 1) as f32 * samples_per_pixel) as usize;
                let end = end.min(self.samples.len());

                let mut min = f32::MAX;
                let mut max = f32::MIN;
                for &s in &self.samples[start..end] {
                    min = min.min(s);
                    max = max.max(s);
                }
                let mid = (min + max) * 0.5;

                egui::pos2(
                    rect.left() + x as f32,
                    center_y - mid.clamp(-1.0, 1.0) * half_h,
                )
            })
            .collect();

        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(1.0, self.color),
        ));
    }
}
