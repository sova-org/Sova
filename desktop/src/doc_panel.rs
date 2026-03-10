use std::collections::BTreeMap;

use crate::client_bridge::ClientBridge;
use crate::icons;
use crate::settings::{DocSettings, DocSide, DocTrigger};
use crate::widgets::syntax_highlight::{CompiledSyntax, SyntaxTheme};
use crate::widgets::EditorSettings;
use eframe::egui;
use egui::containers::panel::Side;
use egui::text::{LayoutJob, LayoutSection, TextWrapping};
use egui::{TextBuffer, TextFormat};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use sova_core::scene::script::Script;
use sova_core::schedule::SchedulerMessage;
use sova_core::vm::language::{LanguageDocumentation, LanguageElement};
use sova_server::ClientMessage;

const GENERAL_ARTICLES_EN: &[(&str, &str)] = &[
    ("Getting Started", include_str!("../docs/en/getting-started.md")),
    ("The Scene", include_str!("../docs/en/the-scene.md")),
    ("The Grid", include_str!("../docs/en/the-grid.md")),
    ("Languages", include_str!("../docs/en/languages.md")),
    ("Devices", include_str!("../docs/en/devices.md")),
    ("Timing", include_str!("../docs/en/timing.md")),
    ("Variables", include_str!("../docs/en/variables.md")),
    ("Events", include_str!("../docs/en/events.md")),
    ("Multiplayer", include_str!("../docs/en/multiplayer.md")),
    ("Audio Engine", include_str!("../docs/en/audio-engine.md")),
    ("Visuals (Hydra)", include_str!("../docs/en/visuals.md")),
];
fn general_articles() -> &'static [(&'static str, &'static str)] {
    let locale = rust_i18n::locale();
    match locale.as_ref() {
        "fr" => &[
            ("Pour commencer", include_str!("../docs/fr/getting-started.md")),
            ("La Scène", include_str!("../docs/fr/the-scene.md")),
            ("La Grille", include_str!("../docs/fr/the-grid.md")),
            ("Langages", include_str!("../docs/fr/languages.md")),
            ("Périphériques", include_str!("../docs/fr/devices.md")),
            ("Timing", include_str!("../docs/fr/timing.md")),
            ("Variables", include_str!("../docs/fr/variables.md")),
            ("Événements", include_str!("../docs/fr/events.md")),
            ("Multijoueur", include_str!("../docs/fr/multiplayer.md")),
            ("Moteur audio", include_str!("../docs/fr/audio-engine.md")),
            ("Visuels (Hydra)", include_str!("../docs/fr/visuals.md")),
        ],
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
    selected_tab: usize,
    search: String,
    md_cache: CommonMarkCache,
    view: Option<DocView>,
    example_output: Option<Result<String, String>>,
    edited_example: String,
    scroll_to_top: bool,
    scroll_toc: bool,
}

impl DocPanel {

    pub fn new(settings: DocSettings) -> Self {
        Self {
            settings,
            hover_expanded: false,
            hover_timer: None,
            selected_tab: 0,
            search: String::new(),
            md_cache: CommonMarkCache::default(),
            view: None,
            example_output: None,
            edited_example: String::new(),
            scroll_to_top: false,
            scroll_toc: false,
        }
    }

    pub fn is_expanded(&self) -> bool {
        !self.settings.collapsed || self.hover_expanded
    }

    fn set_view(&mut self, view: DocView) {
        if self.view.as_ref() != Some(&view) {
            self.scroll_to_top = true;
            self.scroll_toc = true;
        }
        self.view = Some(view);
    }

    pub fn show_side_panel(
        &mut self,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        editor_settings: &EditorSettings,
    ) {
        let side = match self.settings.side {
            DocSide::Left => Side::Left,
            DocSide::Right => Side::Right,
        };

        if self.is_expanded() {
            self.show_expanded(ctx, bridge, side, editor_settings);
        } else {
            self.show_collapsed(ctx, side);
        }
    }

    fn show_collapsed(&mut self, ctx: &egui::Context, side: Side) {
        let panel = egui::SidePanel::new(side, "doc_panel_collapsed")
            .exact_width(COLLAPSED_WIDTH)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(ctx.style().visuals.panel_fill));

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

