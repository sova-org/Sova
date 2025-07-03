use crate::components::grid::rendering::{CellInteraction, CellStyle};
use crate::disk::Theme;
use crate::utils::styles::CommonStyles;
use ratatui::style::Color;

/// Clean, focused color palette for the grid
#[derive(Clone)]
pub struct GridColors {
    pub enabled: Color,
    pub disabled: Color,
    pub playing: Color,
    pub user_cursor: Color,
    pub peer_colors: [Color; 5],
    pub text_dark: Color,
    pub text_light: Color,
}

impl GridColors {
    pub fn for_theme(theme: &Theme) -> Self {
        Self {
            enabled: CommonStyles::accent_cyan_themed(theme)
                .fg
                .unwrap_or(Color::Green),
            disabled: CommonStyles::description_themed(theme)
                .fg
                .unwrap_or(Color::Gray),
            playing: CommonStyles::warning_themed(theme)
                .fg
                .unwrap_or(Color::Yellow),
            user_cursor: CommonStyles::selected_item_themed(theme)
                .bg
                .unwrap_or(Color::White),
            peer_colors: [
                Color::Rgb(99, 179, 237),  // Sky blue
                Color::Rgb(167, 139, 250), // Purple
                Color::Rgb(79, 172, 254),  // Light blue
                Color::Rgb(245, 101, 101), // Light red
                Color::Rgb(251, 191, 36),  // Amber
            ],
            text_dark: Color::Black,
            text_light: Color::White,
        }
    }

    /// Get appropriate text color for background
    pub fn text_for_background(&self, bg: Color) -> Color {
        match bg {
            Color::White | Color::Yellow => self.text_dark,
            Color::Rgb(r, g, b) if (r as u16 + g as u16 + b as u16) > 400 => self.text_dark,
            _ => self.text_light,
        }
    }
}

/// Style resolver with clear precedence
#[derive(Clone)]
pub struct StyleResolver {
    colors: GridColors,
}

impl StyleResolver {
    pub fn for_theme(theme: &Theme) -> Self {
        Self {
            colors: GridColors::for_theme(theme),
        }
    }

    pub fn resolve_style(
        &self,
        is_enabled: bool,
        is_playing: bool,
        interaction: &CellInteraction,
    ) -> CellStyle {
        let background = match interaction {
            CellInteraction::LocalCursor | CellInteraction::LocalSelection => {
                self.colors.user_cursor
            }
            CellInteraction::Peer {
                color_index,
                blink_visible,
                ..
            } => {
                if *blink_visible {
                    self.colors.peer_colors[*color_index % 5]
                } else {
                    self.darken_color(self.colors.peer_colors[*color_index % 5], 0.4)
                }
            }
            CellInteraction::None => {
                if is_playing {
                    self.colors.playing
                } else if is_enabled {
                    self.colors.enabled
                } else {
                    self.colors.disabled
                }
            }
        };

        let text = self.colors.text_for_background(background);

        CellStyle { background, text }
    }

    fn darken_color(&self, color: Color, factor: f32) -> Color {
        match color {
            Color::Rgb(r, g, b) => Color::Rgb(
                (r as f32 * factor) as u8,
                (g as f32 * factor) as u8,
                (b as f32 * factor) as u8,
            ),
            _ => color,
        }
    }
}
