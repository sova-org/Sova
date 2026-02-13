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

        let top_color = self.color;
        let bot_color = self.color.gamma_multiply(0.3);

        let mut mesh = egui::Mesh::default();
        for (i, &mag) in self.bands.iter().enumerate() {
            let h = mag.clamp(0.0, 1.0) * rect.height();
            let x0 = rect.left() + i as f32 * (bar_w + gap);
            let x1 = x0 + bar_w;
            let y_top = rect.bottom() - h;
            let y_bot = rect.bottom();

            let base = mesh.vertices.len() as u32;
            mesh.colored_vertex(egui::pos2(x0, y_top), top_color); // top-left
            mesh.colored_vertex(egui::pos2(x1, y_top), top_color); // top-right
            mesh.colored_vertex(egui::pos2(x1, y_bot), bot_color); // bottom-right
            mesh.colored_vertex(egui::pos2(x0, y_bot), bot_color); // bottom-left
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base, base + 2, base + 3);
        }
        painter.add(egui::Shape::mesh(mesh));
    }
}
