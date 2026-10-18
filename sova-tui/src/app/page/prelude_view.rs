use std::iter;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::Stylize,
    text::{Line, Text},
    widgets::{
        Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Widget,
    },
};
use sova_core::{scene::script::Script, schedule::SchedulerMessage};

use crate::{
    app::{AppPage, AppState},
    event::AppEvent,
    theme::Palette,
    widgets::editor::Editor,
};

const FRAME_WIDTH: u16 = 14;
const FRAME_HEIGHT: u16 = 3;
const LINE_HEADER_WIDTH: u16 = 4;

pub struct PreludeView {
    pub editor: Editor,
    pub editing: bool,
}

impl PreludeView {
    pub fn new() -> Self {
        Self {
            editor: Editor::new(),
            editing: false,
        }
    }

    pub fn update_content(&mut self, state: &AppState) {
        let Some(script) = state.selected_prelude() else {
            self.editor.set_text("");
            return;
        };
        self.editor.set_text(script.content());
        self.editor.lang = script.lang().to_string();
    }

    pub fn upload(&mut self, state: &mut AppState) {
        self.editor.flash.trigger();
        let mut prelude = state.client_state.scene.prelude.clone();
        let script = Script::new(self.editor.content(), self.editor.lang.clone());
        if state.selected_prelude < prelude.len() {
            prelude[state.selected_prelude] = script;
        } else {
            prelude.push(script);
        }
        state
            .events
            .send(SchedulerMessage::SetScenePrelude(prelude));
        self.editor.mark_unedited();
    }

    fn render_frame(
        area: Rect,
        buf: &mut Buffer,
        script: &Script,
        selected: bool,
        palette: &Palette,
    ) {
        let b = if selected {
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(palette.accent)
                .bg(palette.selection)
        } else {
            Block::default().bg(palette.surface)
        };

        b.render(area, buf);

        let mut lines = Vec::new();
        lines.push(Line::from(format!("{}", script.lang())));

        let layout = Layout::vertical([Constraint::Length(lines.len() as u16)])
            .flex(Flex::Center)
            .split(area.inner(Margin::new(2, 0)));

        let t = Text::from(lines);
        let mut p = Paragraph::new(t).style(palette.text());
        if selected {
            p = p.bold()
        }
        p.render(layout[0], buf);
    }

    fn render_header(area: Rect, buf: &mut Buffer, i: usize, selected: bool, palette: &Palette) {
        let b_color = if selected {
            palette.selection
        } else {
            palette.surface
        };

        let b = Block::default().bg(b_color);
        b.render(area, buf);

        let mut lines = Vec::new();
        lines.push(Line::from(format!("P{}", i)));

        let layout = Layout::vertical([Constraint::Length(lines.len() as u16)])
            .flex(Flex::Center)
            .split(area);

        let t = Text::from(lines);
        let mut p = Paragraph::new(t).centered().style(palette.text());
        if selected {
            p = p.bold()
        }
        p.render(layout[0], buf);
    }

