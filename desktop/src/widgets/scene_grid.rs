use std::collections::BTreeSet;

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use sova_core::scene::Scene;

const CELL_WIDTH: f32 = 120.0;
const CELL_HEIGHT: f32 = 40.0;
const HEADER_HEIGHT: f32 = 28.0;
const GAP: f32 = 1.0;
const ADD_BTN_HEIGHT: f32 = 22.0;
const ADD_LINE_WIDTH: f32 = 28.0;
const INDICATOR_RADIUS: f32 = 4.0;
const INDICATOR_X: f32 = 10.0;

pub struct SceneGrid<'a> {
    scene: &'a Scene,
    positions: &'a [Vec<(usize, usize)>],
    cursor: Option<(usize, usize)>,
    selection: &'a BTreeSet<(usize, usize)>,
    accent: Color32,
}

pub struct SceneGridResponse {
    pub clicked: Option<(usize, usize)>,
    pub double_clicked: Option<(usize, usize)>,
    pub secondary_clicked_cell: Option<(usize, usize)>,
    pub secondary_clicked_header: Option<usize>,
    pub enable_toggled: Option<(usize, usize)>,
    pub add_frame_clicked: Option<usize>,
    pub add_line_clicked: bool,
}

impl<'a> SceneGrid<'a> {
    pub fn new(
        scene: &'a Scene,
        positions: &'a [Vec<(usize, usize)>],
        cursor: Option<(usize, usize)>,
        selection: &'a BTreeSet<(usize, usize)>,
        accent: Color32,
    ) -> Self {
        Self {
            scene,
            positions,
            cursor,
            selection,
            accent,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> (egui::Response, SceneGridResponse) {
        let num_lines = self.scene.lines.len();
        let max_frames = self
            .scene
            .lines
            .iter()
            .map(|l| l.frames.len())
            .max()
            .unwrap_or(0);

        let grid_w = num_lines as f32 * (CELL_WIDTH + GAP);
        let grid_h = HEADER_HEIGHT + GAP + max_frames as f32 * (CELL_HEIGHT + GAP);
        let total_w = grid_w + ADD_LINE_WIDTH;
        let total_h = grid_h + ADD_BTN_HEIGHT;
        let size = Vec2::new(
            total_w.max(ADD_LINE_WIDTH),
            total_h.max(HEADER_HEIGHT + ADD_BTN_HEIGHT),
        );

        let (rect, response) =
            ui.allocate_exact_size(size, Sense::click() | Sense::focusable_noninteractive());
        let painter = ui.painter_at(rect);
        let origin = rect.min;

        let text_color = ui.visuals().text_color();
        let dim_text = ui.visuals().weak_text_color();
        let subtle_bg = ui.visuals().faint_bg_color;
        let header_bg = ui.visuals().code_bg_color;
        let selected_tint =
            Color32::from_rgba_unmultiplied(self.accent.r(), self.accent.g(), self.accent.b(), 40);

        for li in 0..num_lines {
            let x = origin.x + li as f32 * (CELL_WIDTH + GAP);

            // Header
            let header_rect =
                Rect::from_min_size(Pos2::new(x, origin.y), Vec2::new(CELL_WIDTH, HEADER_HEIGHT));
            painter.rect_filled(header_rect, 0.0, header_bg);

            let line = &self.scene.lines[li];
            let mut header_label = format!("Line {li}");
            if line.looping {
                header_label.push_str(" \u{21BB}");
            }
            if line.trailing {
                header_label.push_str(" \u{27BF}");
            }
            painter.text(
                header_rect.center(),
                egui::Align2::CENTER_CENTER,
                header_label,
                egui::FontId::proportional(13.0),
                text_color,
            );

            // Cells
            for fi in 0..line.frames.len() {
                let frame = &line.frames[fi];
                let y = origin.y + HEADER_HEIGHT + GAP + fi as f32 * (CELL_HEIGHT + GAP);
                let cell_rect =
                    Rect::from_min_size(Pos2::new(x, y), Vec2::new(CELL_WIDTH, CELL_HEIGHT));

                let is_cursor = self.cursor == Some((li, fi));
                let is_selected = self.selection.contains(&(li, fi));
                let is_playing = self
                    .positions
                    .get(li)
                    .is_some_and(|pos| pos.iter().any(|(f, _)| *f == fi));

                let bg = if !frame.enabled {
                    Color32::from_gray(40)
                } else if is_playing {
                    self.accent
                } else if is_selected {
                    selected_tint
                } else {
                    subtle_bg
                };
                painter.rect_filled(cell_rect, 0.0, bg);

                if is_cursor {
                    painter.rect_stroke(
                        cell_rect,
                        0.0,
                        Stroke::new(2.0, self.accent),
                        egui::StrokeKind::Inside,
                    );
                }

                // Enable indicator
                let ind_center = Pos2::new(cell_rect.min.x + INDICATOR_X, cell_rect.min.y + 12.0);
                if frame.enabled {
                    painter.circle_filled(ind_center, INDICATOR_RADIUS, text_color);
                } else {
                    painter.circle_stroke(
                        ind_center,
                        INDICATOR_RADIUS,
                        Stroke::new(1.0, dim_text),
                    );
                }

                let tc = if !frame.enabled {
                    dim_text
                } else if is_playing {
                    Color32::WHITE
                } else {
                    text_color
                };

                // Frame label
                let default_label;
                let label = match &frame.name {
                    Some(n) => n.as_str(),
                    None => {
                        default_label = format!("Frame {fi}");
                        &default_label
                    }
                };
                painter.text(
                    Pos2::new(
                        cell_rect.center().x + INDICATOR_X / 2.0,
                        cell_rect.min.y + 12.0,
                    ),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(12.0),
                    tc,
                );

                // Duration / repetitions
                let detail = if frame.repetitions > 1 {
                    format!("{:.2} x{}", frame.duration, frame.repetitions)
                } else {
                    format!("{:.2}", frame.duration)
                };
                painter.text(
                    Pos2::new(cell_rect.center().x, cell_rect.max.y - 10.0),
                    egui::Align2::CENTER_CENTER,
                    detail,
                    egui::FontId::proportional(10.0),
                    tc,
                );
            }

            // [+] add frame button below this column
            let add_y =
                origin.y + HEADER_HEIGHT + GAP + line.frames.len() as f32 * (CELL_HEIGHT + GAP);
            let add_rect =
                Rect::from_min_size(Pos2::new(x, add_y), Vec2::new(CELL_WIDTH, ADD_BTN_HEIGHT));
            painter.rect_filled(add_rect, 0.0, header_bg.gamma_multiply(0.5));
            painter.text(
                add_rect.center(),
                egui::Align2::CENTER_CENTER,
                "+",
                egui::FontId::proportional(14.0),
                dim_text,
            );
        }

        // [+] add line header
        let add_line_x = origin.x + num_lines as f32 * (CELL_WIDTH + GAP);
        let add_line_rect = Rect::from_min_size(
            Pos2::new(add_line_x, origin.y),
            Vec2::new(ADD_LINE_WIDTH, HEADER_HEIGHT),
        );
        painter.rect_filled(add_line_rect, 0.0, header_bg.gamma_multiply(0.5));
        painter.text(
            add_line_rect.center(),
            egui::Align2::CENTER_CENTER,
            "+",
            egui::FontId::proportional(14.0),
            dim_text,
        );

        // Hit detection
        let grid_resp = self.detect_clicks(&response, origin, num_lines);

        (response, grid_resp)
    }

    fn detect_clicks(
        &self,
        response: &egui::Response,
        origin: Pos2,
        num_lines: usize,
    ) -> SceneGridResponse {
        let mut result = SceneGridResponse {
            clicked: None,
            double_clicked: None,
            secondary_clicked_cell: None,
            secondary_clicked_header: None,
            enable_toggled: None,
            add_frame_clicked: None,
            add_line_clicked: false,
        };

        let has_interaction =
            response.clicked() || response.double_clicked() || response.secondary_clicked();
        if !has_interaction {
            return result;
        }

        let Some(pos) = response.interact_pointer_pos() else {
            return result;
        };
        let rel = pos - origin;
        let col = (rel.x / (CELL_WIDTH + GAP)) as usize;

        // [+] add line button
        if rel.x >= num_lines as f32 * (CELL_WIDTH + GAP) && rel.y < HEADER_HEIGHT {
            if response.clicked() {
                result.add_line_clicked = true;
            }
            return result;
        }

        if col >= num_lines {
            return result;
        }

        // Header
        if rel.y <= HEADER_HEIGHT {
            if response.secondary_clicked() {
                result.secondary_clicked_header = Some(col);
            }
            return result;
        }

        // Grid area
        let row_offset = rel.y - HEADER_HEIGHT - GAP;
        let fi = (row_offset / (CELL_HEIGHT + GAP)) as usize;
        let line = &self.scene.lines[col];

        // [+] add frame button
        if fi >= line.frames.len() {
            let add_y = line.frames.len() as f32 * (CELL_HEIGHT + GAP);
            if row_offset >= add_y && row_offset < add_y + ADD_BTN_HEIGHT && response.clicked() {
                result.add_frame_clicked = Some(col);
            }
            return result;
        }

        // Cell click — check enable indicator region first
        let cell_local_x = rel.x - col as f32 * (CELL_WIDTH + GAP);
        let cell_local_y = row_offset - fi as f32 * (CELL_HEIGHT + GAP);
        if cell_local_x < INDICATOR_X + INDICATOR_RADIUS + 4.0
            && cell_local_y < 20.0
            && response.clicked()
        {
            result.enable_toggled = Some((col, fi));
        } else if response.double_clicked() {
            result.double_clicked = Some((col, fi));
        } else if response.secondary_clicked() {
            result.secondary_clicked_cell = Some((col, fi));
        } else if response.clicked() {
            result.clicked = Some((col, fi));
        }

        result
    }
}
