use crate::app::AppState;
use crossterm::event::KeyEvent;
use ratatui::{buffer::Buffer, layout::Rect};

mod header;
pub use header::*;
mod footer;
pub use footer::*;
mod scope;
pub use scope::*;
mod vu_meter;
pub use vu_meter::*;
mod floating;
pub use floating::*;
mod help;
pub use help::*;

pub mod popup;
pub mod log_widget;
pub mod list_choice;

pub trait InteractiveWidget {
    fn on_key_event(&mut self, event: KeyEvent, state: &mut AppState);
    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &mut AppState);
}