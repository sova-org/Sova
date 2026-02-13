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

        if self.samples.len() < 2 {
            return;
        }

        let num_bins = 256.min(self.samples.len());
        let bin_size = self.samples.len() / num_bins;
        let peaks: Vec<(f32, f32)> = (0..num_bins)
            .map(|i| {
                let start = i * bin_size;
                let end = (start + bin_size).min(self.samples.len());
                self.samples[start..end]
                    .iter()
                    .fold((f32::MAX, f32::MIN), |(mn, mx), &s| (mn.min(s), mx.max(s)))
            })
            .collect();

        let half_h = rect.height() * 0.5;
        let num_peaks = peaks.len() as f32;
        let width = rect.width();

        let peak_x = |i: usize| rect.left() + (i as f32 / (num_peaks - 1.0)) * width;
        let val_y = |v: f32| center_y - v.clamp(-1.0, 1.0) * half_h;

        let fill_color = self.color.gamma_multiply(0.35);
        let mut mesh = egui::Mesh::default();
        for i in 0..peaks.len() - 1 {
            let x0 = peak_x(i);
            let x1 = peak_x(i + 1);
            let (min0, max0) = peaks[i];
            let (min1, max1) = peaks[i + 1];

            let base = mesh.vertices.len() as u32;
            mesh.colored_vertex(egui::pos2(x0, val_y(max0)), fill_color);
            mesh.colored_vertex(egui::pos2(x0, val_y(min0)), fill_color);
            mesh.colored_vertex(egui::pos2(x1, val_y(max1)), fill_color);
            mesh.colored_vertex(egui::pos2(x1, val_y(min1)), fill_color);
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base + 1, base + 2, base + 3);
        }
        painter.add(egui::Shape::mesh(mesh));

        let stroke = egui::Stroke::new(1.0, self.color);
        let top_line: Vec<egui::Pos2> = peaks
            .iter()
            .enumerate()
            .map(|(i, &(_, max))| egui::pos2(peak_x(i), val_y(max)))
            .collect();
        let bot_line: Vec<egui::Pos2> = peaks
            .iter()
            .enumerate()
            .map(|(i, &(min, _))| egui::pos2(peak_x(i), val_y(min)))
            .collect();
        painter.add(egui::Shape::line(top_line, stroke));
        painter.add(egui::Shape::line(bot_line, stroke));
    }
}
