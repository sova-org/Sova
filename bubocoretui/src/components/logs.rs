use crate::app::App;
use crate::components::Component;
use chrono::{DateTime, Local};
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};
use std::fmt;

/// Represents a single log message with its timestamp, level, and content.
#[derive(Clone, Debug)]
pub struct LogEntry {
    /// The time the log entry was created.
    pub timestamp: DateTime<Local>,
    /// The severity level of the log entry.
    pub level: LogLevel,
    /// The textual content of the log message.
    pub message: String,
}

/// Defines the severity levels for log entries.
#[derive(Clone, Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

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

/// A UI component responsible for displaying application logs.
///
/// This component handles rendering the log entries with appropriate styling,
/// scrolling behavior, and user interactions like clearing logs.
#[allow(dead_code)] // TODO: Remove if LogsComponent struct itself isn't directly used elsewhere
pub struct LogsComponent;

impl LogsComponent {
    /// Creates a new instance of the `LogsComponent`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for LogsComponent {
    /// Handles key events relevant to the logs component.
    ///
    /// Implements scrolling (Up/Down, PageUp/PageDown, Home/End) and log clearing (Ctrl+L).
    /// Manages the `is_following` state to automatically scroll to the bottom for new logs,
    /// unless the user manually scrolls up.
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        let total_lines = app.logs.len();
        // The maximum scroll position index (0-based).
        let theoretical_max_scroll = total_lines.saturating_sub(1);

