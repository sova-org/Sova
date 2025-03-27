use crate::App;
use ratatui::{Frame, prelude::Rect, style::Style};
use tui_big_text::{BigText, PixelSize};

pub fn draw(frame: &mut Frame, _app: &App, _area: Rect) {
    let big_text = BigText::builder()
        .centered()
        .pixel_size(PixelSize::Full)
        .style(Style::new())
        .lines(vec!["".into(), "BuboCore".into()])
        .build();
    frame.render_widget(big_text, frame.area());
}
