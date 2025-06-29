//! Layout utility functions for TUI components.
//!
//! This module provides common layout calculations used across multiple components.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Creates a centered rectangle within a given area using percentage-based sizing.
///
/// This function calculates a rectangle that is centered both horizontally and vertically
/// within the provided area `r`. The size of the resulting rectangle is specified as
/// percentages of the parent area's width and height.
///
/// # Arguments
///
/// * `percent_x` - The width of the resulting rectangle as a percentage of the parent area's width
/// * `percent_y` - The height of the resulting rectangle as a percentage of the parent area's height
/// * `r` - The parent area (Rect) within which to center the new rectangle
///
/// # Returns
///
/// A new `Rect` that is centered within the parent area with the specified dimensions
///
/// # Example
///
/// ```
/// use ratatui::layout::Rect;
/// use tui::utils::layout::centered_rect;
///
/// let parent = Rect::new(0, 0, 100, 50);
/// let centered = centered_rect(50, 30, parent);
/// // Creates a rectangle that is 50% of parent width and 30% of parent height,
/// // centered within the parent area
/// ```
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Ensure percentages are within bounds
    let percent_x = percent_x.min(100);
    let percent_y = percent_y.min(100);

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Creates a centered rectangle within a given area using fixed dimensions.
///
/// This function calculates a rectangle that is centered both horizontally and vertically
/// within the provided area `r`. The size of the resulting rectangle is specified as
/// fixed width and height values.
///
/// # Arguments
///
/// * `width` - The fixed width of the resulting rectangle
/// * `height` - The fixed height of the resulting rectangle
/// * `r` - The parent area (Rect) within which to center the new rectangle
///
/// # Returns
///
/// A new `Rect` that is centered within the parent area with the specified fixed dimensions
///
/// # Example
///
/// ```
/// use ratatui::layout::Rect;
/// use tui::utils::layout::centered_rect_fixed;
///
/// let parent = Rect::new(0, 0, 100, 50);
/// let centered = centered_rect_fixed(30, 10, parent);
/// // Creates a rectangle that is exactly 30x10 units, centered within the parent area
/// ```
pub fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let horizontal_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(width),
        Constraint::Fill(1),
    ])
    .split(horizontal_layout[1])[1]
}