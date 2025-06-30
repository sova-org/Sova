
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
