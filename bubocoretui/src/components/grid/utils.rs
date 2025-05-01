use bubocorelib::scene::line::Line;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Contains information about the grid's rendering dimensions and frame counts.
/// 
/// This struct is used to track the visible area and total frame count of the grid,
/// which is essential for calculating scroll positions and determining what portion
/// of the grid should be displayed.
/// 
/// # Fields
/// 
/// * `visible_height` - The number of rows that can be displayed in the current viewport
/// * `max_frames` - The total number of frames in the longest line of the grid
pub struct GridRenderInfo {
    pub visible_height: usize,
    pub max_frames: usize,
}

