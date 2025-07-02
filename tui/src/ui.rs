use crate::app::{App, Mode};
use crate::components::Component;
use corelib::lang::variable::VariableValue;
use crate::components::devices::DevicesComponent;
use crate::components::editor::EditorComponent;
use crate::components::grid::GridComponent;
use crate::components::help::HelpComponent;
use crate::components::logs::LogsComponent;
use crate::components::options::OptionsComponent;
use crate::components::saveload::SaveLoadComponent;
use crate::components::screensaver::ScreensaverComponent;
use crate::components::splash::SplashComponent;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Widget},
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
    app: &'a App,
}

impl<'a> Widget for ContextBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme_colors = get_ui_theme_colors(&self.app.client_config.theme);
        let base_style = Style::default()
            .bg(theme_colors.context_bar_bg)
            .fg(theme_colors.context_bar_fg);
        buf.set_style(area, base_style);

        let mode_style = Style::default()
            .fg(theme_colors.mode_text_fg)
            .bg(theme_colors.mode_text_bg)
            .add_modifier(Modifier::BOLD);

        let mode_width_guess: u16 = 16;

        // Ensure constraints don't exceed area width
        let available_width = area.width;
        let actual_mode_width = mode_width_guess.min(available_width);
        let message_min_width: u16 = 1;

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
        let mut mode_text_inner = match self.mode {
            Mode::Editor => "EDITOR",
            Mode::Grid => "GRID",
            Mode::Options => "OPTIONS",
            Mode::Splash => "WELCOME",
            Mode::Help => "HELP",
            Mode::Devices => "DEVICES",
            Mode::Logs => "LOGS",
            Mode::SaveLoad => "FILES",
            Mode::Screensaver => "SLEEPING",
        }
        .to_string();

        // Add Vim mode if applicable
        if self.mode == Mode::Editor
            && self.app.client_config.editing_mode == crate::disk::EditingMode::Vim
        {
            mode_text_inner = format!(
                "{} ({}) ",
                mode_text_inner,
                self.app.editor.vim_state.mode_display_string()
            );
        }

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
                    let mut truncated: String = self
                        .message
                        .chars()
                        .take(max_message_width.saturating_sub(3))
                        .collect();
                    truncated.push_str("...");
                    truncated
                } else {
                    self.message
                        .chars()
                        .next()
                        .map_or(String::new(), |c| c.to_string())
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
                Style::default().fg(theme_colors.context_bar_fg),
            );
        }
    }
}

/// Widget to display global variables (A-Z).
/// Renders the single-letter global variables in a compact format.
struct GlobalVariablesWidget<'a> {
    variables: &'a std::collections::HashMap<String, VariableValue>,
}

impl<'a> Widget for GlobalVariablesWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 4 || area.width < 8 {
            return;
        }
        
        // Create thick border block
        let border_style = Style::default()
            .fg(Color::White)
            .bg(Color::Reset); // Transparent background
            
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(border_style);
        
        // Calculate inner area before rendering
        let inner_area = block.inner(area);
        
        // Render the block
        block.render(area, buf);
        
        if inner_area.width < 8 || inner_area.height < 1 {
            return;
        }
        
        // Only show the 8 existing global variables
        let existing_vars = ['A', 'B', 'C', 'D', 'W', 'X', 'Y', 'Z'];
        
        // Calculate box width for 8 equal sections  
        let box_width = if inner_area.width >= 8 {
            inner_area.width / 8
        } else {
            1
        };
        
        // Styles for variable names (bold) and values (normal)
        let var_name_style = Style::default()
            .fg(Color::White)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD);
        let var_value_style = Style::default()
            .fg(Color::White)
            .bg(Color::Reset);
        
        // Render each variable in its own box
        for (i, &letter) in existing_vars.iter().enumerate() {
            let box_x = inner_area.x + (i as u16 * box_width);
            let box_y = inner_area.y;
            
            // Skip if box would extend beyond area
            if box_x >= inner_area.x + inner_area.width {
                break;
            }
            
            // Variable name (centered on first line)
            let var_name = letter.to_string();
            let name_x = box_x + (box_width.saturating_sub(1)) / 2;
            if name_x < inner_area.x + inner_area.width && box_y < inner_area.y + inner_area.height {
                buf.cell_mut(Position::new(name_x, box_y)).unwrap().set_char(letter).set_style(var_name_style);
            }
            
            // Variable value (centered on second line if we have space)
            if inner_area.height >= 2 && box_y + 1 < inner_area.y + inner_area.height {
                let value_str = if let Some(value) = self.variables.get(&var_name) {
                    format_variable_value(value)
                } else {
                    "nil".to_string()
                };
                
                // Truncate value if too long for box
                let display_value = if value_str.len() > box_width as usize {
                    if box_width >= 3 {
                        format!("{}…", &value_str[..box_width.saturating_sub(1) as usize])
                    } else {
                        value_str.chars().take(box_width as usize).collect()
                    }
                } else {
                    value_str
                };
                
                // Center the value text
                let value_len = display_value.chars().count() as u16;
                let value_start_x = box_x + (box_width.saturating_sub(value_len)) / 2;
                
                // Render each character of the value
                for (char_idx, ch) in display_value.chars().enumerate() {
                    let char_x = value_start_x + char_idx as u16;
                    if char_x < inner_area.x + inner_area.width {
                        buf.cell_mut(Position::new(char_x, box_y + 1)).unwrap().set_char(ch).set_style(var_value_style);
                    }
                }
            } else if inner_area.height >= 1 {
                // If only one line available, show just the variable name
                // (already handled above)
            }
        }
    }
}

