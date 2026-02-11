use eframe::egui;

use crate::widgets::EditorSettings;

pub struct OptionsPanel {
    pub open: bool,
}

impl OptionsPanel {
    pub fn new() -> Self {
        Self { open: false }
    }

    pub fn show(&mut self, ctx: &egui::Context, editor_settings: &mut EditorSettings) {
        egui::Window::new("Options")
            .open(&mut self.open)
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                egui::CollapsingHeader::new("Editor")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Font size");
                            ui.add(
                                egui::DragValue::new(&mut editor_settings.font_size)
                                    .range(8.0..=32.0)
                                    .speed(0.5),
                            );
                        });
                        ui.checkbox(&mut editor_settings.line_numbers, "Line numbers");
                        ui.checkbox(&mut editor_settings.word_wrap, "Word wrap");
                        ui.checkbox(&mut editor_settings.show_whitespace, "Show whitespace");
                        ui.checkbox(
                            &mut editor_settings.highlight_current_line,
                            "Highlight current line",
                        );
                    });
            });
    }
}
