mod about_dialog;
mod bottom_bar;
mod code_editor;
mod command_palette;
mod confirm_dialog;
pub mod hint;
mod inline_markdown;
pub mod inline_scene_view;
mod spectrum;
pub mod spectrum_analyzer;
pub mod syntax_highlight;
pub mod tip_popup;
mod toast;
mod vu_meter;
mod waveform;

pub use about_dialog::about_dialog;
pub use bottom_bar::bottom_bar;
pub use code_editor::{CodeEditor, EditorContext, EditorSettings, PeerCursor};
pub use command_palette::{CommandId, CommandPalette, PaletteAction, PanelStates};
pub use confirm_dialog::{ConfirmAction, ConfirmDialog};
pub use inline_markdown::append_inline_markdown;
pub use spectrum::Spectrum;
pub use syntax_highlight::SyntaxThemePref;
pub use toast::{ToastLevel, ToastStack};
pub use vu_meter::VuMeter;
pub use spectrum_analyzer::SpectrumAnalyzer;
pub use waveform::Waveform;

pub fn smooth(buffer: &mut Vec<f32>, source: &[f32], factor: f32) {
    buffer.resize(source.len(), 0.0);
    for (b, &s) in buffer.iter_mut().zip(source) {
        *b = *b * factor + s * (1.0 - factor);
    }
}

/// Downsample using Largest-Triangle-Three-Buckets (LTTB).
/// Preserves waveform shape far better than min/max decimation.
pub fn downsample_lttb(output: &mut Vec<f32>, source: &[f32], target_len: usize) {
    output.clear();
    let n = source.len();
    if n == 0 || target_len == 0 {
        return;
    }
    if n <= target_len {
        output.extend_from_slice(source);
        return;
    }
    output.reserve(target_len);

    // Always keep first point
    output.push(source[0]);

    let bucket_count = target_len - 2;
    let bucket_size = (n - 2) as f64 / bucket_count as f64;
    let mut prev_selected = 0usize;

    for i in 0..bucket_count {
        let bucket_start = ((i as f64 * bucket_size) as usize) + 1;
        let bucket_end = (((i + 1) as f64 * bucket_size) as usize + 1).min(n - 1);

        // Average of the *next* bucket (look-ahead for triangle area)
        let next_start = bucket_end;
        let next_end = if i + 2 < bucket_count {
            (((i + 2) as f64 * bucket_size) as usize + 1).min(n - 1)
        } else {
            n - 1
        };
        let next_len = (next_end - next_start + 1).max(1) as f32;
        let avg_next: f32 =
            source[next_start..=next_end].iter().sum::<f32>() / next_len;
        let avg_next_x = (next_start + next_end) as f32 * 0.5;

        let prev_x = prev_selected as f32;
        let prev_y = source[prev_selected];

        let mut best_idx = bucket_start;
        let mut best_area = -1.0_f32;
        for (j, &sample) in source[bucket_start..=bucket_end].iter().enumerate() {
            let j = j + bucket_start;
            let area = ((prev_x - avg_next_x) * (sample - prev_y)
                - (prev_x - j as f32) * (avg_next - prev_y))
                .abs();
            if area > best_area {
                best_area = area;
                best_idx = j;
            }
        }

        output.push(source[best_idx]);
        prev_selected = best_idx;
    }

    // Always keep last point
    output.push(source[n - 1]);
}

/// Phosphor-style trace: tracks the min/max envelope of where the waveform
/// has been. Instantly expands to new extremes, slowly decays back.
pub fn apply_trace(
    trace: &mut Vec<(f32, f32)>,
    current: &[f32],
    persistence: f32,
) {
    if trace.len() != current.len() {
        trace.clear();
        trace.extend(current.iter().map(|&v| (v, v)));
        return;
    }

    let keep = persistence.clamp(0.0, 0.98);
    let take = 1.0 - keep;

    for (t, &v) in trace.iter_mut().zip(current) {
        t.0 = if v < t.0 { v } else { t.0 * keep + v * take };
        t.1 = if v > t.1 { v } else { t.1 * keep + v * take };
    }
}

pub fn align_trigger(buffer: &mut Vec<f32>, source: &[f32]) {
    buffer.clear();
    if source.is_empty() {
        return;
    }

    let len = source.len();
    if len < 3 {
        buffer.extend_from_slice(source);
        return;
    }

    let search_start = len / 16;
    let search_end = (len / 3).max(search_start + 1).min(len - 1);
    let mut best_index = None;
    let mut best_score = f32::MIN;

    for i in search_start..search_end {
        let a = source[i];
        let b = source[i + 1];
        if a <= 0.0 && b > 0.0 {
            let slope = b - a;
            let closeness = 1.0 - (a.abs() + b.abs()).min(1.0);
            let score = slope * 2.0 + closeness;
            if score > best_score {
                best_score = score;
                best_index = Some(i);
            }
        }
    }

    if let Some(start) = best_index {
        buffer.extend_from_slice(&source[start..]);
        buffer.extend_from_slice(&source[..start]);
    } else {
        buffer.extend_from_slice(source);
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

pub(crate) fn cycled_accent(accent: eframe::egui::Color32, index: usize) -> eframe::egui::Color32 {
    const N: usize = 8;
    if index % N == 0 {
        return accent;
    }
    let (h, s, l) = rgb_to_hsl(accent.r(), accent.g(), accent.b());
    let rotated = (h + (index % N) as f32 * (360.0 / N as f32)) % 360.0;
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
                        .button(crate::icons::rich(crate::icons::DOCK))
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
