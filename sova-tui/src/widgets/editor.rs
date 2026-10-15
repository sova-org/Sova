use std::{collections::VecDeque, ops::Range, time::Duration};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Position, prelude::{Buffer, Rect}, text::{Line, Span}, widgets::{StatefulWidget, Widget},
};
use ratatui_themes::Style;
use ropey::Rope;

use crate::{
    app::AppState,
    compiled_syntax::CompiledSyntax,
    flash::Flash,
    theme::Palette,
    widgets::list_choice::ListChoice,
};

const EDITOR_FLASH_MS: u64 = 100;

pub type CursorPos = (usize, usize);
pub type EditorState = (String, CursorPos);

const MAX_HISTORY : usize = 100;

#[derive(Debug, Default, Clone)]
pub struct EditorHistory {
    past: VecDeque<EditorState>,
    future: Vec<EditorState>,
}

impl EditorHistory {
    pub fn add(&mut self, state: EditorState) {
        self.past.push_back(state);
        self.future.clear();
        while self.past.len() > MAX_HISTORY {
            let init = self.past.pop_front().unwrap();
            let _ = self.past.pop_front();
            self.past.push_front(init);
        }
    }

    pub fn current(&self) -> Option<&EditorState> {
        self.past.back()
    }

    pub fn current_mut(&mut self) -> Option<&mut EditorState> {
        self.past.back_mut()
    }

    pub fn lens(&self) -> (usize, usize) {
        (self.past.len(), self.future.len())
    }

    pub fn len(&self) -> usize {
        self.past.len() + self.future.len()
    }

    pub fn is_empty(&self) -> bool {
        self.is_past_empty() && self.is_future_empty()
    }

    pub fn backward(&mut self) -> Option<&EditorState> {
        if self.past.is_empty() {
            return None;
        } else if self.past.len() == 1 {
            return self.past.back();
        } else {
            let state = self.past.pop_back().unwrap();
            self.future.push(state);
            self.past.back()
        }
    }

    pub fn forward(&mut self) -> Option<&EditorState> {
        let Some(state) = self.future.pop() else {
            return None;
        };
        self.past.push_back(state);
        self.past.back()
    }

    pub fn is_past_empty(&self) -> bool {
        self.past.is_empty()
    }

    pub fn is_future_empty(&self) -> bool {
        self.future.is_empty()
    }

    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }
}

pub struct Editor {
    buffer: Rope,
    cursor: CursorPos,
    selection_anchor: Option<CursorPos>,
    pub lang: String,
    pub flash: Flash,
    pub show_cursor: bool,
    pub lang_popup: ListChoice,
    pub selecting: bool,
    pub history: EditorHistory
}

impl Editor {
    pub fn new() -> Self {
        let mut popup = ListChoice::default();
        popup.title = "Language".to_string();
        popup.content = "Select language for script".to_string();
        Editor {
            buffer: Rope::default(),
            cursor: (0, 0),
            selection_anchor: None,
            lang: String::new(),
            flash: Flash::new(Duration::from_millis(EDITOR_FLASH_MS)),
            show_cursor: true,
            lang_popup: popup,
            selecting: false,
            history: Default::default()
        }
    }

    pub fn cursor_pos(&self) -> CursorPos {
        self.cursor
    }

    pub fn set_cursor_pos(&mut self, pos: CursorPos) {
        self.cursor = pos;
        self.sanitize_cursor();
    }

    pub fn content(&self) -> String {
        self.buffer.to_string()
    }

    pub fn save_state(&mut self) {
        self.history.add((self.content(), self.cursor_pos()));
    }

    pub fn set_text(&mut self, text: &str) {
        //let text = text.replace("\t", "    ");
        self.buffer = Rope::from(text);
        self.cursor = (0, 0);
        self.selection_anchor = None;
        self.history.clear();
        self.save_state();
    }

    pub fn cursor_char_idx(&self) -> usize {
        self.to_char_idx(self.cursor)
    }

    pub fn to_char_idx(&self, (line, row): CursorPos) -> usize {
        let line_begin = self.buffer.line_to_char(line);
        line_begin + row
    }

    pub fn start(&mut self) {
        self.cursor = (0, 0);
    }

    pub fn start_selection(&mut self) {
        self.selection_anchor = Some(self.cursor);
        self.selecting = true;
    }

    pub fn stop_selection(&mut self) {
        self.selecting = false;
    }

    pub fn cancel_selection(&mut self) {
        self.stop_selection();
        self.selection_anchor = None;
    }

    pub fn has_selection(&self) -> bool {
        self.selection_anchor.is_some()
    }

    pub fn select_all(&mut self) {
        self.start();
        self.start_selection();
        self.end();
    }

