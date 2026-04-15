use std::path::PathBuf;

use eframe::egui;
use egui::containers::panel::Side;

use crate::InputOwner;
use crate::panels::chat_panel::ChatPanel;
use crate::client_bridge::ClientBridge;
use crate::icons;
use crate::panels::sample_browser_panel::SampleBrowserPanel;
use crate::settings::ToolsSettings;
use crate::widgets::SceneOpacity;

pub struct ToolsPanel {
    pub settings: ToolsSettings,
}

impl ToolsPanel {
    pub fn new(mut settings: ToolsSettings) -> Self {
        if !settings.show_chat && !settings.show_sample_browser {
            if let Some(tab) = settings.active_tab {
                match tab {
                    crate::settings::ToolsTab::Chat => settings.show_chat = true,
                    crate::settings::ToolsTab::SampleBrowser => {
                        settings.show_sample_browser = true;
                    }
                }
            } else if settings.open {
                settings.show_chat = true;
                settings.show_sample_browser = true;
            }
        }
        settings.open = settings.show_chat || settings.show_sample_browser;
        settings.active_tab = None;
        Self { settings }
    }

    fn sync_open(&mut self) {
        self.settings.open = self.settings.show_chat || self.settings.show_sample_browser;
        self.settings.active_tab = None;
    }

    pub fn toggle_chat(&mut self) {
        self.settings.show_chat = !self.settings.show_chat;
        self.sync_open();
    }

    pub fn toggle_sample_browser(&mut self) {
        self.settings.show_sample_browser = !self.settings.show_sample_browser;
        self.sync_open();
    }

    pub fn close(&mut self) {
        self.settings.show_chat = false;
        self.settings.show_sample_browser = false;
        self.sync_open();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn show_side_panel(
        &mut self,
        ctx: &egui::Context,
        side: Side,
        chat: &mut ChatPanel,
        sample_browser: &mut SampleBrowserPanel,
        bridge: &mut ClientBridge,
        default_path: Option<&std::path::Path>,
        sample_paths: &[PathBuf],
        is_hosting: bool,
        opacity: SceneOpacity,
        input_owner: InputOwner,
    ) -> Option<egui::Rect> {
        if !self.settings.open {
            return None;
        }

        // Always keep the sample browser state up to date
        sample_browser.poll(default_path, sample_paths);

        let show_chat = self.settings.show_chat && !chat.detached;
        let show_sample_browser = self.settings.show_sample_browser && !sample_browser.detached;
        if !show_chat && !show_sample_browser {
            return None;
        }

        let fill = opacity.panel_fill(ctx);
        let panel = egui::SidePanel::new(side, "tools_panel")
            .default_width(self.settings.width)
            .width_range(250.0..=600.0)
            .resizable(true)
            .frame(egui::Frame::side_top_panel(&ctx.style()).fill(fill));

        let r = panel.show(ctx, |ui| {
            egui::TopBottomPanel::top("tools_header")
                .show_separator_line(true)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        let title = match (show_chat, show_sample_browser) {
                            (true, true) => {
                                format!("{} / {}", t!("chat.title"), t!("sample_browser.title"))
                            }
                            (true, false) => t!("chat.title").to_string(),
                            (false, true) => t!("sample_browser.title").to_string(),
                            (false, false) => String::new(),
                        };
                        ui.strong(title);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_icon = match side {
                                Side::Left => icons::CHEVRON_LEFT,
                                Side::Right => icons::CHEVRON_RIGHT,
                            };
                            if ui
                                .button(icons::rich(close_icon))
                                .on_hover_text(t!("tools.collapse"))
                                .clicked()
                            {
                                self.close();
                            }
                        });
                    });
                });

            // browser_rect: only the sample browser portion, not the whole panel.
            // Used by resolve_input_owner for click-to-focus detection.
            let mut browser_rect: Option<egui::Rect> = None;

            match (show_chat, show_sample_browser) {
                (true, true) => {
                    let sample_height = ui.available_height() * 0.5;
                    let br = egui::TopBottomPanel::bottom("tools_sample_browser")
                        .exact_height(sample_height)
                        .show_separator_line(true)
                        .show_inside(ui, |ui| {
                            sample_browser.browser_content(
                                ui,
                                ctx,
                                bridge,
                                sample_paths,
                                true,
                                is_hosting,
                                input_owner,
                            );
                        });
                    browser_rect = Some(br.response.rect);
                    chat.chat_content(ui, bridge, true);
                }
                (true, false) => {
                    chat.chat_content(ui, bridge, true);
                }
                (false, true) => {
                    let br = ui.scope(|ui| {
                        sample_browser.browser_content(
                            ui,
                            ctx,
                            bridge,
                            sample_paths,
                            true,
                            is_hosting,
                            input_owner,
                        );
                    });
                    browser_rect = Some(br.response.rect);
                }
                (false, false) => {}
            }

            browser_rect
        });

        self.settings.width = r.response.rect.width();
        r.inner
    }
}