/// Formats a VariableValue for compact display
fn format_variable_value(value: &VariableValue) -> String {
    match value {
        VariableValue::Integer(i) => i.to_string(),
        VariableValue::Float(f) => format!("{:.2}", f),
        VariableValue::Bool(b) => if *b { "T".to_string() } else { "F".to_string() },
        VariableValue::Str(s) => {
            if s.len() > 8 {
                format!("\"{}...\"", &s[..5])
            } else {
                format!("\"{}\"", s)
            }
        },
        VariableValue::Decimal(sign, num, den) => {
            let val = (*num as f64) / (*den as f64) * (*sign as f64);
            format!("{:.2}", val)
        },
        VariableValue::Dur(_) => "dur".to_string(),
        VariableValue::Func(_) => "fn".to_string(),
        VariableValue::Map(_) => "map".to_string(),
    }
}

/// Widget to display the bottom phase/tempo bar.
/// Renders a full-width phase bar with centered, overlaid status and tempo text.
/// Uses direct buffer manipulation for precise cell control and dynamic background colors.
struct PhaseTempoBarWidget<'a> {
    phase: f64,
    quantum: f64,
    is_playing: bool,
    tempo: f64,
    app: &'a App,
}

impl<'a> Widget for PhaseTempoBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let available_width = area.width as usize;
        if available_width == 0 {
            return;
        }

        let theme_colors = get_ui_theme_colors(&self.app.client_config.theme);

        // Calculate phase bar state
        let bar_fg_color = if self.is_playing {
            theme_colors.tempo_bar_playing
        } else {
            theme_colors.tempo_bar_stopped
        };
        let filled_ratio = if self.quantum > 0.0 {
            (self.phase / self.quantum).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let filled_width = (filled_ratio * available_width as f64).round() as usize;

        let mut bar_chars: Vec<char> = Vec::with_capacity(available_width);
        for i in 0..available_width {
            bar_chars.push(if i < filled_width { '█' } else { ' ' });
        }

        // Prepare overlay text content and style
        let status_symbol = if self.is_playing { '▶' } else { '■' };
        let tempo_text = format!("{:.1} BPM", self.tempo);
        let separator = " | ";
        let overlay_content_str = format!("{}{}{}", status_symbol, separator, tempo_text);
        let overlay_content_chars: Vec<char> = overlay_content_str.chars().collect();
        let overlay_text_width = overlay_content_chars.len();

        let overlay_bold_style = Style::default().add_modifier(Modifier::BOLD);

        // Calculate centering position
        let total_width = available_width;
        let text_width = overlay_text_width;
        let overlay_start_col = if text_width >= total_width {
            0
        } else {
            total_width.saturating_sub(text_width) / 2
        };
        let overlay_end_col = overlay_start_col.saturating_add(text_width); // Use saturating_add

        // Render cell by cell
        let mut overlay_char_idx = 0;
        let y = area.top();

        for col in 0..total_width {
            let x = area.left() + col as u16;
            if col >= bar_chars.len() {
                continue;
            } // Should not happen if width > 0

            let bar_char = bar_chars[col];
            let cell_bg_color = match bar_char {
                '█' => bar_fg_color,
                _ => theme_colors.tempo_bar_bg,
            };

            let pos: Position = (x, y).into();
            let cell = buf.cell_mut(pos).unwrap();

            if col >= overlay_start_col
                && col < overlay_end_col
                && overlay_char_idx < overlay_content_chars.len()
            {
                // Overlay text cell
                let overlay_char = overlay_content_chars[overlay_char_idx];
                let final_fg_color = if cell_bg_color == theme_colors.tempo_bar_bg {
                    theme_colors.tempo_bar_text
                } else {
                    theme_colors.tempo_bar_text_on_bar
                };
                let base_overlay_style = overlay_bold_style;

                cell.set_char(overlay_char)
                    .set_style(base_overlay_style.fg(final_fg_color).bg(cell_bg_color));
                overlay_char_idx += 1;
            } else {
                // Phase bar background cell
                let final_fg_color = match bar_char {
                    '█' => bar_fg_color,
                    _ => theme_colors.tempo_bar_bg, // Make space invisible on background
                };
                cell.set_char(bar_char)
                    .set_style(Style::default().fg(final_fg_color).bg(cell_bg_color));
            }
        }
        // Ensure any remaining area has themed background (if area.width was somehow larger)
        for x in (area.left() + total_width as u16)..area.right() {
            let pos: Position = (x, y).into();
            buf.cell_mut(pos).unwrap().set_bg(theme_colors.tempo_bar_bg);
        }
    }
}

