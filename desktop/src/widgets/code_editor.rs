use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::rc::Rc;

use eframe::egui;
use egui::text::{LayoutJob, LayoutSection};
use egui::{Color32, FontId, Id, TextBuffer, TextEdit, TextFormat};
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};

use super::syntax_highlight::{CompiledSyntax, SyntaxTheme, SyntaxThemePref};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorSettings {
    pub font_size: f32,
    pub line_numbers: bool,
    pub word_wrap: bool,
    pub show_whitespace: bool,
    pub highlight_current_line: bool,
    pub syntax_theme: SyntaxThemePref,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            line_numbers: true,
            word_wrap: false,
            show_whitespace: false,
            highlight_current_line: true,
            syntax_theme: SyntaxThemePref::default(),
        }
    }
}

pub struct CodeEditorOutput {
    pub response: egui::Response,
    pub cursor_line: Option<usize>,
    pub cursor_col: Option<usize>,
}

pub struct CodeEditor {
    search_open: bool,
    search_query: String,
    matches: Rc<Vec<Range<usize>>>,
    current_match: usize,
    cache_hash: u64,
}

impl CodeEditor {
    pub fn new() -> Self {
        Self {
            search_open: false,
            search_query: String::new(),
            matches: Rc::new(Vec::new()),
            current_match: 0,
            cache_hash: 0,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        id: Id,
        text: &mut String,
        settings: &EditorSettings,
        syntax: Option<(&CompiledSyntax, &SyntaxTheme)>,
    ) -> CodeEditorOutput {
        let font_id = FontId::monospace(settings.font_size);
        let is_mac = ui.ctx().os().is_mac();

        // Cmd/Ctrl+F toggles search
        if ui.input(|i| {
            if is_mac {
                i.key_pressed(egui::Key::F) && i.modifiers.mac_cmd
            } else {
                i.key_pressed(egui::Key::F) && i.modifiers.ctrl
            }
        }) {
            self.search_open = !self.search_open;
            if !self.search_open {
                self.search_query.clear();
                self.matches = Rc::new(Vec::new());
            }
        }

        // Search bar
        let mut navigate = Navigate::None;
        if self.search_open {
            ui.horizontal(|ui| {
                let search_id = id.with("search_field");
                let r = ui.add_sized(
                    [200.0, 20.0],
                    TextEdit::singleline(&mut self.search_query).id(search_id),
                );

                if r.gained_focus() || ui.memory(|m| !m.has_focus(search_id)) {
                    r.request_focus();
                }

                if !self.matches.is_empty() {
                    ui.label(format!("{}/{}", self.current_match + 1, self.matches.len()));
                } else if !self.search_query.is_empty() {
                    ui.label("0/0");
                }

                if ui.button("<").clicked() {
                    navigate = Navigate::Prev;
                }
                if ui.button(">").clicked() {
                    navigate = Navigate::Next;
                }
                if ui.button("x").clicked() {
                    self.search_open = false;
                    self.search_query.clear();
                    self.matches = Rc::new(Vec::new());
                }

                if r.has_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.search_open = false;
                        self.search_query.clear();
                        self.matches = Rc::new(Vec::new());
                    } else if ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.shift) {
                        navigate = Navigate::Prev;
                    } else if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        navigate = Navigate::Next;
                    }
                }
            });
        }

        self.recompute_matches(text);

        match navigate {
            Navigate::Next if !self.matches.is_empty() => {
                self.current_match = (self.current_match + 1) % self.matches.len();
            }
            Navigate::Prev if !self.matches.is_empty() => {
                self.current_match =
                    (self.current_match + self.matches.len() - 1) % self.matches.len();
            }
            _ => {}
        }

        let matches = self.matches.clone();
        let current = self.current_match;
        let word_wrap = settings.word_wrap;
        let font_clone = font_id.clone();
        let text_color = ui.visuals().text_color();

        let mut layouter = move |ui: &egui::Ui, text_str: &dyn TextBuffer, wrap_width: f32| {
            let text_s = text_str.as_str();
            let max_width = if word_wrap { wrap_width } else { f32::INFINITY };
            let spans: Vec<(Range<usize>, Color32)> = syntax
                .map(|(cs, theme)| {
                    cs.tokenize(text_s)
                        .map(|(r, cat)| (r, theme.color(cat)))
                        .collect()
                })
                .unwrap_or_default();
            let job = build_layout_job(
                text_s,
                &font_clone,
                text_color,
                max_width,
                &matches,
                current,
                &spans,
            );
            ui.fonts_mut(|f| f.layout_job(job))
        };

        let highlight_current_line = settings.highlight_current_line;
        let show_whitespace = settings.show_whitespace;
        let available_height = ui.available_height();

        let (response, edit_output) = if settings.line_numbers {
            let line_count = text.chars().filter(|c| *c == '\n').count() + 1;
            let digit_count = ((line_count as f32).log10().floor() as usize + 1).max(2);
            let char_width = ui.fonts_mut(|f| f.glyph_width(&font_id, '0'));
            let gutter_width = char_width * (digit_count as f32 + 1.0) + 8.0;

            let mut output = None;
            ui.horizontal_top(|ui| {
                let (gutter_rect, _) =
                    ui.allocate_exact_size(egui::vec2(gutter_width, 0.0), egui::Sense::hover());

                let text_width = ui.available_width();
                let edit_output = TextEdit::multiline(text)
                    .id(id.with("editor"))
                    .font(font_id.clone())
                    .desired_width(text_width)
                    .min_size(egui::vec2(text_width, available_height))
                    .lock_focus(true)
                    .layouter(&mut layouter)
                    .show(ui);

                paint_line_numbers(
                    ui,
                    &edit_output,
                    gutter_rect.min.x,
                    gutter_width,
                    &font_id,
                );

                output = Some(edit_output);
            });

            let edit_output = output.unwrap();
            (edit_output.response.clone(), edit_output)
        } else {
            let edit_output = TextEdit::multiline(text)
                .id(id.with("editor"))
                .font(font_id.clone())
                .min_size(egui::vec2(ui.available_width(), available_height))
                .lock_focus(true)
                .layouter(&mut layouter)
                .show(ui);
            (edit_output.response.clone(), edit_output)
        };

        if highlight_current_line {
            paint_current_line_highlight(ui, &edit_output);
        }

        if show_whitespace {
            paint_whitespace(ui, &edit_output, &font_id);
        }

        let cursor = edit_output
            .cursor_range
            .as_ref()
            .map(|cr| cursor_line_col(text, cr.primary.index));
        CodeEditorOutput {
            response,
            cursor_line: cursor.map(|(l, _)| l),
            cursor_col: cursor.map(|(_, c)| c),
        }
    }

    fn recompute_matches(&mut self, text: &str) {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        self.search_query.hash(&mut hasher);
        let h = hasher.finish();

        if h == self.cache_hash {
            return;
        }
        self.cache_hash = h;

        let matches = Rc::make_mut(&mut self.matches);
        matches.clear();

        if self.search_query.is_empty() {
            self.current_match = 0;
            return;
        }

        let Ok(re) = RegexBuilder::new(&regex::escape(&self.search_query))
            .case_insensitive(true)
            .build()
        else {
            return;
        };

        for m in re.find_iter(text) {
            matches.push(m.start()..m.end());
        }

        if self.current_match >= matches.len() {
            self.current_match = 0;
        }
    }
}

