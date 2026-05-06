use doux::types::{ModuleGroup, ModuleInfo, Source};
use eframe::egui;
use egui_commonmark::CommonMarkViewer;

use crate::client_bridge::ClientBridge;
use crate::icons;
use crate::widgets::EditorSettings;

use super::{
    find_clicked_hook, general_articles, resolve_article_link, tab_underline, toc_marker,
    DocPanel, DocView,
};

impl DocPanel {
    pub(crate) fn show_content(
        &mut self,
        ui: &mut egui::Ui,
        bridge: &ClientBridge,
        editor_settings: &EditorSettings,
    ) {
        let langs = bridge.languages();
        let doux_tab = 1 + langs.len();
        let tab_count = doux_tab + 1;
        self.selected_tab = self.selected_tab.min(tab_count - 1);

        egui::TopBottomPanel::top("doc_tabs").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let r = ui.selectable_label(
                    self.selected_tab == 0,
                    icons::button_text(ui, icons::BOOK, t!("doc.sova")),
                );
                if self.selected_tab == 0 {
                    tab_underline(ui, r.rect);
                }
                if r.clicked() {
                    self.selected_tab = 0;
                    self.search.clear();
                    self.view = None;
                    self.example_output = None;
                    self.edited_example.clear();
                    self.scroll_to_top = true;
                }
                for (i, lang) in langs.iter().enumerate() {
                    let tab_idx = i + 1;
                    let display_name = {
                        let mut c = lang.name.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().to_string() + c.as_str(),
                        }
                    };
                    let r = ui.selectable_label(
                        self.selected_tab == tab_idx,
                        icons::button_text(ui, icons::CODE, &display_name),
                    );
                    if self.selected_tab == tab_idx {
                        tab_underline(ui, r.rect);
                    }
                    if r.clicked() {
                        self.selected_tab = tab_idx;
                        self.search.clear();
                        self.view = None;
                        self.example_output = None;
                        self.edited_example.clear();
                        self.scroll_to_top = true;
                    }
                }

