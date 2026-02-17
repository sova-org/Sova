use std::collections::BTreeMap;

use crate::client_bridge::ClientBridge;
use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use sova_core::schedule::SchedulerMessage;
use sova_core::vm::Language;
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

#[derive(Clone, PartialEq)]
enum DocView {
    GeneralArticle(usize),
    LangArticle(usize),
    LangReference(usize),
}

pub struct DocPanel {
    pub open: bool,
    docs: BTreeMap<String, LanguageDocumentation>,
    selected_tab: usize,
    search: String,
    md_cache: CommonMarkCache,
    view: Option<DocView>,
    example_output: Option<Result<String, String>>,
    edited_example: String,
}

impl DocPanel {
    pub fn new() -> Self {
        let mut docs = BTreeMap::new();

        let languages: Vec<Box<dyn Language>> = vec![
            Box::new(langs::bali::BaliCompiler),
            Box::new(langs::bob::BobCompiler),
            Box::new(langs::boinx::BoinxInterpreterFactory),
            Box::new(langs::forth::ForthInterpreterFactory),
        ];

        for lang in &languages {
            let doc = lang.documentation();
            if !doc.reference.is_empty() || !doc.articles.is_empty() {
                docs.insert(lang.name().to_owned(), doc);
            }
        }

        Self {
            open: false,
            docs,
            selected_tab: 0,
            search: String::new(),
            md_cache: CommonMarkCache::default(),
            view: None,
            example_output: None,
            edited_example: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, bridge: &ClientBridge) {
        if !self.open {
            return;
        }

        let mut open = self.open;

        egui::Window::new(t!("doc.title"))
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_size([600.0, 420.0])
            .max_size([800.0, 600.0])
            .vscroll(false)
            .show(ctx, |ui| {
                let lang_names: Vec<String> = self.docs.keys().cloned().collect();
                let tab_count = 1 + lang_names.len();
                self.selected_tab = self.selected_tab.min(tab_count - 1);

                // Tab bar + filter
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

                // Left: TOC sidebar
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

                // Right: Content (takes remaining space)
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
            });

        self.open = open;
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
                                            SchedulerMessage::RunSnippet(
                                                lang_name.clone(),
                                                self.edited_example.clone(),
                                            ),
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
