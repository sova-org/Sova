use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::rc::Rc;

use eframe::egui;
use egui::text::{LayoutJob, LayoutSection};
use egui::{Color32, FontId, Id, TextBuffer, TextEdit, TextFormat};
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use sova_core::vm::language::{LanguageElement, ReferenceEntry};

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

pub struct PeerCursor {
    pub name: String,
    pub line: usize,
    pub col: usize,
    pub color: Color32,
}

pub struct EditorContext<'a> {
    pub settings: &'a EditorSettings,
    pub syntax: Option<(&'a CompiledSyntax, &'a SyntaxTheme)>,
    pub reference: Option<&'a BTreeMap<LanguageElement, ReferenceEntry>>,
    pub peer_cursors: &'a [PeerCursor],
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
        ctx: &EditorContext,
    ) -> CodeEditorOutput {
        let settings = ctx.settings;
        let syntax = ctx.syntax;
        let reference = ctx.reference;
        let peer_cursors = ctx.peer_cursors;
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

        if !peer_cursors.is_empty() {
            paint_peer_cursors(ui, &edit_output, &font_id, peer_cursors);
        }

        if let Some(ref_map) = reference {
            show_hover_tooltip(ui, &edit_output, text, ref_map, syntax);
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
    let bg = ui.visuals().extreme_bg_color;
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

fn paint_peer_cursors(
    ui: &egui::Ui,
    output: &egui::text_edit::TextEditOutput,
    font_id: &FontId,
    peers: &[PeerCursor],
) {
    let galley = &output.galley;
    let galley_pos = output.galley_pos;
    let painter = ui.painter();

    // Build a map of logical line → first row index
    let mut line_to_row: Vec<usize> = Vec::new();
    for (i, _) in galley.rows.iter().enumerate() {
        let is_new_line = i == 0 || galley.rows[i - 1].ends_with_newline;
        if is_new_line {
            line_to_row.push(i);
        }
    }

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

        // Compute x position from glyph positions
        let x = if row.glyphs.is_empty() {
            0.0
        } else if peer.col == 0 {
            row.glyphs[0].pos.x
        } else if peer.col <= row.glyphs.len() {
            let g = &row.glyphs[peer.col - 1];
            g.pos.x + g.advance_width
        } else {
            let g = row.glyphs.last().unwrap();
            g.pos.x + g.advance_width
        };

        let screen_x = galley_pos.x + x;
        let screen_y = galley_pos.y + row.pos.y;
        let row_height = row.size.y;

        // Caret line
        painter.line_segment(
            [
                egui::pos2(screen_x, screen_y),
                egui::pos2(screen_x, screen_y + row_height),
            ],
            egui::Stroke::new(2.0, peer.color),
        );

        // Name label above caret
        let label_bg = Color32::from_rgba_unmultiplied(
            peer.color.r(),
            peer.color.g(),
            peer.color.b(),
            180,
        );
        let label_font = FontId::monospace(font_id.size * 0.7);
        let label_galley = painter.layout_no_wrap(
            peer.name.clone(),
            label_font,
            Color32::WHITE,
        );
        let label_w = label_galley.size().x + 4.0;
        let label_h = label_galley.size().y + 2.0;
        let label_rect = egui::Rect::from_min_size(
            egui::pos2(screen_x, screen_y - label_h),
            egui::vec2(label_w, label_h),
        );
        painter.rect_filled(label_rect, 0.0, label_bg);
        painter.galley(
            egui::pos2(label_rect.min.x + 2.0, label_rect.min.y + 1.0),
            label_galley,
            Color32::WHITE,
        );
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

fn word_at_byte_offset(text: &str, offset: usize) -> Option<&str> {
    let bytes = text.as_bytes();
    if offset >= bytes.len() || !is_word_byte(bytes[offset]) {
        return None;
    }
    let mut start = offset;
    while start > 0 && is_word_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = offset;
    while end < bytes.len() && is_word_byte(bytes[end]) {
        end += 1;
    }
    Some(&text[start..end])
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'>' | b'@' | b'#')
}

fn byte_offset_at_pos(
    galley: &egui::Galley,
    galley_pos: egui::Pos2,
    screen_pos: egui::Pos2,
) -> Option<usize> {
    let local_x = screen_pos.x - galley_pos.x;
    let local_y = screen_pos.y - galley_pos.y;

    let mut byte_offset = 0usize;
    for row in &galley.rows {
        let row_bottom = row.pos.y + row.size.y;
        if local_y >= row.pos.y && local_y < row_bottom {
            for (i, glyph) in row.glyphs.iter().enumerate() {
                let glyph_end = if i + 1 < row.glyphs.len() {
                    row.glyphs[i + 1].pos.x
                } else {
                    glyph.pos.x + glyph.advance_width
                };
                if local_x >= glyph.pos.x && local_x < glyph_end {
                    let intra: usize = row.glyphs[..i].iter().map(|g| g.chr.len_utf8()).sum();
                    return Some(byte_offset + intra);
                }
            }
            return None;
        }
        let row_bytes: usize = row.glyphs.iter().map(|g| g.chr.len_utf8()).sum();
        byte_offset += row_bytes + if row.ends_with_newline { 1 } else { 0 };
    }
    None
}

fn lookup_reference<'a>(
    word: &str,
    reference: &'a BTreeMap<LanguageElement, ReferenceEntry>,
) -> Option<&'a ReferenceEntry> {
    let word_lower = word.to_ascii_lowercase();
    for (elem, entry) in reference {
        let key = match elem {
            LanguageElement::Word(w) => w,
            LanguageElement::Brackets(open, _) => open,
        };
        if key.to_ascii_lowercase() == word_lower {
            return Some(entry);
        }
        if entry.aliases.iter().any(|a| a.to_ascii_lowercase() == word_lower) {
            return Some(entry);
        }
    }
    None
}

