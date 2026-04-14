use std::collections::BTreeMap;

use eframe::egui;
use egui::text::{LayoutJob, LayoutSection, TextWrapping};
use egui::{TextBuffer, TextFormat};
use sova_core::scene::script::Script;
use sova_core::schedule::SchedulerMessage;
use sova_core::vm::language::{LanguageDocumentation, LanguageElement};
use sova_server::ClientMessage;

use crate::client_bridge::ClientBridge;
use crate::icons;
use crate::widgets::{
    syntax_highlight::{CompiledSyntax, SyntaxTheme},
    EditorSettings,
};

use super::markdown::{show_highlighted_markdown, show_run_status_pill};
use super::{DocPanel, DocView, MarkdownRunner};

impl DocPanel {
    pub(crate) fn show_lang_toc(
        &mut self,
        ui: &mut egui::Ui,
        doc: &LanguageDocumentation,
        needle: &str,
    ) {
        if !doc.articles.is_empty() {
            ui.strong(t!("doc.articles").as_ref());
            ui.add_space(4.0);
            for (i, (title, content)) in doc.articles.iter().enumerate() {
                if !needle.is_empty()
                    && !title.to_lowercase().contains(needle)
                    && !content.to_lowercase().contains(needle)
                {
                    continue;
                }
                let selected = self.view == Some(DocView::LangArticle(i));
                let r = ui.selectable_label(selected, title);
                if selected {
                    let accent = ui.visuals().selection.bg_fill;
                    ui.painter().line_segment(
                        [r.rect.left_top(), r.rect.left_bottom()],
                        egui::Stroke::new(crate::theme::STROKE_EMPHASIS, accent),
                    );
                }
                if selected && self.scroll_toc {
                    r.scroll_to_me(Some(egui::Align::Center));
                    self.scroll_toc = false;
                }
                if r.clicked() {
                    self.set_view(DocView::LangArticle(i));
                    self.example_output = None;
                    self.edited_example.clear();
                }
            }
            ui.add_space(8.0);
        }

        if doc.reference.is_empty() {
            return;
        }

        let ref_entries: Vec<_> = doc.reference.iter().collect();
        let searching = !needle.is_empty();

        struct TocItem {
            index: usize,
            label: String,
            example: Option<String>,
            category: String,
            desc_lower: String,
            alias_lower: Vec<String>,
        }

        let items: Vec<TocItem> = ref_entries
            .iter()
            .enumerate()
            .map(|(i, (elem, entry))| TocItem {
                index: i,
                label: element_label(elem),
                example: entry.example.clone(),
                category: entry
                    .category
                    .clone()
                    .unwrap_or_else(|| "Other".to_string()),
                desc_lower: entry.description.to_lowercase(),
                alias_lower: entry.aliases.iter().map(|a| a.to_lowercase()).collect(),
            })
            .collect();

        let matches_search = |item: &TocItem| -> bool {
            !searching
                || item.label.to_lowercase().contains(needle)
                || item.desc_lower.contains(needle)
                || item.alias_lower.iter().any(|a| a.contains(needle))
        };

        let mut categories: Vec<(String, Vec<usize>)> = Vec::new();
        let mut cat_index: BTreeMap<String, usize> = BTreeMap::new();

        for (i, item) in items.iter().enumerate() {
            if let Some(&idx) = cat_index.get(&item.category) {
                categories[idx].1.push(i);
            } else {
                cat_index.insert(item.category.clone(), categories.len());
                categories.push((item.category.clone(), vec![i]));
            }
        }

        let has_categories =
            categories.len() > 1 || categories.first().is_some_and(|(name, _)| name != "Other");

        let show_item = |panel: &mut DocPanel, ui: &mut egui::Ui, item: &TocItem| {
            let selected = panel.view == Some(DocView::LangReference(item.index));
            let r = ui.selectable_label(selected, &item.label);
            if selected {
                let accent = ui.visuals().selection.bg_fill;
                ui.painter().line_segment(
                    [r.rect.left_top(), r.rect.left_bottom()],
                    egui::Stroke::new(crate::theme::STROKE_EMPHASIS, accent),
                );
            }
            if selected && panel.scroll_toc {
                r.scroll_to_me(Some(egui::Align::Center));
                panel.scroll_toc = false;
            }
            if r.clicked() {
                panel.set_view(DocView::LangReference(item.index));
                panel.example_output = None;
                panel.edited_example = item.example.clone().unwrap_or_default();
            }
        };

        if has_categories {
            for (cat_name, indices) in &categories {
                let visible: Vec<_> = indices
                    .iter()
                    .filter(|&&i| matches_search(&items[i]))
                    .copied()
                    .collect();

                if visible.is_empty() {
                    continue;
                }

                let header =
                    egui::CollapsingHeader::new(egui::RichText::new(cat_name).strong().size(12.0))
                        .default_open(!searching)
                        .open(if searching { Some(true) } else { None });

                header.show(ui, |ui| {
                    for i in visible {
                        show_item(self, ui, &items[i]);
                    }
                });
            }
        } else {
            ui.strong(t!("doc.reference").as_ref());
            ui.add_space(4.0);
            for item in &items {
                if !matches_search(item) {
                    continue;
                }
                show_item(self, ui, item);
            }
        }
    }