                let r = ui.selectable_label(
                    self.selected_tab == doux_tab,
                    icons::button_text(ui, icons::MUSIC_NOTE, "Doux"),
                );
                if self.selected_tab == doux_tab {
                    tab_underline(ui, r.rect);
                }
                if r.clicked() {
                    self.selected_tab = doux_tab;
                    self.search.clear();
                    self.view = None;
                    self.example_output = None;
                    self.edited_example.clear();
                    self.scroll_to_top = true;
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(t!("doc.filter").as_ref())
                        .weak()
                        .small(),
                );
                egui::TextEdit::singleline(&mut self.search)
                    .hint_text("…")
                    .desired_width(ui.available_width())
                    .show(ui);
            });
            ui.add_space(4.0);
        });

        let needle = self.search.to_lowercase();
        let selected = self.selected_tab;

        egui::SidePanel::left("doc_toc")
            .resizable(true)
            .default_width(140.0)
            .width_range(100.0..=220.0)
            .frame(
                egui::Frame::NONE
                    .inner_margin(4.0)
                    .fill(ui.visuals().panel_fill),
            )
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if selected == 0 {
                        self.show_general_toc(ui, &needle);
                    } else if selected == doux_tab {
                        self.show_doux_toc(ui, &needle);
                    } else {
                        let lang = &langs[selected - 1];
                        self.show_lang_toc(ui, &lang.documentation, &needle);
                    }
                });
            });

        let mut nav_target: Option<String> = None;

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.inner_margin(egui::Margin {
                left: 16,
                right: 16,
                top: 8,
                bottom: 8,
            }))
            .show_inside(ui, |ui| {
                let mut scroll = egui::ScrollArea::vertical();
                if self.scroll_to_top {
                    scroll = scroll.vertical_scroll_offset(0.0);
                    self.scroll_to_top = false;
                }
                scroll.show(ui, |ui| {
                    nav_target = if selected == 0 {
                        self.show_general_content(ui)
                    } else if selected == doux_tab {
                        self.show_doux_content(ui);
                        None
                    } else {
                        let lang = &langs[selected - 1];
                        self.show_lang_content(
                            ui,
                            &lang.name,
                            &lang.documentation,
                            bridge,
                            editor_settings,
                        )
                    };
                });
            });

        if let Some(slug) = nav_target
            && let Some(view) = resolve_article_link(&slug)
        {
            let tab = match &view {
                DocView::GeneralArticle(_) => 0,
                DocView::DouxModule(_) => doux_tab,
                _ => self.selected_tab,
            };
            self.selected_tab = tab;
            self.set_view(view);
            self.example_output = None;
            self.edited_example.clear();
        }
    }

    fn show_general_toc(&mut self, ui: &mut egui::Ui, needle: &str) {
        ui.strong(t!("doc.articles").as_ref());
        ui.add_space(4.0);
        for (i, (_, title, content)) in general_articles().iter().enumerate() {
            if !needle.is_empty()
                && !title.to_lowercase().contains(needle)
                && !content.to_lowercase().contains(needle)
            {
                continue;
            }
            let selected = self.view == Some(DocView::GeneralArticle(i));
            let r = ui.selectable_label(selected, *title);
            if selected {
                toc_marker(ui, r.rect);
            }
            if selected && self.scroll_toc {
                r.scroll_to_me(Some(egui::Align::Center));
                self.scroll_toc = false;
            }
            if r.clicked() {
                self.set_view(DocView::GeneralArticle(i));
                self.example_output = None;
            }
        }
    }

    fn show_general_content(&mut self, ui: &mut egui::Ui) -> Option<String> {
        let articles = general_articles();
        match &self.view {
            Some(DocView::GeneralArticle(idx)) => {
                if let Some((_, title, content)) = articles.get(*idx) {
                    if *idx == 0 {
                        show_welcome_header(ui);
                    } else {
                        ui.heading(*title);
                    }
                    ui.add_space(8.0);
                    CommonMarkViewer::new().show(ui, &mut self.md_cache, content);
                }
            }
            _ => {
                if let Some((_, _title, content)) = articles.first() {
                    show_welcome_header(ui);
                    ui.add_space(8.0);
                    CommonMarkViewer::new().show(ui, &mut self.md_cache, content);
                }
            }
        }
        find_clicked_hook(&self.md_cache)
    }


    fn show_doux_toc(&mut self, ui: &mut egui::Ui, needle: &str) {
        let modules = doux::all_modules();
        let searching = !needle.is_empty();

        let groups: &[(ModuleGroup, &str)] = &[
            (ModuleGroup::Source, "Sources"),
            (ModuleGroup::Synthesis, "Synthesis"),
            (ModuleGroup::Effect, "Effects"),
        ];

        for &(group, label) in groups {
            let group_modules: Vec<(usize, &&ModuleInfo)> = modules
                .iter()
                .enumerate()
                .filter(|(_, m)| m.group == group)
                .filter(|(_, m)| {
                    !searching
                        || m.name.contains(needle)
                        || m.description.to_lowercase().contains(needle)
                        || m.params.iter().any(|p| {
                            p.name.contains(needle) || p.description.to_lowercase().contains(needle)
                        })
                })
                .collect();

            if group_modules.is_empty() {
                continue;
            }

            let header =
                egui::CollapsingHeader::new(egui::RichText::new(label).strong().size(12.0))
                    .default_open(!searching)
                    .open(if searching { Some(true) } else { None });

            header.show(ui, |ui| {
                for (idx, module) in &group_modules {
                    let selected = self.view == Some(DocView::DouxModule(*idx));
                    let r = ui.selectable_label(selected, module.name);
                    if selected {
                        toc_marker(ui, r.rect);
                    }
                    if selected && self.scroll_toc {
                        r.scroll_to_me(Some(egui::Align::Center));
                        self.scroll_toc = false;
                    }
                    if r.clicked() {
                        self.set_view(DocView::DouxModule(*idx));
                    }
                }
            });
        }
    }

    fn show_doux_content(&mut self, ui: &mut egui::Ui) {
        let modules = doux::all_modules();
        let idx = match &self.view {
            Some(DocView::DouxModule(i)) => *i,
            _ => {
                ui.heading("Doux");
                ui.add_space(8.0);
                ui.label("Select a module from the sidebar to view its parameters.");
                return;
            }
        };

        let Some(module) = modules.get(idx) else {
            return;
        };

        let group_label = match module.group {
            ModuleGroup::Source => "Source",
            ModuleGroup::Synthesis => "Synthesis",
            ModuleGroup::Effect => "Effect",
        };
        ui.label(
            egui::RichText::new(group_label)
                .small()
                .color(ui.visuals().weak_text_color()),
        );

        ui.heading(module.name);

        // For sources, show aliases and category
        if module.group == ModuleGroup::Source {
            for source in Source::all() {
                let info = source.info();
                if info.module.name == module.name {
                    if !info.aliases.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("Aliases: {}", info.aliases.join(", ")))
                                .italics()
                                .color(ui.visuals().weak_text_color()),
                        );
                    }
                    ui.label(
                        egui::RichText::new(format!("{:?}", info.category))
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );
                    if let Some(d) = &info.drum_defaults {
                        ui.label(
                            egui::RichText::new(format!(
                                "Defaults: freq={} Hz, attack={}, decay={}, sustain={}, release={}",
                                d.freq, d.attack, d.decay, d.sustain, d.release
                            ))
                            .small()
                            .color(ui.visuals().weak_text_color()),
                        );
                    }
                    break;
                }
            }
        }

        ui.separator();
        ui.add_space(4.0);

        ui.label(module.description);

        if module.params.is_empty() {
            return;
        }

        ui.add_space(8.0);

        let accent = ui.visuals().selection.bg_fill;
        let dimmed = egui::Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 40);
        let weak = ui.visuals().weak_text_color();
        let mono = egui::FontId::monospace(13.0);

        for (i, param) in module.params.iter().enumerate() {
            if i > 0 {
                let rect = ui.available_rect_before_wrap();
                ui.painter().line_segment(
                    [rect.left_top(), egui::pos2(rect.right(), rect.top())],
                    egui::Stroke::new(crate::theme::STROKE_HAIRLINE, dimmed),
                );
                ui.add_space(4.0);
            }

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(param.name).font(mono.clone()).strong());
                if !param.aliases.is_empty() {
                    ui.label(
                        egui::RichText::new(format!("({})", param.aliases.join(", ")))
                            .small()
                            .color(weak),
                    );
                }
            });

            ui.label(param.description);

            if param.min != 0.0 || param.max != 0.0 {
                ui.label(
                    egui::RichText::new(format!(
                        "default: {}  range: {} .. {}",
                        param.default, param.min, param.max,
                    ))
                    .small()
                    .color(weak),
                );
            } else {
                ui.label(
                    egui::RichText::new(format!("default: {}", param.default))
                        .small()
                        .color(weak),
                );
            }

            ui.add_space(4.0);
        }

        // Prev / Next navigation
        let total = modules.len();
        ui.add_space(12.0);
        ui.separator();
        ui.horizontal(|ui| {
            if ui
                .add_enabled(idx > 0, egui::Button::new(icons::rich(icons::CHEVRON_LEFT)))
                .clicked()
            {
                self.set_view(DocView::DouxModule(idx - 1));
            }
            ui.label(format!("{} / {}", idx + 1, total));
            if ui
                .add_enabled(
                    idx + 1 < total,
                    egui::Button::new(icons::rich(icons::CHEVRON_RIGHT)),
                )
                .clicked()
            {
                self.set_view(DocView::DouxModule(idx + 1));
            }
        });
    }
}

pub(crate) fn show_welcome_header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add(
            egui::Image::new(egui::include_image!("../../../assets/icon.png"))
                .fit_to_exact_size(egui::vec2(48.0, 48.0)),
        );
        ui.vertical(|ui| {
            ui.heading(egui::RichText::new("Sova").size(24.0).strong());
            ui.label(egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).weak());
        });
    });
    ui.add_space(8.0);
    let accent = ui.visuals().selection.bg_fill;
    let dimmed = egui::Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 60);
    let rect = ui.available_rect_before_wrap();
    ui.painter().line_segment(
        [rect.left_top(), egui::pos2(rect.right(), rect.top())],
        egui::Stroke::new(crate::theme::STROKE_HAIRLINE, dimmed),
    );
    ui.add_space(8.0);
}
