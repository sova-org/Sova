use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::{Buffer, Rect}, style::Stylize, text::{Line, Span, Text}, widgets::{Paragraph, StatefulWidget, Widget}};
use ratatui_themes::Style;
use ropey::Rope;

use crate::{app::{AppPage, AppState}};

pub struct Editor {
    buffer: Rope,
    cursor: (usize, usize),
    selection_begin: (usize, usize),
    selection_end: (usize, usize)
}

impl Editor {
    pub fn new() -> Self {
        Editor { 
            buffer: Rope::default(), 
            cursor: (0, 0),
            selection_begin: (0, 0),
            selection_end: (0, 0)
        }
    }

    pub fn update_content(&mut self, state: &AppState) {
        let Some(frame) = state.selected_frame() else {
            self.buffer = Rope::default();
            return;
        };
        self.set_text(frame.script().content());
    }

    pub fn set_text(&mut self, text: &str) {
        let text = text.replace("\t", "    ");
        self.buffer = Rope::from(text);
        self.cursor = (0, 0);
        self.selection_begin = (0, 0);
        self.selection_end = (0, 0);
    }

    pub fn cursor_char_idx(&self) -> usize {
        self.to_char_idx(self.cursor)
    }

    pub fn to_char_idx(&self, (line, row): (usize, usize)) -> usize {
        let line_begin = self.buffer.line_to_char(line);
        line_begin + row
    }

    pub fn next_char(&mut self) {
        let line = self.buffer.line(self.cursor.0);
        let lines = self.buffer.len_lines();
        if self.cursor.1 + 1 < line.len_chars() {
            self.cursor.1 += 1;
        } else if self.cursor.0 + 1 < lines {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        } else if self.cursor.0 + 1 == lines {
            self.cursor.1 = line.len_chars();
        }
    }

    pub fn previous_char(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            let line = self.buffer.line(self.cursor.0);
            self.cursor.1 = line.len_chars().saturating_sub(1);
        }
    }

    pub fn next_line(&mut self) {
        if self.cursor.0 + 1 < self.buffer.len_lines() {
            self.cursor.0 += 1;
        }
        let line = self.buffer.line(self.cursor.0);
        self.cursor.1 = self.cursor.1.min(line.len_chars().saturating_sub(1));
    }

    pub fn previous_line(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
        }
        let line = self.buffer.line(self.cursor.0);
        self.cursor.1 = self.cursor.1.min(line.len_chars().saturating_sub(1));
    }

    pub fn on_key_event(&mut self, event: KeyEvent, state: &mut AppState) {
        //self.cursor = self.cursor.min(self.buffer.len_chars());
        let char_idx = self.cursor_char_idx();
        match event.code {
            KeyCode::Enter if event.modifiers.contains(KeyModifiers::CONTROL) => {
                
            }
            KeyCode::Backspace => {
                if char_idx > 0 {
                    self.previous_char();
                    let begin = char_idx - 1;
                    self.buffer.remove(begin..char_idx);
                }
            }
            KeyCode::Enter => {
                self.buffer.insert(char_idx, "\n");
                self.cursor.0 += 1;
                self.cursor.1 = 0;
            }
            KeyCode::Left => {
                self.previous_char();
            }
            KeyCode::Right => {
                self.next_char();
            }
            KeyCode::Up => {
                self.previous_line();
            }
            KeyCode::Down => {
                self.next_line();
            }
            KeyCode::Home => (),
            KeyCode::End => (),
            KeyCode::PageUp => {
                state.go_to_previous_frame();
            }
            KeyCode::PageDown => {
                state.go_to_next_frame();
            }
            KeyCode::Tab => {
                self.buffer.insert(char_idx, "    ");
                self.cursor.1 += 4;
            }
            KeyCode::BackTab => (),
            KeyCode::Delete => {
                if char_idx < self.buffer.len_chars() {
                    self.buffer.remove(char_idx .. (char_idx + 1));
                }
            }
            KeyCode::Insert => (),
            KeyCode::F(_) => (),
            KeyCode::Char(c) => {
                self.buffer.insert_char(char_idx, c);
                self.cursor.1 += 1;
            },
            KeyCode::Null => (),
            KeyCode::Esc => {
                state.go_to(AppPage::Scene);
            }
            _ => ()
        }
    }

    fn render_line_numbers(&self, area: Rect, buf: &mut Buffer, state: &mut AppState) -> u16 {
        let first_col_len = self.buffer.len_lines().to_string().len() as u16;
        for l in 0..self.buffer.len_lines() {
            let line_index = Span::styled(
                l.to_string(), 
                Style::default().fg(
                    if l as usize == self.cursor.0 { state.palette.foreground } else { state.palette.muted }
                ).bg(state.palette.background)
            );
            let line_index = Line::from(line_index).right_aligned();
            line_index.render(Rect { 
                x: area.x, 
                y: area.y + l as u16, 
                width: first_col_len, 
                height: 1
            }, buf);
        }
        first_col_len
    }

    fn render_buffer(&self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let Some(frame) = state.selected_frame() else {
            return;
        };

        let lang = frame.script().lang();
        let mut lines : Vec<Line> = if let Some(syn) = state.compiled_syntax.get(lang) {
            self.buffer.lines().map(|line| {
                let line_str = line.to_string();
                let split = syn.split(&line_str);
                split.into_iter().map(|(sub, cat)| {
                    Span::styled(sub, if let Some(cat) = cat { 
                        state.palette.token_color(cat)
                    } else { 
                        state.palette.foreground 
                    })
                }).collect()
            }).collect()
        } else {
            self.buffer.lines().map(|line| {
                let spans = vec![Span::styled(line.to_string(), state.palette.text())];
                Line::from(spans)
            }).collect()
        };

        if self.cursor.0 < lines.len() {
            let line = std::mem::take(&mut lines[self.cursor.0]);
            lines[self.cursor.0] = line.bg(state.palette.selection);
        }

        let text = Text::from(lines);
        let p = Paragraph::new(text).bg(state.palette.surface);
        p.render(area, buf);
    }

    fn render_cursor(&self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let char_idx = self.cursor_char_idx();
        let char = if char_idx < self.buffer.len_chars() {
            let at_idx = self.buffer.char(char_idx);
            if at_idx.is_control() { ' ' } else { at_idx }
        } else {
            ' '
        };
        let cursor_render = Span::styled(char.to_string(), state.palette.text().reversed());
        cursor_render.render(Rect { 
            x: area.x + (self.cursor.1 as u16), 
            y: area.y + (self.cursor.0 as u16), 
            width: 1, height: 1 
        }, buf);
    }
}

impl StatefulWidget for &Editor {
    type State = AppState;

    fn render(self, mut area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let first_col_len = self.render_line_numbers(area, buf, state);

        area.x += first_col_len + 1;
        area.width = area.width.saturating_sub(first_col_len + 2);
        
        self.render_buffer(area, buf, state);
        self.render_cursor(area, buf, state);
    }
}