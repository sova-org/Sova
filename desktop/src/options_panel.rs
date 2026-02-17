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
                let ctx = ui.ctx().clone();
                let hint = |r: &egui::Response, text: &'static str| {
                    if r.hovered() { crate::widgets::hint::set(&ctx, text); }
                };

                egui::CollapsingHeader::new("Appearance")
                    .default_open(true)
                    .show(ui, |ui| {
                        hint(&ui.label("Theme"), "Color scheme for the interface");
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

                        hint(&ui.label("UI Scale"), "Global zoom level for all UI elements");
                        let r = ui.add(
                            egui::Slider::new(&mut appearance.zoom, 0.75..=2.0)
                                .step_by(0.05)
                                .suffix("x"),
                        );
                        hint(&r, "Global zoom level for all UI elements");
                        changed |= r.changed();

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            hint(&ui.label("Accent color"), "Primary highlight color used across the UI");
                            let r = ui.color_edit_button_srgb(&mut appearance.accent_color);
                            hint(&r, "Primary highlight color used across the UI");
                            changed |= r.changed();
                        });

                        ui.add_space(4.0);

                        hint(&ui.label("Spacing"), "Density of UI elements");
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

                        let r = ui.checkbox(&mut appearance.window_shadows, "Window shadows");
                        hint(&r, "Enable drop shadows on floating windows");
                        changed |= r.changed();
                    });

                egui::CollapsingHeader::new("Editor")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            hint(&ui.label("Font size"), "Code editor font size in points");
                            let r = ui.add(
                                egui::DragValue::new(&mut editor_settings.font_size)
                                    .range(8.0..=32.0)
                                    .speed(0.5),
                            );
                            hint(&r, "Code editor font size in points");
                        });
                        hint(
                            &ui.checkbox(&mut editor_settings.line_numbers, "Line numbers"),
                            "Show line numbers in the code editor",
                        );
                        hint(
                            &ui.checkbox(&mut editor_settings.word_wrap, "Word wrap"),
                            "Wrap long lines instead of horizontal scrolling",
                        );
                        hint(
                            &ui.checkbox(&mut editor_settings.show_whitespace, "Show whitespace"),
                            "Render spaces and tabs as visible markers",
                        );
                        hint(
                            &ui.checkbox(
                                &mut editor_settings.highlight_current_line,
                                "Highlight current line",
                            ),
                            "Highlight the line where the cursor is",
                        );
                    });
            });

        changed
    }
}
