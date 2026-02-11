use eframe::egui;

use crate::settings::{AppearanceSettings, SpacingPref, ThemePref};
use crate::widgets::EditorSettings;

pub struct OptionsPanel {
    pub open: bool,
}

impl OptionsPanel {
    pub fn new() -> Self {
        Self { open: false }
    }

    /// Returns `true` if appearance settings were changed.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        editor_settings: &mut EditorSettings,
        appearance: &mut AppearanceSettings,
    ) -> bool {
        let mut changed = false;

        egui::Window::new("Options")
            .open(&mut self.open)
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                egui::CollapsingHeader::new("Appearance")
                    .default_open(true)
                    .show(ui, |ui| {
                        // Theme
                        ui.label("Theme");
                        ui.horizontal(|ui| {
                            changed |= ui
                                .selectable_value(&mut appearance.theme, ThemePref::System, "System")
                                .changed();
                            changed |= ui
                                .selectable_value(&mut appearance.theme, ThemePref::Dark, "Dark")
                                .changed();
                            changed |= ui
                                .selectable_value(&mut appearance.theme, ThemePref::Light, "Light")
                                .changed();
                        });

                        ui.add_space(4.0);

                        // UI Scale
                        ui.label("UI Scale");
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut appearance.zoom, 0.75..=2.0)
                                    .step_by(0.05)
                                    .suffix("x"),
                            )
                            .changed();

                        ui.add_space(4.0);

                        // Accent Color
                        ui.horizontal(|ui| {
                            ui.label("Accent color");
                            changed |= ui.color_edit_button_srgb(&mut appearance.accent_color).changed();
                        });

                        ui.add_space(4.0);

                        // Spacing
                        ui.label("Spacing");
                        changed |= ui
                            .radio_value(&mut appearance.spacing, SpacingPref::Compact, "Compact")
                            .changed();
                        changed |= ui
                            .radio_value(&mut appearance.spacing, SpacingPref::Normal, "Normal")
                            .changed();
                        changed |= ui
                            .radio_value(&mut appearance.spacing, SpacingPref::Comfortable, "Comfortable")
                            .changed();

                        ui.add_space(4.0);

                        // Window Shadows
                        changed |= ui
                            .checkbox(&mut appearance.window_shadows, "Window shadows")
                            .changed();
                    });

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

        changed
    }
}