        match key_event.code {
            // Scroll up: Disable following and move scroll position up.
            KeyCode::Up | KeyCode::PageUp | KeyCode::Home => {
                app.interface.components.logs_state.is_following = false;
                let current_pos = app.interface.components.logs_state.scroll_position;
                let new_pos = match key_event.code {
                    KeyCode::Up => current_pos.saturating_sub(1),
                    KeyCode::PageUp | KeyCode::Home => 0, // Jump to top
                    _ => unreachable!(),
                };
                app.interface.components.logs_state.scroll_position = new_pos;
                Ok(true)
            }
            // Scroll down: Move scroll position down, potentially re-enable following.
            KeyCode::Down | KeyCode::PageDown | KeyCode::End => {
                if total_lines > 0 {
                    let current_pos = app.interface.components.logs_state.scroll_position;
                    let new_pos = match key_event.code {
                        KeyCode::Down => (current_pos + 1).min(theoretical_max_scroll),
                        KeyCode::PageDown | KeyCode::End => theoretical_max_scroll, // Jump to bottom
                        _ => unreachable!(),
                    };
                    app.interface.components.logs_state.scroll_position = new_pos;
                    // If scrolled to the absolute bottom, resume following new logs.
                    if new_pos >= theoretical_max_scroll {
                        app.interface.components.logs_state.is_following = true;
                    }
                }
                Ok(true)
            }
            // Clear logs: Reset logs and scroll position.
            KeyCode::Char('l') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.logs.clear();
                app.interface.components.logs_state.scroll_position = 0;
                app.interface.components.logs_state.is_following = true;
                app.set_status_message("Logs cleared.".to_string());
                Ok(true)
            }
            // Ignore other keys for this component.
            _ => Ok(false),
        }
    }

    /// Renders the logs component within the designated area.
    ///
    /// Calculates the visible log lines based on scroll position and `is_following` state.
    /// Formats lines with timestamps, levels, and applies zebra striping for readability.
    /// Uses a `Paragraph` widget with wrapping enabled to display the logs.
    /// Also renders a help line indicating available keybindings.
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::White)); // Default border color

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Split area into main log view and a small help line at the bottom.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)]) // Log area, Help line
            .split(inner_area);

        let log_area = chunks[0];
        let help_area = chunks[1];

        // Avoid drawing if the area is too small.
        if log_area.height == 0 || log_area.width == 0 {
            return;
        }

        let total_lines = app.logs.len();
        // Number of lines that can fit in the log_area vertically.
        let num_displayable_lines = log_area.height as usize;
        // The maximum scroll position index considering the display height.
        let max_scroll_for_view = total_lines.saturating_sub(num_displayable_lines);

        // Determine the actual scroll position to start rendering from.
        let current_scroll = if app.interface.components.logs_state.is_following {
            max_scroll_for_view // If following, scroll to the very bottom.
        } else {
            // Otherwise, use the stored position, clamped by the maximum possible scroll.
            app.interface
                .components
                .logs_state
                .scroll_position
                .min(max_scroll_for_view)
        };

        // Calculate the range of log entries to display.
        let start_index = current_scroll;
        let end_index = (start_index + num_displayable_lines).min(total_lines);

        let zebra_fg_color = Color::Black;
        let zebra_bg_color = Color::White;

        // Format and style the visible log lines.
        let log_lines: Vec<Line> = app
            .logs
            .range(start_index..end_index) // Get the slice of logs to display
            .enumerate()
            .map(|(i, log)| {
                let original_line = format_log_entry(log);

                // Determine the style for zebra striping (alternating backgrounds).
                let is_striped_line = (start_index + i) % 2 == 1;
                let stripe_style = Style::default().bg(zebra_bg_color).fg(zebra_fg_color);

                // Apply the stripe style only to specific spans (level, message), preserving others.
                let styled_spans: Vec<Span> = original_line
                    .spans
                    .into_iter()
                    .enumerate()
                    .map(|(span_idx, span)| {
                        // Apply stripe only to spans after the timestamp and separator (index > 1)
                        if is_striped_line && span_idx > 1 {
                            Span::styled(span.content, span.style.patch(stripe_style))
                        } else {
                            // Keep original style for timestamp/separator or non-striped lines
                            span
                        }
                    })
                    .collect();

                Line::from(styled_spans)
            })
            .collect();

        // Create a Paragraph widget to display the log lines with wrapping.
        let log_content = Paragraph::new(Text::from(log_lines))
            .wrap(Wrap { trim: false }) // Enable wrapping, don't trim whitespace.
            .style(Style::default()); // Base style for the paragraph itself.

        frame.render_widget(log_content, log_area);

        // Render the help text at the bottom.
        let help_style = Style::default().fg(Color::DarkGray);
        let key_style = Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD);
        let help_spans = vec![
            Span::styled("↑↓", key_style),
            Span::styled(": Scroll | ", help_style),
            Span::styled("PgUp/PgDn", key_style),
            Span::styled(": Jump | ", help_style),
            Span::styled("Home/End", key_style),
            Span::styled(": Top/Bottom | ", help_style),
            Span::styled("Ctrl+L", key_style),
            Span::styled(": Clear", help_style),
        ];
        let help = Paragraph::new(Line::from(help_spans)).alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}

/// Formats a single log entry into a `Line` with appropriate styling for timestamp and level.
fn format_log_entry(log: &LogEntry) -> Line {
    let time_str = log.timestamp.format("%H:%M:%S").to_string();
    let time_separator = " => ";

    // Define level string and its specific style based on LogLevel.
    let (level_str, level_style) = match log.level {
        LogLevel::Info => (" INFO ", Style::default().fg(Color::Black).bg(Color::White)),
        LogLevel::Warn => (
            " WARN ",
            Style::default().fg(Color::White).bg(Color::Yellow),
        ),
        LogLevel::Error => (
            " ERROR ",
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
        LogLevel::Debug => (
            " DEBUG ",
            Style::default().fg(Color::White).bg(Color::Magenta),
        ),
    };

    // Construct the line with styled spans.
    Line::from(vec![
        Span::styled(time_str, Style::default().fg(Color::White)), // Timestamp style (white)
        Span::styled(time_separator, Style::default().fg(Color::White)), // Separator style (white)
        Span::styled(level_str, level_style), // Level style
        Span::raw(" "), // Spacer
        Span::raw(&log.message), // Log message content (will inherit line style)
    ])
}
