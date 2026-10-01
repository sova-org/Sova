use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Margin, Rect}, style::Stylize, symbols::scrollbar::Set, text::{Line, Text}, widgets::{Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget}};
use sova_core::scene::Frame;

use crate::{app::AppState, event::AppEvent, theme::Palette};

const FRAME_WIDTH : u16 = 14;
const FRAME_HEIGHT : u16 = 4;
const FRAME_HEADER_HEIGHT : u16 = 1;
const LINE_HEADER_WIDTH : u16 = 5;

pub struct SceneView;

impl SceneView {

    fn render_frame(area: Rect, buf: &mut Buffer, _j: usize, frame: &Frame, selected: bool, palette: &Palette) {
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
        lines.push(Line::from(format!("{}' x {}", frame.duration, frame.repetitions)));
        lines.push(Line::from(format!("{}", frame.script().lang())));

        let layout = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(lines.len() as u16),
            Constraint::Min(0)
        ]).split(area.inner(Margin::new(2, 0)));

        let t = Text::from(lines);
        let mut p = Paragraph::new(t).style(palette.text());
        if selected {
            p = p.bold()
        }
        p.render(layout[1], buf);
    }

    fn render_frame_header(area: Rect, buf: &mut Buffer, i: usize, selected: bool, palette: &Palette) {
        let b_color = if selected {
            palette.selection
        } else {
            palette.surface
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
        let mut p = Paragraph::new(t).centered().style(palette.text());
        if selected {
            p = p.bold()
        }
        p.render(layout[1], buf);
    }

    fn render_line_header(
        area: Rect, 
        buf: &mut Buffer, 
        i: usize, 
        line: &sova_core::scene::Line, 
        selected: bool, 
        palette: &Palette
    ) {
        let b_color = if selected {
            palette.selection
        } else {
            palette.surface
        };

        let b = Block::default().bg(b_color);
        b.render(area, buf);

        let mut lines = Vec::new();
        lines.push(Line::from(format!("L{}", i)));
        lines.push(Line::from(format!("{}{}{}", 
            if line.manual { nerd_font_symbols::oct::OCT_GEAR } else { " " },
            if line.looping { nerd_font_symbols::oct::OCT_SYNC } else { " " },
            if line.trailing { nerd_font_symbols::md::MD_ARROW_COLLAPSE_RIGHT } else { " " },
        )));

        let layout = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(lines.len() as u16),
            Constraint::Min(0)
        ]).split(area);

        let t = Text::from(lines);
        let mut p = Paragraph::new(t).centered().style(palette.text());;
        if selected {
            p = p.bold()
        }
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
        let mut x_scroll_state = ScrollbarState::new(max_frames.saturating_sub(visible_frames) + 1)
            .position(first_frame);
        let mut y_scroll_state = ScrollbarState::new(state.scene_image.n_lines().saturating_sub(visible_lines) + 1)
            .position(first_line);
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
            Self::render_frame_header(header_area, buf, f_j, f_j == state.selected.1, &state.palette);
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
            Self::render_line_header(header_area, buf, f_i, line, f_i == state.selected.0, &state.palette);
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
                    selected,
                    &state.palette
                );
            }
        }
        if visible_lines < state.scene_image.n_lines() {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(state.palette.surface);
            scrollbar.render(area.inner(Margin { horizontal: 0, vertical: 1 }), buf, &mut y_scroll_state);
        }
        if visible_frames < max_frames {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                .style(state.palette.surface);
            scrollbar.render(area.inner(Margin { horizontal: 1, vertical: 0 }), buf, &mut x_scroll_state);
        }
    }
}