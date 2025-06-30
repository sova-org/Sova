//! Common styles for TUI components.
//!
//! This module provides centralized style definitions to ensure consistency
//! across all UI components and make styling changes easier to manage.

use crate::disk::Theme;
use ratatui::style::{Color, Modifier, Style};

/// Collection of commonly used styles throughout the TUI application.
///
/// This struct provides static methods for creating consistent styles
/// that are used across multiple components. Centralizing these styles
/// makes it easy to maintain visual consistency and apply theme changes.
pub struct CommonStyles;

/// Color scheme definition for different themes
struct ColorScheme {
    // Text colors
    text_primary: Color,
    text_key_binding: Color,
    text_description: Color,
    text_value: Color,
    
    // State colors
    warning: Color,
    error: Color,
    success: Color,
    
    // Background colors
    selected_bg: Color,
    highlight_bg: Color,
    
    // Accent colors
    accent_primary: Color,
    accent_secondary: Color,
}

impl ColorScheme {
    fn classic() -> Self {
        Self {
            text_primary: Color::White,
            text_key_binding: Color::Gray,
            text_description: Color::DarkGray,
            text_value: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            success: Color::Green,
            selected_bg: Color::DarkGray,
            highlight_bg: Color::Blue,
            accent_primary: Color::Cyan,
            accent_secondary: Color::Magenta,
        }
    }
    
    fn ocean() -> Self {
        Self {
            text_primary: Color::Rgb(240, 248, 255), // Alice blue
            text_key_binding: Color::Rgb(70, 130, 180), // Steel blue
            text_description: Color::Rgb(119, 136, 153), // Light slate gray
            text_value: Color::Rgb(72, 209, 204), // Medium turquoise
            warning: Color::Rgb(255, 215, 0), // Gold
            error: Color::Rgb(220, 20, 60), // Crimson
            success: Color::Rgb(46, 139, 87), // Sea green
            selected_bg: Color::Rgb(25, 25, 112), // Midnight blue
            highlight_bg: Color::Rgb(0, 100, 148), // Dark cerulean
            accent_primary: Color::Rgb(0, 191, 255), // Deep sky blue
            accent_secondary: Color::Rgb(138, 43, 226), // Blue violet
        }
    }
    
    fn forest() -> Self {
        Self {
            text_primary: Color::Rgb(245, 245, 220), // Beige
            text_key_binding: Color::Rgb(107, 142, 35), // Olive drab
            text_description: Color::Rgb(128, 128, 128), // Gray
            text_value: Color::Rgb(50, 205, 50), // Lime green
            warning: Color::Rgb(255, 140, 0), // Dark orange
            error: Color::Rgb(178, 34, 34), // Fire brick
            success: Color::Rgb(34, 139, 34), // Forest green
            selected_bg: Color::Rgb(85, 107, 47), // Dark olive green
            highlight_bg: Color::Rgb(46, 125, 50), // Dark green
            accent_primary: Color::Rgb(154, 205, 50), // Yellow green
            accent_secondary: Color::Rgb(147, 112, 219), // Medium purple
        }
    }
    
    fn for_theme(theme: &Theme) -> Self {
        match theme {
            Theme::Classic => Self::classic(),
            Theme::Ocean => Self::ocean(),
            Theme::Forest => Self::forest(),
        }
    }
}

impl CommonStyles {
    /// Style for key binding text (e.g., "Ctrl+S", "Enter", "Esc").
    /// 
    /// Uses theme-appropriate color with bold modifier, used in help text and status indicators.
    pub fn key_binding() -> Style {
        let scheme = ColorScheme::classic(); // Default to classic for compatibility
        Style::default().fg(scheme.text_key_binding).add_modifier(Modifier::BOLD)
    }

    /// Style for active values, important text, or positive indicators.
    /// 
    /// Uses theme-appropriate color with bold modifier, used for enabled states, active values,
    /// and positive status indicators.
    pub fn value_text() -> Style {
        let scheme = ColorScheme::classic();
        Style::default().fg(scheme.text_value).add_modifier(Modifier::BOLD)
    }

    /// Style for selected items in lists and menus.
    /// 
    /// Bold text with theme-appropriate background, used for highlighting the
    /// currently selected item in lists, menus, and option sets.
    pub fn selected_item() -> Style {
        let scheme = ColorScheme::classic();
        Style::default().add_modifier(Modifier::BOLD).bg(scheme.selected_bg)
    }

