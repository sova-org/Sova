use eframe::egui;

const DB_MIN: f32 = -60.0;
const DB_MAX: f32 = 0.0;
const DB_RANGE: f32 = DB_MAX - DB_MIN;

const YELLOW_DB: f32 = -12.0;
const RED_DB: f32 = -3.0;

const GREEN: egui::Color32 = egui::Color32::from_rgb(50, 200, 80);
const YELLOW: egui::Color32 = egui::Color32::from_rgb(220, 200, 40);
const RED: egui::Color32 = egui::Color32::from_rgb(220, 50, 50);

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

        let vertical = rect.height() > rect.width();

        let bg = ui.visuals().extreme_bg_color;
        painter.rect_filled(rect, 0.0, bg);

        let rms_t = ((self.rms_db - DB_MIN) / DB_RANGE).clamp(0.0, 1.0);
        let peak_t = ((self.peak_db - DB_MIN) / DB_RANGE).clamp(0.0, 1.0);

        let yellow_t = (YELLOW_DB - DB_MIN) / DB_RANGE;
        let red_t = (RED_DB - DB_MIN) / DB_RANGE;

        // Meter fill via mesh with color zones
        if rms_t > 0.0 {
            let mut mesh = egui::Mesh::default();

            let zones: &[(f32, f32, egui::Color32, egui::Color32)] = &[
                (0.0, yellow_t, GREEN, GREEN),
                (yellow_t, red_t, YELLOW, YELLOW),
                (red_t, 1.0, RED, RED),
            ];

            for &(zone_start, zone_end, c_start, c_end) in zones {
                let seg_start = zone_start;
                let seg_end = zone_end.min(rms_t);
                if seg_start >= rms_t || seg_start >= seg_end {
                    continue;
                }

                let base = mesh.vertices.len() as u32;
                if vertical {
                    // Bottom = 0 dB min, top = peak. Meter fills upward.
                    let y0 = rect.bottom() - seg_end * rect.height();
                    let y1 = rect.bottom() - seg_start * rect.height();
                    mesh.colored_vertex(egui::pos2(rect.left(), y0), c_end);
                    mesh.colored_vertex(egui::pos2(rect.right(), y0), c_end);
                    mesh.colored_vertex(egui::pos2(rect.right(), y1), c_start);
                    mesh.colored_vertex(egui::pos2(rect.left(), y1), c_start);
                } else {
                    let x0 = rect.left() + seg_start * rect.width();
                    let x1 = rect.left() + seg_end * rect.width();
                    mesh.colored_vertex(egui::pos2(x0, rect.top()), c_start);
                    mesh.colored_vertex(egui::pos2(x1, rect.top()), c_end);
                    mesh.colored_vertex(egui::pos2(x1, rect.bottom()), c_end);
                    mesh.colored_vertex(egui::pos2(x0, rect.bottom()), c_start);
                }
                mesh.add_triangle(base, base + 1, base + 2);
                mesh.add_triangle(base, base + 2, base + 3);
            }

            painter.add(egui::Shape::mesh(mesh));
        }

        // Peak hold indicator
        if peak_t > 0.0 {
            let peak_color = if self.peak_db >= RED_DB {
                RED
            } else if self.peak_db >= YELLOW_DB {
                YELLOW
            } else {
                GREEN
            };
            let stroke = egui::Stroke::new(2.0, peak_color);
            if vertical {
                let y = rect.bottom() - peak_t * rect.height();
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    stroke,
                );
            } else {
                let x = rect.left() + peak_t * rect.width();
                painter.line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    stroke,
                );
            }
        }

        // dB tick marks
        let tick_color = ui.visuals().weak_text_color();
        let tick_stroke = egui::Stroke::new(0.5, tick_color);
        let font = egui::FontId::monospace(9.0);

        for (&db, &label) in TICK_DBS.iter().zip(TICK_LABELS.iter()) {
            let t = (db - DB_MIN) / DB_RANGE;

            if vertical {
                let y = rect.bottom() - t * rect.height();
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.left() + 4.0, y)],
                    tick_stroke,
                );
                painter.text(
                    egui::pos2(rect.left() + 6.0, y),
                    egui::Align2::LEFT_CENTER,
                    label,
                    font.clone(),
                    tick_color,
                );
            } else {
                let x = rect.left() + t * rect.width();
                painter.line_segment(
                    [
                        egui::pos2(x, rect.bottom()),
                        egui::pos2(x, rect.bottom() - 4.0),
                    ],
                    tick_stroke,
                );
                painter.text(
                    egui::pos2(x, rect.bottom() - 6.0),
                    egui::Align2::CENTER_BOTTOM,
                    label,
                    font.clone(),
                    tick_color,
                );
            }
        }
    }
}
