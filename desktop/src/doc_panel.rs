use std::collections::BTreeMap;

use crate::client_bridge::ClientBridge;
use crate::icons;
use crate::settings::{DocSettings, DocSide, DocTrigger};
use eframe::egui;
use egui::containers::panel::Side;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use sova_core::scene::script::Script;
use sova_core::schedule::SchedulerMessage;
use sova_core::vm::language::{LanguageDocumentation, LanguageElement};
use sova_server::ClientMessage;

const GENERAL_ARTICLES_EN: &[(&str, &str)] = &[(
    "Getting Started",
    include_str!("../docs/en/getting-started.md"),
)];
const GENERAL_ARTICLES_FR: &[(&str, &str)] = &[(
    "Pour commencer",
    include_str!("../docs/fr/getting-started.md"),
)];
const GENERAL_ARTICLES_ES: &[(&str, &str)] = &[(
    "Primeros pasos",
    include_str!("../docs/es/getting-started.md"),
)];
const GENERAL_ARTICLES_IT: &[(&str, &str)] = &[(
    "Per iniziare",
    include_str!("../docs/it/getting-started.md"),
)];

fn general_articles() -> &'static [(&'static str, &'static str)] {
    let locale = rust_i18n::locale();
    match locale.as_ref() {
        "fr" => GENERAL_ARTICLES_FR,
        "es" => GENERAL_ARTICLES_ES,
        "it" => GENERAL_ARTICLES_IT,
        _ => GENERAL_ARTICLES_EN,
    }
}

const COLLAPSED_WIDTH: f32 = 24.0;
const HOVER_DELAY_SECS: f64 = 0.2;

#[derive(Clone, PartialEq)]
enum DocView {
    GeneralArticle(usize),
    LangArticle(usize),
    LangReference(usize),
}

pub struct DocPanel {
    pub settings: DocSettings,
    hover_expanded: bool,
    hover_timer: Option<f64>,
    docs: BTreeMap<String, LanguageDocumentation>,
    selected_tab: usize,
    search: String,
    md_cache: CommonMarkCache,
    view: Option<DocView>,
    example_output: Option<Result<String, String>>,
    edited_example: String,
}

impl DocPanel {
    pub fn new(settings: DocSettings) -> Self {
        let center = langs::create_language_center();
        let mut docs = BTreeMap::new();
        for (name, (doc, _syn)) in center.all_languages_definitions() {
            if !doc.reference.is_empty() || !doc.articles.is_empty() {
                docs.insert(name, doc);
            }
        }

        Self {
            settings,
            hover_expanded: false,
            hover_timer: None,
            docs,
            selected_tab: 0,
            search: String::new(),
            md_cache: CommonMarkCache::default(),
            view: None,
            example_output: None,
            edited_example: String::new(),
        }
    }

    pub fn is_expanded(&self) -> bool {
        !self.settings.collapsed || self.hover_expanded
    }

    pub fn show_side_panel(&mut self, ctx: &egui::Context, bridge: &ClientBridge) {
        let side = match self.settings.side {
            DocSide::Left => Side::Left,
            DocSide::Right => Side::Right,
        };

        if self.is_expanded() {
            self.show_expanded(ctx, bridge, side);
        } else {
            self.show_collapsed(ctx, side);
        }
    }