    pub(crate) fn show_lang_content(
        &mut self,
        ui: &mut egui::Ui,
        lang: &str,
        doc: &LanguageDocumentation,
        bridge: &ClientBridge,
        editor_settings: &EditorSettings,
    ) -> Option<String> {
        let syntax = bridge.syntax_map.get(lang);
        let mut clicked_slug: Option<String> = None;
        match &self.view {
            Some(DocView::LangArticle(idx)) => {
                if let Some((title, content)) = doc.articles.get(*idx) {
                    let theme = SyntaxTheme::from_pref(editor_settings.syntax_theme);
                    ui.heading(title);
                    ui.add_space(8.0);
                    let mut runner = MarkdownRunner {
                        bridge,
                        lang_name: lang,
                    };
                    clicked_slug = show_highlighted_markdown(
                        ui,
                        &mut self.md_cache,
                        content,
                        syntax,
                        &theme,
                        Some(&mut runner),
                    );
                }
            }
            Some(DocView::LangReference(idx)) => {
                let ref_entries: Vec<_> = doc.reference.iter().collect();
                let total = ref_entries.len();
                let idx = *idx;
                if let Some((elem, entry)) = ref_entries.get(idx) {
                    let heading = element_label(elem);

                    if let Some(cat) = &entry.category {
                        egui::Frame::NONE
                            .fill(ui.visuals().faint_bg_color)
                            .inner_margin(egui::Margin::symmetric(6, 2))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(cat)
                                        .small()
                                        .color(ui.visuals().weak_text_color()),
                                );
                            });
                        ui.add_space(4.0);
                    }

                    ui.heading(&heading);

                    if let Some(sig) = &entry.signature {
                        ui.label(
                            egui::RichText::new(sig)
                                .monospace()
                                .size(13.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                    }

                    if !entry.aliases.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("Aliases: {}", entry.aliases.join(", ")))
                                .italics()
                                .small()
                                .color(ui.visuals().weak_text_color()),
                        );
                    }

                    ui.separator();
                    ui.add_space(8.0);

                    {
                        let theme = SyntaxTheme::from_pref(editor_settings.syntax_theme);
                        clicked_slug = show_highlighted_markdown(
                            ui,
                            &mut self.md_cache,
                            &entry.description,
                            syntax,
                            &theme,
                            None,
                        );
                    }

                    if let Some(example) = &entry.example {
                        if self.edited_example.is_empty() {
                            self.edited_example = example.clone();
                        }

                        ui.add_space(8.0);
                        ui.strong(t!("doc.example").as_ref());
                        ui.add_space(4.0);

                        self.show_example_editor(ui, syntax, editor_settings);

                        ui.add_space(4.0);

                        let lang_name = lang.to_owned();
                        let connected = bridge.is_connected();
                        ui.horizontal(|ui| {
                            let run_btn = egui::Button::new(t!("doc.run").as_ref());
                            if ui.add_enabled(connected, run_btn).clicked() {
                                bridge.send(ClientMessage::SchedulerControl(
                                    SchedulerMessage::RunSnippet(
                                        Script::new(
                                            self.edited_example.clone(),
                                            lang_name.clone(),
                                        ),
                                        1.0,
                                    ),
                                ));
                                self.example_output = Some(Ok(t!("doc.sent").into()));
                            }
                            if ui
                                .button(icons::button_text(ui, icons::REFRESH, t!("doc.reset")))
                                .clicked()
                            {
                                self.edited_example = example.clone();
                                self.example_output = None;
                            }
                        });

                        if let Some(result) = &self.example_output {
                            show_run_status_pill(ui, result);
                        }
                    }

                    let prev_example = if idx > 0 {
                        ref_entries
                            .get(idx - 1)
                            .and_then(|(_, e)| e.example.clone())
                    } else {
                        None
                    };
                    let next_example = if idx + 1 < total {
                        ref_entries
                            .get(idx + 1)
                            .and_then(|(_, e)| e.example.clone())
                    } else {
                        None
                    };

