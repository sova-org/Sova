mod about_dialog;
mod bottom_bar;
mod code_editor;
pub(crate) mod signal;
mod combo;
mod command_palette;
mod confirm_dialog;
pub mod hint;
mod inline_markdown;
pub mod inline_scene_view;
mod opacity;
pub mod shortcut;
mod spectrum;
pub mod spectrum_analyzer;
pub mod syntax_highlight;
mod toast;
mod vu_meter;
mod waveform;

pub use about_dialog::about_dialog;
pub use bottom_bar::bottom_bar;
pub use code_editor::{CodeEditor, EditorContext, EditorSettings, PeerCursor};
pub(crate) use code_editor::syntax_layout_job;
pub use combo::searchable_string_list as combo_searchable_string_list;
pub use combo::string_list as combo_string_list;
pub use command_palette::{CommandId, CommandPalette, PaletteAction, PanelStates};
pub use confirm_dialog::{ConfirmAction, ConfirmDialog};
pub use inline_markdown::append_inline_markdown;
pub use opacity::SceneOpacity;
pub use spectrum::Spectrum;
pub use spectrum_analyzer::SpectrumAnalyzer;
pub use syntax_highlight::SyntaxThemePref;
pub use toast::{ToastLevel, ToastStack};
pub use vu_meter::VuMeter;
pub use waveform::Waveform;
pub(crate) use signal::{smooth, decay_peaks, downsample_lttb, apply_trace, align_trigger};

/// Returns true when `response` lost focus because `key` was pressed, and
/// consumes that key so it cannot fall through to other keyboard handlers in
/// the same frame.
pub fn consume_key_on_lost_focus(
    ui: &mut eframe::egui::Ui,
    response: &eframe::egui::Response,
    key: eframe::egui::Key,
) -> bool {
    response.lost_focus() && ui.input_mut(|i| i.consume_key(eframe::egui::Modifiers::NONE, key))
}

/// Floating window inside the main viewport. Standardises every embedded
/// panel toggle (scope, spectrum, chat, sample browser, …) so the resizable +
/// collapsible + default_size + open scaffolding lives in one place.
///
/// Pass `frame = Some(...)` only when the default `egui::Window` frame needs
/// to be overridden (e.g. zero inner margin for the scope waveform).
pub fn embedded_window(
    ctx: &eframe::egui::Context,
    title: impl Into<eframe::egui::WidgetText>,
    open: &mut bool,
    default_size: [f32; 2],
    frame: Option<eframe::egui::Frame>,
    content: impl FnOnce(&mut eframe::egui::Ui),
) {
    use eframe::egui;
    let mut window = egui::Window::new(title)
        .open(open)
        .resizable(true)
        .collapsible(true)
        .default_size(default_size);
    if let Some(f) = frame {
        window = window.frame(f);
    }
    window.show(ctx, content);
}

pub fn show_detached_viewport(
    ctx: &eframe::egui::Context,
    open: &mut bool,
    detached: &mut bool,
    title: &str,
    size: [f32; 2],
    appearance: &crate::settings::AppearanceSettings,
    content: impl FnOnce(&mut eframe::egui::Ui),
) {
    use eframe::egui;

    let vp_id = egui::ViewportId::from_hash_of(title);
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

            crate::theme::apply_appearance(ctx, appearance);

            if ctx.input(|i| i.viewport().close_requested()) {
                *open = false;
                *detached = false;
                return;
            }

            egui::TopBottomPanel::top(egui::Id::new(title).with("toolbar")).show(ctx, |ui| {
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
