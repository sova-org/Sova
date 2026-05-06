use eframe::egui;

use crate::client_bridge::ClientBridge;
use crate::theme::username_color;
use crate::widgets::syntax_highlight::SyntaxTheme;
use crate::widgets::syntax_layout_job;

use super::compact_duration_label;

const PREVIEW_MAX_WIDTH: f32 = 520.0;
const PREVIEW_MAX_LINES: usize = 16;

pub(super) fn show_frame_preview(
    ui: &mut egui::Ui,
    li: usize,
    fi: usize,
    frame: &sova_core::scene::Frame,
    bridge: &ClientBridge,
    theme: &SyntaxTheme,
    editing_names: &[&str],
) {
    ui.set_max_width(PREVIEW_MAX_WIDTH);

    let live_text = bridge
        .frame_text_id_at(li, fi)
        .and_then(|id| bridge.frame_doc_text(id))
        .unwrap_or_else(|| frame.script().content().to_string());
    let lang = frame.script().lang();

    show_header(ui, li, fi, lang, frame.duration);

    if live_text.is_empty() {
        ui.label(
            egui::RichText::new(t!("scene.preview.empty"))
                .small()
                .italics()
                .color(ui.visuals().weak_text_color()),
        );
    } else {
        show_code(ui, &live_text, lang, theme, bridge);
    }

    show_peer_strip(ui, li, fi, bridge);
    show_editing_names(ui, editing_names);
}

fn show_header(ui: &mut egui::Ui, li: usize, fi: usize, lang: &str, duration: f64) {
    ui.label(
        egui::RichText::new(t!(
            "scene.preview.header",
            li = li + 1,
            fi = format!("{:02}", fi + 1),
            lang = lang,
            bars = compact_duration_label(duration),
        ))
        .small()
        .strong()
        .color(ui.visuals().text_color()),
    );
    ui.separator();
}

fn show_code(
    ui: &mut egui::Ui,
    text: &str,
    lang: &str,
    theme: &SyntaxTheme,
    bridge: &ClientBridge,
) {
    let (visible, hidden) = truncate_to_lines(text, PREVIEW_MAX_LINES);
    if let Some(syntax) = bridge.syntax_map.get(lang) {
        let mut job = syntax_layout_job(visible, syntax, theme, ui);
        job.wrap.max_width = PREVIEW_MAX_WIDTH;
        ui.label(job);
    } else {
        ui.label(
            egui::RichText::new(visible)
                .monospace()
                .color(ui.visuals().text_color()),
        );
    }
    if hidden > 0 {
        ui.label(
            egui::RichText::new(t!("scene.preview.more_lines", n = hidden))
                .small()
                .italics()
                .color(ui.visuals().weak_text_color()),
        );
    }
}

fn show_peer_strip(ui: &mut egui::Ui, li: usize, fi: usize, bridge: &ClientBridge) {
    let mut carets = bridge.text_cursors_for_frame(li, fi);
    if carets.is_empty() {
        return;
    }
    carets.sort_by(|a, b| a.0.cmp(&b.0));
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        for (name, line, col) in &carets {
            let color = username_color(name);
            ui.label(
                egui::RichText::new(format!("● {} ({}:{})", name, line + 1, col + 1))
                    .small()
                    .color(color),
            );
        }
    });
}

fn show_editing_names(ui: &mut egui::Ui, editing_names: &[&str]) {
    if editing_names.is_empty() {
        return;
    }
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(t!("scene.editing", names = editing_names.join(", ")))
            .small()
            .color(ui.visuals().weak_text_color()),
    );
}

fn truncate_to_lines(text: &str, max_lines: usize) -> (&str, usize) {
    let mut count = 0;
    let mut cut = text.len();
    for (i, ch) in text.char_indices() {
        if ch == '\n' {
            count += 1;
            if count == max_lines {
                cut = i;
                break;
            }
        }
    }
    if count < max_lines {
        return (text, 0);
    }
    let total = text.bytes().filter(|b| *b == b'\n').count() + 1;
    let visible_lines = max_lines;
    let hidden = total.saturating_sub(visible_lines);
    (&text[..cut], hidden)
}
