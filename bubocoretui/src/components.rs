use ratatui::prelude::Rect;

pub mod editor;
pub mod grid;
pub mod help;
pub mod options;
pub mod splash;

pub fn inner_area(area: Rect) -> Rect {
    let inner = area;
    Rect {
        x: inner.x + 1,
        y: inner.y + 1,
        width: inner.width.saturating_sub(2),
        height: inner.height.saturating_sub(2),
    }
}
