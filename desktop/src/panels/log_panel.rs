use crate::theme::COLOR_ERROR;
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

const SEVERITY_LABELS: [&str; 5] = ["F", "E", "W", "I", "D"];
const SEVERITIES: [Severity; 5] = [
    Severity::Fatal,
    Severity::Error,
    Severity::Warn,
    Severity::Info,
    Severity::Debug,
];

#[derive(PartialEq)]
enum LogTab {
    Server,
    Client,
}

pub struct LogPanel {
    rx: mpsc::Receiver<LogEntry>,
    server_logs: VecDeque<LogMessage>,
    client_logs: VecDeque<LogMessage>,
    active_tab: LogTab,
    severity_filter: [bool; 5],
    search: String,
}

impl LogPanel {
    pub fn new(rx: mpsc::Receiver<LogEntry>) -> Self {
        Self {
            rx,
            server_logs: VecDeque::new(),
            client_logs: VecDeque::new(),
            active_tab: LogTab::Server,
            severity_filter: [true; 5],
            search: String::new(),
        }
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

    pub fn show_inside(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active_tab, LogTab::Server, t!("log.server"));
            ui.selectable_value(&mut self.active_tab, LogTab::Client, t!("log.client"));

            ui.separator();

            for (i, sev) in SEVERITIES.iter().enumerate() {
                let active = self.severity_filter[i];
                let color = severity_color(sev);
                let dimmed = color.linear_multiply(0.3);
                let btn_color = if active { color } else { dimmed };
                let btn = egui::Button::new(
                    egui::RichText::new(SEVERITY_LABELS[i])
                        .color(btn_color)
                        .strong(),
                )
                .frame(false);
                if ui.add(btn).clicked() {
                    self.severity_filter[i] = !self.severity_filter[i];
                }
            }

            ui.separator();

            let search_edit = egui::TextEdit::singleline(&mut self.search)
                .hint_text(t!("log.search"))
                .desired_width(120.0);
            ui.add(search_edit);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(crate::icons::rich(crate::icons::TRASH))
                    .on_hover_text(t!("log.clear"))
                    .clicked()
                {
                    match self.active_tab {
                        LogTab::Server => self.server_logs.clear(),
                        LogTab::Client => self.client_logs.clear(),
                    }
                }
            });
        });

        ui.separator();

        let logs = match self.active_tab {
            LogTab::Server => &self.server_logs,
            LogTab::Client => &self.client_logs,
        };

        let search_lower = self.search.to_lowercase();

        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink(false)
            .show(ui, |ui| {
                let stripe = egui::Color32::from_white_alpha(10);
                let mut visible_idx = 0usize;
                for log in logs.iter() {
                    if !self.severity_filter[severity_index(&log.level)] {
                        continue;
                    }
                    if !search_lower.is_empty() && !log.msg.to_lowercase().contains(&search_lower) {
                        continue;
                    }

                    let color = severity_color(&log.level);
                    let resp =
                        ui.colored_label(color, egui::RichText::new(log.to_string()).monospace());
                    if visible_idx % 2 == 1 {
                        let row = egui::Rect::from_x_y_ranges(
                            ui.clip_rect().x_range(),
                            resp.rect.y_range(),
                        );
                        ui.painter().rect_filled(row, 0.0, stripe);
                    }
                    visible_idx += 1;
                }
            });
    }
}

fn severity_index(severity: &Severity) -> usize {
    match severity {
        Severity::Fatal => 0,
        Severity::Error => 1,
        Severity::Warn => 2,
        Severity::Info => 3,
        Severity::Debug => 4,
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
