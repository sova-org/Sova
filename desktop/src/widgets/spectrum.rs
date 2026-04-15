use eframe::egui::{self, Color32};

const DB_MIN: f32 = -60.0;
const DB_RANGE: f32 = 60.0;

// Color zone thresholds (same as VU meter)
const YELLOW_T: f32 = (-12.0 - DB_MIN) / DB_RANGE; // ~0.8
const RED_T: f32 = (-3.0 - DB_MIN) / DB_RANGE; // ~0.95

const YELLOW: Color32 = Color32::from_rgb(220, 200, 40);
const RED: Color32 = Color32::from_rgb(220, 50, 50);

fn mag_to_t(mag: f32) -> f32 {
    let db = 20.0 * mag.max(1e-7).log10();
    ((db - DB_MIN) / DB_RANGE).clamp(0.0, 1.0)
}

fn color_for_t(accent: Color32, t: f32) -> Color32 {
    if t < YELLOW_T {
        accent
    } else if t < RED_T {
        let f = (t - YELLOW_T) / (RED_T - YELLOW_T);
        lerp_color(accent, YELLOW, f)
    } else {
        let f = ((t - RED_T) / (1.0 - RED_T)).min(1.0);
        lerp_color(YELLOW, RED, f)
    }
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let inv = 1.0 - t;
    Color32::from_rgb(
        (a.r() as f32 * inv + b.r() as f32 * t) as u8,
        (a.g() as f32 * inv + b.g() as f32 * t) as u8,
        (a.b() as f32 * inv + b.b() as f32 * t) as u8,
    )
}

/// Spatial smoothing: 3-tap kernel [0.25, 0.5, 0.25] applied in-place.
fn spatial_smooth(heights: &mut [f32]) {
    let n = heights.len();
    if n < 3 {
        return;
    }
    let mut prev = heights[0];
    for i in 1..n - 1 {
        let smoothed = prev * 0.25 + heights[i] * 0.5 + heights[i + 1] * 0.25;
        prev = heights[i];
        heights[i] = smoothed;
    }
}

pub struct Spectrum<'a> {
    bands: &'a [f32],
    peaks: Option<&'a [f32]>,
    color: Color32,
    gradient_strength: f32,
}

impl<'a> Spectrum<'a> {
    pub fn new(bands: &'a [f32], color: Color32) -> Self {
        Self {
            bands,
            peaks: None,
            color,
            gradient_strength: 0.3,
        }
    }

    pub fn peaks(mut self, p: &'a [f32]) -> Self {
        self.peaks = Some(p);
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

        let n = self.bands.len();
        if n < 2 {
            return;
        }

        let painter = ui.painter_at(rect);
        let accent = self.color;

        let x_of = |i: usize| rect.left() + (i as f32 / (n - 1) as f32) * rect.width();
        let y_of = |t: f32| rect.bottom() - t * rect.height();

        let mut heights: Vec<f32> = self.bands.iter().map(|&m| mag_to_t(m)).collect();
        spatial_smooth(&mut heights);

        // Filled area with per-vertex color based on magnitude.
        let mut mesh = egui::Mesh::default();
        for i in 0..n - 1 {
            let x0 = x_of(i);
            let x1 = x_of(i + 1);
            let y_top0 = y_of(heights[i]);
            let y_top1 = y_of(heights[i + 1]);
            let y_bot = rect.bottom();

            let top_c0 = color_for_t(accent, heights[i]);
            let top_c1 = color_for_t(accent, heights[i + 1]);
            let bot_c0 = top_c0.gamma_multiply(self.gradient_strength);
            let bot_c1 = top_c1.gamma_multiply(self.gradient_strength);

            let base = mesh.vertices.len() as u32;
            mesh.colored_vertex(egui::pos2(x0, y_top0), top_c0);
            mesh.colored_vertex(egui::pos2(x1, y_top1), top_c1);
            mesh.colored_vertex(egui::pos2(x1, y_bot), bot_c1);
            mesh.colored_vertex(egui::pos2(x0, y_bot), bot_c0);
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base, base + 2, base + 3);
        }
        painter.add(egui::Shape::mesh(mesh));

        // Curve line along the top of the fill.
        let top_points: Vec<egui::Pos2> = heights
            .iter()
            .enumerate()
            .map(|(i, &t)| egui::pos2(x_of(i), y_of(t)))
            .collect();
        let line_color = color_for_t(accent, heights.iter().cloned().fold(0.0f32, f32::max));
        painter.add(egui::Shape::line(
            top_points,
            egui::Stroke::new(1.5, line_color),
        ));

        // Peak hold line.
        if let Some(peaks) = self.peaks {
            let peak_color = accent.gamma_multiply(0.55);
            let peak_points: Vec<egui::Pos2> = peaks
                .iter()
                .enumerate()
                .map(|(i, &m)| egui::pos2(x_of(i), y_of(mag_to_t(m))))
                .collect();
            painter.add(egui::Shape::line(
                peak_points,
                egui::Stroke::new(1.0, peak_color),
            ));
        }
    }
}
