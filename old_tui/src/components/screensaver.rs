//! Screensaver component for SovaTUI
//!
//! Displays a simple message after a period of inactivity.
//! Any key press dismisses the screensaver.

use crate::app::App;
use crate::components::Component;
use crate::components::logs::LogLevel;
use crate::utils::styles::CommonStyles;
use color_eyre::Result as EyreResult;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
};
use std::time::Instant;

/// Enum representing different bitfield patterns for the screensaver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BitfieldPattern {
    AndXorTime,        // (x & y) ^ t
    XorXorTime,        // x ^ y ^ t
    OrXorTime,         // (x | y) ^ t
    AndAddOrTime,      // x.wrapping_add(y).wrapping_add(t_int),
    XorAndOrTime,      // (x ^ y) & (x | y) & t_int,
    XorAddTime,        // x.wrapping_add(y) ^ t_int,
    MulXorTime,        // x.wrapping_mul(y) ^ t_int,
    ShiftXorY,         // (x << (t % 4)) ^ y
    AddSubXor,         // (x + t) ^ (y - t)
    OrAndXor,          // (x | t) ^ (y & t)
    ModXorY,           // (x % (t|1)) ^ y
    MulAddXor,         // x.wrapping_mul(y.wrapping_add(t))
    DiagWave,          // (x + y + t) % M (simple diagonal wave)
    ExpandingSquare,   // (x.max(y) + t) % M (expanding squares)
    XorDistFromCenter, // ((dx*dx + dy*dy).sqrt() ^ t) % M (needs center calc)
    TanXYT,            // ((tan(x*t*0.01) * tan(y*t*0.01)) * 100) % M (needs float calc)
    PatternCount,      // Helper (MUST BE LAST)
}

impl BitfieldPattern {
    // Method to get the next pattern in the cycle
    pub fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = (current_index + 1) % (BitfieldPattern::PatternCount as usize);
        // This conversion is safe as long as PatternCount is the last variant
        // and the enum variants are contiguous from 0.
        unsafe { std::mem::transmute(next_index as u8) }
    }

    // Default pattern to start with
    pub fn default_pattern() -> Self {
        BitfieldPattern::AndXorTime
    }
}

/// Component responsible for displaying the screensaver.
pub struct ScreensaverComponent;

impl Default for ScreensaverComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreensaverComponent {
    /// Creates a new `ScreensaverComponent`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for ScreensaverComponent {
    /// Handles key events for the Screensaver component.
    /// Any key press exits the screensaver mode and returns to the previous mode.
    fn handle_key_event(&self, app: &mut App, _key_event: KeyEvent) -> EyreResult<bool> {
        app.interface.screen.mode = app.interface.screen.previous_mode; // Restore previous mode
        app.last_interaction_time = Instant::now(); // Reset inactivity timer
        app.add_log(LogLevel::Info, "Screensaver dismissed.".to_string());
        Ok(true) // Consume the key event
    }

    /// Draws the Screensaver component.
    /// Displays a dynamic pattern synchronized to the musical tempo and phase.
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let buf = frame.buffer_mut();

        let current_pattern = app.interface.components.screensaver_pattern;
        let start_time = app.interface.components.screensaver_start_time;
        let tempo = app.server.link.session_state.tempo();

        // Use elapsed time for animation but scale by tempo for musical sync
        let elapsed = start_time.elapsed().as_secs_f32();
        let tempo_scale = (tempo as f32 / 120.0).max(0.3).min(3.0); // Scale relative to 120 BPM
        let t_int = (elapsed * 5.0 * tempo_scale) as u16;

        // Define characters and colors for mapping
        // Using block characters for density
        let characters = [' ', '░', '▒', '▓', '█'];
        // Use theme-aware grayscale colors
        let colors = get_screensaver_colors(&app.client_config.theme);
        let num_levels = characters.len().min(colors.len());

        // Calculate center coordinates of the drawing area
        let center_x = area.x + area.width / 2;
        let center_y = area.y + area.height / 2;

