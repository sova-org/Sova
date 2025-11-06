use std::cmp::min;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
use sova_core::schedule::{ActionTiming, SchedulerMessage};

use crate::{app::AppState, event::AppEvent, popup::PopupValue};

const LINE_RECT_WIDTH: f64 = 16.0;
const LINE_RECT_HEIGHT: f64 = 3.0;

const FRAME_RECT_HEIGHT: f64 = 4.0;

fn set_selected(state: &mut AppState, line_index: usize, frame_index: usize) {
    let before = state.selected;
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
    if before != state.selected {
        state.events.send(AppEvent::ChangeScript);
    }
}

#[derive(Default)]
pub struct SceneWidget;

impl SceneWidget {

    pub fn compute_start_coordinates(&self, state: &AppState, area: Rect) -> (f64, f64) {
        let (width, height) = (f64::from(area.width), f64::from(area.height));
        let x_selected = 1.0 + (state.selected.0 as f64) * LINE_RECT_WIDTH;
        let y_selected = height - LINE_RECT_HEIGHT;
        let y_selected = y_selected - (FRAME_RECT_HEIGHT * (state.selected.1 + 1) as f64);

        let x = if x_selected + LINE_RECT_WIDTH > width {
            x_selected + LINE_RECT_WIDTH - width
        } else {
            0.0
        };
        let y = if y_selected < 0.0 {
            y_selected 
        } else {
            0.0
        };

        (x,y)
    }

    pub fn process_event(&mut self, state: &mut AppState, event: KeyEvent) {
        let selected = state.selected;
        match event.code {
            KeyCode::Up => set_selected(state, selected.0, selected.1.saturating_sub(1)),
            KeyCode::Down => set_selected(state, selected.0, selected.1 + 1),
            KeyCode::Left => set_selected(state, selected.0.saturating_sub(1), selected.1),
            KeyCode::Right => set_selected(state, selected.0 + 1, selected.1),
            KeyCode::Char('i') => {
                let (line_index, frame_index) = state.selected;
                let msg = if state.scene_image.is_empty() || state.scene_image.line(line_index).unwrap().is_empty() {
                    SchedulerMessage::AddFrame(line_index, frame_index, Default::default(), ActionTiming::Immediate)
                } else {
                    SchedulerMessage::AddFrame(line_index, frame_index + 1, Default::default(), ActionTiming::Immediate)
                };
                state.events.send(msg.into());
            } 
            KeyCode::Char('l') => {
                let (line_index, _) = state.selected;
                let msg = if state.scene_image.is_empty() {
                    SchedulerMessage::AddLine(0, Default::default(), ActionTiming::Immediate)
                } else {
                    SchedulerMessage::AddLine(line_index + 1, Default::default(), ActionTiming::Immediate)
                };
                state.events.send(msg.into());
            } 
            KeyCode::Char('r') => {
                let (line_index, frame_index) = state.selected;
                if event.modifiers == KeyModifiers::CONTROL {
                    if !state.scene_image.is_empty() {
                        state.events.send(SchedulerMessage::RemoveLine(line_index, ActionTiming::Immediate).into());
                    }
                } else {
                    if state.selected_frame().is_some() {
                        state.events.send(SchedulerMessage::RemoveFrame(line_index, frame_index, ActionTiming::Immediate).into());
                    }
                }
            }
            KeyCode::Char('d') if state.selected_frame().is_some() => {
                let (line_index, frame_index) = state.selected;
                let cloned = state.selected_frame().unwrap().clone();
                let dur = cloned.duration;
                state.events.send(AppEvent::Popup(
                    "Frame duration".to_owned(), 
                    "Which frame duration (beats) to apply to frame ?".to_owned(), 
                    PopupValue::Float(dur), 
                    Box::new(move |state, value| {
                        let mut new = cloned;
                        new.duration = value.float();
                        state.events.send(SchedulerMessage::SetFrames(vec![(
                            line_index, frame_index, new
                        )], ActionTiming::Immediate).into());
                    })
                ));
            }
            KeyCode::Char('m') if state.selected_frame().is_some() => {
                let (line_index, frame_index) = state.selected;
                let mut cloned = state.selected_frame().unwrap().clone();
                cloned.enabled = !cloned.enabled;
                state.events.send(SchedulerMessage::SetFrames(vec![(
                    line_index, frame_index, cloned
                )], ActionTiming::Immediate).into());
            }
            KeyCode::Char('y') if state.selected_frame().is_some() => {
                let (line_index, frame_index) = state.selected;
                let msg = if event.modifiers == KeyModifiers::CONTROL {
                    SchedulerMessage::AddLine(line_index + 1, state.scene_image.line(line_index).unwrap().clone(), ActionTiming::Immediate)
                } else {
                    SchedulerMessage::AddFrame(line_index, frame_index + 1, state.selected_frame().unwrap().clone(), ActionTiming::Immediate)
                };
                state.events.send(msg.into());
            }
            _ => (),
        }
    }

    pub fn get_help() -> &'static str {
        "\
        I: insert frame after  R: remove frame     M: toggle frame\n\
        L: insert line after   C-R: remove line    Y: copy frame after\n\
        Arrows: move           D: change duration  C-Y: copy line after\
        "
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
                    Color::LightMagenta
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
                text.light_magenta().bold()
            } else {
                Span::from(text)
            };
            ctx.print(x_offset + text_offset, y_top + LINE_RECT_HEIGHT / 2.0, text);

            let line_pos = pos.get(line_index).cloned().unwrap_or((0, 0));

            for (frame_index, frame) in line.frames.iter().enumerate() {
                let selected_frame = state.selected == (line_index, frame_index);
                let color = if selected_frame {
                    Color::LightMagenta
                } else {
                    Color::White
                };

                let y_frame = y_top - (FRAME_RECT_HEIGHT * (frame_index + 1) as f64);
                let rect = Rectangle {
                    x: x_offset,
                    y: y_frame,
                    width: LINE_RECT_WIDTH,
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

                let (mut frame_name, frame_infos) = if selected_frame {
                    (frame_name.light_magenta().bold(), frame_infos.light_magenta().bold())
                } else {
                    (Span::from(frame_name), Span::from(frame_infos))
                };

                if frame_index == line_pos.0 {
                    frame_name = frame_name.bg(Color::White).fg(Color::Black);
                }
                if !frame.enabled {
                    frame_name = frame_name.crossed_out().gray();
                }

                let x = 2.0 + x_offset;
                ctx.print(x, y_frame + 2.0, frame_name);
                ctx.print(x, y_frame + 1.0, frame_infos);
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
        let (x,y) = self.compute_start_coordinates(state, area);
        set_selected(state, state.selected.0, state.selected.1);
        Canvas::default()
            .marker(Marker::Braille)
            .paint(|ctx| {
                self.draw_scene(state, ctx, area);
            })
            .x_bounds([x, x + f64::from(area.width)])
            .y_bounds([y, y + f64::from(area.height)])
            .render(area, buf);
    }
}
