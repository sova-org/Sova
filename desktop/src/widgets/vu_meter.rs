use eframe::egui::{self, Color32};

const DB_MIN: f32 = -60.0;
const DB_MAX: f32 = 0.0;
const DB_RANGE: f32 = DB_MAX - DB_MIN;

const YELLOW_DB: f32 = -12.0;
const RED_DB: f32 = -3.0;

const YELLOW_DIM: Color32 = Color32::from_rgb(120, 160, 30);
const YELLOW_BRIGHT: Color32 = Color32::from_rgb(220, 200, 40);
const RED_DIM: Color32 = Color32::from_rgb(200, 100, 20);
const RED_BRIGHT: Color32 = Color32::from_rgb(220, 50, 50);
const LANE_BG: Color32 = Color32::from_rgb(30, 30, 30);

fn dim_color(c: Color32) -> Color32 {
    Color32::from_rgb(
        (c.r() as f32 * 0.4) as u8,
        (c.g() as f32 * 0.4) as u8,
        (c.b() as f32 * 0.4) as u8,
    )
}

pub struct VuMeter {
    rms_db: f32,
    peak_db: f32,
}

impl VuMeter {
    pub fn new(rms_db: f32, peak_db: f32) -> Self {
        Self { rms_db, peak_db }
    }

    /// Paint just the gradient bar and peak indicator into the given rect.
    pub fn paint_bar(&self, painter: &egui::Painter, bar_rect: egui::Rect, accent: Color32) {
        painter.rect_filled(bar_rect, 0.0, LANE_BG);

        let accent_dim = dim_color(accent);
        let rms_t = ((self.rms_db - DB_MIN) / DB_RANGE).clamp(0.0, 1.0);
        let peak_t = ((self.peak_db - DB_MIN) / DB_RANGE).clamp(0.0, 1.0);
        let yellow_t = (YELLOW_DB - DB_MIN) / DB_RANGE;
        let red_t = (RED_DB - DB_MIN) / DB_RANGE;

        if rms_t > 0.0 {
            let mut mesh = egui::Mesh::default();
            let zones: &[(f32, f32, Color32, Color32)] = &[
                (0.0, yellow_t, accent_dim, accent),
                (yellow_t, red_t, YELLOW_DIM, YELLOW_BRIGHT),
                (red_t, 1.0, RED_DIM, RED_BRIGHT),
            ];

            for &(zone_start, zone_end, c_bottom, c_top) in zones {
                let seg_end = zone_end.min(rms_t);
                if zone_start >= rms_t || zone_start >= seg_end {
                    continue;
                }
                let y_top = bar_rect.bottom() - seg_end * bar_rect.height();
                let y_bot = bar_rect.bottom() - zone_start * bar_rect.height();
                let base = mesh.vertices.len() as u32;
                mesh.colored_vertex(egui::pos2(bar_rect.left(), y_top), c_top);
                mesh.colored_vertex(egui::pos2(bar_rect.right(), y_top), c_top);
                mesh.colored_vertex(egui::pos2(bar_rect.right(), y_bot), c_bottom);
                mesh.colored_vertex(egui::pos2(bar_rect.left(), y_bot), c_bottom);
                mesh.add_triangle(base, base + 1, base + 2);
                mesh.add_triangle(base, base + 2, base + 3);
            }
            painter.add(egui::Shape::mesh(mesh));
        }

        if peak_t > 0.0 {
            let peak_color = if self.peak_db >= RED_DB {
                RED_BRIGHT
            } else if self.peak_db >= YELLOW_DB {
                YELLOW_BRIGHT
            } else {
                accent
            };
            let y = bar_rect.bottom() - peak_t * bar_rect.height();
            painter.line_segment(
                [
                    egui::pos2(bar_rect.left(), y),
                    egui::pos2(bar_rect.right(), y),
                ],
                egui::Stroke::new(2.0, peak_color),
            );
        }
    }
}
