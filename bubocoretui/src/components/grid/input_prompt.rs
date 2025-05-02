use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};
use tui_textarea::TextArea;

/// A widget that renders a text input prompt with a title and custom styling.
///
/// This widget wraps a `TextArea` component to provide a styled input field with a title.
/// It's commonly used for user input prompts in the grid interface, such as when
/// renaming lines or entering frame data.
///
/// # Fields
///
/// * `textarea` - A reference to the underlying text input area
/// * `title` - The title displayed in the widget's border
/// * `style` - The visual styling applied to the widget
///
/// # Lifetime Parameters
///
/// * `'a` - The lifetime of the reference to the `TextArea`
pub struct InputPromptWidget<'a> {
    pub textarea: &'a TextArea<'a>,
    pub title: String,
    pub style: Style,
}

impl<'a> InputPromptWidget<'a> {
    pub fn new(textarea: &'a TextArea<'a>, title: String, style: Style) -> Self {
        Self { textarea, title, style }
    }
}

impl<'a> Widget for InputPromptWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut textarea_to_render = self.textarea.clone();
        textarea_to_render.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", self.title))
                .style(self.style),
        );
        textarea_to_render.set_style(Style::default().fg(Color::White));
        textarea_to_render.render(area, buf);
    }
}