#[derive(PartialEq)]
enum Navigate {
    None,
    Next,
    Prev,
}

fn build_layout_job(
    text: &str,
    font_id: &FontId,
    text_color: Color32,
    max_width: f32,
    matches: &[Range<usize>],
    current_match: usize,
    syntax_spans: &[(Range<usize>, Color32)],
) -> LayoutJob {
    let default_fmt = TextFormat::simple(font_id.clone(), text_color);

    let mut job = LayoutJob {
        text: text.to_owned(),
        wrap: egui::text::TextWrapping {
            max_width,
            ..Default::default()
        },
        ..Default::default()
    };

    // Build base sections from syntax tokens (or one default section)
    if syntax_spans.is_empty() {
        job.sections.push(LayoutSection {
            leading_space: 0.0,
            byte_range: 0..text.len(),
            format: default_fmt.clone(),
        });
    } else {
        let mut pos = 0;
        for (range, color) in syntax_spans {
            if range.start > pos {
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: pos..range.start,
                    format: default_fmt.clone(),
                });
            }
            job.sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: range.clone(),
                format: TextFormat::simple(font_id.clone(), *color),
            });
            pos = range.end;
        }
        if pos < text.len() {
            job.sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: pos..text.len(),
                format: default_fmt.clone(),
            });
        }
    }

    // Overlay search highlights on top of syntax coloring (skip stale matches)
    let matches: Vec<Range<usize>> = matches
        .iter()
        .filter(|m| m.end <= text.len())
        .cloned()
        .collect();
    if !matches.is_empty() {
        let highlight_bg = Color32::from_rgba_unmultiplied(255, 255, 0, 60);
        let current_bg = Color32::from_rgba_unmultiplied(255, 180, 0, 120);

        let base_sections = std::mem::take(&mut job.sections);
        for section in base_sections {
            split_section_with_highlights(
                section,
                &matches,
                current_match,
                highlight_bg,
                current_bg,
                &mut job.sections,
            );
        }
    }

    job
}

