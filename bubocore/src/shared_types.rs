use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

/// Represents the user's selection in the grid component.
/// Shared between server and clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridSelection {
    /// The starting cell of the selection (usually where the selection began).
    pub start: (usize, usize), // (row, col)
    /// The ending cell of the selection (usually the current cursor position).
    pub end: (usize, usize),   // (row, col)
}

impl GridSelection {
    /// Creates a new selection starting and ending at the same cell.
    pub fn single(row: usize, col: usize) -> Self {
        Self { start: (row, col), end: (row, col) }
    }

    /// Checks if the selection covers only a single cell.
    pub fn is_single(&self) -> bool {
        self.start == self.end
    }

    /// Returns the normalized bounds of the selection.
    /// Returns ((top_row, left_col), (bottom_row, right_col)).
    pub fn bounds(&self) -> ((usize, usize), (usize, usize)) {
        let top = min(self.start.0, self.end.0);
        let left = min(self.start.1, self.end.1);
        let bottom = max(self.start.0, self.end.0);
        let right = max(self.start.1, self.end.1);
        ((top, left), (bottom, right))
    }

    /// Returns the primary cursor position (usually the 'end' position).
    pub fn cursor_pos(&self) -> (usize, usize) {
        self.end
    }
}

impl Default for GridSelection {
    fn default() -> Self {
        Self::single(0, 0)
    }
}

// Placeholder for richer device info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: usize,
    pub name: String,
    pub kind: DeviceKind,
    pub is_connected: bool,
    // Consider adding is_input/is_output flags or refining DeviceKind
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeviceKind {
    Midi,
    Osc,
    Log, // Added Log for completeness
    Other,
}

/// Data structure representing a single frame being pasted.
/// Sent from client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastedFrameData {
    pub length: f64,
    pub is_enabled: bool,
    pub script_content: Option<String>,
} 