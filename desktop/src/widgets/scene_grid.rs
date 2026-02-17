use std::collections::{BTreeSet, HashMap};

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

#[derive(Clone, Copy, PartialEq)]
pub enum InlineEditRegion {
    Label,
    Detail,
}

pub struct InlineEdit<'a> {
    pub line: usize,
    pub frame: usize,
    pub region: InlineEditRegion,
    pub buf: &'a mut String,
    pub request_focus: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InlineEditAction {
    Active,
    Committed,
    Cancelled,
    Tabbed,
    BackTabbed,
}

pub struct SceneGrid<'a> {
    scene: &'a Scene,
    positions: &'a [Vec<(usize, usize)>],
    cursor: Option<(usize, usize)>,
    selection: &'a BTreeSet<(usize, usize)>,
    peer_editing: &'a HashMap<(usize, usize), Vec<String>>,
    peer_cursors: &'a HashMap<String, (usize, usize)>,
    accent: Color32,
}

pub struct SceneGridResponse {
    pub clicked: Option<(usize, usize)>,
    pub double_clicked: Option<(usize, usize)>,
    pub secondary_clicked_cell: Option<(usize, usize)>,
    pub secondary_clicked_header: Option<usize>,
    pub enable_toggled: Option<(usize, usize)>,
    pub looping_toggled: Option<usize>,
    pub trailing_toggled: Option<usize>,
    pub add_frame_clicked: Option<usize>,
    pub add_line_clicked: bool,
    pub edit_action: Option<InlineEditAction>,
}

impl<'a> SceneGrid<'a> {
    pub fn new(
        scene: &'a Scene,
        positions: &'a [Vec<(usize, usize)>],
        cursor: Option<(usize, usize)>,
        selection: &'a BTreeSet<(usize, usize)>,
        peer_editing: &'a HashMap<(usize, usize), Vec<String>>,
        peer_cursors: &'a HashMap<String, (usize, usize)>,
        accent: Color32,
    ) -> Self {
        Self {
            scene,
            positions,
            cursor,
            selection,
            peer_editing,
            peer_cursors,
            accent,
        }
    }

