mod about_dialog;
mod bottom_bar;
mod code_editor;
mod command_palette;
mod confirm_dialog;
mod scene_grid;
mod scope;
mod spectrum;
mod step_editor;
mod vu_meter;

pub use about_dialog::about_dialog;
pub use bottom_bar::bottom_bar;
pub use code_editor::{CodeEditor, EditorSettings};
pub use command_palette::{CommandId, CommandPalette, PaletteAction};
pub use confirm_dialog::{ConfirmAction, ConfirmDialog};
pub use scene_grid::{InlineEdit, InlineEditAction, InlineEditRegion, SceneGrid, SceneGridResponse};
pub use scope::Scope;
pub use spectrum::Spectrum;
pub use step_editor::StepEditorManager;
pub use vu_meter::VuMeter;

pub const COLOR_OK: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(100, 200, 100);
pub const COLOR_ERROR: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(200, 100, 100);
pub const COLOR_MUTED: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(128, 128, 128);
