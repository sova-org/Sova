use eframe::egui;
use egui::{Color32, FontId};
use sova_core::vm::interpreter::Annotation;

use super::PeerCursor;

pub(super) fn paint_line_numbers(
    ui: &egui::Ui,
    output: &egui::text_edit::TextEditOutput,
    gutter_x: f32,
    gutter_width: f32,
    font_id: &FontId,
) {
    let galley = &output.galley;
    let galley_pos = output.galley_pos;
    let painter = ui.painter();

    let gutter_rect = egui::Rect::from_min_size(
        egui::pos2(gutter_x, output.text_clip_rect.min.y),
        egui::vec2(gutter_width, output.text_clip_rect.height()),
    );
    let bg = ui.visuals().extreme_bg_color;
    painter.rect_filled(gutter_rect, 0.0, bg);

    let num_width = gutter_width - 8.0;
    let line_num_color = ui.visuals().weak_text_color();
    let mut line_num = 1u32;
    for (i, placed_row) in galley.rows.iter().enumerate() {
        let is_new_line = i == 0 || galley.rows[i - 1].ends_with_newline;
        if is_new_line {
            let row_y = galley_pos.y + placed_row.pos.y;
            painter.text(
                egui::pos2(gutter_x + num_width, row_y),
                egui::Align2::RIGHT_TOP,
                format!("{line_num}"),
                font_id.clone(),
                line_num_color,
            );
            line_num += 1;
        }
    }
}

pub(super) fn paint_current_line_highlight(
    ui: &egui::Ui,
    output: &egui::text_edit::TextEditOutput,
) {
    let Some(cursor_range) = &output.cursor_range else {
        return;
    };

    let galley = &output.galley;
    let galley_pos = output.galley_pos;
    let cursor_byte = cursor_range.primary.index;

    // Find which row contains the cursor byte offset
    let mut byte_offset = 0;
    let mut row_index = 0;
    for (i, row) in galley.rows.iter().enumerate() {
        let row_bytes: usize = row.glyphs.iter().map(|g| g.chr.len_utf8()).sum();
        let row_end = byte_offset + row_bytes + if row.ends_with_newline { 1 } else { 0 };
        if cursor_byte < row_end || i == galley.rows.len() - 1 {
            row_index = i;
            break;
        }
        byte_offset = row_end;
    }

    if let Some(row) = galley.rows.get(row_index) {
        let row_rect = egui::Rect::from_min_size(
            egui::pos2(output.text_clip_rect.min.x, galley_pos.y + row.pos.y),
            egui::vec2(output.text_clip_rect.width(), row.size.y),
        );

        let highlight_color = if ui.visuals().dark_mode {
            Color32::from_rgba_unmultiplied(255, 255, 255, 16)
        } else {
            Color32::from_rgba_unmultiplied(0, 0, 0, 16)
        };

        ui.painter().rect_filled(row_rect, 0.0, highlight_color);
    }
}

pub(super) fn paint_whitespace(
    ui: &egui::Ui,
    output: &egui::text_edit::TextEditOutput,
    font_id: &FontId,
) {
    let galley = &output.galley;
    let galley_pos = output.galley_pos;
    let painter = ui.painter();
    let ws_color = if ui.visuals().dark_mode {
        Color32::from_rgba_unmultiplied(255, 255, 255, 40)
    } else {
        Color32::from_rgba_unmultiplied(0, 0, 0, 40)
    };

    let ws_font = FontId::monospace(font_id.size * 0.8);

    for row in &galley.rows {
        for glyph in &row.glyphs {
            let ch = glyph.chr;
            let symbol = match ch {
                ' ' => "\u{00B7}",  // middle dot
                '\t' => "\u{2192}", // rightwards arrow
                _ => continue,
            };

            let pos = egui::pos2(galley_pos.x + glyph.pos.x, galley_pos.y + row.pos.y);

            painter.text(
                pos,
                egui::Align2::LEFT_TOP,
                symbol,
                ws_font.clone(),
                ws_color,
            );
        }
    }
}

