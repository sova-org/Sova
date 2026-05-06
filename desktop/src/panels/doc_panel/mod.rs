mod docs_ui;
mod lang_docs;
mod markdown;
mod settings_ui;

use crate::client_bridge::ClientBridge;
use crate::icons;
use crate::panels::audio_panel::AudioPanel;
use crate::panels::options_panel::OptionsPanel;
use crate::panels::server_panel::{ServerAction, ServerPanel};
use crate::settings::{AppearanceSettings, DocSettings, DocSide, DocTrigger};
use crate::theme::STROKE_EMPHASIS;
use crate::widgets::{EditorSettings, SceneOpacity};
use eframe::egui;
use egui::containers::panel::Side;
use egui_commonmark::CommonMarkCache;

impl From<DocSide> for Side {
    fn from(side: DocSide) -> Self {
        match side {
            DocSide::Left => Self::Left,
            DocSide::Right => Self::Right,
        }
    }
}

/// Accent underline drawn along the bottom of a horizontal tab button when selected.
pub(super) fn tab_underline(ui: &egui::Ui, rect: egui::Rect) {
    let accent = ui.visuals().selection.bg_fill;
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        egui::Stroke::new(STROKE_EMPHASIS, accent),
    );
}

/// Accent marker drawn along the left edge of a TOC list item when selected.
pub(super) fn toc_marker(ui: &egui::Ui, rect: egui::Rect) {
    let accent = ui.visuals().selection.bg_fill;
    ui.painter().line_segment(
        [rect.left_top(), rect.left_bottom()],
        egui::Stroke::new(STROKE_EMPHASIS, accent),
    );
}

pub struct SettingsContext<'a> {
    pub server: &'a mut ServerPanel,
    pub audio: &'a mut AudioPanel,
    pub options: &'a mut OptionsPanel,
    pub devices: &'a mut crate::panels::devices_panel::DevicesPanel,
    pub logs: &'a mut crate::panels::log_panel::LogPanel,
    pub editor_settings: &'a mut EditorSettings,
    pub appearance: &'a mut AppearanceSettings,
    pub view_mode: &'a mut crate::scene_panel::ViewMode,
    pub show_phase_bar: &'a mut bool,
}

const GENERAL_ARTICLES_EN: &[(&str, &str, &str)] = &[
    ("about", "About Sova", include_str!("../../../docs/en/about.md")),
    (
        "the-scene",
        "The Scene",
        include_str!("../../../docs/en/the-scene.md"),
    ),
    ("timing", "Timing", include_str!("../../../docs/en/timing.md")),
    (
        "languages",
        "Languages",
        include_str!("../../../docs/en/languages.md"),
    ),
    ("devices", "Devices", include_str!("../../../docs/en/devices.md")),
    (
        "variables",
        "Variables",
        include_str!("../../../docs/en/variables.md"),
    ),
    (
        "audio-engine",
        "Audio Engine",
        include_str!("../../../docs/en/audio-engine.md"),
    ),
    (
        "multiplayer",
        "Multiplayer",
        include_str!("../../../docs/en/multiplayer.md"),
    ),
];
pub(crate) fn general_articles() -> &'static [(&'static str, &'static str, &'static str)] {
    // FR articles deferred — serve EN for all locales until FR rewrite is done
    GENERAL_ARTICLES_EN
}

const COLLAPSED_WIDTH: f32 = 30.0;
const HOVER_DELAY_SECS: f64 = 0.2;

pub(crate) fn resolve_article_link(slug: &str) -> Option<DocView> {
    if let Some(i) = general_articles().iter().position(|(s, _, _)| *s == slug) {
        return Some(DocView::GeneralArticle(i));
    }
    None
}

pub(crate) fn find_clicked_hook(cache: &CommonMarkCache) -> Option<String> {
    cache
        .link_hooks()
        .iter()
        .find_map(|(k, v)| if *v { Some(k.clone()) } else { None })
}

#[derive(Clone, PartialEq)]
pub(crate) enum DocView {
    GeneralArticle(usize),
    LangArticle(usize),
    LangReference(usize),
    DouxModule(usize),
}

/// Lets `show_highlighted_markdown` make fenced code blocks runnable in this pass.
/// `None` is passed when blocks should render inertly (Hydra, general docs, reference descriptions).
pub(crate) struct MarkdownRunner<'a> {
    pub(crate) bridge: &'a ClientBridge,
    pub(crate) lang_name: &'a str,
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
    Server = 0,
    Appearance = 1,
    Devices = 2,
    Logs = 3,
}

impl SettingsTab {
    pub(crate) fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Appearance,
            2 => Self::Devices,
            3 => Self::Logs,
            _ => Self::Server,
        }
    }

    pub(crate) fn label(self) -> String {
        match self {
            Self::Server => t!("config.title").into(),
            Self::Appearance => t!("options.title").into(),
            Self::Devices => t!("devices.title").into(),
            Self::Logs => t!("log.title").into(),
        }
    }

    pub(crate) fn icon(self) -> &'static str {
        match self {
            Self::Server => icons::CPU,
            Self::Appearance => icons::PALETTE,
            Self::Devices => icons::PLUGS_CONNECTED,
            Self::Logs => icons::FILE_TEXT,
        }
    }
}

