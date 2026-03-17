use std::collections::BTreeMap;

use crate::audio_panel::AudioPanel;
use crate::client_bridge::ClientBridge;
use crate::icons;
use crate::options_panel::OptionsPanel;
use crate::server_panel::{ServerAction, ServerPanel};
use crate::settings::{AppearanceSettings, DocSettings, DocSide, DocTrigger};
use crate::visuals;
use crate::widgets::syntax_highlight::{CompiledSyntax, SyntaxTheme};
use crate::widgets::EditorSettings;
use eframe::egui;
use egui::containers::panel::Side;
use egui::text::{LayoutJob, LayoutSection, TextWrapping};
use egui::{TextBuffer, TextFormat};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use sova_core::scene::script::Script;
use sova_core::schedule::SchedulerMessage;
use doux::types::{ModuleGroup, ModuleInfo, Source};
use sova_core::vm::language::{LanguageDocumentation, LanguageElement};
use sova_server::ClientMessage;

pub struct SettingsContext<'a> {
    pub server: &'a mut ServerPanel,
    pub audio: &'a mut AudioPanel,
    pub options: &'a mut OptionsPanel,
    pub devices: &'a mut crate::devices_panel::DevicesPanel,
    pub logs: &'a mut crate::log_panel::LogPanel,
    pub editor_settings: &'a mut EditorSettings,
    pub appearance: &'a mut AppearanceSettings,
    pub dismissed_tips: &'a mut Vec<String>,
}

const GENERAL_ARTICLES_EN: &[(&str, &str, &str)] = &[
    ("about", "About Sova", include_str!("../docs/en/about.md")),
    ("getting-started", "Getting Started", include_str!("../docs/en/getting-started.md")),
    ("the-scene", "The Scene", include_str!("../docs/en/the-scene.md")),
    ("timing", "Timing", include_str!("../docs/en/timing.md")),
    ("languages", "Languages", include_str!("../docs/en/languages.md")),
    ("events", "Events", include_str!("../docs/en/events.md")),
    ("devices", "Devices", include_str!("../docs/en/devices.md")),
    ("variables", "Variables", include_str!("../docs/en/variables.md")),
    ("audio-engine", "Audio Engine", include_str!("../docs/en/audio-engine.md")),
    ("multiplayer", "Multiplayer", include_str!("../docs/en/multiplayer.md")),
];
fn general_articles() -> &'static [(&'static str, &'static str, &'static str)] {
    // FR articles deferred — serve EN for all locales until FR rewrite is done
    GENERAL_ARTICLES_EN
}

const HYDRA_ARTICLES: &[(&str, &str, &str)] = &[
    ("hydra-intro", "Introduction", include_str!("../docs/en/hydra/intro.md")),
    ("hydra-chaining", "Chaining", include_str!("../docs/en/hydra/chaining.md")),
    ("hydra-sources", "Sources", include_str!("../docs/en/hydra/sources.md")),
    ("hydra-geometry", "Geometry", include_str!("../docs/en/hydra/geometry.md")),
    ("hydra-color", "Color", include_str!("../docs/en/hydra/color.md")),
    ("hydra-blending", "Blending", include_str!("../docs/en/hydra/blending.md")),
    ("hydra-modulation", "Modulation", include_str!("../docs/en/hydra/modulation.md")),
    ("hydra-buffers", "Buffers", include_str!("../docs/en/hydra/buffers.md")),
    ("hydra-feedback", "Feedback", include_str!("../docs/en/hydra/feedback.md")),
    ("hydra-animation", "Animation", include_str!("../docs/en/hydra/animation.md")),
    ("hydra-text", "Text", include_str!("../docs/en/hydra/text.md")),
    ("hydra-differences", "Differences", include_str!("../docs/en/hydra/differences.md")),
];
fn hydra_articles() -> &'static [(&'static str, &'static str, &'static str)] {
    HYDRA_ARTICLES
}

const COLLAPSED_WIDTH: f32 = 24.0;
const HOVER_DELAY_SECS: f64 = 0.2;

fn resolve_article_link(slug: &str) -> Option<DocView> {
    if let Some(i) = general_articles().iter().position(|(s, _, _)| *s == slug) {
        return Some(DocView::GeneralArticle(i));
    }
    if let Some(i) = hydra_articles().iter().position(|(s, _, _)| *s == slug) {
        return Some(DocView::HydraArticle(i));
    }
    None
}

