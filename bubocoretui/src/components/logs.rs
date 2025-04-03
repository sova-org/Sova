use crate::App;
use crate::components::Component;
use crate::app::{LogEntry, LogLevel};
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Rect, Constraint, Layout, Direction},
    style::{Color, Style, Modifier},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};
use std::fmt;

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Debug => write!(f, "DEBUG"),
        }
    }
}

pub struct LogsState {
    pub scroll_position: usize,
}

impl LogsState {
    pub fn new() -> Self {
        Self {
            scroll_position: 0,
        }
    }
}

pub struct LogsComponent;

impl LogsComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for LogsComponent {
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        match key_event.code {
            KeyCode::Up => {
                if app.logs.len() > 0 {
                    app.interface.components.logs_state.scroll_position = 
                        app.interface.components.logs_state.scroll_position.saturating_sub(1);
                }
                Ok(true)
            }
            KeyCode::Down => {
                if app.logs.len() > 0 {
                    let max_scroll = app.logs.len().saturating_sub(1);
                     app.interface.components.logs_state.scroll_position = 
                         (app.interface.components.logs_state.scroll_position + 1).min(max_scroll);
                }
                Ok(true)
            }
            KeyCode::Char('l') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.logs.clear();
                app.interface.components.logs_state.scroll_position = 0;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Application/Server Logs ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(inner_area);
        
        let log_area = chunks[0];
        let help_area = chunks[1];

        if log_area.height == 0 || log_area.width == 0 {
            return; // Not enough space
        }

        let log_lines: Vec<Line> = app.logs.iter().map(format_log_entry).collect();

        let num_lines_to_show = log_area.height as usize;
        let total_lines = log_lines.len();
        let start_index = total_lines.saturating_sub(num_lines_to_show);
        let visible_log_lines_slice = &log_lines[start_index..];

        let log_content = Paragraph::new(visible_log_lines_slice.to_vec())
            .style(Style::default());

        frame.render_widget(log_content, log_area);

        let help_text = "↑↓: Scroll | Ctrl+L: Clear";
        let help = Paragraph::new(Text::from(help_text))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}

fn format_log_entry(log: &LogEntry) -> Line {
    let time_str = log.timestamp.format("%H:%M:%S").to_string();

    let (level_str, level_style) = match log.level {
        LogLevel::Info => ("INFO ", Style::default().fg(Color::Cyan)),
        LogLevel::Warn => ("WARN ", Style::default().fg(Color::Yellow)),
        LogLevel::Error => (
            "ERROR",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
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