        for row in area.top()..area.bottom() {
            for col in area.left()..area.right() {
                let x = col;
                let y = row;

                // Calculate value using the helper function and current pattern from App state
                let formula_val = calculate_screensaver_value(
                    current_pattern,
                    x,
                    y,
                    t_int,
                    center_x,
                    center_y,
                    num_levels as u16, // Use num_levels for the final modulo
                );

                // Map the value to a character and color index (already done by calculate_screensaver_value)
                let level_index = formula_val as usize;

                // Bounds check for safety, although modulo should handle it
                if level_index < num_levels {
                    let char_to_draw = characters[level_index];
                    let cell_color = colors[level_index];
                    let cell_style = Style::default().fg(cell_color);

                    // Get mutable reference to the cell and set properties
                    let position: ratatui::layout::Position = (col, row).into();
                    if let Some(cell) = buf.cell_mut(position) {
                        cell.set_char(char_to_draw);
                        cell.set_style(cell_style);
                    }
                } else {
                    // Fallback for safety, should not happen with modulo
                    let position: ratatui::layout::Position = (col, row).into();
                    if let Some(cell) = buf.cell_mut(position) {
                        cell.set_char('?');
                        cell.set_style(CommonStyles::error_themed(&app.client_config.theme));
                    }
                }
            }
        }
    }
}

fn calculate_screensaver_value(
    pattern: BitfieldPattern,
    x: u16,
    y: u16,
    t_int: u16,
    center_x: u16,
    center_y: u16,
    max_val: u16,
) -> u16 {
    // Calculate the raw value based on the pattern
    let raw_val = match pattern {
        BitfieldPattern::AndXorTime => (x & y) ^ t_int,
        BitfieldPattern::XorXorTime => x ^ y ^ t_int,
        BitfieldPattern::OrXorTime => (x | y) ^ t_int,
        BitfieldPattern::AndAddOrTime => x.wrapping_add(y).wrapping_add(t_int),
        BitfieldPattern::XorAndOrTime => (x ^ y) & (x | y) & t_int,
        BitfieldPattern::XorAddTime => x.wrapping_add(y) ^ t_int,
        BitfieldPattern::MulXorTime => x.wrapping_mul(y) ^ t_int,
        BitfieldPattern::ShiftXorY => (x << (t_int % 4)) ^ y,
        BitfieldPattern::AddSubXor => (x.wrapping_add(t_int)) ^ (y.wrapping_sub(t_int)),
        BitfieldPattern::OrAndXor => (x | t_int) ^ (y & t_int),
        BitfieldPattern::ModXorY => (x % (t_int.max(1))) ^ y,
        BitfieldPattern::MulAddXor => x.wrapping_mul(y.wrapping_add(t_int)),
        BitfieldPattern::DiagWave => x.wrapping_add(y).wrapping_add(t_int),
        BitfieldPattern::ExpandingSquare => x.max(y).wrapping_add(t_int),
        BitfieldPattern::XorDistFromCenter => {
            let dx = x.abs_diff(center_x);
            let dy = y.abs_diff(center_y);
            // Use f32 for sqrt, then cast back
            let dist = ((dx as f32 * dx as f32 + dy as f32 * dy as f32).sqrt()) as u16;
            dist ^ t_int
        }
        BitfieldPattern::TanXYT => {
            let tan_x = (x as f32 * t_int as f32 * 0.01).tan();
            let tan_y = (y as f32 * t_int as f32 * 0.01).tan();
            // Multiply, scale, ensure it's positive before casting
            ((tan_x * tan_y * 100.0).abs()) as u16
        }
        BitfieldPattern::PatternCount => 0, // Default/fallback
    };

    // Apply final modulo if max_val is valid
    if max_val > 0 {
        raw_val % max_val
    } else {
        raw_val
    }
}

/// Get theme-appropriate colors for the screensaver patterns
/// Creates a gradient using CommonStyles theme colors for consistency
fn get_screensaver_colors(theme: &crate::disk::Theme) -> [Color; 5] {
    use crate::utils::styles::CommonStyles;

    // Create a gradient from dark to light using theme colors
    let dark = CommonStyles::description_themed(theme)
        .fg
        .unwrap_or(Color::Gray);
    let medium = CommonStyles::default_text_themed(theme)
        .fg
        .unwrap_or(Color::White);
    let accent1 = CommonStyles::accent_cyan_themed(theme)
        .fg
        .unwrap_or(Color::Cyan);
    let accent2 = CommonStyles::accent_magenta_themed(theme)
        .fg
        .unwrap_or(Color::Magenta);
    let bright = CommonStyles::value_text_themed(theme)
        .fg
        .unwrap_or(Color::Green);

    [dark, medium, accent1, accent2, bright]
}
