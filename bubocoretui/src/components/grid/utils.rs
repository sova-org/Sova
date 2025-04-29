use ratatui::prelude::*;
use bubocorelib::scene::Line;
/// Creates a centered rectangle within a given area.
///
/// This function calculates a new rectangle that is centered both horizontally and vertically
/// within the provided area. The size of the new rectangle is specified as percentages of
/// the original area's dimensions.
///
/// # Arguments
///
/// * `percent_x` - The width of the new rectangle as a percentage of the original width (0-100)
/// * `percent_y` - The height of the new rectangle as a percentage of the original height (0-100)
/// * `r` - The original rectangle to center within
///
/// # Returns
///
/// A new `Rect` that is centered within the original area with the specified percentage dimensions
///
/// # Example
///
/// ```
/// let area = Rect::new(0, 0, 100, 100);
/// let centered = centered_rect(60, 60, area);
/// // centered will be a 60x60 rectangle centered within the 100x100 area
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

/// Holds the data needed to render a single cell in the grid.
/// 
/// This struct contains all the information required to render a cell in the timeline grid,
/// including its position, associated line data, and display properties.
/// 
/// # Fields
/// 
/// * `frame_idx` - The row index of the frame within its line (0-based)
/// * `col_idx` - The column index of the line in the scene (0-based)
/// * `line` - Optional reference to the line data containing the frame. None if the line doesn't exist
/// * `col_width` - The width in characters that the cell should occupy when rendered
pub struct GridCellData<'a> {
    pub frame_idx: usize,
    pub col_idx: usize,
    pub line: Option<&'a Line>,
    pub col_width: u16,
}
