use crate::app::{App, Mode};
use color_eyre::Result as EyreResult;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Clear, Paragraph},
};
use crate::components::Component;
use crate::components::editor::EditorComponent;
use crate::components::grid::GridComponent;
use crate::components::help::HelpComponent;
use crate::components::navigation::NavigationComponent;
use crate::components::options::OptionsComponent;
use crate::components::splash::SplashComponent;
use crate::components::devices::DevicesComponent;
use crate::components::logs::{LogsComponent, LogLevel};
use crate::components::saveload::SaveLoadComponent;
use std::time::{Duration, Instant};

/// Flash UI effect on evaluation
pub struct Flash {
    pub is_flashing: bool,
    pub flash_start: Option<Instant>,
    pub flash_duration: Duration,
    pub flash_color: Color,
}

/// Main UI drawing function
/// 
/// This function is called on each tick
/// It checks the flash status and draws the UI components
/// It also draws the top and bottom bars
/// 
/// # Arguments
/// 
/// * `frame` - The frame to draw on
/// * `app` - The application state
pub fn ui(frame: &mut Frame, app: &mut App) {
    check_flash_status(app);

    // Constraints are adjusted based on the display of the phase bar
    let top_bar_height = if app.settings.show_phase_bar { 1 } else { 0 };

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // Phase bar
            Constraint::Length(top_bar_height), 
            // Central area
            Constraint::Min(1),
            // Bottom bar
            Constraint::Length(1),
        ])
        .split(frame.area());

    let top_bar = main_layout[0];
    let main_area = main_layout[1];
    let bottom_bar = main_layout[2];

    draw_top_bar(frame, app, top_bar);

    // Draw the active component
    match app.interface.screen.mode {
        Mode::Splash => SplashComponent::new().draw(app, frame, main_area),
        Mode::Editor => EditorComponent::new().draw(app, frame, main_area),
        Mode::Grid => GridComponent::new().draw(app, frame, main_area),
        Mode::Options => OptionsComponent::new().draw(app, frame, main_area),
        Mode::Help => HelpComponent::new().draw(app, frame, main_area),
        Mode::Devices => DevicesComponent::new().draw(app, frame, main_area),
        Mode::Logs => LogsComponent::new().draw(app, frame, main_area),
        Mode::SaveLoad => SaveLoadComponent::new().draw(app, frame, main_area),
        Mode::Navigation => NavigationComponent::new().draw(app, frame, main_area),
    }

    // Draw the bottom bar
    if let Err(e) = draw_bottom_bar(frame, app, bottom_bar) {
        app.add_log(LogLevel::Error, format!("Error drawing bottom bar: {}", e));
    }

    // Flash effect (when needed)
    if app.interface.screen.flash.is_flashing {
        frame.render_widget(Clear, frame.area());
        frame.render_widget(
            Block::default().style(Style::default().bg(app.interface.screen.flash.flash_color)),
            frame.area(),
        );
    }
}

/// Check to update the flash status
fn check_flash_status(app: &mut App) {
    if app.interface.screen.flash.is_flashing {
        if let Some(start_time) = app.interface.screen.flash.flash_start {
            if start_time.elapsed() > app.interface.screen.flash.flash_duration {
                app.interface.screen.flash.is_flashing = false;
                app.interface.screen.flash.flash_start = None;
            }
        }
    }
}

