mod about_dialog;
mod bottom_bar;
mod code_editor;
mod command_palette;
mod confirm_dialog;
pub mod hint;
mod scene_grid;
mod spectrum;
mod step_editor;
pub mod syntax_highlight;
pub mod tip_popup;
mod vu_meter;
mod waveform;

pub use about_dialog::about_dialog;
pub use bottom_bar::bottom_bar;
pub use code_editor::{CodeEditor, EditorSettings};
pub use syntax_highlight::SyntaxThemePref;
pub use command_palette::{CommandId, CommandPalette, PaletteAction};
pub use confirm_dialog::{ConfirmAction, ConfirmDialog};
pub use scene_grid::{
    HeaderEditField, HeaderInlineEdit, InlineEdit, InlineEditAction, InlineEditRegion, SceneGrid,
    SceneGridResponse,
};
pub use spectrum::Spectrum;
pub use step_editor::StepEditorManager;
pub use vu_meter::VuMeter;
pub use waveform::Waveform;

pub const COLOR_OK: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(100, 200, 100);
pub const COLOR_ERROR: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(200, 100, 100);
pub const COLOR_MUTED: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(128, 128, 128);

pub fn username_color(name: &str) -> eframe::egui::Color32 {
    let mut hash: u32 = 0;
    for b in name.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as u32);
    }
    let hue = (hash % 360) as f32;
    let (r, g, b) = hsl_to_rgb(hue, 0.65, 0.55);
    eframe::egui::Color32::from_rgb(r, g, b)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
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

pub fn fuzzy_score(needle: &str, haystack: &str) -> Option<i32> {
    let needle: Vec<char> = needle.to_lowercase().chars().collect();
    let hay: Vec<char> = haystack.to_lowercase().chars().collect();
    let mut score: i32 = 0;
    let mut hi = 0;
    let mut prev_match = false;

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

    Some(score)
}
