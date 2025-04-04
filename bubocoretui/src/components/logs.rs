use crate::App;
use crate::components::Component;
use crate::app::{LogEntry, LogLevel};
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Rect, Constraint, Layout, Direction},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, ListItem, BorderType},
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
    pub is_following: bool,
}

impl LogsState {
    pub fn new() -> Self {
        Self {
            scroll_position: 0,
            is_following: true,
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

    fn before_draw(&mut self, _app: &mut App) -> EyreResult<()> {
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool> {
        let total_lines = app.logs.len();
        let theoretical_max_scroll = total_lines.saturating_sub(1);

        match key_event.code {
            // Scroll up the logs
            KeyCode::Up | KeyCode::PageUp | KeyCode::Home => {
                app.interface.components.logs_state.is_following = false;
                match key_event.code {
                    KeyCode::Up => {
                        app.interface.components.logs_state.scroll_position =
                            app.interface.components.logs_state.scroll_position.saturating_sub(1);
                    }
                    KeyCode::PageUp | KeyCode::Home => {
                        app.interface.components.logs_state.scroll_position = 0;
                    }
                    _ => unreachable!(),
                }
                Ok(true)
            }
            // Scroll down the logs
            KeyCode::Down | KeyCode::PageDown | KeyCode::End => {
                if total_lines > 0 {
                    let mut new_scroll_pos = app.interface.components.logs_state.scroll_position;
                    match key_event.code {
                        KeyCode::Down => {
                            new_scroll_pos = (new_scroll_pos + 1).min(theoretical_max_scroll);
                        }
                        KeyCode::PageDown | KeyCode::End => {
                            new_scroll_pos = theoretical_max_scroll as usize;
                        }
                         _ => unreachable!(),
                    }
                    app.interface.components.logs_state.scroll_position = new_scroll_pos;
                    // Check if we reached the bottom and resume following
                    if new_scroll_pos >= theoretical_max_scroll {
                         app.interface.components.logs_state.is_following = true;
                    }
                }
                Ok(true)
            }
            // Clear the logs
            KeyCode::Char('l') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.logs.clear();
                app.interface.components.logs_state.scroll_position = 0;
                app.interface.components.logs_state.is_following = true;
                app.set_status_message("Logs cleared.".to_string());
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Draw the logs component
    /// 
    /// # Arguments
    /// 
    /// * `app`: The application state
    /// * `frame`: The frame to draw on
    /// * `area`: The area to draw on
    /// 
    /// # Returns
    /// 
    /// * `()`
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let title = " Logs ";
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::White));

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

        let total_lines = app.logs.len();
        let num_lines_to_show = log_area.height as usize;
        // Calculate the max scroll based on the current view height
        let max_scroll_for_view = total_lines.saturating_sub(num_lines_to_show);

        // Determine the scroll position for rendering
        let current_scroll =
            if app.interface.components.logs_state.is_following {
                max_scroll_for_view // If following, always show the end
            } else {
                // Otherwise, use the stored position, clamped to the view
                app.interface.components.logs_state.scroll_position.min(max_scroll_for_view)
            };

        let start_index = current_scroll;
        let end_index = (start_index + num_lines_to_show).min(total_lines);
        let zebra_color = Color::DarkGray;

        let log_lines: Vec<ListItem> = app.logs.range(start_index..end_index)
            .enumerate()
            .map(|(i, log)| {
                let line = format_log_entry(log);
                let style = if (start_index + i) % 2 == 1 {
                    Style::default().bg(zebra_color)
                } else {
                    Style::default()
                };
                ListItem::new(line).style(style)
            })
            .collect();

        let log_content = ratatui::widgets::List::new(log_lines)
            .style(Style::default());

        frame.render_widget(log_content, log_area);

        // Help text remains the same
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        let help_spans = vec![
            Span::styled("↑↓", key_style), Span::styled(": Scroll | ", help_style),
            Span::styled("PgUp/PgDn", key_style), Span::styled(": Jump | ", help_style),
            Span::styled("Home/End", key_style), Span::styled(": Top/Bottom | ", help_style),
            Span::styled("Ctrl+L", key_style), Span::styled(": Clear", help_style),
        ];
        let help = Paragraph::new(Line::from(help_spans))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}

/// Format a log entry into a line
/// 
/// # Arguments
/// 
/// * `log`: The log entry to format
/// 
/// # Returns
///
/// * `Line`: The formatted log entry
fn format_log_entry(log: &LogEntry) -> Line {
    let time_str = log.timestamp.format("%H:%M:%S").to_string();

    let (level_str, level_style) = match log.level {
        LogLevel::Info => (" INFO ", Style::default().fg(Color::Black).bg(Color::White)),
        LogLevel::Warn => (" WARN ", Style::default().fg(Color::White).bg(Color::Yellow)),
        LogLevel::Error => (
            " ERROR ",
            Style::default().fg(Color::White)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD),
        ),
        LogLevel::Debug => (" DEBUG ", Style::default().fg(Color::White).bg(Color::Magenta)),
    };

    Line::from(vec![
        Span::styled(time_str, Style::default().bg(Color::White).fg(Color::Black)),
        Span::styled(level_str, level_style),
        Span::raw(" "),
        Span::raw(&log.message),
    ])
} 