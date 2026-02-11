mod about_dialog;
mod bottom_bar;

pub use about_dialog::about_dialog;
pub use bottom_bar::bottom_bar;

pub const COLOR_OK: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(100, 200, 100);
pub const COLOR_ERROR: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(200, 100, 100);