    pub fn show(
        self,
        ui: &mut egui::Ui,
        inline_edit: Option<&mut InlineEdit<'_>>,
    ) -> (egui::Response, SceneGridResponse) {
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

        let edit_coords = inline_edit
            .as_ref()
            .map(|e| (e.line, e.frame, e.region));

        // Precompute reverse map: (line, frame) → list of peer names with cursor there
        let mut cursor_at_cell: HashMap<(usize, usize), Vec<&str>> = HashMap::new();
        for (name, &(li, fi)) in self.peer_cursors {
            cursor_at_cell.entry((li, fi)).or_default().push(name.as_str());
        }

        let mut peer_tags: Vec<(Rect, &str, Color32)> = Vec::new();

        for li in 0..num_lines {
            let x = origin.x + li as f32 * (CELL_WIDTH + GAP);

            // Header
            let header_rect =
                Rect::from_min_size(Pos2::new(x, origin.y), Vec2::new(CELL_WIDTH, HEADER_HEIGHT));
            painter.rect_filled(header_rect, 0.0, header_bg);

            let line = &self.scene.lines[li];
            painter.text(
                Pos2::new(header_rect.left() + 6.0, header_rect.center().y),
                egui::Align2::LEFT_CENTER,
                t!("scene.line_label", i = li),
                egui::FontId::proportional(13.0),
                text_color,
            );

            let indicator_font = egui::FontId::proportional(14.0);
            let trailing_x = header_rect.right() - 14.0;
            let looping_x = trailing_x - 20.0;
            let looping_color = if line.looping { self.accent } else { dim_text };
            let trailing_color = if line.trailing { self.accent } else { dim_text };

            painter.text(
                Pos2::new(looping_x, header_rect.center().y),
                egui::Align2::CENTER_CENTER,
                crate::icons::LOOPING,
                indicator_font.clone(),
                looping_color,
            );
            painter.text(
                Pos2::new(trailing_x, header_rect.center().y),
                egui::Align2::CENTER_CENTER,
                crate::icons::TRAILING,
                indicator_font,
                trailing_color,
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

                // Peer editing indicators
                if let Some(editors) = self.peer_editing.get(&(li, fi)) {
                    let dot_y = cell_rect.min.y + 6.0;
                    for (i, name) in editors.iter().enumerate().take(3) {
                        let dot_x = cell_rect.max.x - 6.0 - i as f32 * 8.0;
                        painter.circle_filled(
                            Pos2::new(dot_x, dot_y),
                            3.0,
                            super::username_color(name),
                        );
                    }
                }

                // Peer cursor borders + collect name tags
                if let Some(peers) = cursor_at_cell.get(&(li, fi)) {
                    for name in peers.iter().take(3) {
                        let color = super::username_color(name);
                        painter.rect_stroke(
                            cell_rect,
                            0.0,
                            Stroke::new(2.0, color),
                            egui::StrokeKind::Inside,
                        );
                        peer_tags.push((cell_rect, name, color));
                    }
                }

                let tc = if !frame.enabled {
                    dim_text
                } else if is_playing {
                    Color32::WHITE
                } else {
                    text_color
                };

                let editing_label = edit_coords == Some((li, fi, InlineEditRegion::Label));
                let editing_detail = edit_coords == Some((li, fi, InlineEditRegion::Detail));

                // Frame label
                if !editing_label {
                    let default_label;
                    let label = match &frame.name {
                        Some(n) => n.as_str(),
                        None => {
                            default_label = t!("scene.frame_label", i = fi);
                            default_label.as_ref()
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
                }

                // Duration / repetitions
                if !editing_detail {
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

        // Floating peer name tags (drawn after cells so they render on top)
        {
            let tag_font = egui::FontId::proportional(10.0);
            let pad_x = 3.0;
            let pad_y = 1.0;
            let mut offset_x = 0.0_f32;
            let mut prev_cell: Option<Rect> = None;
            for (cell_rect, name, color) in &peer_tags {
                if prev_cell != Some(*cell_rect) {
                    offset_x = 0.0;
                    prev_cell = Some(*cell_rect);
                }
                let galley =
                    painter.layout_no_wrap((*name).to_string(), tag_font.clone(), Color32::WHITE);
                let text_size = galley.size();
                let tag_w = text_size.x + pad_x * 2.0;
                let tag_h = text_size.y + pad_y * 2.0;
                let tag_pos =
                    Pos2::new(cell_rect.min.x + 20.0 + offset_x, cell_rect.min.y - tag_h);
                let tag_rect = Rect::from_min_size(tag_pos, Vec2::new(tag_w, tag_h));
                painter.rect_filled(tag_rect, 0.0, *color);
                painter.galley(
                    Pos2::new(tag_pos.x + pad_x, tag_pos.y + pad_y),
                    galley,
                    Color32::WHITE,
                );
                offset_x += tag_w + 2.0;
            }
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

        // Inline edit widget
        let edit_action = if let Some(edit) = inline_edit {
            let x = origin.x + edit.line as f32 * (CELL_WIDTH + GAP);
            let y = origin.y + HEADER_HEIGHT + GAP + edit.frame as f32 * (CELL_HEIGHT + GAP);

            let edit_rect = match edit.region {
                InlineEditRegion::Label => Rect::from_min_size(
                    Pos2::new(x + INDICATOR_X * 2.0, y + 1.0),
                    Vec2::new(CELL_WIDTH - INDICATOR_X * 2.0 - 2.0, CELL_HEIGHT / 2.0 - 1.0),
                ),
                InlineEditRegion::Detail => Rect::from_min_size(
                    Pos2::new(x + 2.0, y + CELL_HEIGHT / 2.0),
                    Vec2::new(CELL_WIDTH - 4.0, CELL_HEIGHT / 2.0 - 1.0),
                ),
            };

            let font = match edit.region {
                InlineEditRegion::Label => egui::FontId::proportional(12.0),
                InlineEditRegion::Detail => egui::FontId::proportional(10.0),
            };

            let resp = ui.put(
                edit_rect,
                egui::TextEdit::singleline(edit.buf)
                    .font(font)
                    .horizontal_align(egui::Align::Center)
                    .frame(false)
                    .margin(egui::Margin::ZERO),
            );

            if edit.request_focus {
                resp.request_focus();
            }

            if resp.lost_focus() {
                let (enter, tab, shift_tab) = ui.input(|i| {
                    (
                        i.key_pressed(egui::Key::Enter),
                        i.key_pressed(egui::Key::Tab) && !i.modifiers.shift,
                        i.key_pressed(egui::Key::Tab) && i.modifiers.shift,
                    )
                });
                if enter {
                    Some(InlineEditAction::Committed)
                } else if tab {
                    Some(InlineEditAction::Tabbed)
                } else if shift_tab {
                    Some(InlineEditAction::BackTabbed)
                } else {
                    Some(InlineEditAction::Cancelled)
                }
            } else {
                Some(InlineEditAction::Active)
            }
        } else {
            None
        };

        // Peer hover tooltips
        if response.hovered()
            && let Some(pos) = ui.ctx().pointer_hover_pos()
        {
            let rel = pos - origin;
            let col = (rel.x / (CELL_WIDTH + GAP)) as usize;
            if col < num_lines && rel.y > HEADER_HEIGHT + GAP {
                let row_offset = rel.y - HEADER_HEIGHT - GAP;
                let fi = (row_offset / (CELL_HEIGHT + GAP)) as usize;
                let mut parts = Vec::new();
                if let Some(editors) = self.peer_editing.get(&(col, fi))
                    && !editors.is_empty()
                {
                    parts.push(t!("scene.editing", names = editors.join(", ")).into());
                }
                if let Some(peers) = cursor_at_cell.get(&(col, fi))
                    && !peers.is_empty()
                {
                    parts.push(peers.join(", "));
                }
                if !parts.is_empty() {
                    response.clone().on_hover_text(parts.join("\n"));
                }
            }
        }

        // Hit detection
        let mut grid_resp = self.detect_clicks(&response, origin, num_lines);
        grid_resp.edit_action = edit_action;

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
            looping_toggled: None,
            trailing_toggled: None,
            add_frame_clicked: None,
            add_line_clicked: false,
            edit_action: None,
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
            if response.clicked() {
                let cell_local_x = rel.x - col as f32 * (CELL_WIDTH + GAP);
                let trailing_x = CELL_WIDTH - 14.0;
                let looping_x = trailing_x - 20.0;
                if (cell_local_x - looping_x).abs() < 10.0 {
                    result.looping_toggled = Some(col);
                } else if (cell_local_x - trailing_x).abs() < 10.0 {
                    result.trailing_toggled = Some(col);
                }
            } else if response.secondary_clicked() {
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
