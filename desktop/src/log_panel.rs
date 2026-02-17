use crate::widgets::COLOR_ERROR;
use eframe::egui;
use sova_core::protocol::log::{LogMessage, Severity};
use std::collections::VecDeque;
use std::sync::mpsc;

pub struct LogEntry {
    pub source: LogSource,
    pub message: LogMessage,
}

pub enum LogSource {
    Server,
    Client,
}

const MAX_LOGS: usize = 500;

#[derive(PartialEq)]
enum LogTab {
    Server,
    Client,
}

pub struct LogPanel {
    pub collapsed: bool,
    height: f32,
    rx: mpsc::Receiver<LogEntry>,
    server_logs: VecDeque<LogMessage>,
    client_logs: VecDeque<LogMessage>,
    active_tab: LogTab,
}

impl LogPanel {
    pub fn new(rx: mpsc::Receiver<LogEntry>, height: f32) -> Self {
        Self {
            collapsed: true,
            height,
            rx,
            server_logs: VecDeque::new(),
            client_logs: VecDeque::new(),
            active_tab: LogTab::Server,
        }
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn poll(&mut self) {
        while let Ok(entry) = self.rx.try_recv() {
            let logs = match entry.source {
                LogSource::Server => &mut self.server_logs,
                LogSource::Client => &mut self.client_logs,
            };
            logs.push_back(entry.message);
            if logs.len() > MAX_LOGS {
                logs.pop_front();
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let panel_height = if self.collapsed { 0.0 } else { self.height };

        let resp = egui::TopBottomPanel::bottom("logs")
            .resizable(!self.collapsed)
            .default_height(panel_height)
            .max_height(400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.active_tab, LogTab::Server, t!("log.server"));
                    ui.selectable_value(&mut self.active_tab, LogTab::Client, t!("log.client"));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let label = if self.collapsed {
                            crate::icons::CHEVRON_UP
                        } else {
                            crate::icons::CHEVRON_DOWN
                        };
                        if ui.button(label).clicked() {
                            self.collapsed = !self.collapsed;
                        }
                    });
                });

                if self.collapsed {
                    return;
                }

                ui.separator();

                let logs = match self.active_tab {
                    LogTab::Server => &self.server_logs,
                    LogTab::Client => &self.client_logs,
                };

                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let stripe = egui::Color32::from_white_alpha(10);
                        for (i, log) in logs.iter().enumerate() {
                            let color = severity_color(&log.level);
                            let resp = ui.colored_label(
                                color,
                                egui::RichText::new(log.to_string()).monospace(),
                            );
                            if i % 2 == 1 {
                                let row = egui::Rect::from_x_y_ranges(
                                    ui.clip_rect().x_range(),
                                    resp.rect.y_range(),
                                );
                                ui.painter().rect_filled(row, 0.0, stripe);
                            }
                        }
                    });
            });

        if !self.collapsed {
            self.height = resp.response.rect.height();
        }
    }
}

fn severity_color(severity: &Severity) -> egui::Color32 {
    match severity {
        Severity::Fatal | Severity::Error => COLOR_ERROR,
        Severity::Warn => egui::Color32::from_rgb(200, 180, 100),
        Severity::Info => egui::Color32::from_rgb(200, 200, 200),
        Severity::Debug => egui::Color32::from_rgb(130, 130, 130),
    }
}
