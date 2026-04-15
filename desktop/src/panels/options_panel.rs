use eframe::egui;
use sova_core::vm::language::LanguageDefinition;

use crate::scene_panel::ViewMode;
use crate::settings::{AppearanceSettings, DocSettings, DocSide, DocTrigger};
use crate::widgets::{EditorSettings, SyntaxThemePref};

pub struct OptionsPanel {
    system_fonts: Option<Vec<String>>,
}

fn options_grid(id: &'static str) -> egui::Grid {
    egui::Grid::new(id).num_columns(2).spacing([8.0, 4.0])
}

impl OptionsPanel {
    pub fn new() -> Self {
        Self { system_fonts: None }
    }

    /// Returns `true` if appearance settings were changed.
    #[allow(clippy::too_many_arguments)]
    pub fn show_inside(
        &mut self,
        ui: &mut egui::Ui,
        editor_settings: &mut EditorSettings,
        appearance: &mut AppearanceSettings,
        doc_settings: &mut DocSettings,
        languages: &[LanguageDefinition],
        view_mode: &mut ViewMode,
        show_phase_bar: &mut bool,
    ) -> bool {
        let mut changed = false;
        let system_fonts = self
            .system_fonts
            .get_or_insert_with(crate::fonts::list_system_fonts);

        use crate::widgets::hint;

        egui::CollapsingHeader::new(t!("options.appearance"))
            .default_open(true)
            .show(ui, |ui| {
                options_grid("appearance_grid").show(ui, |ui| {
                    // --- Dropdowns & pickers ---

                    // UI font
                    ui.label(t!("options.ui_font"));
                    if crate::widgets::combo_searchable_string_list(
                        ui,
                        "ui_font_selector",
                        &mut appearance.ui_font,
                        Some("Default"),
                        system_fonts,
                    ) {
                        changed = true;
                    }
                    ui.end_row();

                    // Font size
                    let r = hint::labeled(
                        ui,
                        t!("options.ui_font_size"),
                        t!("options.hint.ui_font_size"),
                        |ui| {
                            ui.add(
                                egui::DragValue::new(&mut appearance.ui_font_size)
                                    .range(10.0..=20.0)
                                    .speed(0.5),
                            )
                        },
                    );
                    changed |= r.changed();
                    ui.end_row();

                    // Accent color
                    let r = hint::labeled(
                        ui,
                        t!("options.accent_color"),
                        t!("options.hint.accent_color"),
                        |ui| ui.color_edit_button_srgb(&mut appearance.accent_color),
                    );
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
                    let r = hint::labeled(
                        ui,
                        t!("options.ui_scale"),
                        t!("options.hint.ui_scale"),
                        |ui| {
                            ui.add(
                                egui::Slider::new(&mut appearance.zoom, 0.75..=2.0)
                                    .step_by(0.05)
                                    .suffix("x"),
                            )
                        },
                    );
                    changed |= r.changed();
                    ui.end_row();

                    // Animation speed
                    let r = hint::labeled(
                        ui,
                        t!("options.animation_speed"),
                        t!("options.hint.animation_speed"),
                        |ui| {
                            ui.add(
                                egui::Slider::new(&mut appearance.animation_time, 0.0..=0.5)
                                    .step_by(0.01)
                                    .suffix("s"),
                            )
                        },
                    );
                    changed |= r.changed();
                    ui.end_row();

                    // Background brightness
                    let mut brightness = appearance.bg_brightness as f32;
                    let r = hint::labeled(
                        ui,
                        t!("options.bg_brightness"),
                        t!("options.hint.bg_brightness"),
                        |ui| ui.add(egui::Slider::new(&mut brightness, 10.0..=50.0).step_by(1.0)),
                    );
                    if r.changed() {
                        appearance.bg_brightness = brightness as u8;
                        changed = true;
                    }
                    ui.end_row();

                    // Scene opacity (only when visuals enabled)
                    if appearance.visuals_enabled {
                        ui.label(t!("options.scene_opacity"));
                        let r = ui.add(egui::Slider::new(&mut appearance.scene_opacity, 0.0..=1.0));
                        changed |= r.changed();
                        ui.end_row();
                    }

                    // --- Checkboxes ---

                    // Window shadows
                    let r = hint::labeled(
                        ui,
                        t!("options.window_shadows"),
                        t!("options.hint.window_shadows"),
                        |ui| ui.checkbox(&mut appearance.window_shadows, ""),
                    );
                    changed |= r.changed();
                    ui.end_row();

                    // Visuals enabled
                    let r = hint::labeled(
                        ui,
                        t!("options.visuals_enabled"),
                        t!("options.hint.visuals_enabled"),
                        |ui| ui.checkbox(&mut appearance.visuals_enabled, ""),
                    );
                    changed |= r.changed();
                    ui.end_row();
                });
            });

        egui::CollapsingHeader::new(t!("options.sidebar"))
            .default_open(true)
            .show(ui, |ui| {
                options_grid("sidebar_grid").show(ui, |ui| {
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

        egui::CollapsingHeader::new(t!("options.scene"))
            .default_open(true)
            .show(ui, |ui| {
                options_grid("scene_grid").show(ui, |ui| {
                    ui.label(t!("options.scene_view_mode"));
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            view_mode,
                            ViewMode::Sequencer,
                            t!("options.scene_view_mode.sequencer"),
                        );
                        ui.radio_value(
                            view_mode,
                            ViewMode::Classic,
                            t!("options.scene_view_mode.classic"),
                        );
                    });
                    ui.end_row();

                    hint::labeled(
                        ui,
                        t!("options.show_phase_bar"),
                        t!("options.hint.show_phase_bar"),
                        |ui| ui.checkbox(show_phase_bar, ""),
                    );
                    ui.end_row();
                });
            });

        egui::CollapsingHeader::new(t!("options.editor"))
            .default_open(true)
            .show(ui, |ui| {
                options_grid("editor_grid").show(ui, |ui| {
                    // Font size
                    hint::labeled(
                        ui,
                        t!("options.font_size"),
                        t!("options.hint.font_size"),
                        |ui| {
                            ui.add(
                                egui::DragValue::new(&mut editor_settings.font_size)
                                    .range(8.0..=32.0)
                                    .speed(0.5),
                            )
                        },
                    );
                    ui.end_row();

                    // Editor font
                    ui.label(t!("options.editor_font"));
                    if crate::widgets::combo_searchable_string_list(
                        ui,
                        "editor_font_selector",
                        &mut appearance.editor_font,
                        Some("Default"),
                        system_fonts,
                    ) {
                        changed = true;
                    }
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
                        let names: Vec<&str> = languages.iter().map(|l| l.name.as_str()).collect();
                        crate::widgets::combo_string_list(
                            ui,
                            "default_language_selector",
                            &mut editor_settings.default_language,
                            None,
                            &names,
                        );
                    }
                    ui.end_row();

                    hint::labeled(
                        ui,
                        t!("options.line_numbers"),
                        t!("options.hint.line_numbers"),
                        |ui| ui.checkbox(&mut editor_settings.line_numbers, ""),
                    );
                    ui.end_row();

                    hint::labeled(
                        ui,
                        t!("options.word_wrap"),
                        t!("options.hint.word_wrap"),
                        |ui| ui.checkbox(&mut editor_settings.word_wrap, ""),
                    );
                    ui.end_row();

                    hint::labeled(
                        ui,
                        t!("options.show_whitespace"),
                        t!("options.hint.show_whitespace"),
                        |ui| ui.checkbox(&mut editor_settings.show_whitespace, ""),
                    );
                    ui.end_row();

                    hint::labeled(
                        ui,
                        t!("options.highlight_current_line"),
                        t!("options.hint.highlight_current_line"),
                        |ui| ui.checkbox(&mut editor_settings.highlight_current_line, ""),
                    );
                    ui.end_row();

                    hint::labeled(
                        ui,
                        t!("options.code_completion"),
                        t!("options.hint.code_completion"),
                        |ui| ui.checkbox(&mut editor_settings.code_completion, ""),
                    );
                    ui.end_row();
                });
            });

        changed
    }
}
