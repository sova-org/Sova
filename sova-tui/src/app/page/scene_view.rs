use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Margin, Rect}, style::{Color, Stylize}, text::{Line, Text}, widgets::{Block, Paragraph, StatefulWidget, Widget}};
use sova_core::scene::Frame;

use crate::{app::AppState, event::AppEvent};

const FRAME_WIDTH : u16 = 12;
const FRAME_HEIGHT : u16 = 4;
const FRAME_HEADER_HEIGHT : u16 = 1;
const LINE_HEADER_WIDTH : u16 = 5;

pub struct SceneView;

impl SceneView {

    fn render_frame(area: Rect, buf: &mut Buffer, _j: usize, frame: &Frame, selected: bool) {
        let b_color = if selected {
            Color::Rgb(127, 0, 0)
        } else {
            Color::Rgb(127, 127, 127)
        };

        let b = Block::default().bg(b_color);
        let block_area = b.inner(area);
        b.render(area, buf);

        let mut lines = Vec::new();
        lines.push(Line::from(format!("{}' x {}", frame.duration, frame.repetitions)));
        lines.push(Line::from(format!("{}", frame.script().lang())));

        let layout = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(lines.len() as u16),
            Constraint::Min(0)
        ]).split(block_area.inner(Margin::new(1, 0)));

        let t = Text::from(lines);
        let p = Paragraph::new(t);
        p.render(layout[1], buf);
    }

    fn render_frame_header(area: Rect, buf: &mut Buffer, i: usize, selected: bool) {
        let b_color = if selected {
            Color::Rgb(127, 0, 0)
        } else {
            Color::Rgb(127, 127, 127)
        };

        let b = Block::default().bg(b_color);
        b.render(area, buf);

        let mut lines = Vec::new();
        lines.push(Line::from(format!("F{}", i)));

        let layout = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(lines.len() as u16),
            Constraint::Min(0)
        ]).split(area);

        let t = Text::from(lines);
        let p = Paragraph::new(t).centered();
        p.render(layout[1], buf);
    }

    fn render_line_header(area: Rect, buf: &mut Buffer, i: usize, line: &sova_core::scene::Line, selected: bool) {
        let b_color = if selected {
            Color::Rgb(127, 0, 0)
        } else {
            Color::Rgb(127, 127, 127)
        };

        let b = Block::default().bg(b_color);
        b.render(area, buf);

        let mut lines = Vec::new();
        lines.push(Line::from(format!("L{}", i)));
        lines.push(Line::from(format!("{}{}{}", 
            if line.manual { "M" } else { " " },
            if line.looping { "L" } else { " " },
            if line.trailing { "T" } else { " " },
        )));

        let layout = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(lines.len() as u16),
            Constraint::Min(0)
        ]).split(area);

        let t = Text::from(lines);
        let p = Paragraph::new(t).centered();
        p.render(layout[1], buf);
    }

    pub fn on_key_event(event: KeyEvent, state: &mut AppState) {
        let prev = state.selected;
        match event.code {
            KeyCode::Right => {
                let max_f = state.selected_line().map(|l| l.n_frames().saturating_sub(1)).unwrap_or(0);
                state.selected.1 = std::cmp::min(state.selected.1 + 1, max_f);
            }
            KeyCode::Left => {
                state.selected.1 = state.selected.1.saturating_sub(1)
            }
            KeyCode::Down => {
                let max_l = state.scene_image.n_lines().saturating_sub(1);
                state.selected.0 = std::cmp::min(state.selected.0 + 1, max_l);
                state.selected.1 = std::cmp::min(
                    state.selected.1, 
                    state.selected_line().map(|l| l.n_frames().saturating_sub(1)).unwrap_or_default()
                );
            }
            KeyCode::Up => {
                state.selected.0 = state.selected.0.saturating_sub(1);
                state.selected.1 = std::cmp::min(
                    state.selected.1, 
                    state.selected_line().map(|l| l.n_frames().saturating_sub(1)).unwrap_or_default()
                );
            }
            _ => ()
        }
        if prev != state.selected {
            state.events.send(AppEvent::RefreshScript);
        }
    }

}

impl StatefulWidget for SceneView {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut max_frames = 0;
        for line in state.scene_image.lines.iter() {
            max_frames = std::cmp::max(max_frames, line.n_frames());
        }
        let visible_lines = (
            area.height.saturating_sub(FRAME_HEADER_HEIGHT + 1) / 
            (FRAME_HEIGHT + 1)
        ) as usize;
        let visible_frames = (
            area.width.saturating_sub(LINE_HEADER_WIDTH + 1) / 
            (FRAME_WIDTH + 1)
        ) as usize;
        let mid_h = visible_lines / 2;
        let mid_w = visible_frames / 2;
        let first_line = if state.selected.0 > mid_h {
            std::cmp::min(state.selected.0 - mid_h, state.scene_image.n_lines().saturating_sub(visible_lines))
        } else {
            0
        };
        let first_frame = if state.selected.1 > mid_w {
            std::cmp::min(state.selected.1 - mid_w, max_frames.saturating_sub(visible_frames))
        } else {
            0
        };
        for j in 0..max_frames {
            if j >= visible_frames {
                break;
            }
            let header_area = Rect::new(
                LINE_HEADER_WIDTH + 1 + (j as u16) * (FRAME_WIDTH + 1), 
                0, 
                FRAME_WIDTH, 
                FRAME_HEADER_HEIGHT
            );
            let f_j = j + first_frame;
            Self::render_frame_header(header_area, buf, f_j, f_j == state.selected.1);
        }
        for (i, line) in state.scene_image.lines[first_line..].iter().enumerate() {
            if i >= visible_lines {
                break;
            }
            let header_area = Rect::new(
                0, 
                FRAME_HEADER_HEIGHT + 1 + (i as u16) * (FRAME_HEIGHT + 1), 
                LINE_HEADER_WIDTH, 
                FRAME_HEIGHT
            );
            let f_i = i + first_line;
            Self::render_line_header(header_area, buf, f_i, line, f_i == state.selected.0);
            if line.n_frames() <= first_frame {
                continue;
            }
            for (j, frame) in line.frames[first_frame..].iter().enumerate() {
                if j >= visible_frames {
                    break;
                }
                let frame_area = Rect::new(
                    LINE_HEADER_WIDTH + 1 + (j as u16) * (FRAME_WIDTH + 1), 
                    FRAME_HEADER_HEIGHT + 1 + (i as u16) * (FRAME_HEIGHT + 1), 
                    FRAME_WIDTH, 
                    FRAME_HEIGHT
                );
                
                let f_j = j + first_frame;
                let selected = (f_i, f_j) == state.selected;
                Self::render_frame(
                    frame_area, 
                    buf, 
                    f_j, 
                    frame, 
                    selected
                );
            }
        }
    }
}