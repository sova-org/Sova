use std::collections::BTreeMap;

use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use sova_core::vm::language::{LanguageDocumentation, LanguageElement};
use sova_core::vm::Language;

const GENERAL_ARTICLES: &[(&str, &str)] = &[
    ("Getting Started", include_str!("../docs/getting-started.md")),
];

pub struct DocPanel {
    pub open: bool,
    docs: BTreeMap<String, LanguageDocumentation>,
    selected_tab: usize,
    search: String,
    md_cache: CommonMarkCache,
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
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }

        let screen = ctx.content_rect();
        let mut open = self.open;

        egui::Window::new("Documentation")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_width(500.0)
            .default_height(400.0)
            .max_height(screen.height() * 0.85)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(screen.center())
            .vscroll(false)
            .show(ctx, |ui| {
                let lang_names: Vec<String> = self.docs.keys().cloned().collect();
                let tab_count = 1 + lang_names.len();
                self.selected_tab = self.selected_tab.min(tab_count - 1);

                ui.horizontal(|ui| {
                    if ui.selectable_label(self.selected_tab == 0, "Sova").clicked() {
                        self.selected_tab = 0;
                        self.search.clear();
                    }
                    for (i, name) in lang_names.iter().enumerate() {
                        let tab_idx = i + 1;
                        if ui.selectable_label(self.selected_tab == tab_idx, name).clicked() {
                            self.selected_tab = tab_idx;
                            self.search.clear();
                        }
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut self.search);
                });

                ui.separator();

                let needle = self.search.to_lowercase();
                let selected = self.selected_tab;

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if selected == 0 {
                        show_general_articles(ui, &mut self.md_cache, &needle);
                    } else {
                        let doc = &self.docs[&lang_names[selected - 1]];
                        show_language_doc(ui, &mut self.md_cache, doc, &needle);
                    }
                });
            });

        self.open = open;
    }
}

fn show_general_articles(ui: &mut egui::Ui, cache: &mut CommonMarkCache, needle: &str) {
    for (title, content) in GENERAL_ARTICLES {
        if !needle.is_empty()
            && !title.to_lowercase().contains(needle)
            && !content.to_lowercase().contains(needle)
        {
            continue;
        }
        egui::CollapsingHeader::new(*title)
            .default_open(GENERAL_ARTICLES.len() == 1)
            .show(ui, |ui| {
                CommonMarkViewer::new().show(ui, cache, content);
            });
    }
}

fn show_language_doc(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    doc: &LanguageDocumentation,
    needle: &str,
) {
    if !doc.articles.is_empty() {
        ui.heading("Articles");
        ui.add_space(4.0);
        for (title, content) in &doc.articles {
            if !needle.is_empty()
                && !title.to_lowercase().contains(needle)
                && !content.to_lowercase().contains(needle)
            {
                continue;
            }
            egui::CollapsingHeader::new(title).show(ui, |ui| {
                CommonMarkViewer::new().show(ui, cache, content);
            });
        }
        ui.add_space(8.0);
    }

    if !doc.reference.is_empty() {
        ui.heading("Reference");
        ui.add_space(4.0);

        egui::Grid::new("doc_ref_grid")
            .num_columns(2)
            .min_col_width(80.0)
            .striped(true)
            .show(ui, |ui| {
                for (elem, desc) in &doc.reference {
                    let label = match elem {
                        LanguageElement::Word(w) => w.clone(),
                        LanguageElement::Brackets(open, close) => {
                            format!("{open} ... {close}")
                        }
                    };

                    if !needle.is_empty()
                        && !label.to_lowercase().contains(needle)
                        && !desc.to_lowercase().contains(needle)
                    {
                        continue;
                    }

                    ui.strong(&label);
                    ui.label(desc);
                    ui.end_row();
                }
            });
    }
}
