use crate::app::{App, Mode};
use crate::components::Component;
use crate::components::devices::DevicesComponent;
use crate::components::editor::EditorComponent;
use crate::components::grid::GridComponent;
use crate::components::help::HelpComponent;
use crate::components::logs::LogsComponent;
use crate::components::navigation::NavigationComponent;
use crate::components::options::OptionsComponent;
use crate::components::saveload::SaveLoadComponent;
use crate::components::splash::SplashComponent;
use ratatui::{
    buffer::Buffer,
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Clear, Widget},
};
use std::time::{Duration, Instant};
use unicode_width::UnicodeWidthStr;

/// Represents a flash effect state for the UI.
pub struct Flash {
    /// Whether the flash effect is currently active.
    pub is_flashing: bool,
    /// The time when the flash effect started.
    pub flash_start: Option<Instant>,
    /// The duration of the flash effect.
    pub flash_duration: Duration,
    /// The color of the flash effect.
    pub flash_color: Color,
}

/// Widget to display the top context bar (Mode | Message).
struct ContextBarWidget<'a> {
    mode: Mode,
    message: &'a str,
}

impl<'a> Widget for ContextBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let base_style = Style::default().bg(Color::White).fg(Color::Black);
        buf.set_style(area, base_style);

        let mode_style = Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD);

        let mode_width_guess: u16 = 12;

        // Ensure constraints don't exceed area width
        let available_width = area.width;
        let actual_mode_width = mode_width_guess.min(available_width);
        let message_min_width: u16 = 1; // Allow message area to shrink

        let constraints = [
            Constraint::Length(actual_mode_width),
            Constraint::Min(message_min_width),
        ];

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        let mode_area = chunks[0];
        let message_area = chunks[1];

        // 1. Draw Mode
        let mode_text_inner = match self.mode {
             Mode::Editor => "EDITOR", Mode::Grid => "GRID", Mode::Options => "OPTIONS",
             Mode::Splash => "WELCOME", Mode::Help => "HELP", Mode::Devices => "DEVICES",
             Mode::Logs => "LOGS", Mode::Navigation => "NAVIGATION", Mode::SaveLoad => "FILES",
         };
        let mode_text_padded = format!(" {} ", mode_text_inner);
        buf.set_stringn(
            mode_area.left(),
            mode_area.top(),
            &mode_text_padded,
            mode_area.width as usize,
            mode_style,
        );

        // 2. Draw Message (Centered)
        let max_message_width = message_area.width as usize;
        if max_message_width > 0 {
            let truncated_message = if self.message.width() > max_message_width {
                 if max_message_width >= 3 {
                     let mut truncated: String = self.message.chars().take(max_message_width.saturating_sub(3)).collect();
                     truncated.push_str("...");
                     truncated
                 } else {
                     self.message.chars().next().map_or(String::new(), |c| c.to_string())
                 }
            } else {
                self.message.to_string()
            };
            let msg_width = truncated_message.width() as u16;
            let msg_x = message_area.left() + message_area.width.saturating_sub(msg_width) / 2;
            buf.set_stringn(
                msg_x,
                message_area.top(),
                &truncated_message,
                msg_width as usize, // Draw only the calculated width
                Style::default().fg(Color::Black),
            );
        }
    }
}

/// Widget to display the bottom phase/tempo bar.
/// Renders a full-width phase bar with centered, overlaid status and tempo text.
/// Uses direct buffer manipulation for precise cell control and dynamic background colors.
struct PhaseTempoBarWidget {
    phase: f64,
    quantum: f64,
    is_playing: bool,
    tempo: f64,
}

