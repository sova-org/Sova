mod about_dialog;
mod bottom_bar;
mod code_editor;
mod command_palette;
mod confirm_dialog;
pub mod hint;
mod scene_grid;
mod spectrum;
mod step_editor;
mod vu_meter;
mod waveform;

pub use about_dialog::about_dialog;
pub use bottom_bar::bottom_bar;
pub use code_editor::{CodeEditor, EditorSettings};
pub use command_palette::{CommandId, CommandPalette, PaletteAction};
pub use confirm_dialog::{ConfirmAction, ConfirmDialog};
pub use scene_grid::{InlineEdit, InlineEditAction, InlineEditRegion, SceneGrid, SceneGridResponse};
pub use spectrum::Spectrum;
pub use step_editor::StepEditorManager;
pub use vu_meter::VuMeter;
pub use waveform::Waveform;

pub const COLOR_OK: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(100, 200, 100);
pub const COLOR_ERROR: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(200, 100, 100);
pub const COLOR_MUTED: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(128, 128, 128);

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
                    if ui.button(crate::icons::DOCK).on_hover_text("Dock back").clicked() {
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