    fn show_expanded(
        &mut self,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        side: Side,
        editor_settings: &EditorSettings,
    ) {
        let panel = egui::SidePanel::new(side, "doc_panel_expanded")
            .default_width(self.settings.width)
            .width_range(200.0..=800.0)
            .resizable(true);

        let r = panel.show(ctx, |ui| {
            self.show_header(ui);
            ui.separator();
            self.show_content(ui, bridge, editor_settings);
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

    fn show_content(
        &mut self,
        ui: &mut egui::Ui,
        bridge: &ClientBridge,
        editor_settings: &EditorSettings,
    ) {
        let langs = bridge.languages();
        let tab_count = 1 + langs.len();
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
                    self.scroll_to_top = true;
                }
                for (i, lang) in langs.iter().enumerate() {
                    let tab_idx = i + 1;
                    if ui
                        .selectable_label(self.selected_tab == tab_idx, &lang.name)
                        .clicked()
                    {
                        self.selected_tab = tab_idx;
                        self.search.clear();
                        self.view = None;
                        self.example_output = None;
                        self.edited_example.clear();
                        self.scroll_to_top = true;
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
                        let lang = &langs[selected - 1];
                        self.show_lang_toc(ui, &lang.documentation, &needle);
                    }
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let mut scroll = egui::ScrollArea::vertical();
            if self.scroll_to_top {
                scroll = scroll.vertical_scroll_offset(0.0);
                self.scroll_to_top = false;
            }
            scroll.show(ui, |ui| {
                if selected == 0 {
                    self.show_general_content(ui);
                } else {
                    let lang = &langs[selected - 1];
                    self.show_lang_content(ui, &lang.name, &lang.documentation, bridge, editor_settings);
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
            let r = ui.selectable_label(selected, *title);
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

    fn show_general_content(&mut self, ui: &mut egui::Ui) {
        let articles = general_articles();
        match &self.view {
            Some(DocView::GeneralArticle(idx)) => {
                if let Some((title, content)) = articles.get(*idx) {
                    if *idx == 0 {
                        show_welcome_header(ui);
                    } else {
                        ui.heading(*title);
                    }
                    ui.add_space(4.0);
                    CommonMarkViewer::new().show(ui, &mut self.md_cache, content);
                }
            }
            _ => {
                if let Some((_title, content)) = articles.first() {
                    show_welcome_header(ui);
                    ui.add_space(4.0);
                    CommonMarkViewer::new().show(ui, &mut self.md_cache, content);
                }
            }
        }
    }

}

fn show_welcome_header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add(
            egui::Image::new(egui::include_image!("../assets/icon.png"))
                .fit_to_exact_size(egui::vec2(48.0, 48.0)),
        );
        ui.vertical(|ui| {
            ui.heading(egui::RichText::new("Sova").size(24.0).strong());
            ui.label(
                egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).weak(),
            );
        });
    });
    ui.add_space(8.0);
}

impl DocPanel {
    fn show_lang_toc(&mut self, ui: &mut egui::Ui, doc: &LanguageDocumentation, needle: &str) {
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
                let r = ui.selectable_label(selected, title);
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

        // Build TOC items: (index, label, example, category, aliases)
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

        // Build category groups preserving insertion order
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

        let has_categories = categories.len() > 1
            || categories
                .first()
                .is_some_and(|(name, _)| name != "Other");

        let show_item = |panel: &mut DocPanel, ui: &mut egui::Ui, item: &TocItem| {
            let selected = panel.view == Some(DocView::LangReference(item.index));
            let r = ui.selectable_label(selected, &item.label);
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

                let header = egui::CollapsingHeader::new(
                    egui::RichText::new(cat_name).strong().size(12.0),
                )
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
            for item in &items {
                if !matches_search(item) {
                    continue;
                }
                show_item(self, ui, item);
            }
        }
    }

    fn show_lang_content(
        &mut self,
        ui: &mut egui::Ui,
        lang: &str,
        doc: &LanguageDocumentation,
        bridge: &ClientBridge,
        editor_settings: &EditorSettings,
    ) {
        let syntax = bridge.syntax_map.get(lang);
        match &self.view {
            Some(DocView::LangArticle(idx)) => {
                if let Some((title, content)) = doc.articles.get(*idx) {
                    let theme = SyntaxTheme::from_pref(editor_settings.syntax_theme);
                    ui.heading(title);
                    ui.add_space(4.0);
                    show_highlighted_markdown(
                        ui,
                        &mut self.md_cache,
                        content,
                        syntax,
                        &theme,
                    );
                }
            }
            Some(DocView::LangReference(idx)) => {
                let ref_entries: Vec<_> = doc.reference.iter().collect();
                let total = ref_entries.len();
                let idx = *idx;
                if let Some((elem, entry)) = ref_entries.get(idx) {
                    // Clone what we need so self is free for mutation
                    let entry_category = entry.category.clone();
                    let entry_aliases = entry.aliases.clone();
                    let entry_description = entry.description.clone();
                    let entry_example = entry.example.clone();
                    let heading = element_label(elem);

                    // Category badge
                    if let Some(cat) = &entry_category {
                        ui.label(
                            egui::RichText::new(cat)
                                .small()
                                .color(ui.visuals().weak_text_color()),
                        );
                    }

                    ui.heading(&heading);

                    // Aliases
                    if !entry_aliases.is_empty() {
                        ui.label(
                            egui::RichText::new(format!(
                                "Aliases: {}",
                                entry_aliases.join(", ")
                            ))
                            .italics()
                            .color(ui.visuals().weak_text_color()),
                        );
                    }

                    ui.separator();

                    // Description
                    {
                        let theme = SyntaxTheme::from_pref(editor_settings.syntax_theme);
                        show_highlighted_markdown(
                            ui,
                            &mut self.md_cache,
                            &entry_description,
                            syntax,
                            &theme,
                        );
                    }

                    if let Some(example) = &entry_example {
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
                                        Script::new(self.edited_example.clone(), lang_name.clone())
                                    ),
                                ));
                                self.example_output = Some(Ok(t!("doc.sent").into()));
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

                    // Prev / Next navigation
                    let prev_example = if idx > 0 {
                        ref_entries.get(idx - 1).and_then(|(_, e)| e.example.clone())
                    } else {
                        None
                    };
                    let next_example = if idx + 1 < total {
                        ref_entries.get(idx + 1).and_then(|(_, e)| e.example.clone())
                    } else {
                        None
                    };

                    ui.add_space(12.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(idx > 0, egui::Button::new(icons::CHEVRON_LEFT))
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
                                egui::Button::new(icons::CHEVRON_RIGHT),
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
                    ui.add_space(4.0);
                    show_highlighted_markdown(
                        ui,
                        &mut self.md_cache,
                        content,
                        syntax,
                        &theme,
                    );
                } else if let Some((elem, entry)) = doc.reference.iter().next() {
                    ui.heading(element_label(elem));
                    ui.add_space(4.0);
                    ui.label(&entry.description);
                }
            }
            _ => {}
        }
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

        let mut layouter =
            move |ui: &egui::Ui, text_buf: &dyn TextBuffer, wrap_width: f32| {
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
                    let default_fmt =
                        TextFormat::simple(font_clone.clone(), text_color);
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
                            format: TextFormat::simple(
                                font_clone.clone(),
                                theme.color(cat),
                            ),
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

        egui::Frame::NONE
            .fill(bg)
            .inner_margin(8.0)
            .show(ui, |ui| {
                egui::TextEdit::multiline(&mut self.edited_example)
                    .font(font_id)
                    .desired_rows(row_count)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter)
                    .show(ui);
            });
    }
}

fn element_label(elem: &LanguageElement) -> String {
    match elem {
        LanguageElement::Word(w) => w.clone(),
        LanguageElement::Brackets(open, close) => format!("{open} ... {close}"),
    }
}

/// Render markdown with syntax-highlighted code blocks.
/// Splits on ``` fences, renders prose via CommonMarkViewer and code blocks
/// as syntax-highlighted labels in a dark frame.
fn show_highlighted_markdown(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    md: &str,
    syntax: Option<&CompiledSyntax>,
    theme: &SyntaxTheme,
) {
    let font_id = egui::FontId::monospace(13.0);
    let text_color = ui.visuals().text_color();
    let bg = ui.visuals().extreme_bg_color;

    let mut rest = md;
    let mut section_id = 0u32;
    while let Some(fence_start) = rest.find("```") {
        let prose = &rest[..fence_start];
        if !prose.trim().is_empty() {
            ui.push_id(section_id, |ui| {
                CommonMarkViewer::new().show(ui, cache, prose);
            });
            section_id += 1;
        }

        // Skip the opening ``` and optional language tag line
        let after_fence = &rest[fence_start + 3..];
        let after_tag = match after_fence.find('\n') {
            Some(nl) => &after_fence[nl + 1..],
            None => {
                // Malformed: no closing fence
                rest = after_fence;
                continue;
            }
        };

        // Find closing ```
        let (code, remainder) = match after_tag.find("```") {
            Some(end) => {
                let code = &after_tag[..end];
                let skip = end + 3;
                let rem = &after_tag[skip..];
                // Skip trailing newline after closing fence
                let rem = rem.strip_prefix('\n').unwrap_or(rem);
                (code, rem)
            }
            None => {
                // No closing fence: treat remainder as code
                (after_tag, "")
            }
        };

        let code = code.strip_suffix('\n').unwrap_or(code);

        egui::Frame::NONE
            .fill(bg)
            .inner_margin(8.0)
            .show(ui, |ui| {
                let job = build_highlighted_job(code, &font_id, text_color, syntax, theme);
                ui.add(egui::Label::new(job).selectable(true));
            });

        rest = remainder;
    }

    // Remaining prose after last code block
    if !rest.trim().is_empty() {
        ui.push_id(section_id, |ui| {
            CommonMarkViewer::new().show(ui, cache, rest);
        });
    }
}

fn build_highlighted_job(
    code: &str,
    font_id: &egui::FontId,
    text_color: egui::Color32,
    syntax: Option<&CompiledSyntax>,
    theme: &SyntaxTheme,
) -> LayoutJob {
    let default_fmt = TextFormat::simple(font_id.clone(), text_color);
    let mut job = LayoutJob {
        text: code.to_owned(),
        wrap: TextWrapping {
            max_width: f32::INFINITY,
            ..Default::default()
        },
        ..Default::default()
    };

    if let Some(cs) = syntax {
        let mut pos = 0;
        for (range, cat) in cs.tokenize(code) {
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
                format: TextFormat::simple(font_id.clone(), theme.color(cat)),
            });
            pos = range.end;
        }
        if pos < code.len() {
            job.sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: pos..code.len(),
                format: default_fmt,
            });
        }
    } else {
        job.sections.push(LayoutSection {
            leading_space: 0.0,
            byte_range: 0..code.len(),
            format: default_fmt,
        });
    }

    job
}