impl Widget for PhaseTempoBarWidget {
     fn render(self, area: Rect, buf: &mut Buffer) {
        let available_width = area.width as usize;
        if available_width == 0 { return; }

        // Calculate phase bar state
        let bar_fg_color = if self.is_playing { Color::Green } else { Color::Red };
        let filled_ratio = if self.quantum > 0.0 { (self.phase / self.quantum).clamp(0.0, 1.0) } else { 0.0 };
        let filled_width = (filled_ratio * available_width as f64).round() as usize;

        let mut bar_chars: Vec<char> = Vec::with_capacity(available_width);
        for i in 0..available_width {
            bar_chars.push(if i < filled_width { '█' } else { ' ' });
        }

        // Prepare overlay text content and style
        let (status_symbol, _) = if self.is_playing { ('▶', Color::Green) } else { ('■', Color::Red) };
        let tempo_text = format!("{:.1} BPM", self.tempo);
        let separator = " | ";
        let overlay_content_str = format!("{}{}{}", status_symbol, separator, tempo_text);
        let overlay_content_chars: Vec<char> = overlay_content_str.chars().collect();
        let overlay_text_width = overlay_content_chars.len();

        let overlay_bold_style = Style::default().add_modifier(Modifier::BOLD);

        // Calculate centering position
        let total_width = available_width;
        let text_width = overlay_text_width;
        let overlay_start_col = if text_width >= total_width { 0 } else { total_width.saturating_sub(text_width) / 2 };
        let overlay_end_col = overlay_start_col.saturating_add(text_width); // Use saturating_add

        // Render cell by cell
        let mut overlay_char_idx = 0;
        let y = area.top();

        for col in 0..total_width {
             let x = area.left() + col as u16;
             if col >= bar_chars.len() { continue; } // Should not happen if width > 0

             let bar_char = bar_chars[col];
             let cell_bg_color = match bar_char {
                 '█' => bar_fg_color,
                 _ => Color::White,
             };

             let cell = buf.get_mut(x, y);

             if col >= overlay_start_col && col < overlay_end_col && overlay_char_idx < overlay_content_chars.len() {
                 // Overlay text cell
                 let overlay_char = overlay_content_chars[overlay_char_idx];
                 let final_fg_color = if cell_bg_color == Color::White { Color::Black } else { Color::White };
                 let base_overlay_style = overlay_bold_style;

                 cell.set_char(overlay_char)
                     .set_style(base_overlay_style.fg(final_fg_color).bg(cell_bg_color));
                 overlay_char_idx += 1;

             } else {
                 // Phase bar background cell
                 let final_fg_color = match bar_char {
                     '█' => bar_fg_color,
                     _ => Color::White, // Make space invisible on white background
                 };
                 cell.set_char(bar_char)
                     .set_style(Style::default().fg(final_fg_color).bg(cell_bg_color));
             }
         }
         // Ensure any remaining area has a white background (if area.width was somehow larger)
         for x in (area.left() + total_width as u16)..area.right() {
             buf.get_mut(x, y).set_bg(Color::White);
         }
     }
}

/// Main UI drawing function.
///
/// Called on each tick to render the application frame. It sets up the main layout,
/// renders the top context bar, the bottom phase/tempo bar, the active central component,
/// and handles the flash effect.
pub fn ui(frame: &mut Frame, app: &mut App) {
    check_flash_status(app);

    let top_bar_height = 1;
    let bottom_bar_height = 1;

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_bar_height),
            Constraint::Min(1),
            Constraint::Length(bottom_bar_height),
        ])
        .split(frame.area());

    let top_bar_area = main_layout[0];
    let main_area = main_layout[1];
    let bottom_bar_area = main_layout[2];

    // Render top context bar
    let context_widget = ContextBarWidget {
        mode: app.interface.screen.mode,
        message: &app.interface.components.bottom_message,
    };
    frame.render_widget(context_widget, top_bar_area);

    // Render active component in the main area
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

    // Render bottom phase/tempo bar
    let phase_widget = PhaseTempoBarWidget {
        phase: app.server.link.get_phase(),
        quantum: app.server.link.quantum.max(1.0),
        is_playing: app.server.is_transport_playing,
        tempo: app.server.link.session_state.tempo(),
    };
    frame.render_widget(phase_widget, bottom_bar_area);


    // Render flash effect if active
     if app.interface.screen.flash.is_flashing {
         frame.render_widget(Clear, frame.area());
         frame.render_widget(
             Block::default().style(Style::default().bg(app.interface.screen.flash.flash_color)),
             frame.area(),
         );
     }
}

/// Checks and updates the flash effect status.
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
