use std::collections::{BTreeSet, HashMap};
use std::fmt::Write;
use std::sync::Arc;
use std::time::Instant;

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use egui::text::LayoutJob;
use sova_core::scene::Scene;

const EXPANDED_WIDTH: f32 = 170.0;
const COLLAPSED_WIDTH: f32 = 90.0;
const ROW_HEIGHT: f32 = 28.0;
const HEADER_HEIGHT: f32 = 28.0;
const GAP: f32 = 1.0;
const ADD_BTN_HEIGHT: f32 = 22.0;
const ADD_LINE_WIDTH: f32 = 28.0;
const PREVIEW_PAD: f32 = 4.0;
const MAX_PREVIEW_HEIGHT: f32 = 100.0;

// Sub-column fixed widths
const ENABLE_W: f32 = 16.0;
const DUR_W: f32 = 40.0;
const REPS_W: f32 = 28.0;

#[derive(Clone, Copy, PartialEq)]
pub enum InlineEditRegion {
    Name,
    Duration,
    Repetitions,
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

#[derive(Clone, Copy, PartialEq)]
pub enum HeaderEditField {
    Speed,
    StartFrame,
    EndFrame,
}

pub struct HeaderInlineEdit<'a> {
    pub line: usize,
    pub field: HeaderEditField,
    pub buf: &'a mut String,
    pub request_focus: bool,
}

pub struct SceneGrid<'a> {
    scene: &'a Scene,
    positions: &'a [Vec<(usize, usize)>],
    progress: &'a [f32],
    cursor: Option<(usize, usize)>,
    selection: &'a BTreeSet<(usize, usize)>,
    peer_editing: &'a HashMap<(usize, usize), Vec<String>>,
    peer_cursors: &'a HashMap<String, (usize, usize, Option<(usize, usize)>)>,
    compilation_flashes: &'a HashMap<(usize, usize), (bool, Instant)>,
    mutation_flashes: &'a HashMap<(usize, usize), Instant>,
    accent: Color32,
    focused_line: Option<usize>,
    available: Vec2,
    visuals_enabled: bool,
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
    pub subcol_clicked: Option<((usize, usize), InlineEditRegion)>,
    pub speed_clicked: Option<usize>,
    pub start_frame_clicked: Option<usize>,
    pub end_frame_clicked: Option<usize>,
    pub header_edit_action: Option<InlineEditAction>,
}