pub(super) fn paint_peer_cursors(
    ui: &egui::Ui,
    output: &egui::text_edit::TextEditOutput,
    font_id: &FontId,
    peers: &[PeerCursor],
) {
    let galley = &output.galley;
    let galley_pos = output.galley_pos;
    let painter = ui.painter();
    let line_to_row = build_line_to_row(galley);

    for peer in peers {
        let row_idx = if peer.line < line_to_row.len() {
            line_to_row[peer.line]
        } else if let Some(&last) = line_to_row.last() {
            last
        } else {
            continue;
        };

        let Some(row) = galley.rows.get(row_idx) else {
            continue;
        };

        let x = glyph_x(&row.glyphs, peer.col);
        let screen_x = galley_pos.x + x;
        let screen_y = galley_pos.y + row.pos.y;
        let row_height = row.size.y;

        // Caret line
        painter.line_segment(
            [
                egui::pos2(screen_x, screen_y),
                egui::pos2(screen_x, screen_y + row_height),
            ],
            egui::Stroke::new(crate::theme::STROKE_EMPHASIS, peer.color),
        );

        // Name label above caret
        let label_bg =
            Color32::from_rgba_unmultiplied(peer.color.r(), peer.color.g(), peer.color.b(), 180);
        let label_font = FontId::monospace(font_id.size * 0.7);
        let label_galley = painter.layout_no_wrap(peer.name.clone(), label_font, Color32::WHITE);
        let label_w = label_galley.size().x + 4.0;
        let label_h = label_galley.size().y + 2.0;
        let clip = ui.clip_rect();
        let label_y = if screen_y - label_h < clip.min.y {
            screen_y + row_height
        } else {
            screen_y - label_h
        };
        let label_rect =
            egui::Rect::from_min_size(egui::pos2(screen_x, label_y), egui::vec2(label_w, label_h));
        painter.rect_filled(label_rect, 0.0, label_bg);
        painter.galley(
            egui::pos2(label_rect.min.x + 2.0, label_rect.min.y + 1.0),
            label_galley,
            Color32::WHITE,
        );
    }
}

pub(super) fn build_line_to_row(galley: &egui::text::Galley) -> Vec<usize> {
    let mut map = Vec::new();
    for (i, _) in galley.rows.iter().enumerate() {
        let is_new_line = i == 0 || galley.rows[i - 1].ends_with_newline;
        if is_new_line {
            map.push(i);
        }
    }
    map
}

pub(super) fn glyph_x(glyphs: &[egui::epaint::text::Glyph], col: usize) -> f32 {
    if glyphs.is_empty() {
        0.0
    } else if col == 0 {
        glyphs[0].pos.x
    } else if col <= glyphs.len() {
        let g = &glyphs[col - 1];
        g.pos.x + g.advance_width
    } else {
        let g = glyphs.last().expect("non-empty: guarded by is_empty check above");
        g.pos.x + g.advance_width
    }
}

pub(super) fn paint_annotations(
    ui: &egui::Ui,
    output: &egui::text_edit::TextEditOutput,
    font_id: &FontId,
    annotations: &[Annotation],
) {
    let galley = &output.galley;
    let galley_pos = output.galley_pos;
    let painter = ui.painter();
    let line_to_row = build_line_to_row(galley);

    let annotation_font = FontId::monospace(font_id.size * 0.85);
    let text_color = ui.visuals().weak_text_color();
    let highlight_color = Color32::from_rgba_unmultiplied(255, 200, 60, 40);

    // Collect InsertText annotations per line to paint at end of line
    let mut line_texts: std::collections::BTreeMap<usize, String> =
        std::collections::BTreeMap::new();

    for annotation in annotations {
        match annotation {
            Annotation::InsertText(text, pos) => {
                let row_idx = if pos.line < line_to_row.len() {
                    line_to_row[pos.line]
                } else {
                    continue;
                };
                if galley.rows.get(row_idx).is_none() {
                    continue;
                }
                let entry = line_texts.entry(pos.line).or_default();
                if !entry.is_empty() {
                    entry.push(' ');
                }
                entry.push_str(text);
            }
            Annotation::Highlight(start, end) => {
                let start_row = if start.line < line_to_row.len() {
                    line_to_row[start.line]
                } else {
                    continue;
                };
                let end_row = if end.line < line_to_row.len() {
                    line_to_row[end.line]
                } else {
                    continue;
                };

                for row_idx in start_row..=end_row {
                    let Some(row) = galley.rows.get(row_idx) else {
                        continue;
                    };
                    let x_start = if row_idx == start_row {
                        glyph_x(&row.glyphs, start.col.unwrap_or(0))
                    } else {
                        0.0
                    };
                    let x_end = if row_idx == end_row {
                        glyph_x(&row.glyphs, end.col.unwrap_or(row.glyphs.len()))
                    } else {
                        row.rect().width()
                    };

                    let rect = egui::Rect::from_min_size(
                        egui::pos2(galley_pos.x + x_start, galley_pos.y + row.pos.y),
                        egui::vec2(x_end - x_start, row.size.y),
                    );
                    painter.rect_filled(rect, 0.0, highlight_color);
                }
            }
            Annotation::InsertBitmap(..) => {}
        }
    }

    // Paint collected InsertText at end of each line, clipped to editor width
    let clip = ui.clip_rect();
    for (line, text) in &line_texts {
        let row_idx = line_to_row[*line];
        let Some(row) = galley.rows.get(row_idx) else {
            continue;
        };
        let line_end_x = glyph_x(&row.glyphs, row.glyphs.len());
        let screen_x = galley_pos.x + line_end_x + 8.0;
        if screen_x >= clip.max.x {
            continue;
        }
        let screen_y = galley_pos.y + row.pos.y;
        let avail = clip.max.x - screen_x;
        let galley = painter.layout(text.to_string(), annotation_font.clone(), text_color, avail);
        painter.galley(egui::pos2(screen_x, screen_y), galley, text_color);
    }
}