    pub fn select_line(&mut self) {
        self.start_of_line();
        self.start_selection();
        self.end_of_line();
    }

    pub fn reverse_selection(&mut self) {
        let Some(anchor) = &mut self.selection_anchor else {
            return;
        };
        std::mem::swap(&mut self.cursor, anchor)
    }

    pub fn selection_range(&self) -> Range<usize> {
        let Some(anchor) = self.selection_anchor else {
            return 0..0;
        };
        let mut a = self.to_char_idx(anchor);
        let mut b = self.to_char_idx(self.cursor);
        if b < a {
            std::mem::swap(&mut a, &mut b);
        }
        // if b < self.buffer.len_chars() {
        //     b += 1;
        // }
        a..b
    }

    pub fn selection_begin(&self) -> Option<CursorPos> {
        let Some(anchor) = self.selection_anchor else {
            return None;
        };
        if anchor < self.cursor {
            Some(anchor)
        } else {
            Some(self.cursor)
        }
    }

    pub fn selection_end(&self) -> Option<CursorPos> {
        let Some(anchor) = self.selection_anchor else {
            return None;
        };
        if anchor > self.cursor {
            Some(anchor)
        } else {
            Some(self.cursor)
        }
    }

    pub fn selected_text(&self) -> String {
        let range = self.selection_range();
        self.buffer.slice(range).to_string()
    }

    pub fn remove_selection(&mut self) {
        let Some(begin) = self.selection_begin() else {
            return;
        };
        let range = self.selection_range();
        self.buffer.remove(range);
        self.cursor = begin;
        self.cancel_selection();
    }

    pub fn replace_selection(&mut self, text: String) {
        self.remove_selection();
        self.insert_text_before(text);
    }

    pub fn sanitize_cursor(&mut self) {
        self.cursor = self.sanitize_pos(self.cursor);
    }

    pub fn current_char(&self) -> Option<char> {
        let idx = self.cursor_char_idx();
        if idx >= self.buffer.len_chars() {
            return None;
        }
        Some(self.buffer.char(idx))
    }