pub(crate) const SETTINGS_TABS: [SettingsTab; 4] = [
    SettingsTab::Server,
    SettingsTab::Appearance,
    SettingsTab::Devices,
    SettingsTab::Logs,
];

pub struct DocPanel {
    pub settings: DocSettings,
    hover_expanded: bool,
    hover_timer: Option<f64>,
    pub(crate) selected_tab: usize,
    pub(crate) search: String,
    pub(crate) md_cache: CommonMarkCache,
    pub(crate) view: Option<DocView>,
    pub(crate) example_output: Option<Result<String, String>>,
    pub(crate) edited_example: String,
    pub(crate) scroll_to_top: bool,
    pub(crate) scroll_toc: bool,
}

impl DocPanel {
    pub fn new(settings: DocSettings) -> Self {
        let mut md_cache = CommonMarkCache::default();
        for (slug, _, _) in general_articles() {
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
        }
    }

    pub fn is_expanded(&self) -> bool {
        !self.settings.collapsed || self.hover_expanded
    }

    pub(crate) fn set_view(&mut self, view: DocView) {
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

    pub fn show_expanded_side_panel(
        &mut self,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        settings: SettingsContext<'_>,
    ) -> (ServerAction, bool, bool) {
        if !self.is_expanded() {
            return (ServerAction::None, false, false);
        }
        self.show_expanded(ctx, bridge, Side::from(self.settings.side), settings)
    }

    pub fn show_collapsed_side_panel(&mut self, ctx: &egui::Context, opacity: SceneOpacity) {
        if self.is_expanded() {
            return;
        }
        self.show_collapsed(ctx, Side::from(self.settings.side), opacity);
    }

    fn show_collapsed(&mut self, ctx: &egui::Context, side: Side, opacity: SceneOpacity) {
        let fill = opacity.panel_fill(ctx);
        let panel = egui::SidePanel::new(side, "doc_panel_collapsed")
            .exact_width(COLLAPSED_WIDTH)
            .resizable(false)
            .show_separator_line(true)
            .frame(egui::Frame::NONE.fill(fill));

        let r = panel.show(ctx, |ui| {
            let rect = ui.max_rect();
            let icon_size = egui::vec2(COLLAPSED_WIDTH, 24.0);
            let weak = ui.visuals().weak_text_color();
            let icon = icons::rich(icons::BOOK).color(weak).size(16.0);
            ui.put(
                egui::Rect::from_center_size(rect.center(), icon_size),
                egui::Label::new(icon),
            );
        });

        let hover_pos = ctx.input(|i| i.pointer.hover_pos().unwrap_or_default());
        let strip_rect = r.response.rect;
        let hovering = strip_rect.contains(hover_pos);
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
        settings: SettingsContext<'_>,
    ) -> (ServerAction, bool, bool) {
        let mut server_action = ServerAction::None;
        let mut appearance_changed = false;
        let mut pick_sample_folder = false;

        let panel = egui::SidePanel::new(side, "doc_panel_expanded")
            .default_width(self.settings.width)
            .width_range(200.0..=800.0)
            .resizable(true);

        let r = panel.show(ctx, |ui| {
            egui::TopBottomPanel::top("sidebar_mode_tabs").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    let mode = self.mode();
                    let doc_r = ui.selectable_label(
                        mode == SidebarMode::Docs,
                        icons::button_text(ui, icons::BOOK, t!("doc.title")),
                    );
                    if mode == SidebarMode::Docs {
                        tab_underline(ui, doc_r.rect);
                    }
                    if doc_r.clicked() {
                        self.settings.mode = SidebarMode::Docs as u8;
                    }

                    let settings_r = ui.selectable_label(
                        mode == SidebarMode::Settings,
                        icons::button_text(ui, icons::GEAR, t!("settings.title")),
                    );
                    if mode == SidebarMode::Settings {
                        tab_underline(ui, settings_r.rect);
                    }
                    if settings_r.clicked() {
                        self.settings.mode = SidebarMode::Settings as u8;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let collapse_icon = match self.settings.side {
                            DocSide::Left => icons::CHEVRON_LEFT,
                            DocSide::Right => icons::CHEVRON_RIGHT,
                        };
                        if ui
                            .button(icons::rich(collapse_icon))
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
                            .button(icons::rich(icons::SWAP))
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

            match self.mode() {
                SidebarMode::Docs => {
                    self.show_content(ui, bridge, settings.editor_settings);
                }
                SidebarMode::Settings => {
                    let (sa, ac, pf) = self.show_settings_content(ui, bridge, settings);
                    server_action = sa;
                    appearance_changed = ac;
                    pick_sample_folder = pf;
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

        (server_action, appearance_changed, pick_sample_folder)
    }
}
