use crate::app::{App, LogEntry};
use bubocorelib::server::LogLevel;
use crate::components::Component;
use chrono::{DateTime, Local};
use ratatui::{
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crossterm::event::KeyEvent;
use color_eyre::Result as EyreResult;
use crate::event::Action;

pub struct LogsComponent;

impl LogsComponent {
    pub fn new() -> Self {
        Self {}
    }

    fn format_log_entry(log: &LogEntry) -> Line {
        let timestamp: DateTime<Local> = log.timestamp.into();
        let time_str = timestamp.format("%H:%M:%S").to_string();

        let (level_str, level_style) = match log.level {
            LogLevel::Info => ("INFO", Style::default().fg(Color::Cyan)),
            LogLevel::Warn => ("WARN", Style::default().fg(Color::Yellow)),
            LogLevel::Error => ("ERROR", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            LogLevel::Debug => ("DEBUG", Style::default().fg(Color::Gray)),
        };

        Line::from(vec![
            Span::styled(time_str, Style::default().fg(Color::DarkGray)),
            Span::raw(" ["),
            Span::styled(level_str, level_style),
            Span::raw("] "),
            Span::raw(&log.message),
        ])
    }
}

impl Component for LogsComponent {

    fn handle_key_event(
        &mut self,
        _key_event: KeyEvent,
    ) -> EyreResult<Option<Action>> {
        // Currently, the logs component doesn't handle any keys directly.
        // It might in the future (e.g., for scrolling).
        Ok(None) // Return None indicating no action taken
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let log_block = Block::default()
            .title("Logs")
            .borders(Borders::ALL)
            .style(Style::default()); // Inherit background from parent

        // Calculate inner area for text
        let inner_area = log_block.inner(area);
        frame.render_widget(log_block.clone(), area); // Clone block to render border first

        if inner_area.height == 0 || inner_area.width == 0 {
            return; // Not enough space to draw logs
        }

        let log_lines: Vec<Line> = app.logs.iter().map(Self::format_log_entry).collect();

        let num_lines_to_show = inner_area.height as usize;
        let start_index = log_lines.len().saturating_sub(num_lines_to_show);
        let visible_log_lines_slice = &log_lines[start_index..];

        // Apply zebra striping
        let styled_log_lines: Vec<Line> = visible_log_lines_slice
            .iter()
            .enumerate()
            .map(|(i, line)| {
                // Use start_index + i to determine the effective index for consistent striping
                let effective_index = start_index + i;
                let bg_style = if effective_index % 2 == 0 {
                    Style::default() // Default background for even lines
                } else {
                    Style::default().bg(Color::DarkGray) // Use DarkGray for odd lines
                };
                // Clone the line and apply the background style to the whole line
                let mut styled_line = line.clone();
                styled_line.patch_style(bg_style);
                styled_line
            })
            .collect();

        let log_paragraph = Paragraph::new(styled_log_lines)
            .style(Style::default()); // Base style without background

        frame.render_widget(log_paragraph, inner_area);
    }
}