impl<'a> SceneGrid<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        scene: &'a Scene,
        positions: &'a [Vec<(usize, usize)>],
        progress: &'a [f32],
        cursor: Option<(usize, usize)>,
        selection: &'a BTreeSet<(usize, usize)>,
        peer_editing: &'a HashMap<(usize, usize), Vec<String>>,
        peer_cursors: &'a HashMap<String, (usize, usize, Option<(usize, usize)>)>,
        compilation_flashes: &'a HashMap<(usize, usize), (bool, Instant)>,
        mutation_flashes: &'a HashMap<(usize, usize), Instant>,
        accent: Color32,
        focused_line: Option<usize>,
        available: Vec2,
        visuals_enabled: bool,
    ) -> Self {
        Self {
            scene,
            positions,
            progress,
            cursor,
            selection,
            peer_editing,
            peer_cursors,
            compilation_flashes,
            mutation_flashes,
            accent,
            focused_line,
            available,
            visuals_enabled,
        }
    }

    fn is_expanded(&self, li: usize) -> bool {
        self.focused_line.is_none() || self.focused_line == Some(li)
    }

    fn paint_flashes(&self, painter: &egui::Painter, rect: Rect, li: usize, fi: usize) {
        if let Some(t) = self.mutation_flashes.get(&(li, fi)) {
            let alpha = (1.0 - t.elapsed().as_secs_f32() / 1.2).max(0.0) * 40.0;
            if alpha > 0.5 {
                painter.rect_filled(
                    rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(200, 200, 220, alpha as u8),
                );
            }
        }
        if let Some(&(success, ref t)) = self.compilation_flashes.get(&(li, fi)) {
            let alpha = (1.0 - t.elapsed().as_secs_f32()).max(0.0) * 60.0;
            if alpha > 0.5 {
                let color = if success {
                    Color32::from_rgba_unmultiplied(80, 200, 80, alpha as u8)
                } else {
                    Color32::from_rgba_unmultiplied(200, 80, 80, alpha as u8)
                };
                painter.rect_filled(rect, 0.0, color);
            }
        }
    }

    pub fn show(
        self,
        ui: &mut egui::Ui,
        inline_edit: Option<&mut InlineEdit<'_>>,
        header_edit: Option<&mut HeaderInlineEdit<'_>>,
        code_preview: Option<LayoutJob>,
    ) -> (egui::Response, SceneGridResponse) {
        let num_lines = self.scene.lines.len();

        // Adaptive scale: fill available width proportionally
        let natural_w: f32 = (0..num_lines)
            .map(|li| {
                if self.is_expanded(li) {
                    EXPANDED_WIDTH
                } else {
                    COLLAPSED_WIDTH
                }
            })
            .sum::<f32>()
            + num_lines.saturating_sub(1) as f32 * GAP;

        let target_w = self.available.x - ADD_LINE_WIDTH - GAP;
        let scale = if natural_w > 0.0 {
            (target_w / natural_w).clamp(1.0, 3.0)
        } else {
            1.0
        };

        // Column X positions and widths (scaled)
        let (col_xs, col_ws): (Vec<f32>, Vec<f32>) = {
            let mut xs = Vec::with_capacity(num_lines);
            let mut ws = Vec::with_capacity(num_lines);
            let mut x = 0.0;
            for li in 0..num_lines {
                xs.push(x);
                let w = if self.is_expanded(li) {
                    EXPANDED_WIDTH * scale
                } else {
                    COLLAPSED_WIDTH * scale
                };
                ws.push(w);
                x += w + GAP;
            }
            (xs, ws)
        };

        // Layout code preview galley for the cursor frame
        let preview_galley: Option<Arc<egui::Galley>> =
            if let Some(job) = code_preview
                && let Some((cur_li, _)) = self.cursor
                && self.is_expanded(cur_li)
            {
                Some(ui.fonts_mut(|f| f.layout_job(job)))
            } else {
                None
            };
        let preview_height = preview_galley
            .as_ref()
            .map(|g| g.size().y.min(MAX_PREVIEW_HEIGHT))
            .unwrap_or(0.0);

        // Per-column Y offsets (cursor frame expands to show code preview)
        let offsets: Vec<Vec<f32>> = self
            .scene
            .lines
            .iter()
            .enumerate()
            .map(|(li, line)| {
                let mut ys = Vec::with_capacity(line.frames.len() + 1);
                let mut y = 0.0;
                for fi in 0..line.frames.len() {
                    ys.push(y);
                    let h = if self.cursor == Some((li, fi)) && preview_galley.is_some() {
                        ROW_HEIGHT + 2.0 * PREVIEW_PAD + preview_height
                    } else {
                        ROW_HEIGHT
                    };
                    y += h + GAP;
                }
                ys.push(y);
                ys
            })
            .collect();

        let max_col_h = offsets
            .iter()
            .map(|o| *o.last().unwrap_or(&0.0))
            .fold(0.0f32, f32::max);

        let add_line_x = if num_lines > 0 {
            col_xs[num_lines - 1] + col_ws[num_lines - 1] + GAP
        } else {
            0.0
        };
        let content_w = add_line_x + ADD_LINE_WIDTH;
        let content_h = HEADER_HEIGHT + GAP + max_col_h + ADD_BTN_HEIGHT;
        let size = Vec2::new(
            content_w.max(self.available.x),
            content_h.max(self.available.y),
        );

        let (rect, response) =
            ui.allocate_exact_size(size, Sense::click() | Sense::focusable_noninteractive());
        let painter = ui.painter_at(rect);

        let offset_x = ((rect.width() - content_w) / 2.0).max(0.0);
        let offset_y = ((rect.height() - content_h) / 2.0).max(0.0);
        let origin = rect.min + Vec2::new(offset_x, offset_y);

        let text_color = ui.visuals().text_color();
        let dim_text = ui.visuals().weak_text_color();

        let translucent = |c: Color32, a: u8| Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a);

        let (header_bg, subtle_bg, selected_tint, disabled_bg, playing_accent, add_btn_bg) =
            if self.visuals_enabled {
                (
                    translucent(ui.visuals().code_bg_color, 225),
                    translucent(ui.visuals().faint_bg_color, 210),
                    Color32::from_rgba_unmultiplied(self.accent.r(), self.accent.g(), self.accent.b(), 50),
                    Color32::from_rgba_unmultiplied(40, 40, 40, 185),
                    translucent(self.accent, 225),
                    translucent(ui.visuals().code_bg_color, 175),
                )
            } else {
                let hbg = ui.visuals().code_bg_color;
                (
                    hbg,
                    ui.visuals().faint_bg_color,
                    Color32::from_rgba_unmultiplied(self.accent.r(), self.accent.g(), self.accent.b(), 40),
                    Color32::from_gray(40),
                    self.accent,
                    hbg.gamma_multiply(0.5),
                )
            };

        let edit_coords = inline_edit.as_ref().map(|e| (e.line, e.frame, e.region));
        let header_edit_coords = header_edit.as_ref().map(|e| (e.line, e.field));

        let mut cursor_at_cell: HashMap<(usize, usize), Vec<&str>> = HashMap::new();
        for (name, &(li, fi, _)) in self.peer_cursors {
            cursor_at_cell
                .entry((li, fi))
                .or_default()
                .push(name.as_str());
        }

        let mut peer_tags: Vec<(Rect, &str, Color32)> = Vec::new();
        let mut cell_buf = String::with_capacity(16);

        for li in 0..num_lines {
            let col_x = origin.x + col_xs[li];
            let col_w = col_ws[li];
            let expanded = self.is_expanded(li);
            let line = &self.scene.lines[li];
            let (name_x, _name_w, dur_x, reps_x) = sub_cols(col_w);

            // Header
            let header_rect =
                Rect::from_min_size(Pos2::new(col_x, origin.y), Vec2::new(col_w, HEADER_HEIGHT));
            painter.rect_filled(header_rect, 0.0, header_bg);

            if expanded {
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

                // Speed factor
                if header_edit_coords != Some((li, HeaderEditField::Speed)) {
                    cell_buf.clear();
                    let _ = write!(cell_buf, "\u{00d7}{:.1}", line.speed_factor);
                    let speed_color = if (line.speed_factor - 1.0).abs() < 0.01 { dim_text } else { self.accent };
                    painter.text(
                        Pos2::new(header_rect.left() + 62.0, header_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &cell_buf,
                        egui::FontId::proportional(11.0),
                        speed_color,
                    );
                }

                // Frame range (always visible, two separate entries)
                let last_fi = line.frames.len().saturating_sub(1);
                let end_x = looping_x - 18.0;
                let start_x = end_x - 24.0;
                let range_font = egui::FontId::proportional(11.0);

                if header_edit_coords != Some((li, HeaderEditField::EndFrame)) {
                    let e = line.end_frame.unwrap_or(last_fi);
                    cell_buf.clear();
                    let _ = write!(cell_buf, "{}", e);
                    let color = if line.end_frame.is_some() { self.accent } else { dim_text };
                    painter.text(
                        Pos2::new(end_x, header_rect.center().y),
                        egui::Align2::CENTER_CENTER,
                        &cell_buf,
                        range_font.clone(),
                        color,
                    );
                }

                if header_edit_coords != Some((li, HeaderEditField::StartFrame)) {
                    let s = line.start_frame.unwrap_or(0);
                    cell_buf.clear();
                    let _ = write!(cell_buf, "{}", s);
                    let color = if line.start_frame.is_some() { self.accent } else { dim_text };
                    painter.text(
                        Pos2::new(start_x, header_rect.center().y),
                        egui::Align2::CENTER_CENTER,
                        &cell_buf,
                        range_font,
                        color,
                    );
                }
            } else {
                // Collapsed header: line label + compact looping/trailing icons
                painter.text(
                    Pos2::new(header_rect.left() + 6.0, header_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    t!("scene.line_label", i = li),
                    egui::FontId::proportional(11.0),
                    dim_text,
                );
                let indicator_font = egui::FontId::proportional(12.0);
                let trailing_x = header_rect.right() - 10.0;
                let looping_x = trailing_x - 16.0;
                painter.text(
                    Pos2::new(looping_x, header_rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    crate::icons::LOOPING,
                    indicator_font.clone(),
                    if line.looping { self.accent } else { dim_text },
                );
                painter.text(
                    Pos2::new(trailing_x, header_rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    crate::icons::TRAILING,
                    indicator_font,
                    if line.trailing { self.accent } else { dim_text },
                );

                // Speed factor (collapsed)
                if header_edit_coords != Some((li, HeaderEditField::Speed)) {
                    cell_buf.clear();
                    let _ = write!(cell_buf, "\u{00d7}{:.1}", line.speed_factor);
                    let speed_color = if (line.speed_factor - 1.0).abs() < 0.01 { dim_text } else { self.accent };
                    let clip = Rect::from_min_size(
                        Pos2::new(header_rect.left() + 44.0, header_rect.top()),
                        Vec2::new(looping_x - header_rect.left() - 48.0, HEADER_HEIGHT),
                    );
                    let clipped = ui.painter().with_clip_rect(clip);
                    clipped.text(
                        Pos2::new(header_rect.left() + 44.0, header_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &cell_buf,
                        egui::FontId::proportional(10.0),
                        speed_color,
                    );
                }
            }

            // Frames
            for (fi, frame) in line.frames.iter().enumerate() {
                let y = origin.y + HEADER_HEIGHT + GAP + offsets[li][fi];

                let is_cursor = self.cursor == Some((li, fi));
                let is_selected = self.selection.contains(&(li, fi));
                let is_playing = self
                    .positions
                    .get(li)
                    .is_some_and(|pos| pos.iter().any(|(f, _)| *f == fi));

                if expanded {
                    let has_preview = is_cursor && preview_galley.is_some();
                    let cell_h = if has_preview {
                        ROW_HEIGHT + 2.0 * PREVIEW_PAD + preview_height
                    } else {
                        ROW_HEIGHT
                    };
                    let row = Rect::from_min_size(Pos2::new(col_x, y), Vec2::new(col_w, ROW_HEIGHT));
                    let cell_rect = Rect::from_min_size(Pos2::new(col_x, y), Vec2::new(col_w, cell_h));

                    // Background: playing = horizontal progress, disabled = gray, selected = tint
                    if is_playing && frame.enabled {
                        let prog = self.progress.get(li).copied().unwrap_or(0.0);
                        let fill_w = col_w * prog;
                        painter.rect_filled(
                            Rect::from_min_size(row.min, Vec2::new(fill_w, ROW_HEIGHT)),
                            0.0,
                            playing_accent,
                        );
                        painter.rect_filled(
                            Rect::from_min_max(
                                Pos2::new(row.min.x + fill_w, row.min.y),
                                row.max,
                            ),
                            0.0,
                            selected_tint,
                        );
                    } else {
                        let bg = if !frame.enabled {
                            disabled_bg
                        } else if has_preview {
                            header_bg
                        } else if is_selected {
                            selected_tint
                        } else {
                            subtle_bg
                        };
                        painter.rect_filled(row, 0.0, bg);
                    }

                    self.paint_flashes(&painter, row, li, fi);

                    // Code preview area
                    if has_preview {
                        let code_area = Rect::from_min_max(
                            Pos2::new(col_x, y + ROW_HEIGHT),
                            Pos2::new(col_x + col_w, y + cell_h),
                        );
                        painter.rect_filled(code_area, 0.0, header_bg);
                        if let Some(ref galley) = preview_galley {
                            let clip = code_area.shrink(PREVIEW_PAD);
                            let clipped = painter.with_clip_rect(clip);
                            clipped.galley(clip.min, galley.clone(), text_color);
                        }
                    }

                    if is_cursor {
                        painter.rect_stroke(
                            cell_rect,
                            0.0,
                            Stroke::new(2.0, self.accent),
                            egui::StrokeKind::Inside,
                        );
                    }

                    // Enable dot
                    let dot_center = Pos2::new(col_x + ENABLE_W / 2.0, y + ROW_HEIGHT / 2.0);
                    if frame.enabled {
                        painter.circle_filled(dot_center, 3.0, text_color);
                    } else {
                        painter.circle_stroke(dot_center, 3.0, Stroke::new(1.0, dim_text));
                    }

                    let tc = if !frame.enabled {
                        dim_text
                    } else if is_playing {
                        Color32::WHITE
                    } else {
                        text_color
                    };

                    // Name
                    if edit_coords != Some((li, fi, InlineEditRegion::Name)) {
                        let default_label;
                        let label = match &frame.name {
                            Some(n) => n.as_str(),
                            None => {
                                default_label = t!("scene.frame_label", i = fi);
                                default_label.as_ref()
                            }
                        };
                        painter.text(
                            Pos2::new(col_x + name_x + 4.0, y + ROW_HEIGHT / 2.0),
                            egui::Align2::LEFT_CENTER,
                            label,
                            egui::FontId::proportional(11.0),
                            tc,
                        );
                    }

                    // Duration
                    if edit_coords != Some((li, fi, InlineEditRegion::Duration)) {
                        cell_buf.clear();
                        let _ = write!(cell_buf, "{:.2}", frame.duration);
                        painter.text(
                            Pos2::new(col_x + dur_x + DUR_W - 4.0, y + ROW_HEIGHT / 2.0),
                            egui::Align2::RIGHT_CENTER,
                            &cell_buf,
                            egui::FontId::proportional(10.0),
                            tc,
                        );
                    }

                    // Repetitions
                    if edit_coords != Some((li, fi, InlineEditRegion::Repetitions)) {
                        cell_buf.clear();
                        let _ = write!(cell_buf, "x{}", frame.repetitions);
                        painter.text(
                            Pos2::new(col_x + reps_x + REPS_W / 2.0, y + ROW_HEIGHT / 2.0),
                            egui::Align2::CENTER_CENTER,
                            &cell_buf,
                            egui::FontId::proportional(10.0),
                            dim_text,
                        );
                    }

                    // Peer editing dots
                    if let Some(editors) = self.peer_editing.get(&(li, fi)) {
                        let dot_y = row.min.y + 6.0;
                        for (i, name) in editors.iter().enumerate().take(3) {
                            let dot_x = row.max.x - 6.0 - i as f32 * 8.0;
                            painter.circle_filled(
                                Pos2::new(dot_x, dot_y),
                                3.0,
                                super::username_color(name),
                            );
                        }
                    }

                    // Peer cursor borders + name tags
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
                } else {
                    // Collapsed row — enable dot + name, no duration/reps
                    let row =
                        Rect::from_min_size(Pos2::new(col_x, y), Vec2::new(col_w, ROW_HEIGHT));

                    if is_playing && frame.enabled {
                        let prog = self.progress.get(li).copied().unwrap_or(0.0);
                        let fill_w = col_w * prog;
                        painter.rect_filled(
                            Rect::from_min_size(row.min, Vec2::new(fill_w, ROW_HEIGHT)),
                            0.0,
                            playing_accent,
                        );
                        painter.rect_filled(
                            Rect::from_min_max(
                                Pos2::new(row.min.x + fill_w, row.min.y),
                                row.max,
                            ),
                            0.0,
                            selected_tint,
                        );
                    } else {
                        let bg = if !frame.enabled {
                            disabled_bg
                        } else if is_selected {
                            selected_tint
                        } else {
                            subtle_bg
                        };
                        painter.rect_filled(row, 0.0, bg);
                    }

                    self.paint_flashes(&painter, row, li, fi);

                    if is_cursor {
                        painter.rect_stroke(
                            row,
                            0.0,
                            Stroke::new(1.5, self.accent),
                            egui::StrokeKind::Inside,
                        );
                    }

                    // Enable dot
                    let dot_center =
                        Pos2::new(col_x + ENABLE_W / 2.0, y + ROW_HEIGHT / 2.0);
                    if frame.enabled {
                        painter.circle_filled(dot_center, 3.0, text_color);
                    } else {
                        painter.circle_stroke(dot_center, 3.0, Stroke::new(1.0, dim_text));
                    }

                    // Name (clipped to collapsed width)
                    let tc = if !frame.enabled {
                        dim_text
                    } else if is_playing {
                        Color32::WHITE
                    } else {
                        text_color
                    };
                    let default_label;
                    let label = match &frame.name {
                        Some(n) => n.as_str(),
                        None => {
                            default_label = t!("scene.frame_label", i = fi);
                            default_label.as_ref()
                        }
                    };
                    let clip = Rect::from_min_size(
                        Pos2::new(col_x + ENABLE_W, y),
                        Vec2::new(col_w - ENABLE_W, ROW_HEIGHT),
                    );
                    let clipped = ui.painter().with_clip_rect(clip);
                    clipped.text(
                        Pos2::new(col_x + ENABLE_W + 4.0, y + ROW_HEIGHT / 2.0),
                        egui::Align2::LEFT_CENTER,
                        label,
                        egui::FontId::proportional(11.0),
                        tc,
                    );

                    // Peer cursor borders
                    if let Some(peers) = cursor_at_cell.get(&(li, fi)) {
                        for name in peers.iter().take(3) {
                            let color = super::username_color(name);
                            painter.rect_stroke(
                                row,
                                0.0,
                                Stroke::new(1.5, color),
                                egui::StrokeKind::Inside,
                            );
                        }
                    }
                }
            }

            // [+] add frame button
            let add_y = origin.y + HEADER_HEIGHT + GAP + *offsets[li].last().unwrap_or(&0.0);
            let add_rect =
                Rect::from_min_size(Pos2::new(col_x, add_y), Vec2::new(col_w, ADD_BTN_HEIGHT));
            painter.rect_filled(add_rect, 0.0, add_btn_bg);
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
        let add_line_rect = Rect::from_min_size(
            Pos2::new(origin.x + add_line_x, origin.y),
            Vec2::new(ADD_LINE_WIDTH, HEADER_HEIGHT),
        );
        painter.rect_filled(add_line_rect, 0.0, add_btn_bg);
        painter.text(
            add_line_rect.center(),
            egui::Align2::CENTER_CENTER,
            "+",
            egui::FontId::proportional(14.0),
            dim_text,
        );

        // Inline edit widget
        let edit_action = if let Some(edit) = inline_edit {
            if self.is_expanded(edit.line) {
                let edit_col_w = col_ws.get(edit.line).copied().unwrap_or(EXPANDED_WIDTH);
                let col_x = origin.x + col_xs.get(edit.line).copied().unwrap_or(0.0);
                let row_y = origin.y
                    + HEADER_HEIGHT
                    + GAP
                    + offsets
                        .get(edit.line)
                        .and_then(|o| o.get(edit.frame).copied())
                        .unwrap_or(0.0);

                let (en_x, en_w, ed_x, er_x) = sub_cols(edit_col_w);
                let (edit_x, edit_w, align) = match edit.region {
                    InlineEditRegion::Name => {
                        (col_x + en_x + 2.0, en_w - 4.0, egui::Align::Min)
                    }
                    InlineEditRegion::Duration => {
                        (col_x + ed_x + 2.0, DUR_W - 4.0, egui::Align::Max)
                    }
                    InlineEditRegion::Repetitions => {
                        (col_x + er_x + 2.0, REPS_W - 4.0, egui::Align::Center)
                    }
                };
                let edit_rect = Rect::from_min_size(
                    Pos2::new(edit_x, row_y + 1.0),
                    Vec2::new(edit_w, ROW_HEIGHT - 2.0),
                );

                let font = egui::FontId::proportional(match edit.region {
                    InlineEditRegion::Name => 11.0,
                    _ => 10.0,
                });

                let resp = ui.put(
                    edit_rect,
                    egui::TextEdit::singleline(edit.buf)
                        .font(font)
                        .horizontal_align(align)
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
                Some(InlineEditAction::Cancelled)
            }
        } else {
            None
        };

        // Header inline edit widget
        let header_edit_action = if let Some(edit) = header_edit {
            let edit_col_w = col_ws.get(edit.line).copied().unwrap_or(EXPANDED_WIDTH);
            let col_x = origin.x + col_xs.get(edit.line).copied().unwrap_or(0.0);
            let (edit_x, edit_w) = match edit.field {
                HeaderEditField::Speed => (col_x + 58.0, 44.0),
                HeaderEditField::StartFrame => {
                    let looping_x = edit_col_w - 34.0;
                    (col_x + looping_x - 36.0 - 10.0, 22.0)
                }
                HeaderEditField::EndFrame => {
                    let looping_x = edit_col_w - 34.0;
                    (col_x + looping_x - 12.0 - 10.0, 22.0)
                }
            };
            let edit_rect = Rect::from_min_size(
                Pos2::new(edit_x, origin.y + 1.0),
                Vec2::new(edit_w, HEADER_HEIGHT - 2.0),
            );
            let resp = ui.put(
                edit_rect,
                egui::TextEdit::singleline(edit.buf)
                    .font(egui::FontId::proportional(11.0))
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

        // Hover hints and peer tooltips
        if response.hovered()
            && let Some(pos) = ui.ctx().pointer_hover_pos()
        {
            let ctx = ui.ctx();
            let rel = pos - origin;

            if let Some((col, col_x, col_w)) = find_column(&col_xs, &col_ws, rel.x) {
                let expanded = self.is_expanded(col);

                if rel.y <= HEADER_HEIGHT {
                    let cell_local_x = rel.x - col_x;
                    let (looping_x, trailing_x) = if expanded {
                        (col_w - 34.0, col_w - 14.0)
                    } else {
                        (col_w - 26.0, col_w - 10.0)
                    };
                    if (cell_local_x - looping_x).abs() < 10.0 {
                        super::hint::set(ctx, t!("scene.hint.looping"));
                    } else if (cell_local_x - trailing_x).abs() < 10.0 {
                        super::hint::set(ctx, t!("scene.hint.trailing"));
                    } else if cell_local_x >= 55.0 && cell_local_x < col_w - 85.0 {
                        super::hint::set(ctx, t!("scene.hint.speed"));
                    } else if expanded {
                        let end_x = looping_x - 18.0;
                        let start_x = end_x - 24.0;
                        if (cell_local_x - end_x).abs() < 12.0 || (cell_local_x - start_x).abs() < 12.0 {
                            super::hint::set(ctx, t!("scene.hint.range"));
                        }
                    } else {
                        super::hint::set(ctx, t!("scene.hint.header"));
                    }
                } else if rel.y > HEADER_HEIGHT + GAP {
                    let row_offset = rel.y - HEADER_HEIGHT - GAP;
                    let line = &self.scene.lines[col];
                    let fi = frame_at_y(&offsets[col], row_offset, line.frames.len());

                    if let Some(fi) = fi {
                        let cell_local_x = rel.x - col_x;
                        if cell_local_x < ENABLE_W {
                            super::hint::set(ctx, t!("scene.hint.enable"));
                        } else {
                            super::hint::set(ctx, t!("scene.hint.cell"));
                        }

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
                    } else {
                        let add_y = *offsets[col].last().unwrap_or(&0.0);
                        if row_offset >= add_y && row_offset < add_y + ADD_BTN_HEIGHT {
                            super::hint::set(ctx, t!("scene.hint.add_frame"));
                        }
                    }
                }
            } else if rel.x >= add_line_x && rel.y < HEADER_HEIGHT {
                super::hint::set(ctx, t!("scene.hint.add_line"));
            }
        }

        // Hit detection
        let mut grid_resp =
            self.detect_clicks(&response, origin, num_lines, &col_xs, &col_ws, &offsets);
        grid_resp.edit_action = edit_action;
        grid_resp.header_edit_action = header_edit_action;

        (response, grid_resp)
    }

    fn detect_clicks(
        &self,
        response: &egui::Response,
        origin: Pos2,
        num_lines: usize,
        col_xs: &[f32],
        col_ws: &[f32],
        offsets: &[Vec<f32>],
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
            subcol_clicked: None,
            speed_clicked: None,
            start_frame_clicked: None,
            end_frame_clicked: None,
            header_edit_action: None,
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

        let Some((col, col_x, col_w)) = find_column(col_xs, col_ws, rel.x) else {
            let al_x = if num_lines > 0 {
                col_xs[num_lines - 1] + col_ws[num_lines - 1] + GAP
            } else {
                0.0
            };
            if rel.x >= al_x && rel.y < HEADER_HEIGHT && response.clicked() {
                result.add_line_clicked = true;
            }
            return result;
        };

        let expanded = self.is_expanded(col);

        // Header
        if rel.y <= HEADER_HEIGHT {
            let cell_local_x = rel.x - col_x;
            let (looping_x, trailing_x) = if expanded {
                (col_w - 34.0, col_w - 14.0)
            } else {
                (col_w - 26.0, col_w - 10.0)
            };
            if response.clicked() {
                if (cell_local_x - looping_x).abs() < 10.0 {
                    result.looping_toggled = Some(col);
                } else if (cell_local_x - trailing_x).abs() < 10.0 {
                    result.trailing_toggled = Some(col);
                }
            } else if response.double_clicked() {
                if cell_local_x >= 55.0 && cell_local_x < col_w - 85.0 {
                    result.speed_clicked = Some(col);
                } else if expanded {
                    let end_x = looping_x - 18.0;
                    let start_x = end_x - 24.0;
                    if (cell_local_x - end_x).abs() < 12.0 {
                        result.end_frame_clicked = Some(col);
                    } else if (cell_local_x - start_x).abs() < 12.0 {
                        result.start_frame_clicked = Some(col);
                    }
                }
            } else if response.secondary_clicked() {
                result.secondary_clicked_header = Some(col);
            }
            return result;
        }

        // Grid area
        let row_offset = rel.y - HEADER_HEIGHT - GAP;
        let line = &self.scene.lines[col];
        let fi = frame_at_y(&offsets[col], row_offset, line.frames.len());

        if let Some(fi) = fi {
            if expanded {
                let cell_local_x = rel.x - col_x;

                if cell_local_x < ENABLE_W && response.clicked() {
                    result.enable_toggled = Some((col, fi));
                } else if response.double_clicked() {
                    result.double_clicked = Some((col, fi));
                } else if response.secondary_clicked() {
                    result.secondary_clicked_cell = Some((col, fi));
                } else if response.clicked() {
                    if self.cursor == Some((col, fi)) {
                        let (_, sc_name_w, sc_dur_x, _) = sub_cols(col_w);
                        let region = if cell_local_x < ENABLE_W + sc_name_w {
                            None
                        } else if cell_local_x < sc_dur_x + DUR_W {
                            Some(InlineEditRegion::Duration)
                        } else {
                            Some(InlineEditRegion::Repetitions)
                        };
                        if let Some(r) = region {
                            result.subcol_clicked = Some(((col, fi), r));
                        }
                    } else {
                        result.clicked = Some((col, fi));
                    }
                }
            } else {
                let cell_local_x = rel.x - col_x;
                if cell_local_x < ENABLE_W && response.clicked() {
                    result.enable_toggled = Some((col, fi));
                } else if response.double_clicked() {
                    result.double_clicked = Some((col, fi));
                } else if response.secondary_clicked() {
                    result.secondary_clicked_cell = Some((col, fi));
                } else if response.clicked() {
                    result.clicked = Some((col, fi));
                }
            }
        } else {
            let add_y = *offsets[col].last().unwrap_or(&0.0);
            if row_offset >= add_y && row_offset < add_y + ADD_BTN_HEIGHT && response.clicked() {
                result.add_frame_clicked = Some(col);
            }
        }

        result
    }
}

fn sub_cols(col_w: f32) -> (f32, f32, f32, f32) {
    let name_x = ENABLE_W;
    let name_w = (col_w - ENABLE_W - DUR_W - REPS_W).max(40.0);
    let dur_x = name_x + name_w;
    let reps_x = dur_x + DUR_W;
    (name_x, name_w, dur_x, reps_x)
}

fn find_column(col_xs: &[f32], col_ws: &[f32], x: f32) -> Option<(usize, f32, f32)> {
    for i in 0..col_xs.len() {
        if x >= col_xs[i] && x < col_xs[i] + col_ws[i] {
            return Some((i, col_xs[i], col_ws[i]));
        }
    }
    None
}

fn frame_at_y(offsets: &[f32], y: f32, num_frames: usize) -> Option<usize> {
    for fi in 0..num_frames {
        let top = offsets[fi];
        let h = offsets[fi + 1] - GAP - top;
        if y >= top && y < top + h {
            return Some(fi);
        }
    }
    None
}
