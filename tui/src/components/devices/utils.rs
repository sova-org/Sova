use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Creates a centered rectangle within a given area.
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
/// use bubocoretui::components::devices::utils::centered_rect;
///
/// let parent = Rect::new(0, 0, 100, 50);
/// let centered = centered_rect(50, 30, parent);
/// // Creates a rectangle that is 50% of parent width and 30% of parent height,
/// // centered within the parent area
/// ```
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