    pub fn sanitize_pos(&self, (mut y, mut x): CursorPos) -> CursorPos {
        let len_lines = self.buffer.len_lines();
        if len_lines == 0 {
            return (0, 0);
        }
        let last_line = len_lines - 1;
        if y >= len_lines {
            y = last_line;
        }
        let line = self.buffer.line(y);
        let len_chars = line.len_chars();
        if len_chars == 0 {
            return (y, 0);
        }
        if x >= len_chars && y == last_line {
            x = len_chars;
        } else if x >= len_chars {
            x = len_chars - 1;
        }
        (y, x)
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

    pub fn end(&mut self) {
        let lines = self.buffer.len_lines();
        if lines == 0 {
            self.cursor = (0, 0);
            return;
        }
        self.cursor.0 = lines - 1;
        self.cursor.1 = self
            .buffer
            .line(self.cursor.0)
            .len_chars();
            //.saturating_sub(1);
    }

    pub fn end_of_line(&mut self) {
        let char_idx = self.cursor_char_idx();
        let line = self.buffer.char_to_line(char_idx);
        let line = self.buffer.line(line);
        self.cursor.1 = line.len_chars().saturating_sub(1);
    }

    pub fn start_of_line(&mut self) {
        self.cursor.1 = 0;
    }

    pub fn next_word(&mut self) -> bool {
        let len = self.buffer.len_chars();
        let mut current = self.cursor_char_idx();
        if current == len {
            return false;
        }
        let mut curr_whitespace = self.buffer.char(current).is_whitespace();
        let init_whitespace = curr_whitespace;
        while !(init_whitespace ^ curr_whitespace) {
            self.next_char();
            current = self.cursor_char_idx();
            if current >= len {
                break;
            }
            curr_whitespace = self.buffer.char(current).is_whitespace();
        }
        true
    }

    pub fn previous_word(&mut self) -> bool {
        let mut current = self.cursor_char_idx();
        if current == 0 {
            return false;
        }
        let mut curr_whitespace = self.buffer.char(current - 1).is_whitespace();
        let init_whitespace = curr_whitespace;
        while !(init_whitespace ^ curr_whitespace) {
            self.previous_char();
            current = self.cursor_char_idx();
            if current == 0 {
                break;
            }
            curr_whitespace = self.buffer.char(current - 1).is_whitespace();
        }
        true
    }

    pub fn insert_text_before(&mut self, text: String) {
        //let text = text.replace("\t", "    ");
        let idx = self.cursor_char_idx();
        let mut first = true;
        for line in text.lines() {
            let len = line.len();
            if !first {
                self.cursor.0 += 1;
                self.cursor.1 = len;
            } else {
                self.cursor.1 += len;
            }
            first = false;
        }
        self.buffer.insert(idx, &text);
    }

    pub fn insert_text_after(&mut self, text: String) {
        //let text = text.replace("\t", "    ");
        self.buffer.insert(self.cursor_char_idx(), &text);
    }

    pub fn remove_next_char(&mut self) -> bool {
        let char_idx = self.cursor_char_idx();
        if char_idx < self.buffer.len_chars() {
            self.buffer.remove(char_idx..(char_idx + 1));
            return true;
        }
        false
    }

    pub fn remove_previous_char(&mut self) -> bool {
        let char_idx = self.cursor_char_idx();
        if char_idx > 0 {
            self.previous_char();
            let begin = char_idx - 1;
            self.buffer.remove(begin..char_idx);
            return true;
        }
        false
    }

    pub fn undo(&mut self) {
        let Some((content, cursor)) = self.history.backward() else {
            return;
        };
        self.buffer = Rope::from_str(content);
        self.cursor = *cursor;
        self.cancel_selection();
    }

    pub fn redo(&mut self) {
        let Some((content, cursor)) = self.history.forward() else {
            return;
        };
        self.buffer = Rope::from_str(content);
        self.cursor = *cursor;
        self.cancel_selection();
    }

    pub fn on_key_event(&mut self, event: KeyEvent, state: &mut AppState) {
        if self.lang_popup.showing {
            self.lang_popup.process_event(event);
            if self.lang_popup.done {
                self.lang = self.lang_popup.selected();
            }
            return;
        }
        match event.code {
            KeyCode::Char('l') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.lang_popup.choices = state
                    .client_state
                    .languages
                    .iter()
                    .map(|l| l.name.clone())
                    .collect();
                self.lang_popup.select(&self.lang);
                self.lang_popup.open();
                return;
            }
            KeyCode::Char('z') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.undo();
            }
            KeyCode::Char('y') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.redo();
            }
            KeyCode::Char('w') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.selecting {
                    self.next_word();
                } else {
                    self.start_selection();
                    self.next_word();
                    self.stop_selection();
                }
            }
            KeyCode::Char('b') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.selecting {
                    self.previous_word();
                } else {
                    self.start_selection();
                    self.previous_word();
                    self.stop_selection();
                }
            }
            KeyCode::Char('a') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.selecting {
                    self.select_all();
                } else {
                    self.select_all();
                    self.stop_selection();
                }
            }
            KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                let selection = self.selected_text();
                if let Some(cb) = &mut state.clipboard
                    && !selection.is_empty()
                {
                    match cb.set_text(selection) {
                        Ok(_) => state.positive("Copied text to clipboard!"),
                        Err(_) => state.negative("Unable to copy text!"),
                    }
                }
            }
            KeyCode::Char('v') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                let Some(cb) = &mut state.clipboard else {
                    state.negative("Unable to paste from clipboard !");
                    return;
                };
                let Ok(to_insert) = cb.get_text() else {
                    state.negative("Unable to paste from clipboard !");
                    return;
                };
                if to_insert.is_empty() {
                    return;
                }
                self.replace_selection(to_insert);
                self.save_state();
            }
            KeyCode::Backspace if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.start_selection();
                let moved = self.previous_word();
                self.remove_selection();
                if moved {
                    self.save_state();
                }
            }
            KeyCode::Delete if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.start_selection();
                let moved = self.next_word();
                self.remove_selection();
                if moved {
                    self.save_state();
                }
            }
            KeyCode::Backspace => {
                if self.has_selection() {
                    self.remove_selection();
                    self.save_state();
                } else if self.remove_previous_char() {
                    self.save_state();
                }
            }
            KeyCode::Delete => {
                if self.has_selection() {
                    self.remove_selection();
                    self.save_state();
                } else if self.remove_next_char() {
                    self.save_state();
                }
            }
            KeyCode::Enter => {
                if self.has_selection() {
                    self.remove_selection();
                }
                self.buffer.insert(self.cursor_char_idx(), "\n");
                self.cursor.0 += 1;
                self.cursor.1 = 0;
                self.save_state();
            }
            KeyCode::Left => {
                if !self.selecting {
                    self.cancel_selection();
                }
                self.previous_char();
            }
            KeyCode::Right => {
                if !self.selecting {
                    self.cancel_selection();
                }
                self.next_char();
            }
            KeyCode::Up => {
                if !self.selecting {
                    self.cancel_selection();
                }
                self.previous_line();
            }
            KeyCode::Down => {
                if !self.selecting {
                    self.cancel_selection();
                }
                self.next_line();
            }
            KeyCode::Home => {
                self.start_of_line();
            }
            KeyCode::End => {
                self.end_of_line();
            }
            KeyCode::Tab => {
                if self.has_selection() {
                    self.remove_selection();
                }
                self.buffer.insert(self.cursor_char_idx(), "\t");
                self.cursor.1 += 1;
                self.save_state();
            }
            KeyCode::BackTab => (),
            KeyCode::Insert => (),
            KeyCode::F(_) => (),
            KeyCode::Char(c) => {
                if self.has_selection() {
                    self.remove_selection();
                }
                self.buffer.insert_char(self.cursor_char_idx(), c);
                self.cursor.1 += 1;
                self.save_state();
            }
            KeyCode::Null => (),
            _ => (),
        }
        self.sanitize_cursor();
        if self.history.lens().0 == 1 {
            self.history.current_mut().unwrap().1 = self.cursor;
        }
    }

    fn render_line_numbers(
        &self,
        from: usize,
        area: Rect,
        buf: &mut Buffer,
        palette: &Palette,
    ) -> u16 {
        let first_col_len = self.buffer.len_lines().to_string().len().max(2) as u16;
        for l in 0..self.buffer.len_lines().min(area.height as usize) {
            let real_l = from + l;
            let line_index = Span::styled(
                real_l.to_string(),
                Style::default()
                    .fg(if real_l == self.cursor.0 {
                        palette.foreground
                    } else {
                        palette.muted
                    })
                    .bg(palette.background),
            );
            let line_index = Line::from(line_index).right_aligned();
            line_index.render(
                Rect {
                    x: area.x,
                    y: area.y + l as u16,
                    width: first_col_len,
                    height: 1,
                },
                buf,
            );
        }
        first_col_len
    }

    fn render_iter(
        &self,
        first_line: usize, 
        first_col: usize, 
        area: Rect,
        buf: &mut Buffer,
        palette: &Palette,
        syntax: Option<&CompiledSyntax>,
    ) {
        let content = self.buffer.to_string();
        let select_range = self.selection_range();
        let cursor_idx = self.cursor_char_idx();
        let slices = syntax
            .map(|syn| syn.split(&content))
            .unwrap_or(vec![(content, None)]);
        let mut pos = (0, 0);
        let mut index = 0;
        for (group, cat) in slices {
            for c in group.chars() {
                if pos.0 >= first_line && 
                    pos.1 >= first_col &&
                    pos.0 + first_line < area.height as usize &&
                    pos.1 + first_col < area.width as usize
                {
                    let mut style = palette.text();
                    if let Some(cat) = cat {
                        style = style.fg(palette.token_color(cat));
                    }
                    if select_range.contains(&index) {
                        style = style.bg(palette.secondary);
                    }
                    if index == cursor_idx {
                        style = style.reversed();
                    }
                    let cell_pos = Position {
                        x: pos.1 as u16 + area.x,
                        y: pos.0 as u16 + area.y,
                    };
                    if let Some(cell) = buf.cell_mut(cell_pos) {
                        cell.set_style(style);
                        if !c.is_control() {
                            cell.set_char(c);
                        }
                    }
                }
                if c == '\n' {
                    pos.0 += 1;
                    pos.1 = 0;
                } else if c == '\t' {
                    pos.1 += 4;
                } else {
                    pos.1 += 1;
                }
                if pos.0 + first_line >= area.height as usize {
                    return;
                }
                index += 1;
            }
        }
        if index == cursor_idx {
            let style = Style::default().reversed();
            let cell_pos = Position {
                x: pos.1 as u16 + area.x,
                y: pos.0 as u16 + area.y,
            };
            if let Some(cell) = buf.cell_mut(cell_pos) {
                cell.set_style(style);
            }
        }
    }
}

impl StatefulWidget for &Editor {
    type State = AppState;

    fn render(self, mut area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let height_us = area.height as usize;
        let first_line = self
            .cursor
            .0
            .saturating_sub(3 * height_us / 4)
            .min(self.buffer.len_lines().saturating_sub(height_us));
        let width_us = area.width as usize;
        let first_col = self.cursor.1.saturating_sub(3 * width_us / 4);

        let first_col_len = self.render_line_numbers(first_line, area, buf, &state.palette);

        area.x += first_col_len + 1;
        area.width = area.width.saturating_sub(first_col_len + 2);

        Line::from(Span::styled(
            self.lang.clone(),
            Style::default().fg(state.palette.muted),
        ))
        .right_aligned()
        .render(
            Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 1,
            },
            buf,
        );

        let syn = state.compiled_syntax.get(&self.lang);
        self.render_iter(first_line, first_col, area, buf, &state.palette, syn);

        if self.lang_popup.showing {
            self.lang_popup.render(area, buf, state);
        }
    }
}