fn find_clicked_hook(cache: &CommonMarkCache) -> Option<String> {
    cache.link_hooks().iter().find_map(|(k, v)| if *v { Some(k.clone()) } else { None })
}

#[derive(Clone, PartialEq)]
enum DocView {
    GeneralArticle(usize),
    LangArticle(usize),
    LangReference(usize),
    HydraArticle(usize),
    DouxModule(usize),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SidebarMode {
    Docs = 0,
    Settings = 1,
}

impl SidebarMode {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Settings,
            _ => Self::Docs,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Config = 0,
    Options = 1,
    Devices = 2,
    Logs = 3,
}

impl SettingsTab {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Options,
            2 => Self::Devices,
            3 => Self::Logs,
            _ => Self::Config,
        }
    }

    fn label(self) -> String {
        match self {
            Self::Config => t!("config.title").into(),
            Self::Options => t!("options.title").into(),
            Self::Devices => t!("devices.title").into(),
            Self::Logs => t!("log.title").into(),
        }
    }
}

const SETTINGS_TABS: [SettingsTab; 4] = [SettingsTab::Config, SettingsTab::Options, SettingsTab::Devices, SettingsTab::Logs];

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
    hydra_syntax: Option<CompiledSyntax>,
}

impl DocPanel {

    pub fn new(settings: DocSettings) -> Self {
        let mut md_cache = CommonMarkCache::default();
        for (slug, _, _) in general_articles().iter().chain(hydra_articles()) {
            md_cache.add_link_hook(*slug);
        }
        Self {
            settings,
            hover_expanded: false,
            hover_timer: None,
            selected_tab: 0,
            search: String::new(),
            md_cache,
            view: None,
            example_output: None,
            edited_example: String::new(),
            scroll_to_top: false,
            scroll_toc: false,
            hydra_syntax: CompiledSyntax::new(&visuals::hydra_syntax()),
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

    pub fn mode(&self) -> SidebarMode {
        SidebarMode::from_u8(self.settings.mode)
    }

    pub fn open_settings_tab(&mut self, tab: SettingsTab) {
        self.settings.mode = SidebarMode::Settings as u8;
        self.settings.settings_tab = tab as u8;
        self.settings.collapsed = false;
        self.settings.pinned = true;
    }

    pub fn is_settings_tab_open(&self, tab: SettingsTab) -> bool {
        !self.settings.collapsed
            && self.mode() == SidebarMode::Settings
            && self.settings.settings_tab == tab as u8
    }

    pub fn is_logs_open(&self) -> bool {
        self.is_settings_tab_open(SettingsTab::Logs)
    }

    pub fn toggle_settings_tab(&mut self, tab: SettingsTab) {
        if !self.settings.collapsed
            && self.mode() == SidebarMode::Settings
            && self.settings.settings_tab == tab as u8
        {
            self.settings.collapsed = true;
        } else {
            self.open_settings_tab(tab);
        }
    }

    pub fn show_side_panel(
        &mut self,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        settings: SettingsContext<'_>,
    ) -> (ServerAction, bool) {
        let side = match self.settings.side {
            DocSide::Left => Side::Left,
            DocSide::Right => Side::Right,
        };

        if self.is_expanded() {
            self.show_expanded(ctx, bridge, side, settings)
        } else {
            self.show_collapsed(ctx, side);
            (ServerAction::None, false)
        }
    }

    fn show_collapsed(&mut self, ctx: &egui::Context, side: Side) {
        let mut top_rect = egui::Rect::NOTHING;
        let mut mid_rect = egui::Rect::NOTHING;
        let mut bottom_rect = egui::Rect::NOTHING;

        let panel = egui::SidePanel::new(side, "doc_panel_collapsed")
            .exact_width(COLLAPSED_WIDTH)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(ctx.style().visuals.panel_fill));

        let r = panel.show(ctx, |ui| {
            let rect = ui.max_rect();
            let third = rect.height() / 3.0;

            top_rect = egui::Rect::from_min_max(
                rect.min,
                egui::pos2(rect.max.x, rect.min.y + third),
            );
            mid_rect = egui::Rect::from_min_max(
                egui::pos2(rect.min.x, rect.min.y + third),
                egui::pos2(rect.max.x, rect.min.y + 2.0 * third),
            );
            bottom_rect = egui::Rect::from_min_max(
                egui::pos2(rect.min.x, rect.min.y + 2.0 * third),
                rect.max,
            );

            let icon_size = egui::vec2(COLLAPSED_WIDTH, 24.0);
            let weak = ui.visuals().weak_text_color();

            let book = egui::RichText::new(icons::BOOK).color(weak).size(16.0);
            ui.put(
                egui::Rect::from_center_size(top_rect.center(), icon_size),
                egui::Label::new(book),
            );

            let gear = egui::RichText::new(icons::GEAR).color(weak).size(16.0);
            ui.put(
                egui::Rect::from_center_size(mid_rect.center(), icon_size),
                egui::Label::new(gear),
            );

            let output = egui::RichText::new(icons::OUTPUT).color(weak).size(16.0);
            ui.put(
                egui::Rect::from_center_size(bottom_rect.center(), icon_size),
                egui::Label::new(output),
            );
        });

        let hover_pos = ctx.input(|i| i.pointer.hover_pos().unwrap_or_default());
        let strip_rect = r.response.rect;
        let hovering_strip = strip_rect.contains(hover_pos);
        let hovering_top = top_rect.contains(hover_pos);
        let hovering_mid = mid_rect.contains(hover_pos);
        let hovering_bottom = bottom_rect.contains(hover_pos);
        let clicked = hovering_strip && ctx.input(|i| i.pointer.primary_clicked());

        let open_settings_tab = |s: &mut DocSettings, tab: SettingsTab| {
            s.mode = SidebarMode::Settings as u8;
            s.settings_tab = tab as u8;
        };

        match self.settings.trigger {
            DocTrigger::Click => {
                if hovering_strip {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if clicked && hovering_top {
                    self.settings.mode = SidebarMode::Docs as u8;
                    self.settings.collapsed = false;
                    self.settings.pinned = true;
                }
                if clicked && hovering_mid {
                    open_settings_tab(&mut self.settings, SettingsTab::Config);
                    self.settings.collapsed = false;
                    self.settings.pinned = true;
                }
                if clicked && hovering_bottom {
                    open_settings_tab(&mut self.settings, SettingsTab::Logs);
                    self.settings.collapsed = false;
                    self.settings.pinned = true;
                }
            }
            DocTrigger::Hover => {
                let start_hover = |me: &mut Self, ctx: &egui::Context| {
                    let now = ctx.input(|i| i.time);
                    if let Some(start) = me.hover_timer {
                        if now - start >= HOVER_DELAY_SECS {
                            me.hover_expanded = true;
                            me.hover_timer = None;
                        }
                    } else {
                        me.hover_timer = Some(now);
                    }
                    ctx.request_repaint();
                };

                if hovering_top {
                    self.settings.mode = SidebarMode::Docs as u8;
                    start_hover(self, ctx);
                } else if hovering_mid {
                    open_settings_tab(&mut self.settings, SettingsTab::Config);
                    start_hover(self, ctx);
                } else if hovering_bottom {
                    open_settings_tab(&mut self.settings, SettingsTab::Logs);
                    start_hover(self, ctx);
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
        settings: SettingsContext<'_>,
    ) -> (ServerAction, bool) {
        let mut server_action = ServerAction::None;
        let mut appearance_changed = false;

        let panel = egui::SidePanel::new(side, "doc_panel_expanded")
            .default_width(self.settings.width)
            .width_range(200.0..=800.0)
            .resizable(true);

        let r = panel.show(ctx, |ui| {
            match self.mode() {
                SidebarMode::Docs => {
                    self.show_content(ui, bridge, settings.editor_settings);
                }
                SidebarMode::Settings => {
                    let (sa, ac) = self.show_settings_content(ui, bridge, settings);
                    server_action = sa;
                    appearance_changed = ac;
                }
            }
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

        (server_action, appearance_changed)
    }

    fn show_settings_content(
        &mut self,
        ui: &mut egui::Ui,
        bridge: &ClientBridge,
        settings: SettingsContext<'_>,
    ) -> (ServerAction, bool) {
        let SettingsContext {
            server, audio, options, devices, logs,
            editor_settings, appearance, dismissed_tips,
        } = settings;
        let mut server_action = ServerAction::None;
        let mut appearance_changed = false;

        egui::TopBottomPanel::top("settings_tabs").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let selected = SettingsTab::from_u8(self.settings.settings_tab);
                for &tab in &SETTINGS_TABS {
                    let r = ui.selectable_label(selected == tab, tab.label());
                    if selected == tab {
                        let accent = ui.visuals().selection.bg_fill;
                        ui.painter().line_segment(
                            [r.rect.left_bottom(), r.rect.right_bottom()],
                            egui::Stroke::new(2.0, accent),
                        );
                    }
                    if r.clicked() {
                        self.settings.settings_tab = tab as u8;
                    }
                }

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
        });

        match SettingsTab::from_u8(self.settings.settings_tab) {
            SettingsTab::Logs => {
                logs.show_inside(ui);
            }
            tab => {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        match tab {
                            SettingsTab::Config => {
                                egui::CollapsingHeader::new(t!("config.server"))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            server_action = server.show_actions(ui);
                                        });
                                        server.show_config(ui);
                                    });

                                egui::CollapsingHeader::new(t!("config.audio"))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            audio.show_restart_button(ui, bridge);
                                        });
                                        audio.show_config(ui);
                                    });

                                if bridge.is_connected() && bridge.audio_state().running {
                                    egui::CollapsingHeader::new(t!("config.audio_status"))
                                        .default_open(true)
                                        .show(ui, |ui| {
                                            audio.show_status_section(ui, bridge);
                                        });
                                }
                            }
                            SettingsTab::Options => {
                                appearance_changed = options.show_inside(
                                    ui,
                                    editor_settings,
                                    appearance,
                                    &mut self.settings,
                                    dismissed_tips,
                                    bridge.languages(),
                                );
                            }
                            SettingsTab::Devices => {
                                devices.show_inside(ui, bridge);
                            }
                            SettingsTab::Logs => unreachable!(),
                        }
                    });
            }
        }

        (server_action, appearance_changed)
    }

    fn show_content(
        &mut self,
        ui: &mut egui::Ui,
        bridge: &ClientBridge,
        editor_settings: &EditorSettings,
    ) {
        let langs = bridge.languages();
        let hydra_tab = 1 + langs.len();
        let doux_tab = hydra_tab + 1;
        let tab_count = doux_tab + 1;
        self.selected_tab = self.selected_tab.min(tab_count - 1);

        egui::TopBottomPanel::top("doc_tabs").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let r = ui.selectable_label(self.selected_tab == 0, t!("doc.sova").as_ref());
                if self.selected_tab == 0 {
                    let accent = ui.visuals().selection.bg_fill;
                    ui.painter().line_segment(
                        [r.rect.left_bottom(), r.rect.right_bottom()],
                        egui::Stroke::new(2.0, accent),
                    );
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
                    let r = ui.selectable_label(self.selected_tab == tab_idx, &lang.name);
                    if self.selected_tab == tab_idx {
                        let accent = ui.visuals().selection.bg_fill;
                        ui.painter().line_segment(
                            [r.rect.left_bottom(), r.rect.right_bottom()],
                            egui::Stroke::new(2.0, accent),
                        );
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

                let r = ui.selectable_label(self.selected_tab == hydra_tab, "Hydra");
                if self.selected_tab == hydra_tab {
                    let accent = ui.visuals().selection.bg_fill;
                    ui.painter().line_segment(
                        [r.rect.left_bottom(), r.rect.right_bottom()],
                        egui::Stroke::new(2.0, accent),
                    );
                }
                if r.clicked() {
                    self.selected_tab = hydra_tab;
                    self.search.clear();
                    self.view = None;
                    self.example_output = None;
                    self.edited_example.clear();
                    self.scroll_to_top = true;
                }

                let r = ui.selectable_label(self.selected_tab == doux_tab, "Doux");
                if self.selected_tab == doux_tab {
                    let accent = ui.visuals().selection.bg_fill;
                    ui.painter().line_segment(
                        [r.rect.left_bottom(), r.rect.right_bottom()],
                        egui::Stroke::new(2.0, accent),
                    );
                }
                if r.clicked() {
                    self.selected_tab = doux_tab;
                    self.search.clear();
                    self.view = None;
                    self.example_output = None;
                    self.edited_example.clear();
                    self.scroll_to_top = true;
                }

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

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(t!("doc.filter").as_ref()).weak().small());
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
            .frame(egui::Frame::NONE.inner_margin(4.0).fill(ui.visuals().panel_fill))
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if selected == 0 {
                        self.show_general_toc(ui, &needle);
                    } else if selected == hydra_tab {
                        self.show_hydra_toc(ui, &needle);
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
            .frame(egui::Frame::NONE.inner_margin(egui::Margin { left: 16, right: 16, top: 8, bottom: 8 }))
            .show_inside(ui, |ui| {
            let mut scroll = egui::ScrollArea::vertical();
            if self.scroll_to_top {
                scroll = scroll.vertical_scroll_offset(0.0);
                self.scroll_to_top = false;
            }
            scroll.show(ui, |ui| {
                nav_target = if selected == 0 {
                    self.show_general_content(ui)
                } else if selected == hydra_tab {
                    self.show_hydra_content(ui, editor_settings)
                } else if selected == doux_tab {
                    self.show_doux_content(ui);
                    None
                } else {
                    let lang = &langs[selected - 1];
                    self.show_lang_content(ui, &lang.name, &lang.documentation, bridge, editor_settings)
                };
            });
        });

        if let Some(slug) = nav_target
            && let Some(view) = resolve_article_link(&slug)
        {
            let tab = match &view {
                DocView::GeneralArticle(_) => 0,
                DocView::HydraArticle(_) => hydra_tab,
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
                let accent = ui.visuals().selection.bg_fill;
                ui.painter().line_segment(
                    [r.rect.left_top(), r.rect.left_bottom()],
                    egui::Stroke::new(2.0, accent),
                );
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

    fn show_hydra_toc(&mut self, ui: &mut egui::Ui, needle: &str) {
        ui.strong(t!("doc.articles").as_ref());
        ui.add_space(4.0);
        for (i, (_, title, content)) in hydra_articles().iter().enumerate() {
            if !needle.is_empty()
                && !title.to_lowercase().contains(needle)
                && !content.to_lowercase().contains(needle)
            {
                continue;
            }
            let selected = self.view == Some(DocView::HydraArticle(i));
            let r = ui.selectable_label(selected, *title);
            if selected {
                let accent = ui.visuals().selection.bg_fill;
                ui.painter().line_segment(
                    [r.rect.left_top(), r.rect.left_bottom()],
                    egui::Stroke::new(2.0, accent),
                );
            }
            if selected && self.scroll_toc {
                r.scroll_to_me(Some(egui::Align::Center));
                self.scroll_toc = false;
            }
            if r.clicked() {
                self.set_view(DocView::HydraArticle(i));
            }
        }
    }

    fn show_hydra_content(&mut self, ui: &mut egui::Ui, editor_settings: &EditorSettings) -> Option<String> {
        let articles = hydra_articles();
        let idx = match &self.view {
            Some(DocView::HydraArticle(i)) => *i,
            _ => 0,
        };
        if let Some((_, title, content)) = articles.get(idx) {
            let theme = SyntaxTheme::from_pref(editor_settings.syntax_theme);
            ui.heading(*title);
            ui.add_space(8.0);
            show_highlighted_markdown(
                ui,
                &mut self.md_cache,
                content,
                self.hydra_syntax.as_ref(),
                &theme,
            )
        } else {
            None
        }
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
                            p.name.contains(needle)
                                || p.description.to_lowercase().contains(needle)
                        })
                })
                .collect();

            if group_modules.is_empty() {
                continue;
            }

            let header = egui::CollapsingHeader::new(
                egui::RichText::new(label).strong().size(12.0),
            )
            .default_open(!searching)
            .open(if searching { Some(true) } else { None });

            header.show(ui, |ui| {
                for (idx, module) in &group_modules {
                    let selected = self.view == Some(DocView::DouxModule(*idx));
                    let r = ui.selectable_label(selected, module.name);
                    if selected {
                        let accent = ui.visuals().selection.bg_fill;
                        ui.painter().line_segment(
                            [r.rect.left_top(), r.rect.left_bottom()],
                            egui::Stroke::new(2.0, accent),
                        );
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

        let Some(module) = modules.get(idx) else { return };

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
                            egui::RichText::new(format!(
                                "Aliases: {}",
                                info.aliases.join(", ")
                            ))
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
                    egui::Stroke::new(1.0, dimmed),
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
                .add_enabled(idx > 0, egui::Button::new(icons::CHEVRON_LEFT))
                .clicked()
            {
                self.set_view(DocView::DouxModule(idx - 1));
            }
            ui.label(format!("{} / {}", idx + 1, total));
            if ui
                .add_enabled(idx + 1 < total, egui::Button::new(icons::CHEVRON_RIGHT))
                .clicked()
            {
                self.set_view(DocView::DouxModule(idx + 1));
            }
        });
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
    let accent = ui.visuals().selection.bg_fill;
    let dimmed = egui::Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 60);
    let rect = ui.available_rect_before_wrap();
    ui.painter().line_segment(
        [rect.left_top(), egui::pos2(rect.right(), rect.top())],
        egui::Stroke::new(1.0, dimmed),
    );
    ui.add_space(8.0);
}

impl DocPanel {
    fn show_lang_toc(&mut self, ui: &mut egui::Ui, doc: &LanguageDocumentation, needle: &str) {
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
                        egui::Stroke::new(2.0, accent),
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
            if selected {
                let accent = ui.visuals().selection.bg_fill;
                ui.painter().line_segment(
                    [r.rect.left_top(), r.rect.left_bottom()],
                    egui::Stroke::new(2.0, accent),
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
            ui.add_space(4.0);
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
    ) -> Option<String> {
        let syntax = bridge.syntax_map.get(lang);
        let mut clicked_slug: Option<String> = None;
        match &self.view {
            Some(DocView::LangArticle(idx)) => {
                if let Some((title, content)) = doc.articles.get(*idx) {
                    let theme = SyntaxTheme::from_pref(editor_settings.syntax_theme);
                    ui.heading(title);
                    ui.add_space(8.0);
                    clicked_slug = show_highlighted_markdown(
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
                    ui.add_space(8.0);

                    // Description
                    {
                        let theme = SyntaxTheme::from_pref(editor_settings.syntax_theme);
                        clicked_slug = show_highlighted_markdown(
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
                                        Script::new(self.edited_example.clone(), lang_name.clone()),
                                        1.0
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
                    ui.add_space(8.0);
                    clicked_slug = show_highlighted_markdown(
                        ui,
                        &mut self.md_cache,
                        content,
                        syntax,
                        &theme,
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

        let frame_response = egui::Frame::NONE
            .fill(bg)
            .inner_margin(egui::Margin { left: 12, right: 8, top: 8, bottom: 8 })
            .show(ui, |ui| {
                egui::TextEdit::multiline(&mut self.edited_example)
                    .font(font_id)
                    .desired_rows(row_count)
                    .desired_width(f32::INFINITY)
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

/// Render markdown with syntax-highlighted code blocks.
/// Splits on ``` fences, renders prose via CommonMarkViewer and code blocks
/// as syntax-highlighted labels in a dark frame.
/// Returns the slug of the first clicked cross-reference link, if any.
fn show_highlighted_markdown(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    md: &str,
    syntax: Option<&CompiledSyntax>,
    theme: &SyntaxTheme,
) -> Option<String> {
    let font_id = egui::FontId::monospace(13.0);
    let text_color = ui.visuals().text_color();
    let bg = ui.visuals().extreme_bg_color;

    let mut clicked_link: Option<String> = None;
    let mut rest = md;
    let mut section_id = 0u32;
    while let Some(fence_start) = rest.find("```") {
        let prose = &rest[..fence_start];
        if !prose.trim().is_empty() {
            ui.push_id(section_id, |ui| {
                CommonMarkViewer::new().show(ui, cache, prose);
            });
            section_id += 1;
            if clicked_link.is_none() {
                clicked_link = find_clicked_hook(cache);
            }
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

        ui.add_space(6.0);
        let frame_response = egui::Frame::NONE
            .fill(bg)
            .inner_margin(egui::Margin { left: 12, right: 8, top: 8, bottom: 8 })
            .show(ui, |ui| {
                let job = build_highlighted_job(code, &font_id, text_color, syntax, theme);
                ui.add(egui::Label::new(job).selectable(true));
            });
        let rect = frame_response.response.rect;
        let accent = ui.visuals().selection.bg_fill;
        ui.painter().line_segment(
            [rect.left_top(), rect.left_bottom()],
            egui::Stroke::new(3.0, accent),
        );
        ui.add_space(6.0);

        rest = remainder;
    }

    // Remaining prose after last code block
    if !rest.trim().is_empty() {
        ui.push_id(section_id, |ui| {
            CommonMarkViewer::new().show(ui, cache, rest);
        });
        if clicked_link.is_none() {
            clicked_link = find_clicked_hook(cache);
        }
    }

    clicked_link
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