fn split_section_with_highlights(
    section: LayoutSection,
    matches: &[Range<usize>],
    current_match: usize,
    highlight_bg: Color32,
    current_bg: Color32,
    out: &mut Vec<LayoutSection>,
) {
    let sec_start = section.byte_range.start;
    let sec_end = section.byte_range.end;
    let base_fmt = &section.format;

    let mut pos = sec_start;
    for (i, m) in matches.iter().enumerate() {
        // Skip matches entirely outside this section
        if m.end <= sec_start || m.start >= sec_end {
            continue;
        }
        let overlap_start = m.start.max(sec_start);
        let overlap_end = m.end.min(sec_end);

        if overlap_start > pos {
            out.push(LayoutSection {
                leading_space: 0.0,
                byte_range: pos..overlap_start,
                format: base_fmt.clone(),
            });
        }
        let bg = if i == current_match { current_bg } else { highlight_bg };
        out.push(LayoutSection {
            leading_space: 0.0,
            byte_range: overlap_start..overlap_end,
            format: TextFormat { background: bg, ..base_fmt.clone() },
        });
        pos = overlap_end;
    }
    if pos < sec_end {
        out.push(LayoutSection {
            leading_space: 0.0,
            byte_range: pos..sec_end,
            format: base_fmt.clone(),
        });
    }
}

fn paint_line_numbers(
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
    let bg = if ui.visuals().dark_mode {
        Color32::from_rgba_unmultiplied(255, 255, 255, 6)
    } else {
        Color32::from_rgba_unmultiplied(0, 0, 0, 6)
    };
    painter.rect_filled(gutter_rect, 0.0, bg);

    let sep_x = gutter_x + gutter_width;
    let sep = if ui.visuals().dark_mode {
        Color32::from_rgba_unmultiplied(255, 255, 255, 15)
    } else {
        Color32::from_rgba_unmultiplied(0, 0, 0, 15)
    };
    painter.line_segment(
        [
            egui::pos2(sep_x, output.text_clip_rect.min.y),
            egui::pos2(sep_x, output.text_clip_rect.max.y),
        ],
        egui::Stroke::new(1.0, sep),
    );

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

fn paint_current_line_highlight(ui: &egui::Ui, output: &egui::text_edit::TextEditOutput) {
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

fn paint_whitespace(ui: &egui::Ui, output: &egui::text_edit::TextEditOutput, font_id: &FontId) {
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

fn cursor_line_col(text: &str, char_offset: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    for (i, ch) in text.chars().enumerate() {
        if i >= char_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}
