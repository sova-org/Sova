use std::cell::RefCell;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    widgets::{
        Block, BorderType, Clear, HighlightSpacing, List, ListItem, ListState, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};

use crate::{app::AppState};

#[derive(Default)]
pub struct ListChoice {
    pub showing: bool,
    pub title: String,
    pub content: String,
    pub index: usize, 
    pub choices: Vec<String>,
    list_state: RefCell<ListState>,
    pub done: bool,
}

impl ListChoice {
    pub fn open(
        &mut self,
    ) {
        self.showing = true;
        self.done = false;
    }

    pub fn select(&mut self, value: &str) {
        for (i, x) in self.choices.iter().enumerate() {
            if x == value {
                self.index = i;
                self.list_state.get_mut().select(Some(i));
            }
        }
    }

    pub fn selected(&self) -> String {
        self.choices.get(self.index).cloned().unwrap_or_default()
    }

    pub fn hide(&mut self) {
        self.showing = false;
    }

    pub fn process_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Esc => self.hide(),
            KeyCode::Enter => self.complete(),
            _ => {
                let len = self.choices.len();
                match event.code {
                    KeyCode::Up if self.list_state.borrow().selected() != Some(0) => {
                        self.list_state.get_mut().select_previous()
                    }
                    KeyCode::Down if self.list_state.borrow().selected() != Some(len - 1) => {
                        self.list_state.get_mut().select_next()
                    }
                    KeyCode::Left => self.list_state.get_mut().select_first(),
                    KeyCode::Right => self.list_state.get_mut().select_last(),
                    code => {
                        if let Some(ch) = code.as_char() && ch.is_ascii_digit() && ch != '0' {
                            let i = (ch as usize).saturating_sub(31) % len;
                            self.list_state.get_mut().select(Some(i));
                        }
                    },
                }
                self.index = self.list_state.borrow().selected().unwrap_or_default();
            },
        }
    }

    pub fn complete(&mut self) {
        self.showing = false;
        self.done = true;
    }

    fn popup_area(area: Rect, percent_x: u16, len: usize, additional_lines: u16) -> Rect {
        let len = 125 * (len as u16) / 100;
        let width = percent_x * area.width / 100;
        let lines = 3 + len / width + u16::from(len % width > 0) + additional_lines;
        let horizontal = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);
        let vertical = Layout::vertical([Constraint::Length(lines)]).flex(Flex::Center);
        let [area] = horizontal.areas(area);
        let [area] = vertical.areas(area);
        area
    }
}

impl StatefulWidget for &ListChoice {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !self.showing {
            return;
        }
        let selected_style = Style::default()
            .bg(state.palette.selection)
            .fg(state.palette.accent)
            .bold();
        let additional_lines = std::cmp::min(10, self.choices.len() as u16);
        let area = ListChoice::popup_area(area, 30, self.content.len(), additional_lines);
        Clear.render(area, buf);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(self.title.as_str())
            .bg(state.palette.surface)
            .fg(state.palette.foreground);
        let layout = Layout::vertical([Constraint::Min(0), Constraint::Length(additional_lines)]);
        let [text_area, input_area] = layout.areas(block.inner(area));
        block.render(area, buf);
        Paragraph::new(self.content.as_str())
            .wrap(Wrap { trim: true })
            .render(text_area, buf);
        let items: Vec<ListItem> = self.choices.iter().map(|s| ListItem::from(s.as_str())).collect();
        let list = List::new(items)
            .style(state.palette.text())
            .highlight_style(selected_style)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);
        let mut state_mut = self.list_state.borrow_mut();
        StatefulWidget::render(list, input_area, buf, &mut state_mut);
    }
}