/// Draw the bottom bar 
/// 
/// This function draws the bottom bar, in charge of displaying
/// the bottom message, the mode, the username, the tempo
/// and the phase bar
/// 
/// # Arguments
/// 
/// * `frame` - The frame to draw on
/// * `app` - The application state
/// * `area` - The area to draw on
/// 
/// # Returns
/// 
/// * `EyreResult<()>` - The result of the draw operation
pub fn draw_bottom_bar(frame: &mut Frame, app: &mut App, area: Rect) -> EyreResult<()> {
    // General style for the bar (white background, default black text)
    let base_style = Style::default().bg(Color::White).fg(Color::Black);
    frame.render_widget(Block::default().style(base_style), area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), 
            Constraint::Percentage(40),
        ])
        .split(area);

    let left_area = chunks[0];
    let right_area = chunks[1];

    // Left side: Mode and Status Message (no changes needed here)
    let mode_text = match app.interface.screen.mode {
        Mode::Editor => "EDITOR",
        Mode::Grid => "GRID",
        Mode::Options => "OPTIONS",
        Mode::Splash => "WELCOME",
        Mode::Help => "HELP",
        Mode::Devices => "DEVICES",
        Mode::Logs => "LOGS",
        Mode::Navigation => "MENU",
        Mode::SaveLoad => "FILES"
    };
    let mode_style = Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD);
    let mode_width = mode_text.len() + 2; 
    let separator_width = 3; 
    let max_message_width = left_area.width.saturating_sub(
        mode_width as u16 + separator_width as u16) as usize;
    let message = &app.interface.components.bottom_message;
    let truncated_message = if message.len() > max_message_width {
         format!("{}...", &message[..max_message_width.saturating_sub(3)])
    } else {
         message.to_string()
    };
    let left_text = Line::from(vec![
        Span::styled(format!(" {} ", mode_text), mode_style),
        Span::raw(" | "),
        Span::styled(truncated_message, Style::default().fg(Color::Black)),
    ]);
    let left_paragraph = Paragraph::new(left_text)
        .style(base_style)
        .alignment(Alignment::Left);
    frame.render_widget(left_paragraph, left_area);

    // Right side: Username, Phase Bar, Tempo (no changes needed here)
    let tempo = app.server.link.session_state.tempo();
    let phase = app.server.link.get_phase();
    let quantum = app.server.link.quantum.max(1.0);
    let username = &app.server.username;
    let is_playing = app.server.is_transport_playing;

    // Mini Phase Bar
    let mini_bar_width = 10; 
    let filled_ratio = (phase / quantum).clamp(0.0, 1.0);
    let filled_count = (filled_ratio * mini_bar_width as f64).round() as usize;
    let empty_count = mini_bar_width - filled_count;
    let mini_bar_str = format!("{}{}", "█".repeat(filled_count), " ".repeat(empty_count));
    let mini_bar_color = if is_playing { Color::Green } else { Color::Red };
    let mini_bar_style = Style::default().fg(mini_bar_color);
    let phase_bar_display_width = mini_bar_width + 2; // Bar + padding/separators

    // Determine Play/Stop status text and style
    let (status_text, status_style) = if is_playing {
        (" ▶ PLAY ", Style::default().bg(Color::Green).fg(Color::Black))
    } else {
        (" ■ STOP ", Style::default().bg(Color::Red).fg(Color::White))
    };
    let status_width = status_text.len();

    let tempo_text = format!(" {:.1} BPM ", tempo);
    let tempo_width = tempo_text.len() + 1;
    // Calculate reserved width: Tempo + Status + PhaseBar + Separators (3 total)
    let reserved_width = tempo_width + status_width + phase_bar_display_width + 3;
    let max_username_width = right_area.width.saturating_sub(reserved_width as u16) as usize;
    let truncated_username = if username.len() > max_username_width {
        format!("{}...", &username[..max_username_width.saturating_sub(3)])
    } else {
        username.clone()
    };
    let right_text = Line::from(vec![
        Span::styled(truncated_username, Style::default().fg(Color::Red)),
        Span::raw(" | "),
        Span::styled(mini_bar_str, mini_bar_style), // Keep the mini bar
        Span::raw(" | "),
        Span::styled(status_text, status_style),    // Add the status text
        Span::raw(" | "),
        Span::styled(tempo_text, Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)),
     ]).alignment(Alignment::Right);
    let right_paragraph = Paragraph::new(right_text)
        .style(base_style);
    frame.render_widget(right_paragraph, right_area);

    Ok(())
}

/// Function used to draw an optional phase bar
/// 
/// This function draws a phase bar on the top of the screen
/// It is optional and can be disabled in the settings
/// 
/// # Arguments
/// 
/// * `frame` - The frame to draw on
/// * `app` - The application state
/// * `area` - The area to draw on
/// 
/// # Returns
/// 
/// * `EyreResult<()>` - The result of the draw operation
fn draw_top_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    if !app.settings.show_phase_bar {
        return;
    }

    let phase = app.server.link.get_phase();
    let quantum = app.server.link.quantum.max(1.0);
    let available_width = area.width as usize;
    // Use the app's transport state flag
    let is_playing = app.server.is_transport_playing;
    let bar_color = if is_playing { Color::Green } else { Color::Red }; // Determine color

    // Ensure phase calculation doesn't lead to NaN or Inf if quantum is tiny
    let filled_ratio = if quantum > 0.0 { (phase / quantum).clamp(0.0, 1.0) } else { 0.0 };
    let filled_width = (filled_ratio * available_width as f64).round() as usize;

    let mut bar = String::with_capacity(available_width);
    for i in 0..available_width {
        if i < filled_width {
            bar.push('█');
        } else {
            bar.push(' ');
        }
    }
    let top_bar = Paragraph::new(Text::from(bar))
        .style(Style::default().bg(bar_color));
    frame.render_widget(top_bar, area);
}