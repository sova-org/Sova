use crate::app::App;
use crate::components::Component;
use crate::utils::styles::CommonStyles;
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

#[derive(Clone, Debug)]
/// Represents a single log entry in the application's logging system.
///
/// Each log entry contains a timestamp indicating when the event occurred,
/// a severity level classifying the importance of the message, and the
/// actual log message content.
///
/// # Fields
///
/// * `timestamp` - The local date and time when the log entry was created
/// * `level` - The severity level of the log entry (Info, Warn, Error, or Debug)
/// * `message` - The actual log message content
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Clone, Debug)]
/// Represents the severity level of a log entry.
///
/// This enum defines the different levels of importance that can be assigned to log messages,
/// following standard logging conventions. Each level has a specific meaning and is typically
/// used to filter and prioritize log messages.
///
/// # Variants
///
/// * `Info` - General operational information about the application's execution
/// * `Warn` - Warning messages indicating potential issues that don't prevent operation
/// * `Error` - Error messages indicating serious problems that may affect functionality
/// * `Debug` - Detailed information useful for debugging and development
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

/// The Logs component responsible for displaying and managing application logs.
///
/// This component provides a user interface for viewing log messages with different severity levels,
/// including scrolling functionality and log clearing capabilities. It maintains the visual state
/// of the log display, including scroll position and auto-follow behavior for new log entries.
///
/// The component implements the `Component` trait to handle key events and rendering, and works
/// in conjunction with `LogsState` to manage the display state and log entries.
pub struct LogsComponent;

impl Default for LogsComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl LogsComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for LogsComponent {
    fn handle_key_event(&self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
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

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(CommonStyles::default_text_themed(&app.client_config.theme));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
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

        let zebra_colors = get_log_zebra_colors(&app.client_config.theme);
        let zebra_fg_color = zebra_colors.0;
        let zebra_bg_color = zebra_colors.1;

        // Format and style the visible log lines.
        let log_lines: Vec<Line> = app
            .logs
            .range(start_index..end_index) // Get the slice of logs to display
            .enumerate()
            .map(|(i, log)| {
                let original_line = format_log_entry(log, app.client_config.theme.clone());

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

        let log_content = Paragraph::new(Text::from(log_lines))
            .wrap(Wrap { trim: false })
            .style(CommonStyles::default_text_themed(&app.client_config.theme));

        frame.render_widget(log_content, log_area);

        // Render the help text at the bottom.
        let help_style = CommonStyles::description_themed(&app.client_config.theme);
        let key_style = CommonStyles::key_binding_themed(&app.client_config.theme);
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

/// Formats a log entry into a styled line for display in the logs view.
///
/// This function takes a log entry and converts it into a formatted line with appropriate
/// styling for different components (timestamp, level indicator, and message). The log level
/// is displayed with distinct colors and styles to make it easily distinguishable:
/// - INFO: Black text on white background
/// - WARN: White text on yellow background
/// - ERROR: Bold white text on red background
/// - DEBUG: White text on magenta background
///
/// # Arguments
///
/// * `log` - A reference to the log entry to format
///
/// # Returns
///
/// A `Line` containing styled spans representing the formatted log entry
///
/// # Example
///
/// ```
/// let log = LogEntry {
///     timestamp: chrono::Local::now(),
///     level: LogLevel::Info,
///     message: "System started".to_string(),
/// };
/// let formatted_line = format_log_entry(&log);
/// ```
fn format_log_entry(log: &LogEntry, theme: crate::disk::Theme) -> Line {
    let time_str = log.timestamp.format("%H:%M:%S").to_string();
    let time_separator = " => ";

    // Define level string and its specific style based on LogLevel.
    let (level_str, level_style) = match log.level {
        LogLevel::Info => (" INFO ", get_log_level_style(LogLevel::Info, &theme)),
        LogLevel::Warn => (" WARN ", get_log_level_style(LogLevel::Warn, &theme)),
        LogLevel::Error => (" ERROR ", get_log_level_style(LogLevel::Error, &theme)),
        LogLevel::Debug => (" DEBUG ", get_log_level_style(LogLevel::Debug, &theme)),
    };

    // Construct the line with styled spans.
    Line::from(vec![
        Span::styled(time_str, get_log_timestamp_style(&theme)), // Timestamp style
        Span::styled(time_separator, get_log_timestamp_style(&theme)), // Separator style
        Span::styled(level_str, level_style),                    // Level style
        Span::raw(" "),                                          // Spacer
        Span::raw(&log.message), // Log message content (will inherit line style)
    ])
}

/// Get theme-appropriate zebra stripe colors for log entries
fn get_log_zebra_colors(theme: &crate::disk::Theme) -> (Color, Color) {
    use crate::disk::Theme;

    match theme {
        Theme::Classic => (Color::Black, Color::White),
        Theme::Ocean => (Color::Rgb(25, 25, 112), Color::Rgb(240, 248, 255)), // Midnight blue on Alice blue
        Theme::Forest => (Color::Rgb(34, 139, 34), Color::Rgb(245, 245, 220)), // Forest green on Beige
    }
}

/// Get theme-appropriate style for log level badges
fn get_log_level_style(level: LogLevel, theme: &crate::disk::Theme) -> Style {
    use crate::disk::Theme;

    match (level, theme) {
        (LogLevel::Info, Theme::Classic) => Style::default().fg(Color::Black).bg(Color::White),
        (LogLevel::Info, Theme::Ocean) => Style::default()
            .fg(Color::Rgb(25, 25, 112))
            .bg(Color::Rgb(240, 248, 255)),
        (LogLevel::Info, Theme::Forest) => Style::default()
            .fg(Color::Rgb(34, 139, 34))
            .bg(Color::Rgb(245, 245, 220)),

        (LogLevel::Warn, Theme::Classic) => Style::default().fg(Color::White).bg(Color::Yellow),
        (LogLevel::Warn, Theme::Ocean) => Style::default()
            .fg(Color::Rgb(240, 248, 255))
            .bg(Color::Rgb(255, 215, 0)),
        (LogLevel::Warn, Theme::Forest) => Style::default()
            .fg(Color::Rgb(245, 245, 220))
            .bg(Color::Rgb(255, 140, 0)),

        (LogLevel::Error, Theme::Classic) => Style::default()
            .fg(Color::White)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD),
        (LogLevel::Error, Theme::Ocean) => Style::default()
            .fg(Color::Rgb(240, 248, 255))
            .bg(Color::Rgb(220, 20, 60))
            .add_modifier(Modifier::BOLD),
        (LogLevel::Error, Theme::Forest) => Style::default()
            .fg(Color::Rgb(245, 245, 220))
            .bg(Color::Rgb(178, 34, 34))
            .add_modifier(Modifier::BOLD),

        (LogLevel::Debug, Theme::Classic) => Style::default().fg(Color::White).bg(Color::Magenta),
        (LogLevel::Debug, Theme::Ocean) => Style::default()
            .fg(Color::Rgb(240, 248, 255))
            .bg(Color::Rgb(138, 43, 226)),
        (LogLevel::Debug, Theme::Forest) => Style::default()
            .fg(Color::Rgb(245, 245, 220))
            .bg(Color::Rgb(147, 112, 219)),
    }
}

/// Get theme-appropriate style for log timestamps
fn get_log_timestamp_style(theme: &crate::disk::Theme) -> Style {
    use crate::disk::Theme;

    match theme {
        Theme::Classic => Style::default().fg(Color::White),
        Theme::Ocean => Style::default().fg(Color::Rgb(240, 248, 255)),
        Theme::Forest => Style::default().fg(Color::Rgb(245, 245, 220)),
    }
}
