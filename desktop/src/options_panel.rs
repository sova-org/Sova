use eframe::egui;
use sova_core::vm::language::LanguageDefinition;

use crate::settings::{AppearanceSettings, DocSettings, DocSide, DocTrigger};
use crate::widgets::{EditorSettings, SyntaxThemePref};

pub struct OptionsPanel {
    system_fonts: Option<Vec<String>>,
}

impl OptionsPanel {
    pub fn new() -> Self {
        Self { system_fonts: None }
    }

    /// Returns `true` if appearance settings were changed.
    pub fn show_inside(
        &mut self,
        ui: &mut egui::Ui,
        editor_settings: &mut EditorSettings,
        appearance: &mut AppearanceSettings,
        doc_settings: &mut DocSettings,
        dismissed_tips: &mut Vec<String>,
        languages: &[LanguageDefinition],
    ) -> bool {
        let mut changed = false;
        let system_fonts = self
            .system_fonts
            .get_or_insert_with(crate::fonts::list_system_fonts);

        use crate::widgets::hint;

        egui::CollapsingHeader::new(t!("options.appearance"))
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("appearance_grid")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // --- Dropdowns & pickers ---

                        // UI font
                        ui.label(t!("options.ui_font"));
                        let ui_display = if appearance.ui_font.is_empty() {
                            "Default".to_string()
                        } else {
                            appearance.ui_font.clone()
                        };
                        egui::ComboBox::from_id_salt("ui_font_selector")
                            .selected_text(&ui_display)
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_value(
                                        &mut appearance.ui_font,
                                        String::new(),
                                        "Default",
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                                for font in system_fonts.iter() {
                                    if ui
                                        .selectable_value(
                                            &mut appearance.ui_font,
                                            font.clone(),
                                            font,
                                        )
                                        .changed()
                                    {
                                        changed = true;
                                    }
                                }
                            });
                        ui.end_row();