    pub fn on_key_event(&mut self, event: KeyEvent, state: &mut AppState) {
        match event.code {
            KeyCode::Esc => {
                if self.editing {
                    if self.editor.lang_popup.showing {
                        self.editor.on_key_event(event, state);
                    } else {
                        self.editor.on_key_event(event, state);
                        self.editing = false;
                    }
                } else {
                    state.go_to(AppPage::Scene);
                }
            }

            KeyCode::Char('i') if !self.editing => {
                let mut prelude = state.client_state.scene.prelude.clone();
                prelude.insert(
                    (state.selected_prelude + 1).min(prelude.len()),
                    Default::default(),
                );
                state
                    .events
                    .send(SchedulerMessage::SetScenePrelude(prelude));
                self.update_content(state);
            }
            KeyCode::Char('b') if !self.editing => {
                let mut prelude = state.client_state.scene.prelude.clone();
                prelude.insert(
                    state.selected_prelude.min(prelude.len()),
                    Default::default(),
                );
                state
                    .events
                    .send(SchedulerMessage::SetScenePrelude(prelude));
                self.update_content(state);
            }
            KeyCode::Char('d') if !self.editing => {
                let Some(script) = state.selected_prelude() else {
                    return;
                };
                let mut prelude = state.client_state.scene.prelude.clone();
                prelude.insert(state.selected_prelude.min(prelude.len()), script.clone());
                state
                    .events
                    .send(SchedulerMessage::SetScenePrelude(prelude));
            }

            KeyCode::Delete if !self.editing => {
                if state.selected_frame().is_none() {
                    state
                        .events
                        .send(AppEvent::Negative("No frame to delete !".to_string()));
                    return;
                }
                let mut prelude = state.client_state.scene.prelude.clone();
                prelude.remove(state.selected_prelude);
                state
                    .events
                    .send(SchedulerMessage::SetScenePrelude(prelude));
                self.update_content(state);
            }

            KeyCode::Down if !self.editing => {
                state.go_to_next_prelude();
                self.update_content(state);
            }
            KeyCode::Up if !self.editing => {
                state.go_to_previous_prelude();
                self.update_content(state);
            }
            KeyCode::PageUp => {
                state.go_to_previous_prelude();
                self.update_content(state);
            }
            KeyCode::PageDown => {
                state.go_to_next_prelude();
                self.update_content(state);
            }
            KeyCode::Enter
                if event.modifiers.intersects(
                    KeyModifiers::CONTROL | KeyModifiers::SHIFT | KeyModifiers::ALT,
                ) =>
            {
                self.upload(state);
            }
            KeyCode::Char('s') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.upload(state);
            }
            KeyCode::Enter if !self.editing => {
                self.editing = true;
            }
            _ if self.editing => self.editor.on_key_event(event, state),
            _ => (),
        }
    }

    fn render_scripts(&mut self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let scene_image = &state.client_state.scene;

        let visible_lines = (area.height / (FRAME_HEIGHT + 1)) as usize;
        let mid_h = visible_lines / 2;
        let first_line = if state.selected_prelude > mid_h {
            std::cmp::min(
                state.selected_prelude - mid_h,
                state
                    .client_state
                    .scene
                    .prelude
                    .len()
                    .saturating_sub(visible_lines),
            )
        } else {
            0
        };

        let mut y_scroll_state =
            ScrollbarState::new(scene_image.n_lines().saturating_sub(visible_lines) + 1)
                .position(first_line);
        for (i, script) in scene_image.prelude[first_line..].iter().enumerate() {
            if i >= visible_lines {
                break;
            }
            let header_area = Rect::new(
                area.x,
                area.y + (i as u16) * (FRAME_HEIGHT + 1),
                LINE_HEADER_WIDTH,
                FRAME_HEIGHT,
            );
            let f_i = i + first_line;
            Self::render_header(
                header_area,
                buf,
                i,
                f_i == state.selected_prelude,
                &state.palette,
            );

            let frame_area = Rect::new(
                area.x + LINE_HEADER_WIDTH + 1,
                area.y + (i as u16) * (FRAME_HEIGHT + 1),
                FRAME_WIDTH,
                FRAME_HEIGHT,
            );

            let selected = f_i == state.selected_prelude;
            Self::render_frame(frame_area, buf, script, selected, &state.palette);
        }
        if visible_lines < scene_image.n_lines() {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .thumb_style(state.palette.selection)
                .style(state.palette.surface)
                .end_symbol(None)
                .begin_symbol(None);
            scrollbar.render(
                area.inner(Margin {
                    horizontal: 0,
                    vertical: 1,
                }),
                buf,
                &mut y_scroll_state,
            );
        }
    }
}

impl StatefulWidget for &mut PreludeView {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = Layout::horizontal([
            Constraint::Length(FRAME_WIDTH + LINE_HEADER_WIDTH + 2),
            Constraint::Min(0),
        ])
        .split(area);
        self.render_scripts(layout[0], buf, state);
        self.editor.show_cursor = self.editing;
        self.editor.render_with(layout[1], buf, &state.palette, &state.compiled_syntax, iter::empty());
    }
}
