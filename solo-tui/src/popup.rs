use std::str::FromStr;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{buffer::Buffer, layout::{Constraint, Flex, Layout, Rect}, style::{Color, Style, Stylize}, widgets::{Block, BorderType, Clear, Paragraph, Widget, Wrap}};
use tui_textarea::{CursorMove, TextArea};

use crate::app::AppState;

#[derive(Default)]
pub enum PopupValue {
    #[default]
    None,
    Bool(bool),
    Text(String),
    Float(f64),
    Int(i64)
}

impl PopupValue {
    pub fn float(self) -> f64 {
        match self {
            Self::Float(x) => x,
            _ => Default::default()
        }
    }
    pub fn bool(self) -> bool {
        match self {
            Self::Bool(x) => x,
            _ => Default::default()
        }
    }
    pub fn int(self) -> i64 {
        match self {
            Self::Int(x) => x,
            _ => Default::default()
        }
    }
    pub fn text(self) -> String {
        match self {
            Self::Text(x) => x,
            _ => Default::default()
        }
    }
}

#[derive(Default)]
pub struct Popup {
    pub showing: bool,
    pub title: String,
    pub content: String,
    pub value: PopupValue,
    callback: Option<Box<dyn FnOnce(&mut AppState, PopupValue)>>,
    text_area: TextArea<'static>
}

impl Popup {

    pub fn open(
        &mut self, 
        title: String, 
        content: String, 
        value: PopupValue, 
        callback: Box<dyn FnOnce(&mut AppState, PopupValue)>
    ) {
        self.title = title;
        self.content = content;
        self.value = value;
        self.callback = Some(callback);
        self.showing = true;

        match &self.value {
            PopupValue::Text(txt) => self.text_area = vec![txt.clone()].into(),
            PopupValue::Float(f) => self.text_area = vec![format!("{f}")].into(),
            PopupValue::Int(i) => self.text_area = vec![format!("{i}")].into(),
            _ => ()
        }
        self.text_area.set_block(Block::bordered()
            .border_style(Color::LightGreen).border_type(BorderType::Rounded)
        );
        self.text_area.move_cursor(CursorMove::End);
    }

    pub fn info(
        &mut self, 
        title: String, 
        content: String, 
    ) {
        self.title = title;
        self.content = content;
        self.value = Default::default();
        self.callback = None;
        self.showing = true;
    }

    pub fn hide(&mut self) {
        self.showing = false;
    }

    fn validate_input<T>(text_area: &mut TextArea, dst: &mut T) 
        where T: FromStr
    {
        let text = text_area.lines().get(0).cloned().unwrap_or_default();
        let mut color = Color::LightGreen;
        match text.parse::<T>() {
            Ok(x) => *dst = x,
            Err(_) => color = Color::LightRed
        }
        text_area.set_block(Block::bordered()
            .border_style(color).border_type(BorderType::Rounded)
        );
    }

    pub fn process_event(&mut self, state: &mut AppState, event: KeyEvent) {
        match event.code {
            KeyCode::Esc => self.hide(),
            KeyCode::Enter => self.complete(state),
            _ => match &mut self.value {
                PopupValue::None => (),
                PopupValue::Bool(b) => match event.code {
                    KeyCode::Left => *b = true,
                    KeyCode::Right => *b = false,
                    _ => ()
                },
                PopupValue::Text(txt) => {
                    self.text_area.input(event);
                    *txt = self.text_area.lines().get(0).cloned().unwrap_or_default()
                },
                PopupValue::Float(f) => {
                    self.text_area.input(event);
                    Self::validate_input(&mut self.text_area, f);
                },
                PopupValue::Int(i) => {
                    self.text_area.input(event);
                    Self::validate_input(&mut self.text_area, i);
                },
            }
        }
    }

    pub fn complete(&mut self, state: &mut AppState) {
        let value = std::mem::take(&mut self.value);
        if let Some(callback) = std::mem::take(&mut self.callback) {
            callback(state, value);
            self.callback = None;
        }
        self.showing = false;
    }

    fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

}

impl Widget for &Popup {

    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.showing {
            let button_block = Block::bordered().border_type(BorderType::Rounded);
            let selected_style = Style::default().bg(Color::White).fg(Color::Black);
            let area = Popup::popup_area(area, 50, 25);
            Clear.render(area, buf);
            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(self.title.as_str())
                .on_black();
            let layout = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]);
            let [text_area, input_area] = layout.areas(block.inner(area));
            block.render(area, buf);
            Paragraph::new(self.content.as_str())
                .wrap(Wrap { trim: true })
                .render(text_area, buf);
            match &self.value {
                PopupValue::None => {
                    let horizontal = Layout::horizontal([Constraint::Length(10)]).flex(Flex::Center);
                    let [input_area] = horizontal.areas(input_area);
                    Paragraph::new("Ok")
                        .on_white()
                        .black()
                        .centered()
                        .block(button_block)
                        .render(input_area, buf)
                }
                PopupValue::Bool(b) => {
                    let horizontal = Layout::horizontal([Constraint::Length(10), Constraint::Length(10)]).flex(Flex::Center);
                    let [yes_area, no_area] = horizontal.areas(input_area);
                    Paragraph::new("Yes")
                        .style(if *b { selected_style } else { Style::default() })
                        .centered()
                        .block(button_block.clone())
                        .render(yes_area, buf);
                    Paragraph::new("No")
                        .style(if !*b { selected_style } else { Style::default() })
                        .centered()
                        .block(button_block)
                        .render(no_area, buf)
                },
                PopupValue::Text(_) 
                    | PopupValue::Float(_)
                    | PopupValue::Int(_) => {
                    self.text_area.render(input_area, buf);
                },
            }
        }
    }
}