    fn show_collapsed(&mut self, ctx: &egui::Context, side: Side) {
        let panel = egui::SidePanel::new(side, "doc_panel_collapsed")
            .exact_width(COLLAPSED_WIDTH)
            .resizable(false)
            .show_separator_line(false);

        let r = panel.show(ctx, |ui| {
            let center = ui.max_rect().center();
            let icon = egui::RichText::new(icons::BOOK)
                .color(ui.visuals().weak_text_color())
                .size(16.0);
            ui.put(
                egui::Rect::from_center_size(center, egui::vec2(COLLAPSED_WIDTH, 24.0)),
                egui::Label::new(icon),
            );
        });

        let strip_rect = r.response.rect;
        let hovering =
            ctx.input(|i| strip_rect.contains(i.pointer.hover_pos().unwrap_or_default()));
        let clicked = hovering && ctx.input(|i| i.pointer.primary_clicked());

        match self.settings.trigger {
            DocTrigger::Click => {
                if hovering {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if clicked {
                    self.settings.collapsed = false;
                    self.settings.pinned = true;
                }
            }
            DocTrigger::Hover => {
                if hovering {
                    let now = ctx.input(|i| i.time);
                    if let Some(start) = self.hover_timer {
                        if now - start >= HOVER_DELAY_SECS {
                            self.hover_expanded = true;
                            self.hover_timer = None;
                        }
                    } else {
                        self.hover_timer = Some(now);
                    }
                    ctx.request_repaint();
                } else {
                    self.hover_timer = None;
                }
            }
        }
    }

    fn show_expanded(&mut self, ctx: &egui::Context, bridge: &ClientBridge, side: Side) {
        let panel = egui::SidePanel::new(side, "doc_panel_expanded")
            .default_width(self.settings.width)
            .width_range(200.0..=800.0)
            .resizable(true);

        let r = panel.show(ctx, |ui| {
            self.show_header(ui);
            ui.separator();
            self.show_content(ui, bridge);
        });

        self.settings.width = r.response.rect.width();

        if self.hover_expanded && !self.settings.pinned {
            let panel_rect = r.response.rect;
            let hovering =
                ctx.input(|i| panel_rect.contains(i.pointer.hover_pos().unwrap_or_default()));
            if !hovering {
                self.hover_expanded = false;
            }
        }
    }

    fn show_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.strong(t!("doc.title").as_ref());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let collapse_icon = match self.settings.side {
                    DocSide::Left => icons::CHEVRON_LEFT,
                    DocSide::Right => icons::CHEVRON_RIGHT,
                };
                if ui
                    .button(collapse_icon)
                    .on_hover_text(t!("doc.collapse"))
                    .clicked()
                {
                    if self.hover_expanded {
                        self.hover_expanded = false;
                    } else {
                        self.settings.collapsed = true;
                    }
                }

                if ui
                    .button(icons::SWAP)
                    .on_hover_text(t!("doc.swap_side"))
                    .clicked()
                {
                    self.settings.side = match self.settings.side {
                        DocSide::Left => DocSide::Right,
                        DocSide::Right => DocSide::Left,
                    };
                }
            });
        });
    }

    fn show_content(&mut self, ui: &mut egui::Ui, bridge: &ClientBridge) {
        let lang_names: Vec<String> = self.docs.keys().cloned().collect();
        let tab_count = 1 + lang_names.len();
        self.selected_tab = self.selected_tab.min(tab_count - 1);

        egui::TopBottomPanel::top("doc_tabs").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.selected_tab == 0, t!("doc.sova").as_ref())
                    .clicked()
                {
                    self.selected_tab = 0;
                    self.search.clear();
                    self.view = None;
                    self.example_output = None;
                    self.edited_example.clear();
                }
                for (i, name) in lang_names.iter().enumerate() {
                    let tab_idx = i + 1;
                    if ui
                        .selectable_label(self.selected_tab == tab_idx, name)
                        .clicked()
                    {
                        self.selected_tab = tab_idx;
                        self.search.clear();
                        self.view = None;
                        self.example_output = None;
                        self.edited_example.clear();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label(t!("doc.filter").as_ref());
                ui.text_edit_singleline(&mut self.search);
            });
        });

        let needle = self.search.to_lowercase();
        let selected = self.selected_tab;

        egui::SidePanel::left("doc_toc")
            .resizable(true)
            .default_width(140.0)
            .width_range(100.0..=220.0)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if selected == 0 {
                        self.show_general_toc(ui, &needle);
                    } else {
                        let lang = &lang_names[selected - 1];
                        self.show_lang_toc(ui, lang, &needle);
                    }
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if selected == 0 {
                    self.show_general_content(ui);
                } else {
                    let lang = lang_names[selected - 1].clone();
                    self.show_lang_content(ui, &lang, bridge);
                }
            });
        });
    }

    fn show_general_toc(&mut self, ui: &mut egui::Ui, needle: &str) {
        ui.strong(t!("doc.articles").as_ref());
        for (i, (title, content)) in general_articles().iter().enumerate() {
            if !needle.is_empty()
                && !title.to_lowercase().contains(needle)
                && !content.to_lowercase().contains(needle)
            {
                continue;
            }
            let selected = self.view == Some(DocView::GeneralArticle(i));
            if ui.selectable_label(selected, *title).clicked() {
                self.view = Some(DocView::GeneralArticle(i));
                self.example_output = None;
            }
        }
    }

    fn show_general_content(&mut self, ui: &mut egui::Ui) {
        let articles = general_articles();
        match &self.view {
            Some(DocView::GeneralArticle(idx)) => {
                if let Some((title, content)) = articles.get(*idx) {
                    ui.heading(*title);
                    ui.add_space(4.0);
                    CommonMarkViewer::new().show(ui, &mut self.md_cache, content);
                }
            }
            _ => {
                if let Some((title, content)) = articles.first() {
                    ui.heading(*title);
                    ui.add_space(4.0);
                    CommonMarkViewer::new().show(ui, &mut self.md_cache, content);
                }
            }
        }
    }

    fn show_lang_toc(&mut self, ui: &mut egui::Ui, lang: &str, needle: &str) {
        let doc = &self.docs[lang];

        if !doc.articles.is_empty() {
            ui.strong(t!("doc.articles").as_ref());
            for (i, (title, content)) in doc.articles.iter().enumerate() {
                if !needle.is_empty()
                    && !title.to_lowercase().contains(needle)
                    && !content.to_lowercase().contains(needle)
                {
                    continue;
                }
                let selected = self.view == Some(DocView::LangArticle(i));
                if ui.selectable_label(selected, title).clicked() {
                    self.view = Some(DocView::LangArticle(i));
                    self.example_output = None;
                    self.edited_example.clear();
                }
            }
            ui.add_space(8.0);
        }

        if !doc.reference.is_empty() {
            ui.strong(t!("doc.reference").as_ref());
            let ref_keys: Vec<_> = doc.reference.keys().collect();
            for (i, elem) in ref_keys.iter().enumerate() {
                let label = element_label(elem);
                let entry = &doc.reference[*elem];
                if !needle.is_empty()
                    && !label.to_lowercase().contains(needle)
                    && !entry.description.to_lowercase().contains(needle)
                {
                    continue;
                }
                let selected = self.view == Some(DocView::LangReference(i));
                if ui.selectable_label(selected, &label).clicked() {
                    self.view = Some(DocView::LangReference(i));
                    self.example_output = None;
                    self.edited_example = entry.example.clone().unwrap_or_default();
                }
            }
        }
    }

    fn show_lang_content(&mut self, ui: &mut egui::Ui, lang: &str, bridge: &ClientBridge) {
        let doc = self.docs[lang].clone();
        match &self.view {
            Some(DocView::LangArticle(idx)) => {
                if let Some((title, content)) = doc.articles.get(*idx) {
                    ui.heading(title);
                    ui.add_space(4.0);
                    CommonMarkViewer::new().show(ui, &mut self.md_cache, content);
                }
            }
            Some(DocView::LangReference(idx)) => {
                let ref_entries: Vec<_> = doc.reference.iter().collect();
                if let Some((elem, entry)) = ref_entries.get(*idx) {
                    ui.heading(element_label(elem));
                    ui.add_space(4.0);
                    ui.label(&entry.description);

                    if let Some(example) = &entry.example {
                        if self.edited_example.is_empty() {
                            self.edited_example = example.clone();
                        }

                        ui.add_space(8.0);
                        ui.strong(t!("doc.example").as_ref());
                        ui.add_space(4.0);

                        egui::Frame::NONE
                            .fill(ui.visuals().extreme_bg_color)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                let row_count = self.edited_example.lines().count().clamp(1, 12);
                                egui::TextEdit::multiline(&mut self.edited_example)
                                    .font(egui::FontId::monospace(13.0))
                                    .desired_rows(row_count)
                                    .desired_width(f32::INFINITY)
                                    .show(ui);
                            });

                        ui.add_space(4.0);

                        let lang_name = lang.to_owned();
                        let connected = bridge.is_connected();
                        ui.horizontal(|ui| {
                            let run_btn = egui::Button::new(t!("doc.run").as_ref());
                            if ui.add_enabled(connected, run_btn).clicked() {
                                match langs::try_compile(&lang_name, &self.edited_example) {
                                    Ok(()) => {
                                        bridge.send(ClientMessage::SchedulerControl(
                                            SchedulerMessage::RunSnippet(Script::new(
                                                self.edited_example.clone(),
                                                lang_name.clone(),
                                            )),
                                        ));
                                        self.example_output = Some(Ok(t!("doc.sent").into()));
                                    }
                                    Err(e) => self.example_output = Some(Err(e)),
                                }
                            }
                            if ui.button(t!("doc.reset").as_ref()).clicked() {
                                self.edited_example = example.clone();
                                self.example_output = None;
                            }
                        });

                        if let Some(result) = &self.example_output {
                            ui.add_space(4.0);
                            match result {
                                Ok(output) => {
                                    egui::Frame::NONE
                                        .fill(egui::Color32::from_rgb(20, 40, 20))
                                        .inner_margin(6.0)
                                        .show(ui, |ui| {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(120, 220, 120),
                                                output,
                                            );
                                        });
                                }
                                Err(err) => {
                                    egui::Frame::NONE
                                        .fill(egui::Color32::from_rgb(50, 20, 20))
                                        .inner_margin(6.0)
                                        .show(ui, |ui| {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(220, 100, 100),
                                                err,
                                            );
                                        });
                                }
                            }
                        }
                    }
                }
            }
            None => {
                if let Some((title, content)) = doc.articles.first() {
                    ui.heading(title);
                    ui.add_space(4.0);
                    CommonMarkViewer::new().show(ui, &mut self.md_cache, content);
                } else if let Some((elem, entry)) = doc.reference.iter().next() {
                    ui.heading(element_label(elem));
                    ui.add_space(4.0);
                    ui.label(&entry.description);
                }
            }
            _ => {}
        }
    }
}

fn element_label(elem: &LanguageElement) -> String {
    match elem {
        LanguageElement::Word(w) => w.clone(),
        LanguageElement::Brackets(open, close) => format!("{open} ... {close}"),
    }
}