fn show_hover_tooltip(
    ui: &egui::Ui,
    output: &egui::text_edit::TextEditOutput,
    text: &str,
    reference: &BTreeMap<LanguageElement, ReferenceEntry>,
    syntax: Option<(&CompiledSyntax, &SyntaxTheme)>,
) {
    if !output.response.hovered() {
        return;
    }
    let Some(pointer) = ui.ctx().pointer_hover_pos() else {
        return;
    };
    let Some(offset) = byte_offset_at_pos(&output.galley, output.galley_pos, pointer) else {
        return;
    };
    let Some(word) = word_at_byte_offset(text, offset) else {
        return;
    };
    let Some(entry) = lookup_reference(word, reference) else {
        return;
    };

    egui::Tooltip::always_open(
        ui.ctx().clone(),
        ui.layer_id(),
        egui::Id::new("doc_tooltip"),
        output.response.rect,
    )
    .at_pointer()
    .show(|ui| {
        ui.set_min_width(350.0);
        if let Some(cat) = &entry.category {
            ui.label(egui::RichText::new(cat).small().weak());
        }
        ui.label(&entry.description);
        if let Some(example) = &entry.example {
            ui.add_space(4.0);
            if let Some((cs, theme)) = syntax {
                let job = syntax_layout_job(example, cs, theme, ui);
                ui.label(job);
            } else {
                ui.label(egui::RichText::new(example).monospace().small());
            }
        }
    });
}

fn syntax_layout_job(
    text: &str,
    cs: &CompiledSyntax,
    theme: &SyntaxTheme,
    ui: &egui::Ui,
) -> LayoutJob {
    let font = FontId::monospace(ui.style().text_styles[&egui::TextStyle::Small].size);
    let default_color = ui.visuals().weak_text_color();
    let spans: Vec<_> = cs
        .tokenize(text)
        .map(|(r, cat)| (r, theme.color(cat)))
        .collect();
    let mut job = LayoutJob {
        text: text.to_owned(),
        ..Default::default()
    };
    if spans.is_empty() {
        job.sections.push(LayoutSection {
            leading_space: 0.0,
            byte_range: 0..text.len(),
            format: TextFormat::simple(font, default_color),
        });
    } else {
        let mut pos = 0;
        for (range, color) in &spans {
            if range.start > pos {
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: pos..range.start,
                    format: TextFormat::simple(font.clone(), default_color),
                });
            }
            job.sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: range.clone(),
                format: TextFormat::simple(font.clone(), *color),
            });
            pos = range.end;
        }
        if pos < text.len() {
            job.sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: pos..text.len(),
                format: TextFormat::simple(font, default_color),
            });
        }
    }
    job
}