/// Main UI drawing function.
///
/// Called on each tick to render the application frame. It sets up the main layout,
/// renders the top context bar, the bottom phase/tempo bar, the active central component,
/// and any overlays like the command palette.
///
/// # Arguments
///
/// * `frame` - Mutable reference to the terminal frame.
/// * `app` - Reference to the main application state.
pub fn ui(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // --- Render differently based on Screensaver mode ---
    if app.interface.screen.mode == Mode::Screensaver {
        // --- Screensaver Mode: Render only the screensaver component fullscreen ---
        ScreensaverComponent::new().draw(app, frame, area);
    } else {
        // --- Normal Mode: Render with Top Bar, Central Area, Bottom Bar ---
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Top bar
                Constraint::Min(0),    // Central area
                Constraint::Length(1), // Bottom bar
            ])
            .split(area);

        let top_bar_area = chunks[0];
        let central_area = chunks[1];
        let bottom_bar_area = chunks[2];

        // Render Top Context Bar
        let top_widget = ContextBarWidget {
            mode: app.interface.screen.mode,
            message: &app.interface.components.bottom_message,
            app,
        };
        frame.render_widget(top_widget, top_bar_area);

        // Render Central Component based on Mode
        match app.interface.screen.mode {
            Mode::Editor => EditorComponent::new().draw(app, frame, central_area),
            Mode::Grid => GridComponent::new().draw(app, frame, central_area),
            Mode::Options => OptionsComponent::new().draw(app, frame, central_area),
            Mode::Splash => SplashComponent::new().draw(app, frame, central_area),
            Mode::Help => HelpComponent::new().draw(app, frame, central_area),
            Mode::Devices => DevicesComponent::new().draw(app, frame, central_area),
            Mode::Logs => LogsComponent::new().draw(app, frame, central_area),
            Mode::SaveLoad => SaveLoadComponent::new().draw(app, frame, central_area),
            Mode::Screensaver => {} // Should not be reached due to the outer if, but needed for exhaustiveness
        }

        // Render Bottom Phase/Tempo Bar
        let bottom_widget = PhaseTempoBarWidget {
            phase: app.server.link.get_phase(),
            quantum: app.server.link.quantum,
            is_playing: app.server.is_transport_playing,
            tempo: app.server.link.session_state.tempo(),
            app,
        };
        frame.render_widget(bottom_widget, bottom_bar_area);

        // --- Render Overlays (Flash, Command Palette) --- Only in Normal Mode? Usually yes.
        // Render Flash Effect (Overlay)
        if app.interface.screen.flash.is_flashing {
            if let Some(start) = app.interface.screen.flash.flash_start {
                if start.elapsed() < app.interface.screen.flash.flash_duration {
                    frame.render_widget(
                        Block::default()
                            .style(Style::default().bg(app.interface.screen.flash.flash_color)),
                        area, // Flash the entire screen (or just central_area?)
                    );
                } else {
                    // Reset flash state after duration
                    app.interface.screen.flash.is_flashing = false;
                    app.interface.screen.flash.flash_start = None;
                }
            } else {
                // Should not happen, but reset if start time is missing
                app.interface.screen.flash.is_flashing = false;
            }
        }

        // Render Command Palette (Overlay)
        app.interface.components.command_palette.draw(app, frame);
    }
}

