use std::cmp::min;

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    symbols::Marker,
    text::Span,
    widgets::{
        StatefulWidget, Widget,
        canvas::{Canvas, Context},
    },
};

use crate::app::AppState;

const LINE_RECT_WIDTH: f64 = 16.0;
const LINE_RECT_HEIGHT: f64 = 3.0;

const FRAME_RECT_WIDTH: f64 = 16.0;
const FRAME_RECT_HEIGHT: f64 = 5.0;

fn set_selected(state: &mut AppState, line_index: usize, frame_index: usize) {
    if state.scene_image.is_empty() {
        state.selected = (0, 0);
        return;
    }
    let line_index = min(line_index, state.scene_image.n_lines() - 1);
    let line = state.scene_image.line(line_index).unwrap();
    if line.is_empty() {
        state.selected = (line_index, 0);
        return;
    }
    let frame_index = min(frame_index, line.n_frames() - 1);
    state.selected = (line_index, frame_index);
}

#[derive(Default)]
pub struct SceneWidget;

impl SceneWidget {
    pub fn process_event(&mut self, state: &mut AppState, event: KeyEvent) -> Result<()> {
        let selected = state.selected;
        match event.code {
            KeyCode::Up => set_selected(state, selected.0, selected.1.saturating_sub(1)),
            KeyCode::Down => set_selected(state, selected.0, selected.1 + 1),
            KeyCode::Left => set_selected(state, selected.0.saturating_sub(1), selected.1),
            KeyCode::Right => set_selected(state, selected.0 + 1, selected.1),
            _ => (),
        }
        Ok(())
    }

    pub fn draw_scene(&self, state: &AppState, ctx: &mut Context, area: Rect) {
        use ratatui::widgets::canvas::*;

        let top = f64::from(area.height);

        let mut on_top = Vec::new();
        let pos = &state.positions;

        for (line_index, line) in state.scene_image.lines.iter().enumerate() {
            let x_offset = 1.0 + line_index as f64 * LINE_RECT_WIDTH;
            let y_top = top - LINE_RECT_HEIGHT;
            let selected_line = state.selected.0 == line_index;
            let rect = Rectangle {
                x: x_offset,
                y: y_top,
                width: LINE_RECT_WIDTH,
                height: LINE_RECT_HEIGHT,
                color: if selected_line {
                    Color::Magenta
                } else {
                    Color::White
                },
            };
            if selected_line {
                on_top.push(rect);
            } else {
                ctx.draw(&rect);
            }
            let text = format!("Line {}", line_index);
            let text_offset = 1.0 + (LINE_RECT_WIDTH / 2.0) - (text.len() as f64 / 2.0);
            let text = if selected_line {
                text.magenta().bold()
            } else {
                Span::from(text)
            };
            ctx.print(x_offset + text_offset, y_top + LINE_RECT_HEIGHT / 2.0, text);

            let line_pos = pos.get(line_index).cloned().unwrap_or((0, 0));

            for (frame_index, frame) in line.frames.iter().enumerate() {
                let selected_frame = state.selected == (line_index, frame_index);
                let color = if selected_frame {
                    Color::Magenta
                } else {
                    Color::White
                };

                let y_frame = y_top - (FRAME_RECT_HEIGHT * (frame_index + 1) as f64);
                let rect = Rectangle {
                    x: x_offset,
                    y: y_frame,
                    width: FRAME_RECT_WIDTH,
                    height: FRAME_RECT_HEIGHT,
                    color,
                };
                if selected_frame {
                    on_top.push(rect);
                } else {
                    ctx.draw(&rect);
                }

                let frame_name = format!("Frame {}", frame_index);
                let frame_infos = format!("{:.2} x {}", frame.duration, frame.repetitions);

                let (mut frame_name, mut frame_infos) = if selected_frame {
                    (frame_name.magenta().bold(), frame_infos.magenta().bold())
                } else {
                    (Span::from(frame_name), Span::from(frame_infos))
                };

                if frame_index == line_pos.0 {
                    frame_name = frame_name.bg(Color::White).fg(Color::Black);
                }

                let x = 2.0 + x_offset;
                ctx.print(x, y_frame + 3.0, frame_name);
                ctx.print(x, y_frame + 2.0, frame_infos);
            }
        }

        for rect in on_top {
            ctx.draw(&rect);
        }
    }
}

impl StatefulWidget for &SceneWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Canvas::default()
            .marker(Marker::Braille)
            .x_bounds([0.0, f64::from(area.width)])
            .y_bounds([0.0, f64::from(area.height)])
            .paint(|ctx| {
                self.draw_scene(state, ctx, area);
            })
            .render(area, buf);
    }
}
