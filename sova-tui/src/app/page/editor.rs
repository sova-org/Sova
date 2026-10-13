use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::{Buffer, Rect}, style::Stylize, text::{Line, Span, Text}, widgets::{Paragraph, StatefulWidget, Widget}};
use ratatui_themes::Style;
use ropey::Rope;
use sova_core::{scene::script::Script, schedule::{ActionTiming, SchedulerMessage}};

use crate::{app::{AppPage, AppState}, compiled_syntax::CompiledSyntax, flash::Flash, theme::Palette, widgets::list_choice::ListChoice};

const EDITOR_FLASH_MS : u64 = 100;

pub struct Editor {
    pub buffer: Rope,
    cursor: (usize, usize),
    selection_begin: (usize, usize),
    selection_end: (usize, usize),
    pub lang: String,
    pub flash: Flash,
    pub show_cursor: bool,
    pub lang_popup: ListChoice
}

impl Editor {
    pub fn new() -> Self {
        let mut popup = ListChoice::default();
        popup.title = "Language".to_string();
        popup.content = "Select language for script".to_string();
        Editor { 
            buffer: Rope::default(), 
            cursor: (0, 0),
            selection_begin: (0, 0),
            selection_end: (0, 0),
            lang: String::new(),
            flash: Flash::new(Duration::from_millis(EDITOR_FLASH_MS)),
            show_cursor: true,
            lang_popup: popup
        }
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
        if self.lang_popup.showing {
            self.lang_popup.process_event(event);
            if self.lang_popup.done {
                self.lang = self.lang_popup.selected();
            }
            return;
        }
        //self.cursor = self.cursor.min(self.buffer.len_chars());
        let char_idx = self.cursor_char_idx();
        match event.code {
            KeyCode::Char('l') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.lang_popup.choices = state.client_state.languages.iter().map(|l| l.name.clone()).collect();
                self.lang_popup.select(&self.lang);
                self.lang_popup.open();
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
            _ => ()
        }
    }

    fn render_line_numbers(&self, from: usize, area: Rect, buf: &mut Buffer, palette: &Palette) -> u16 {
        let first_col_len = self.buffer.len_lines().to_string().len().max(2) as u16;
        for l in 0..self.buffer.len_lines().min(area.height as usize) {
            let real_l = from + l;
            let line_index = Span::styled(
                real_l.to_string(), 
                Style::default().fg(
                    if real_l == self.cursor.0 { palette.foreground } else { palette.muted }
                ).bg(palette.background)
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

    fn render_buffer(&self, first_line: usize, first_col: usize, area: Rect, buf: &mut Buffer, palette: &Palette, syntax: Option<&CompiledSyntax>) {
        let mut lines : Vec<Line> = if let Some(syn) = syntax {
            self.buffer.lines().map(|line| {
                let line_str = line.to_string();
                let split = syn.split(&line_str);
                split.into_iter().map(|(sub, cat)| {
                    Span::styled(sub, if let Some(cat) = cat { 
                        palette.token_color(cat)
                    } else { 
                        palette.foreground 
                    })
                }).collect()
            }).collect()
        } else {
            self.buffer.lines().map(|line| {
                let spans = vec![Span::styled(line.to_string(), palette.text())];
                Line::from(spans)
            }).collect()
        };

        if self.cursor.0 < lines.len() {
            let line = std::mem::take(&mut lines[self.cursor.0]);
            lines[self.cursor.0] = line.bg(palette.selection);
        }

        let text = Text::from(lines);
        let mut p = Paragraph::new(text).bg(palette.surface).scroll((first_line as u16, first_col as u16));

        if self.flash.is_active() {
            p = p.reversed();
        }

        p.render(area, buf);
    }

    fn render_cursor(&self, first_line: usize, first_col: usize, area: Rect, buf: &mut Buffer, palette: &Palette) {
        let char_idx = self.cursor_char_idx();
        let char = if char_idx < self.buffer.len_chars() {
            let at_idx = self.buffer.char(char_idx);
            if at_idx.is_control() { ' ' } else { at_idx }
        } else {
            ' '
        };
        let cursor_render = Span::styled(char.to_string(), palette.text().reversed());
        cursor_render.render(Rect { 
            x: area.x + (self.cursor.1.saturating_sub(first_col) as u16), 
            y: area.y + (self.cursor.0.saturating_sub(first_line) as u16), 
            width: 1, height: 1 
        }, buf);
    }
}

impl StatefulWidget for &Editor {
    type State = AppState;

    fn render(self, mut area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let height_us = area.height as usize;
        let first_line = self.cursor.0
            .saturating_sub(3 * height_us / 4)
            .min(self.buffer.len_lines().saturating_sub(height_us));
        let width_us = area.width as usize;
        let first_col = self.cursor.1
            .saturating_sub(3 * width_us / 4);

        let first_col_len = self.render_line_numbers(first_line, area, buf, &state.palette);

        area.x += first_col_len + 1;
        area.width = area.width.saturating_sub(first_col_len + 2);

        Line::from(Span::styled(self.lang.clone(), Style::default().fg(state.palette.muted)))
            .right_aligned()
            .render(Rect { x: area.x, y: area.y, width: area.width, height: 1 }, buf);

        let syn = state.compiled_syntax.get(&self.lang);
        self.render_buffer(first_line, first_col, area, buf, &state.palette, syn);
        if self.show_cursor { 
            self.render_cursor(first_line, first_col, area, buf, &state.palette);
        }

        if self.lang_popup.showing {
            self.lang_popup.render(area, buf, state);
        }
    }
}

pub struct ScriptEditor {
    pub editor: Editor
}

impl ScriptEditor {
    pub fn new() -> Self {
        Self { editor: Editor::new() }
    }

    pub fn update_content(&mut self, state: &AppState) {
        let Some(frame) = state.selected_frame() else {
            self.editor.buffer = Rope::default();
            return;
        };
        self.editor.set_text(frame.script().content());
        self.editor.lang = frame.script().lang().to_string();
    }

    pub fn upload(&mut self, state: &mut AppState) {
        self.editor.flash.trigger();
        state.events.send(SchedulerMessage::SetScript(
            state.selected.0, 
            state.selected.1, 
            Script::new(self.editor.buffer.to_string(), self.editor.lang.clone()), 
            ActionTiming::Immediate
        ));
    }

    pub fn on_key_event(&mut self, event: KeyEvent, state: &mut AppState) {
        match event.code {
            KeyCode::Enter if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.upload(state);
            }
            KeyCode::PageUp => {
                state.go_to_previous_frame();
            }
            KeyCode::PageDown => {
                state.go_to_next_frame();
            }
            KeyCode::Esc => {
                state.go_to(AppPage::Scene);
            }
            _ => {
                self.editor.on_key_event(event, state);
            }
        }
    }
}

impl StatefulWidget for &ScriptEditor {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.editor.render(area, buf, state);
    }
}