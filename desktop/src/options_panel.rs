use eframe::egui;

use crate::settings::{AppearanceSettings, SpacingPref, ThemePref};
use crate::widgets::{EditorSettings, SyntaxThemePref};

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
        dismissed_tips: &mut Vec<String>,
    ) -> bool {
        let mut changed = false;

        egui::Window::new(t!("options.title"))
            .open(&mut self.open)
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                use crate::widgets::hint;

                egui::CollapsingHeader::new(t!("options.appearance"))
                    .default_open(true)
                    .show(ui, |ui| {
                        let r = ui.label(t!("options.theme"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.theme"));
                        ui.horizontal(|ui| {
                            changed |= ui
                                .selectable_value(
                                    &mut appearance.theme,
                                    ThemePref::System,
                                    t!("options.theme.system"),
                                )
                                .changed();
                            changed |= ui
                                .selectable_value(
                                    &mut appearance.theme,
                                    ThemePref::Dark,
                                    t!("options.theme.dark"),
                                )
                                .changed();
                            changed |= ui
                                .selectable_value(
                                    &mut appearance.theme,
                                    ThemePref::Light,
                                    t!("options.theme.light"),
                                )
                                .changed();
                        });

                        ui.add_space(4.0);

                        let r = ui.label(t!("options.ui_scale"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.ui_scale"));
                        let r = ui.add(
                            egui::Slider::new(&mut appearance.zoom, 0.75..=2.0)
                                .step_by(0.05)
                                .suffix("x"),
                        );
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.ui_scale"));
                        changed |= r.changed();

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            let r = ui.label(t!("options.accent_color"));
                            hint::on_hover(ui.ctx(), &r, t!("options.hint.accent_color"));
                            let r = ui.color_edit_button_srgb(&mut appearance.accent_color);
                            hint::on_hover(ui.ctx(), &r, t!("options.hint.accent_color"));
                            changed |= r.changed();
                        });

                        ui.add_space(4.0);

                        let r = ui.label(t!("options.spacing"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.spacing"));
                        changed |= ui
                            .radio_value(
                                &mut appearance.spacing,
                                SpacingPref::Compact,
                                t!("options.spacing.compact"),
                            )
                            .changed();
                        changed |= ui
                            .radio_value(
                                &mut appearance.spacing,
                                SpacingPref::Normal,
                                t!("options.spacing.normal"),
                            )
                            .changed();
                        changed |= ui
                            .radio_value(
                                &mut appearance.spacing,
                                SpacingPref::Comfortable,
                                t!("options.spacing.comfortable"),
                            )
                            .changed();

                        ui.add_space(4.0);

                        let r = ui.label(t!("options.language"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.language"));
                        let locales = rust_i18n::available_locales!();
                        let current = appearance.locale.clone();
                        egui::ComboBox::from_id_salt("locale_selector")
                            .selected_text(&current)
                            .show_ui(ui, |ui| {
                                for locale in locales {
                                    if ui
                                        .selectable_value(
                                            &mut appearance.locale,
                                            locale.to_string(),
                                            locale,
                                        )
                                        .changed()
                                    {
                                        rust_i18n::set_locale(locale);
                                        changed = true;
                                    }
                                }
                            });

                        ui.add_space(4.0);

                        let r = ui
                            .checkbox(&mut appearance.window_shadows, t!("options.window_shadows"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.window_shadows"));
                        changed |= r.changed();

                        let r = ui.checkbox(
                            &mut appearance.visuals_enabled,
                            t!("options.visuals_enabled"),
                        );
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.visuals_enabled"));
                        changed |= r.changed();
                    });

                egui::CollapsingHeader::new(t!("options.editor"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let r = ui.label(t!("options.font_size"));
                            hint::on_hover(ui.ctx(), &r, t!("options.hint.font_size"));
                            let r = ui.add(
                                egui::DragValue::new(&mut editor_settings.font_size)
                                    .range(8.0..=32.0)
                                    .speed(0.5),
                            );
                            hint::on_hover(ui.ctx(), &r, t!("options.hint.font_size"));
                        });
                        let r = ui.checkbox(
                            &mut editor_settings.line_numbers,
                            t!("options.line_numbers"),
                        );
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.line_numbers"));
                        let r =
                            ui.checkbox(&mut editor_settings.word_wrap, t!("options.word_wrap"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.word_wrap"));
                        let r = ui.checkbox(
                            &mut editor_settings.show_whitespace,
                            t!("options.show_whitespace"),
                        );
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.show_whitespace"));
                        let r = ui.checkbox(
                            &mut editor_settings.highlight_current_line,
                            t!("options.highlight_current_line"),
                        );
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.highlight_current_line"));

                        ui.add_space(4.0);

                        let r = ui.label(t!("options.syntax_theme"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.syntax_theme"));
                        let themes = [
                            (SyntaxThemePref::OneDark, t!("options.syntax_theme.one_dark")),
                            (SyntaxThemePref::Solarized, t!("options.syntax_theme.solarized")),
                            (SyntaxThemePref::Phosphor, t!("options.syntax_theme.phosphor")),
                            (SyntaxThemePref::Dracula, t!("options.syntax_theme.dracula")),
                            (SyntaxThemePref::Monokai, t!("options.syntax_theme.monokai")),
                            (SyntaxThemePref::Gruvbox, t!("options.syntax_theme.gruvbox")),
                            (SyntaxThemePref::Nord, t!("options.syntax_theme.nord")),
                            (SyntaxThemePref::Catppuccin, t!("options.syntax_theme.catppuccin")),
                            (SyntaxThemePref::TokyoNight, t!("options.syntax_theme.tokyo_night")),
                        ];
                        let current_label = themes.iter()
                            .find(|(v, _)| *v == editor_settings.syntax_theme)
                            .map(|(_, l)| l.as_ref())
                            .unwrap_or("");
                        egui::ComboBox::from_id_salt("syntax_theme_selector")
                            .selected_text(current_label)
                            .show_ui(ui, |ui| {
                                for (value, label) in &themes {
                                    ui.selectable_value(
                                        &mut editor_settings.syntax_theme,
                                        *value,
                                        label.as_ref(),
                                    );
                                }
                            });
                    });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                let btn = ui.add_enabled(
                    !dismissed_tips.is_empty(),
                    egui::Button::new(t!("tip.reset")),
                );
                if btn.clicked() {
                    dismissed_tips.clear();
                }
            });

        changed
    }
}