/// Renders the global variables bar in the provided area
pub fn render_global_variables_bar(app: &App, frame: &mut Frame, area: Rect) {
    let widget = GlobalVariablesWidget {
        variables: &app.server.global_variables,
    };
    frame.render_widget(widget, area);
}

/// Theme colors for UI elements
struct UiThemeColors {
    context_bar_bg: Color,
    context_bar_fg: Color,
    mode_text_bg: Color,
    mode_text_fg: Color,
    tempo_bar_bg: Color,
    tempo_bar_playing: Color,
    tempo_bar_stopped: Color,
    tempo_bar_text: Color,
    tempo_bar_text_on_bar: Color,
}

/// Get theme-appropriate colors for UI elements
fn get_ui_theme_colors(theme: &crate::disk::Theme) -> UiThemeColors {
    use crate::disk::Theme;

    match theme {
        Theme::Classic => UiThemeColors {
            context_bar_bg: Color::White,
            context_bar_fg: Color::Black,
            mode_text_bg: Color::Blue,
            mode_text_fg: Color::White,
            tempo_bar_bg: Color::White,
            tempo_bar_playing: Color::Green,
            tempo_bar_stopped: Color::Red,
            tempo_bar_text: Color::Black,
            tempo_bar_text_on_bar: Color::White,
        },
        Theme::Ocean => UiThemeColors {
            context_bar_bg: Color::Rgb(240, 248, 255),  // Alice blue
            context_bar_fg: Color::Rgb(25, 25, 112),    // Midnight blue
            mode_text_bg: Color::Rgb(0, 100, 148),      // Dark cerulean
            mode_text_fg: Color::Rgb(240, 248, 255),    // Alice blue
            tempo_bar_bg: Color::Rgb(240, 248, 255),    // Alice blue
            tempo_bar_playing: Color::Rgb(46, 139, 87), // Sea green
            tempo_bar_stopped: Color::Rgb(220, 20, 60), // Crimson
            tempo_bar_text: Color::Rgb(25, 25, 112),    // Midnight blue
            tempo_bar_text_on_bar: Color::Rgb(240, 248, 255), // Alice blue
        },
        Theme::Forest => UiThemeColors {
            context_bar_bg: Color::Rgb(245, 245, 220),        // Beige
            context_bar_fg: Color::Rgb(34, 139, 34),          // Forest green
            mode_text_bg: Color::Rgb(46, 125, 50),            // Dark green
            mode_text_fg: Color::Rgb(245, 245, 220),          // Beige
            tempo_bar_bg: Color::Rgb(245, 245, 220),          // Beige
            tempo_bar_playing: Color::Rgb(34, 139, 34),       // Forest green
            tempo_bar_stopped: Color::Rgb(178, 34, 34),       // Fire brick
            tempo_bar_text: Color::Rgb(34, 139, 34),          // Forest green
            tempo_bar_text_on_bar: Color::Rgb(245, 245, 220), // Beige
        },
        Theme::Monochrome => UiThemeColors {
            context_bar_bg: Color::White,                     // White background
            context_bar_fg: Color::Black,                     // Black text
            mode_text_bg: Color::Black,                       // Black mode background
            mode_text_fg: Color::White,                       // White mode text
            tempo_bar_bg: Color::White,                       // White tempo bar
            tempo_bar_playing: Color::Black,                  // Black when playing
            tempo_bar_stopped: Color::DarkGray,               // Dark gray when stopped
            tempo_bar_text: Color::Black,                     // Black tempo text
            tempo_bar_text_on_bar: Color::White,              // White text on bar
        },
        Theme::Green => UiThemeColors {
            context_bar_bg: Color::Black,                     // Black background (matrix style)
            context_bar_fg: Color::Rgb(0, 255, 0),            // Bright green text
            mode_text_bg: Color::Rgb(0, 100, 0),              // Dark green mode background
            mode_text_fg: Color::Rgb(0, 255, 0),              // Bright green mode text
            tempo_bar_bg: Color::Black,                       // Black tempo bar background
            tempo_bar_playing: Color::Rgb(0, 255, 0),         // Bright green when playing
            tempo_bar_stopped: Color::Rgb(0, 128, 0),         // Medium green when stopped
            tempo_bar_text: Color::Rgb(0, 255, 0),            // Bright green tempo text
            tempo_bar_text_on_bar: Color::Black,              // Black text on green bar
        },
    }
}