    /// Default text style for most UI content.
    /// 
    /// Uses theme-appropriate primary text color, used as the standard foreground color for most text
    /// elements throughout the interface.
    pub fn default_text() -> Style {
        let scheme = ColorScheme::classic();
        Style::default().fg(scheme.text_primary)
    }

    /// Style for warnings, input prompts, and attention-drawing elements.
    /// 
    /// Uses theme-appropriate warning color, used for input borders, warning messages, and elements
    /// that need to draw user attention without indicating an error.
    pub fn warning() -> Style {
        let scheme = ColorScheme::classic();
        Style::default().fg(scheme.warning)
    }

    /// Style for error messages and critical indicators.
    /// 
    /// Uses theme-appropriate error color, used for error states, failure indicators, and critical
    /// information that requires immediate attention.
    pub fn error() -> Style {
        let scheme = ColorScheme::classic();
        Style::default().fg(scheme.error)
    }

    /// Style for help text, descriptions, and secondary information.
    /// 
    /// Uses theme-appropriate description color, used for less prominent information like help text,
    /// descriptions, and supplementary details.
    pub fn description() -> Style {
        let scheme = ColorScheme::classic();
        Style::default().fg(scheme.text_description)
    }

    // Theme-aware versions of the above methods
    
    /// Style for key binding text with specific theme.
    pub fn key_binding_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.text_key_binding).add_modifier(Modifier::BOLD)
    }

    /// Style for active values with specific theme.
    pub fn value_text_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.text_value).add_modifier(Modifier::BOLD)
    }

    /// Style for selected items with specific theme.
    pub fn selected_item_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().add_modifier(Modifier::BOLD).bg(scheme.selected_bg)
    }

    /// Default text style with specific theme.
    pub fn default_text_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.text_primary)
    }

    /// Warning style with specific theme.
    pub fn warning_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.warning)
    }

    /// Error style with specific theme.
    pub fn error_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.error)
    }

    /// Description style with specific theme.
    pub fn description_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.text_description)
    }

    /// Style for section headers and emphasized text.
    /// 
    /// Uses theme-appropriate primary text color with bold modifier, used for section headers, titles,
    /// and text that should stand out without using color.
    pub fn header() -> Style {
        let scheme = ColorScheme::classic();
        Style::default().fg(scheme.text_primary).add_modifier(Modifier::BOLD)
    }

    /// Style for highlighted backgrounds (e.g., selected rows, focused areas).
    /// 
    /// Uses theme-appropriate highlight background with primary text, used for highlighting focused
    /// or selected areas that need strong visual emphasis.
    pub fn highlight_background() -> Style {
        let scheme = ColorScheme::classic();
        Style::default().bg(scheme.highlight_bg).fg(scheme.text_primary)
    }

    /// Style for accent elements and special indicators.
    /// 
    /// Uses theme-appropriate primary accent color, used for special indicators, accent elements, and
    /// decorative text that should stand out with a distinct color.
    pub fn accent_cyan() -> Style {
        let scheme = ColorScheme::classic();
        Style::default().fg(scheme.accent_primary)
    }

    /// Style for secondary accent elements.
    /// 
    /// Uses theme-appropriate secondary accent color, used for alternative accent elements and special
    /// indicators that need a different color from the primary accent.
    pub fn accent_magenta() -> Style {
        let scheme = ColorScheme::classic();
        Style::default().fg(scheme.accent_secondary)
    }

    /// Style for secondary information with background.
    /// 
    /// Uses theme-appropriate colors, used for duration indicators,
    /// secondary status information, and content that needs subtle emphasis.
    pub fn secondary_info() -> Style {
        let scheme = ColorScheme::classic();
        Style::default().fg(scheme.text_primary).bg(scheme.selected_bg)
    }

    // Additional themed versions for editor.rs
    
    /// Style for magenta accent elements with specific theme.
    pub fn accent_magenta_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.accent_secondary)
    }
    
    /// Style for cyan accent elements with specific theme.
    pub fn accent_cyan_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.accent_primary)
    }
    
    /// Style for section headers with specific theme.
    pub fn header_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.text_primary).add_modifier(Modifier::BOLD)
    }
    
    /// Style for highlighted backgrounds with specific theme.
    pub fn highlight_background_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().bg(scheme.highlight_bg).fg(scheme.text_primary)
    }
}