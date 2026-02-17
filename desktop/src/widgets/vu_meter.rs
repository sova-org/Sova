use eframe::egui::{self, Color32};

const DB_MIN: f32 = -60.0;
const DB_MAX: f32 = 0.0;
const DB_RANGE: f32 = DB_MAX - DB_MIN;

const YELLOW_DB: f32 = -12.0;
const RED_DB: f32 = -3.0;

// Gradient colors (bottom to top within each zone)
const GREEN_DIM: Color32 = Color32::from_rgb(20, 100, 40);
const GREEN_BRIGHT: Color32 = Color32::from_rgb(50, 200, 80);
const YELLOW_DIM: Color32 = Color32::from_rgb(120, 160, 30);
const YELLOW_BRIGHT: Color32 = Color32::from_rgb(220, 200, 40);
const RED_DIM: Color32 = Color32::from_rgb(200, 100, 20);
const RED_BRIGHT: Color32 = Color32::from_rgb(220, 50, 50);

const LABEL_COL_WIDTH: f32 = 14.0;

const TICK_DBS: [f32; 8] = [-48.0, -36.0, -24.0, -12.0, -6.0, -3.0, -1.0, 0.0];
const TICK_LABELS: [&str; 8] = ["-48", "-36", "-24", "-12", "-6", "-3", "-1", " 0"];

pub struct VuMeter {
    rms_db: f32,
    peak_db: f32,
}

impl VuMeter {
    pub fn new(rms_db: f32, peak_db: f32) -> Self {
        Self { rms_db, peak_db }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let avail = ui.available_size();
        let desired = egui::vec2(avail.x.max(24.0), avail.y.max(24.0));
        let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
        let painter = ui.painter_at(rect);

        let bg = ui.visuals().extreme_bg_color;
        painter.rect_filled(rect, 0.0, bg);

        // Split into label column (left) and meter bar (right)
        let label_rect = egui::Rect::from_min_max(
            rect.min,
            egui::pos2(rect.left() + LABEL_COL_WIDTH, rect.bottom()),
        );
        let bar_rect = egui::Rect::from_min_max(
            egui::pos2(rect.left() + LABEL_COL_WIDTH + 1.0, rect.top()),
            rect.max,
        );

        let rms_t = ((self.rms_db - DB_MIN) / DB_RANGE).clamp(0.0, 1.0);
        let peak_t = ((self.peak_db - DB_MIN) / DB_RANGE).clamp(0.0, 1.0);

        let yellow_t = (YELLOW_DB - DB_MIN) / DB_RANGE;
        let red_t = (RED_DB - DB_MIN) / DB_RANGE;

        // Gradient meter fill
        if rms_t > 0.0 {
            let mut mesh = egui::Mesh::default();

            let zones: &[(f32, f32, Color32, Color32)] = &[
                (0.0, yellow_t, GREEN_DIM, GREEN_BRIGHT),
                (yellow_t, red_t, YELLOW_DIM, YELLOW_BRIGHT),
                (red_t, 1.0, RED_DIM, RED_BRIGHT),
            ];

            for &(zone_start, zone_end, c_bottom, c_top) in zones {
                let seg_end = zone_end.min(rms_t);
                if zone_start >= rms_t || zone_start >= seg_end {
                    continue;
                }

                // Bottom = low dB, top = high dB. Meter fills upward.
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

        // Peak hold indicator
        if peak_t > 0.0 {
            let peak_color = if self.peak_db >= RED_DB {
                RED_BRIGHT
            } else if self.peak_db >= YELLOW_DB {
                YELLOW_BRIGHT
            } else {
                GREEN_BRIGHT
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

        // Tick marks and labels in the left column
        let tick_color = ui.visuals().weak_text_color();
        let tick_stroke = egui::Stroke::new(0.5, tick_color);
        let font = egui::FontId::monospace(7.0);

        for (&db, &label) in TICK_DBS.iter().zip(TICK_LABELS.iter()) {
            let t = (db - DB_MIN) / DB_RANGE;
            let y = bar_rect.bottom() - t * bar_rect.height();

            // Tick line from label area toward the bar
            painter.line_segment(
                [
                    egui::pos2(label_rect.right() - 2.0, y),
                    egui::pos2(bar_rect.left(), y),
                ],
                tick_stroke,
            );

            // Label right-aligned in label column
            painter.text(
                egui::pos2(label_rect.right() - 3.0, y),
                egui::Align2::RIGHT_CENTER,
                label,
                font.clone(),
                tick_color,
            );
        }
    }
}