                        // Font size
                        let r = ui.label(t!("options.ui_font_size"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.ui_font_size"));
                        let r = ui.add(
                            egui::DragValue::new(&mut appearance.ui_font_size)
                                .range(10.0..=20.0)
                                .speed(0.5),
                        );
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.ui_font_size"));
                        changed |= r.changed();
                        ui.end_row();

                        // Accent color
                        let r = ui.label(t!("options.accent_color"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.accent_color"));
                        let r = ui.color_edit_button_srgb(&mut appearance.accent_color);
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.accent_color"));
                        changed |= r.changed();
                        ui.end_row();

                        // Language
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
                        ui.end_row();

                        // --- Sliders ---

                        // Scale
                        let r = ui.label(t!("options.ui_scale"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.ui_scale"));
                        let r = ui.add(
                            egui::Slider::new(&mut appearance.zoom, 0.75..=2.0)
                                .step_by(0.05)
                                .suffix("x"),
                        );
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.ui_scale"));
                        changed |= r.changed();
                        ui.end_row();

                        // Animation speed
                        let r = ui.label(t!("options.animation_speed"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.animation_speed"));
                        let r = ui.add(
                            egui::Slider::new(&mut appearance.animation_time, 0.0..=0.5)
                                .step_by(0.01)
                                .suffix("s"),
                        );
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.animation_speed"));
                        changed |= r.changed();
                        ui.end_row();

                        // Background brightness
                        let r = ui.label(t!("options.bg_brightness"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.bg_brightness"));
                        let mut brightness = appearance.bg_brightness as f32;
                        let r =
                            ui.add(egui::Slider::new(&mut brightness, 10.0..=50.0).step_by(1.0));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.bg_brightness"));
                        if r.changed() {
                            appearance.bg_brightness = brightness as u8;
                            changed = true;
                        }
                        ui.end_row();

                        // Scene opacity (only when visuals enabled)
                        if appearance.visuals_enabled {
                            ui.label(t!("options.scene_opacity"));
                            let r = ui.add(
                                egui::Slider::new(&mut appearance.scene_opacity, 0.0..=1.0),
                            );
                            changed |= r.changed();
                            ui.end_row();
                        }

                        // --- Checkboxes ---

                        // Window shadows
                        let r = ui.label(t!("options.window_shadows"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.window_shadows"));
                        let r = ui.checkbox(&mut appearance.window_shadows, "");
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.window_shadows"));
                        changed |= r.changed();
                        ui.end_row();

                        // Visuals enabled
                        let r = ui.label(t!("options.visuals_enabled"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.visuals_enabled"));
                        let r = ui.checkbox(&mut appearance.visuals_enabled, "");
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.visuals_enabled"));
                        changed |= r.changed();
                        ui.end_row();
                    });
            });

        egui::CollapsingHeader::new(t!("options.sidebar"))
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("sidebar_grid")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        let r = ui.label(t!("options.sidebar_side"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.sidebar_side"));
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut doc_settings.side,
                                DocSide::Left,
                                t!("options.sidebar_side.left"),
                            );
                            ui.radio_value(
                                &mut doc_settings.side,
                                DocSide::Right,
                                t!("options.sidebar_side.right"),
                            );
                        });
                        ui.end_row();

                        let r = ui.label(t!("options.sidebar_trigger"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.sidebar_trigger"));
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut doc_settings.trigger,
                                DocTrigger::Click,
                                t!("doc.trigger_click"),
                            );
                            ui.radio_value(
                                &mut doc_settings.trigger,
                                DocTrigger::Hover,
                                t!("doc.trigger_hover"),
                            );
                        });
                        ui.end_row();
                    });
            });

        egui::CollapsingHeader::new(t!("options.editor"))
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("editor_grid")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // Font size
                        let r = ui.label(t!("options.font_size"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.font_size"));
                        let r = ui.add(
                            egui::DragValue::new(&mut editor_settings.font_size)
                                .range(8.0..=32.0)
                                .speed(0.5),
                        );
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.font_size"));
                        ui.end_row();

                        // Editor font
                        ui.label(t!("options.editor_font"));
                        let editor_display = if appearance.editor_font.is_empty() {
                            "Default".to_string()
                        } else {
                            appearance.editor_font.clone()
                        };
                        egui::ComboBox::from_id_salt("editor_font_selector")
                            .selected_text(&editor_display)
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_value(
                                        &mut appearance.editor_font,
                                        String::new(),
                                        "Default",
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                                for font in system_fonts.iter() {
                                    if ui
                                        .selectable_value(
                                            &mut appearance.editor_font,
                                            font.clone(),
                                            font,
                                        )
                                        .changed()
                                    {
                                        changed = true;
                                    }
                                }
                            });
                        ui.end_row();

                        // Syntax theme
                        let r = ui.label(t!("options.syntax_theme"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.syntax_theme"));
                        let themes = [
                            (
                                SyntaxThemePref::OneDark,
                                t!("options.syntax_theme.one_dark"),
                            ),
                            (
                                SyntaxThemePref::Solarized,
                                t!("options.syntax_theme.solarized"),
                            ),
                            (
                                SyntaxThemePref::Phosphor,
                                t!("options.syntax_theme.phosphor"),
                            ),
                            (SyntaxThemePref::Dracula, t!("options.syntax_theme.dracula")),
                            (SyntaxThemePref::Monokai, t!("options.syntax_theme.monokai")),
                            (SyntaxThemePref::Gruvbox, t!("options.syntax_theme.gruvbox")),
                            (SyntaxThemePref::Nord, t!("options.syntax_theme.nord")),
                            (
                                SyntaxThemePref::Catppuccin,
                                t!("options.syntax_theme.catppuccin"),
                            ),
                            (
                                SyntaxThemePref::TokyoNight,
                                t!("options.syntax_theme.tokyo_night"),
                            ),
                        ];
                        let current_label = themes
                            .iter()
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
                        ui.end_row();

                        // Default language
                        let r = ui.label(t!("options.default_language"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.default_language"));
                        if languages.is_empty() {
                            ui.label(&editor_settings.default_language);
                        } else {
                            egui::ComboBox::from_id_salt("default_language_selector")
                                .selected_text(&editor_settings.default_language)
                                .show_ui(ui, |ui| {
                                    for lang in languages {
                                        ui.selectable_value(
                                            &mut editor_settings.default_language,
                                            lang.name.clone(),
                                            &lang.name,
                                        );
                                    }
                                });
                        }
                        ui.end_row();

                        // Line numbers
                        let r = ui.label(t!("options.line_numbers"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.line_numbers"));
                        ui.checkbox(&mut editor_settings.line_numbers, "");
                        ui.end_row();

                        // Word wrap
                        let r = ui.label(t!("options.word_wrap"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.word_wrap"));
                        ui.checkbox(&mut editor_settings.word_wrap, "");
                        ui.end_row();

                        // Show whitespace
                        let r = ui.label(t!("options.show_whitespace"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.show_whitespace"));
                        ui.checkbox(&mut editor_settings.show_whitespace, "");
                        ui.end_row();

                        // Highlight current line
                        let r = ui.label(t!("options.highlight_current_line"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.highlight_current_line"));
                        ui.checkbox(&mut editor_settings.highlight_current_line, "");
                        ui.end_row();

                        // Code completion
                        let r = ui.label(t!("options.code_completion"));
                        hint::on_hover(ui.ctx(), &r, t!("options.hint.code_completion"));
                        ui.checkbox(&mut editor_settings.code_completion, "");
                        ui.end_row();
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

        changed
    }
}
