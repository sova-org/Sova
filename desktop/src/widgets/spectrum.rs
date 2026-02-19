use eframe::egui;

pub struct Spectrum<'a> {
    bands: &'a [f32],
    color: egui::Color32,
    bar_gap: f32,
    gradient_strength: f32,
}

impl<'a> Spectrum<'a> {
    pub fn new(bands: &'a [f32], color: egui::Color32) -> Self {
        Self {
            bands,
            color,
            bar_gap: 0.0,
            gradient_strength: 0.3,
        }
    }

    pub fn bar_gap(mut self, g: f32) -> Self {
        self.bar_gap = g;
        self
    }

    pub fn gradient_strength(mut self, s: f32) -> Self {
        self.gradient_strength = s;
        self
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
        let bar_w = (rect.width() - self.bar_gap * (n - 1) as f32) / n as f32;
        if bar_w <= 0.0 {
            return;
        }

        let hot = egui::Color32::from_rgb(255, 60, 20);
        let accent = self.color;

        let mut mesh = egui::Mesh::default();
        for (i, &mag) in self.bands.iter().enumerate() {
            let m = mag.clamp(0.0, 1.0);
            let h = m * rect.height();
            let x0 = rect.left() + i as f32 * (bar_w + self.bar_gap);
            let x1 = x0 + bar_w;
            let y_top = rect.bottom() - h;
            let y_bot = rect.bottom();

            let top_color = lerp_color(accent, hot, m);
            let bot_color = top_color.gamma_multiply(self.gradient_strength);

            let base = mesh.vertices.len() as u32;
            mesh.colored_vertex(egui::pos2(x0, y_top), top_color);
            mesh.colored_vertex(egui::pos2(x1, y_top), top_color);
            mesh.colored_vertex(egui::pos2(x1, y_bot), bot_color);
            mesh.colored_vertex(egui::pos2(x0, y_bot), bot_color);
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base, base + 2, base + 3);
        }
        painter.add(egui::Shape::mesh(mesh));
    }
}

fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let inv = 1.0 - t;
    egui::Color32::from_rgb(
        (a.r() as f32 * inv + b.r() as f32 * t) as u8,
        (a.g() as f32 * inv + b.g() as f32 * t) as u8,
        (a.b() as f32 * inv + b.b() as f32 * t) as u8,
    )
}