                    ui.add_space(12.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                idx > 0,
                                egui::Button::new(icons::rich(icons::CHEVRON_LEFT)),
                            )
                            .clicked()
                        {
                            let new_idx = idx - 1;
                            self.set_view(DocView::LangReference(new_idx));
                            self.example_output = None;
                            self.edited_example = prev_example.unwrap_or_default();
                        }

                        ui.label(format!("{} / {}", idx + 1, total));

                        if ui
                            .add_enabled(
                                idx + 1 < total,
                                egui::Button::new(icons::rich(icons::CHEVRON_RIGHT)),
                            )
                            .clicked()
                        {
                            let new_idx = idx + 1;
                            self.set_view(DocView::LangReference(new_idx));
                            self.example_output = None;
                            self.edited_example = next_example.unwrap_or_default();
                        }
                    });
                }
            }
            None => {
                if let Some((title, content)) = doc.articles.first() {
                    let theme = SyntaxTheme::from_pref(editor_settings.syntax_theme);
                    ui.heading(title);
                    ui.add_space(8.0);
                    let mut runner = MarkdownRunner {
                        bridge,
                        lang_name: lang,
                    };
                    clicked_slug = show_highlighted_markdown(
                        ui,
                        &mut self.md_cache,
                        content,
                        syntax,
                        &theme,
                        Some(&mut runner),
                    );
                } else if let Some((elem, entry)) = doc.reference.iter().next() {
                    ui.heading(element_label(elem));
                    ui.add_space(8.0);
                    ui.label(&entry.description);
                }
            }
            _ => {}
        }
        clicked_slug
    }

    fn show_example_editor(
        &mut self,
        ui: &mut egui::Ui,
        syntax: Option<&CompiledSyntax>,
        editor_settings: &EditorSettings,
    ) {
        let theme = SyntaxTheme::from_pref(editor_settings.syntax_theme);
        let bg = ui.visuals().extreme_bg_color;
        let text_color = ui.visuals().text_color();
        let row_count = self.edited_example.lines().count().clamp(1, 12);
        let font_id = egui::FontId::monospace(13.0);
        let font_clone = font_id.clone();

        let mut layouter = move |ui: &egui::Ui, text_buf: &dyn TextBuffer, wrap_width: f32| {
            let text_s = text_buf.as_str();
            let mut job = LayoutJob {
                text: text_s.to_owned(),
                wrap: TextWrapping {
                    max_width: wrap_width,
                    ..Default::default()
                },
                ..Default::default()
            };

            if let Some(cs) = syntax {
                let mut pos = 0;
                let default_fmt = TextFormat::simple(font_clone.clone(), text_color);
                for (range, cat) in cs.tokenize(text_s) {
                    if range.start > pos {
                        job.sections.push(LayoutSection {
                            leading_space: 0.0,
                            byte_range: pos..range.start,
                            format: default_fmt.clone(),
                        });
                    }
                    job.sections.push(LayoutSection {
                        leading_space: 0.0,
                        byte_range: range.clone(),
                        format: TextFormat::simple(font_clone.clone(), theme.color(cat)),
                    });
                    pos = range.end;
                }
                if pos < text_s.len() {
                    job.sections.push(LayoutSection {
                        leading_space: 0.0,
                        byte_range: pos..text_s.len(),
                        format: default_fmt,
                    });
                }
            } else {
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: 0..text_s.len(),
                    format: TextFormat::simple(font_clone.clone(), text_color),
                });
            }

            ui.fonts_mut(|f| f.layout_job(job))
        };

        let frame_response = egui::Frame::NONE
            .fill(bg)
            .inner_margin(egui::Margin {
                left: 12,
                right: 8,
                top: 8,
                bottom: 8,
            })
            .show(ui, |ui| {
                egui::TextEdit::multiline(&mut self.edited_example)
                    .font(font_id)
                    .desired_rows(row_count)
                    .desired_width(ui.available_width())
                    .layouter(&mut layouter)
                    .show(ui);
            });
        let rect = frame_response.response.rect;
        let accent = ui.visuals().selection.bg_fill;
        ui.painter().line_segment(
            [rect.left_top(), rect.left_bottom()],
            egui::Stroke::new(3.0, accent),
        );
    }
}

fn element_label(elem: &LanguageElement) -> String {
    match elem {
        LanguageElement::Word(w) => w.clone(),
        LanguageElement::Brackets(open, close) => format!("{open} ... {close}"),
    }
}
