use ratatui::prelude::*;
use ratatui::style::Color;


#[derive(Clone)]
/// Defines the visual styles for different states of grid cells in the timeline view.
/// 
/// Each field represents a different cell state:
/// - `enabled`: Style for active frames that are currently playing
/// - `disabled`: Style for inactive frames that are not playing
/// - `cursor`: Style for the cell where the user's cursor is positioned
/// - `peer_cursor`: Style for cells where other users' cursors are positioned
/// - `start_end_marker`: Style for cells that mark the start or end of a selection
pub struct GridCellStyles {
    pub enabled: Style,
    pub disabled: Style,
    pub cursor: Style,
    pub peer_cursor: Style,
    pub start_end_marker: Style,
}

impl GridCellStyles {
    pub fn default_styles() -> Self {
        Self {
            enabled: Style::default().fg(Color::White).bg(Color::Green),
            disabled: Style::default().fg(Color::White).bg(Color::Red),
            cursor: Style::default().fg(Color::White).bg(Color::Yellow).bold(),
            peer_cursor: Style::default().bg(Color::White).fg(Color::Black),
            start_end_marker: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        }
    }
}