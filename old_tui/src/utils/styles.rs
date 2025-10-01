//! Common styles for TUI components.
//!
//! This module provides centralized style definitions to ensure consistency
//! across all UI components and make styling changes easier to manage.
//!
//! The theming system supports 5 themes: Classic, Ocean, Forest, Monochrome, and Green.
//! Each theme defines a coordinated color palette that extends from UI elements
//! to syntax highlighting in the editor.

use crate::disk::Theme;
use ratatui::style::{Color, Modifier, Style};

/// Collection of commonly used styles throughout the TUI application.
///
/// This struct provides static methods that return consistent styles
/// used across multiple components. All styling should go through these
/// themed methods to ensure visual consistency and easy theme switching.
pub struct CommonStyles;

/// Color scheme definition for different themes.
///
/// Each theme provides a cohesive set of colors for text, state indicators,
/// backgrounds, and accents. This ensures visual consistency across all
/// UI components while allowing for distinct theme personalities.
struct ColorScheme {
    text_primary: Color,
    text_key_binding: Color,
    text_description: Color,
    text_value: Color,
    warning: Color,
    error: Color,
    success: Color,
    selected_bg: Color,
    highlight_bg: Color,
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
            text_primary: Color::Rgb(240, 248, 255),
            text_key_binding: Color::Rgb(70, 130, 180),
            text_description: Color::Rgb(119, 136, 153),
            text_value: Color::Rgb(72, 209, 204),
            warning: Color::Rgb(255, 215, 0),
            error: Color::Rgb(220, 20, 60),
            success: Color::Rgb(46, 139, 87),
            selected_bg: Color::Rgb(25, 25, 112),
            highlight_bg: Color::Rgb(0, 100, 148),
            accent_primary: Color::Rgb(0, 191, 255),
            accent_secondary: Color::Rgb(138, 43, 226),
        }
    }

    fn forest() -> Self {
        Self {
            text_primary: Color::Rgb(245, 245, 220),
            text_key_binding: Color::Rgb(107, 142, 35),
            text_description: Color::Rgb(128, 128, 128),
            text_value: Color::Rgb(50, 205, 50),
            warning: Color::Rgb(255, 140, 0),
            error: Color::Rgb(178, 34, 34),
            success: Color::Rgb(34, 139, 34),
            selected_bg: Color::Rgb(85, 107, 47),
            highlight_bg: Color::Rgb(46, 125, 50),
            accent_primary: Color::Rgb(154, 205, 50),
            accent_secondary: Color::Rgb(147, 112, 219),
        }
    }

    fn monochrome() -> Self {
        Self {
            text_primary: Color::White,
            text_key_binding: Color::Gray,
            text_description: Color::DarkGray,
            text_value: Color::White,
            warning: Color::White,
            error: Color::White,
            success: Color::White,
            selected_bg: Color::Gray,
            highlight_bg: Color::DarkGray,
            accent_primary: Color::White,
            accent_secondary: Color::Gray,
        }
    }

    fn green() -> Self {
        Self {
            text_primary: Color::Rgb(0, 255, 0),
            text_key_binding: Color::Rgb(0, 200, 0),
            text_description: Color::Rgb(0, 128, 0),
            text_value: Color::Rgb(0, 255, 100),
            warning: Color::Rgb(0, 200, 200),
            error: Color::Rgb(255, 255, 255),
            success: Color::Rgb(0, 255, 0),
            selected_bg: Color::Rgb(0, 100, 0),
            highlight_bg: Color::Rgb(0, 150, 0),
            accent_primary: Color::Rgb(0, 255, 150),
            accent_secondary: Color::Rgb(150, 255, 0),
        }
    }

    fn for_theme(theme: &Theme) -> Self {
        match theme {
            Theme::Classic => Self::classic(),
            Theme::Ocean => Self::ocean(),
            Theme::Forest => Self::forest(),
            Theme::Monochrome => Self::monochrome(),
            Theme::Green => Self::green(),
        }
    }
}

impl CommonStyles {
    /// Style for help text, descriptions, and secondary information.
    ///
    /// Uses classic theme description color, used for less prominent information.
    /// Note: Prefer description_themed() for better theme consistency.
    pub fn description() -> Style {
        let scheme = ColorScheme::classic();
        Style::default().fg(scheme.text_description)
    }

    /// Default text style for most UI content.
    ///
    /// Uses classic theme primary text color.
    /// Note: Prefer default_text_themed() for better theme consistency.
    pub fn default_text() -> Style {
        let scheme = ColorScheme::classic();
        Style::default().fg(scheme.text_primary)
    }

    // Theme-aware versions of the above methods

    /// Style for key binding text with specific theme.
    pub fn key_binding_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default()
            .fg(scheme.text_key_binding)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for active values with specific theme.
    pub fn value_text_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default()
            .fg(scheme.text_value)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for selected items with specific theme.
    pub fn selected_item_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);

        // Special case for monochrome: use black text on gray background for visibility
        let text_color = match theme {
            Theme::Monochrome => Color::Black,
            _ => scheme.text_primary,
        };

        Style::default()
            .fg(text_color)
            .bg(scheme.selected_bg)
            .add_modifier(Modifier::BOLD)
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
        Style::default()
            .fg(scheme.text_primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for highlighted backgrounds with specific theme.
    pub fn highlight_background_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default()
            .bg(scheme.highlight_bg)
            .fg(scheme.text_primary)
    }

    // File browser styles - subtle emphasis for file types

    /// Style for file icons and directories.
    pub fn file_directory_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default()
            .fg(scheme.accent_primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for selected files in browser.
    pub fn file_selected_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);

        // Special case for monochrome: use black text on gray background for visibility
        let text_color = match theme {
            Theme::Monochrome => Color::Black,
            _ => scheme.text_primary,
        };

        Style::default()
            .fg(text_color)
            .bg(scheme.selected_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for file status indicators.
    pub fn file_status_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.text_description)
    }

    // Boolean state styles - subtle green/red for true/false

    /// Style for true/enabled boolean states.
    pub fn boolean_true_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.success)
    }

    /// Style for false/disabled boolean states.
    pub fn boolean_false_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.text_description)
    }

    // Selection and highlighting - subtle emphasis without jarring backgrounds

    /// Style for selection highlighting.
    pub fn selection_highlight_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default()
            .fg(scheme.accent_primary)
            .add_modifier(Modifier::BOLD)
    }

    // Log level styles - subtle color coding for log levels

    /// Style for debug log messages.
    pub fn log_debug_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.text_description)
    }

    /// Style for info log messages.
    pub fn log_info_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.text_primary)
    }

    /// Style for warning log messages.
    pub fn log_warn_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default().fg(scheme.warning)
    }

    /// Style for error log messages.
    pub fn log_error_themed(theme: &Theme) -> Style {
        let scheme = ColorScheme::for_theme(theme);
        Style::default()
            .fg(scheme.error)
            .add_modifier(Modifier::BOLD)
    }
}
