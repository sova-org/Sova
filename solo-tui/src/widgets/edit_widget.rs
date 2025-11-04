use crossterm::event::KeyEvent;
use ratatui::{buffer::Buffer, layout::Rect, style::{Style, Stylize}, widgets::{StatefulWidget, Widget}};
use tui_textarea::TextArea;

use crate::app::AppState;

pub struct EditWidget {
    text_area: TextArea<'static>
}

impl EditWidget {

    pub fn new() -> Self {
        let mut text_area : TextArea = Default::default();
        text_area.set_line_number_style(Style::default().dark_gray());
        EditWidget { text_area }
    }

    pub fn get_help() -> &'static str {
        "\
        C-S: Upload
        "
    }

    pub fn process_event(&mut self, state: &mut AppState, event: KeyEvent) { 
        match event.code {
            _ => { 
                self.text_area.input(event);
            }
        }
    }

}

impl StatefulWidget for &EditWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.text_area.render(area, buf);
    }
}