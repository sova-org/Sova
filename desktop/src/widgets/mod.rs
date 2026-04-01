mod about_dialog;
mod bottom_bar;
mod code_editor;
mod command_palette;
mod confirm_dialog;
pub mod hint;
pub mod inline_scene_view;
mod spectrum;
pub mod syntax_highlight;
pub mod tip_popup;
mod toast;
mod vu_meter;
mod waveform;

pub use about_dialog::about_dialog;
pub use bottom_bar::bottom_bar;
pub use code_editor::{CodeEditor, EditorContext, EditorSettings, PeerCursor};
pub use syntax_highlight::SyntaxThemePref;
pub use command_palette::{CommandId, CommandPalette, PaletteAction, PanelStates};
pub use confirm_dialog::{ConfirmAction, ConfirmDialog};
pub use spectrum::Spectrum;
pub use toast::{ToastLevel, ToastStack};
pub use vu_meter::VuMeter;
pub use waveform::Waveform;

pub fn smooth(buffer: &mut Vec<f32>, source: &[f32], factor: f32) {
    buffer.resize(source.len(), 0.0);
    for (b, &s) in buffer.iter_mut().zip(source) {
        *b = *b * factor + s * (1.0 - factor);
    }
}

pub const COLOR_OK: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(100, 200, 100);
pub const COLOR_ERROR: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(200, 100, 100);
pub const COLOR_MUTED: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(128, 128, 128);

pub fn username_color(name: &str) -> eframe::egui::Color32 {
    let mut hash: u32 = 0;
    for b in name.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as u32);
    }
    let hue = (hash % 360) as f32;
    let (r, g, b) = hsl_to_rgb(hue, 0.40, 0.60);
    eframe::egui::Color32::from_rgb(r, g, b)
}

pub(crate) fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match (h as u32) / 60 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

pub(crate) fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    (h, s, l)
}

pub(crate) fn line_accent(accent: eframe::egui::Color32, line_index: usize) -> eframe::egui::Color32 {
    const N: usize = 8;
    if line_index % N == 0 {
        return accent;
    }
    let (h, s, l) = rgb_to_hsl(accent.r(), accent.g(), accent.b());
    let rotated = (h + (line_index % N) as f32 * (360.0 / N as f32)) % 360.0;
    let (r, g, b) = hsl_to_rgb(rotated, s, l);
    eframe::egui::Color32::from_rgb(r, g, b)
}

pub fn show_detached_viewport(
    ctx: &eframe::egui::Context,
    open: &mut bool,
    detached: &mut bool,
    viewport_key: &str,
    title: &str,
    size: [f32; 2],
    appearance: &crate::settings::AppearanceSettings,
    content: impl FnOnce(&mut eframe::egui::Ui),
) {
    use eframe::egui;

    let vp_id = egui::ViewportId::from_hash_of(viewport_key);
    let mut content = Some(content);
    ctx.show_viewport_immediate(
        vp_id,
        egui::ViewportBuilder::default()
            .with_title(title)
            .with_inner_size(size),
        |ctx, class| {
            if class == egui::ViewportClass::Embedded {
                *detached = false;
                return;
            }

            crate::apply_appearance(ctx, appearance);

            if ctx.input(|i| i.viewport().close_requested()) {
                *open = false;
                *detached = false;
                return;
            }

            egui::TopBottomPanel::top(format!("{viewport_key}_toolbar")).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .button(crate::icons::DOCK)
                        .on_hover_text(t!("common.dock_back").to_string())
                        .clicked()
                    {
                        *detached = false;
                    }
                });
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(f) = content.take() {
                    f(ui);
                }
            });
        },
    );
}

#[allow(clippy::too_many_arguments)]
pub fn paint_highlighted_text(
    ui: &eframe::egui::Ui,
    pos: eframe::egui::Pos2,
    text: &str,
    match_indices: &[usize],
    font: eframe::egui::FontId,
    normal_color: eframe::egui::Color32,
    highlight_color: eframe::egui::Color32,
) {
    let painter = ui.painter();
    let chars: Vec<char> = text.chars().collect();
    let mut x = pos.x;

    for (i, &ch) in chars.iter().enumerate() {
        let color = if match_indices.contains(&i) {
            highlight_color
        } else {
            normal_color
        };
        let s = String::from(ch);
        let galley = painter.layout_no_wrap(s, font.clone(), color);
        let char_width = galley.rect.width();
        painter.galley(eframe::egui::pos2(x, pos.y), galley, color);
        x += char_width;
    }
}

pub fn fuzzy_score(needle: &str, haystack: &str) -> Option<(i32, Vec<usize>)> {
    let needle: Vec<char> = needle.to_lowercase().chars().collect();
    let hay: Vec<char> = haystack.to_lowercase().chars().collect();
    let mut score: i32 = 0;
    let mut hi = 0;
    let mut prev_match = false;
    let mut indices = Vec::with_capacity(needle.len());

    for (ni, &nc) in needle.iter().enumerate() {
        let mut found = false;
        while hi < hay.len() {
            if hay[hi] == nc {
                if hi == 0 && ni == 0 {
                    score += 10;
                }
                if prev_match {
                    score += 5;
                }
                if hi > 0 && hay[hi - 1] == ' ' {
                    score += 8;
                }
                score += 1;
                indices.push(hi);
                hi += 1;
                prev_match = true;
                found = true;
                break;
            }
            hi += 1;
            prev_match = false;
        }
        if !found {
            return None;
        }
    }

    Some((score, indices))
}
