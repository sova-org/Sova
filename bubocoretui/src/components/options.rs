use crate::App;
use crate::components::{Component, handle_common_keys, inner_area};
use crate::event::AppEvent;
use crate::app::{LogLevel, LogEntry};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    prelude::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};
use std::error::Error;

pub struct OptionsComponent;

impl OptionsComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for OptionsComponent {
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> Result<bool, Box<dyn Error + 'static>> {
        // First try common key handlers
        if handle_common_keys(app, key_event)? {
            return Ok(true);
        }

        // Options-specific key handling
        match key_event.code {
            KeyCode::Tab => {
                app.events.send(AppEvent::SwitchToEditor);
                Ok(true)
            }
            KeyCode::Char('l') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                // Flush logs
                app.logs.clear();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        // Layout horizontal avec split 60%/40%
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // Boîte de logs (60% width)
        let log_area = main_chunks[0];
        let log_block = Block::default()
            .title("Log")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(log_block.clone(), log_area);

        // Calculate inner area for log text
        let log_text_area = inner_area(log_area);

        if log_text_area.height == 0 || log_text_area.width == 0 {
            return; // Not enough space to draw logs
        }

        // Format log entries
        let log_lines: Vec<Line> = app.logs.iter().map(format_log_entry).collect();

        // Determine how many lines fit and get the latest ones
        let num_lines_to_show = log_text_area.height as usize;
        let start_index = log_lines.len().saturating_sub(num_lines_to_show);
        let visible_log_lines_slice = &log_lines[start_index..];

        // Create the log paragraph
        let log_content = Paragraph::new(visible_log_lines_slice.to_vec())
            .style(Style::default());

        // Render the log content
        frame.render_widget(log_content, log_text_area);

        // Trois boites de taille égale (Devices, Friends, Options)
        let right_side = main_chunks[1];
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(right_side);

        // Devices
        let devices_block = Block::default()
            .title("Devices")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(devices_block, right_chunks[0]);

        let devices_content = if app.server.devices.is_empty() {
            String::from("No devices connected")
        } else {
            app.server.devices.join("\n")
        };

        let devices_text = Paragraph::new(Text::from(devices_content))
            .style(Style::default())
            .block(Block::default());

        let devices_text_area = inner_area(right_chunks[0]);
        frame.render_widget(devices_text, devices_text_area);

        // Friends
        let peers_block = Block::default()
            .title("Friends")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(peers_block, right_chunks[1]);

        let peers_content = if app.server.peers.is_empty() {
            String::from("No peers connected")
        } else {
            app.server.peers.join("\n")
        };

        let peers_text = Paragraph::new(Text::from(peers_content))
            .style(Style::default())
            .block(Block::default());

        let peers_text_area = inner_area(right_chunks[1]);
        frame.render_widget(peers_text, peers_text_area);

        // Options
        let options_block = Block::default()
            .title("Options")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(options_block, right_chunks[2]);

        let options_content = Paragraph::new(Text::from("IDK what to do :))))"))
            .style(Style::default())
            .block(Block::default());

        let options_text_area = inner_area(right_chunks[2]);
        frame.render_widget(options_content, options_text_area);
    }
}

// Helper function to format a single log entry
fn format_log_entry(log: &LogEntry) -> Line {
    let time_str = log.timestamp.format("%H:%M:%S").to_string();

    let (level_str, level_style) = match log.level {
        LogLevel::Info => ("INFO ", Style::default().fg(Color::Cyan)),
        LogLevel::Warn => ("WARN ", Style::default().fg(Color::Yellow)),
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
