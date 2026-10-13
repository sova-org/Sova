use std::collections::BTreeMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{buffer::Buffer, layout::{Constraint, Flex, Layout, Margin, Rect}, style::Stylize, text::{Line, Text}, widgets::{Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget}};
use ratatui_themes::{Color, Style};
use sova_core::{scene::{ExecutionMode, Frame}, schedule::{ActionTiming, SchedulerMessage}};

use crate::{app::{AppPage, AppState}, event::AppEvent, theme::Palette, widgets::popup::PopupValue};

const FRAME_WIDTH : u16 = 14;
const FRAME_HEIGHT : u16 = 4;
const FRAME_HEADER_HEIGHT : u16 = 1;
const LINE_HEADER_WIDTH : u16 = 5;

pub struct SceneView;

impl SceneView {

    fn render_frame(area: Rect, buf: &mut Buffer, col: Color, frame: &Frame, selected: bool, rep: Option<usize>, palette: &Palette) {
        let b = if selected {
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(palette.accent)
                .bg(palette.selection)
        } else {
            Block::default().bg(palette.surface)
        };

        b.render(area, buf);

        if let Some(curr_rep) = rep {
            let active_layout = Layout::vertical([Constraint::Min(0), Constraint::Length(1)])
                .split(area);
            if frame.repetitions > 1 {
                Paragraph::new(format!("{}/{}", curr_rep + 1, frame.repetitions))
                    .right_aligned()
                    .fg(palette.surface)
                    .block(Block::default().bg(col))
                    .render(active_layout[1], buf);
            } else {
                Block::default().bg(col)
                    .fg(palette.surface)
                    .render(active_layout[1], buf);
            }
        }

        let mut lines = Vec::new();
        lines.push(Line::from(format!("{}' x {}", frame.duration, frame.repetitions)));
        lines.push(Line::from(format!("{}", frame.script().lang())));

        let layout = Layout::vertical([
            Constraint::Length(lines.len() as u16),
        ]).flex(Flex::Center).split(area.inner(Margin::new(2, 0)));

        let t = Text::from(lines);
        let mut p = Paragraph::new(t).style(
            if frame.enabled { palette.text() } else { Style::default().fg(palette.muted).crossed_out() }
        );
        if selected {
            p = p.bold()
        }
        p.render(layout[0], buf);
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
            Constraint::Length(lines.len() as u16),
        ]).flex(Flex::Center).split(area);

        let t = Text::from(lines);
        let mut p = Paragraph::new(t).centered().style(palette.text());
        if selected {
            p = p.bold()
        }
        p.render(layout[0], buf);
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
            Constraint::Length(lines.len() as u16),
        ]).flex(Flex::Center).split(area);

        let t = Text::from(lines);
        let mut p = Paragraph::new(t).centered().style(palette.text());
        if selected {
            p = p.bold()
        }
        p.render(layout[0], buf);
    }

    pub fn on_key_event(event: KeyEvent, state: &mut AppState) {
        let prev = state.selected;
        match event.code {
            KeyCode::Esc => {
                state.popup(
                    "Disconnect ?", "Quit session ?", false,
                    |state, x| {
                        if bool::from(x) {
                            state.client_state = Default::default();
                            state.events.send(AppEvent::Disconnected)
                        }
                    },
                );
            }
            KeyCode::Char('t') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                let tempo = state.client_state.clock.tempo;
                state.popup("Tempo", "Set tempo value", tempo, |state, value| {
                    state.events.send(SchedulerMessage::SetTempo(value.into(), ActionTiming::Immediate));
                });
            }
            KeyCode::Char('q') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                let quantum = state.client_state.clock.quantum;
                state.popup("Quantum", "Set quantum value", quantum, |state, value| {
                    state.events.send(SchedulerMessage::SetQuantum(value.into(), ActionTiming::Immediate));
                });
            }

            KeyCode::Char('m') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                let mode = state.client_state.scene.mode;
                let choices = vec![
                    ExecutionMode::Free.to_string(), 
                    ExecutionMode::AtQuantum.to_string(), 
                    ExecutionMode::LongestLine.to_string()
                ];
                let index = match mode {
                    ExecutionMode::Free => 0,
                    ExecutionMode::AtQuantum => 1,
                    ExecutionMode::LongestLine => 2,
                };
                state.popup("Mode", "Set execution mode", PopupValue::Choice(index, choices), |state, value| {
                    let value = ExecutionMode::from(String::from(value));
                    state.events.send(
                        SchedulerMessage::SetSceneMode(value, ActionTiming::Immediate)
                    );
                });
            }

            KeyCode::Char('i') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                state.events.send(
                    SchedulerMessage::AddLine(state.selected.0 + 1, Default::default(), ActionTiming::Immediate)
                );
            }
            KeyCode::Char('b') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                state.events.send(
                    SchedulerMessage::AddLine(state.selected.0, Default::default(), ActionTiming::Immediate)
                );
            }
            KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                let Some(line) = state.selected_line() else {
                    return;
                };
                state.events.send(
                    SchedulerMessage::AddLine(state.selected.0 + 1, line.clone(), ActionTiming::Immediate)
                );
            }

            KeyCode::Char('L') => {
                let Some(line) = state.selected_line() else {
                    return;
                };
                let mut line = line.configuration();
                line.trailing = !line.trailing;
                state.events.send(
                    SchedulerMessage::ConfigureLines(vec![(state.selected.0, line)], ActionTiming::Immediate)
                );
            }
            KeyCode::Char('l') => {
                let Some(line) = state.selected_line() else {
                    return;
                };
                let mut line = line.configuration();
                line.looping = !line.looping;
                state.events.send(
                    SchedulerMessage::ConfigureLines(vec![(state.selected.0, line)], ActionTiming::Immediate)
                );
            }
            KeyCode::Char('m') => {
                let Some(line) = state.selected_line() else {
                    return;
                };
                let mut line = line.configuration();
                line.manual = !line.manual;
                state.events.send(
                    SchedulerMessage::ConfigureLines(vec![(state.selected.0, line)], ActionTiming::Immediate)
                );
            }

            KeyCode::Char('i') => {
                state.events.send(
                    SchedulerMessage::AddFrame(state.selected.0, state.selected.1 + 1, Default::default(), ActionTiming::Immediate)
                );
            }
            KeyCode::Char('b') => {
                state.events.send(
                    SchedulerMessage::AddFrame(state.selected.0, state.selected.1, Default::default(), ActionTiming::Immediate)
                );
            }
            KeyCode::Char('d') => {
                let Some(frame) = state.selected_frame() else {
                    return;
                };
                state.events.send(
                    SchedulerMessage::AddFrame(state.selected.0, state.selected.1 + 1, frame.clone(), ActionTiming::Immediate)
                );
            }

            KeyCode::Char('r') => {
                let Some(frame) = state.selected_frame() else {
                    return;
                };
                state.popup("Repetitions", "How many repetitions ?", frame.repetitions as i64, |state, value| {
                    let value : i64 = value.into();
                    if value < 1 {
                        return;
                    }
                    let Some(frame) = state.selected_frame() else {
                        return;
                    };
                    let mut frame = frame.clone();
                    frame.repetitions = value as usize;
                    state.events.send(
                        SchedulerMessage::SetFrames(vec![(state.selected.0, state.selected.1, frame)], ActionTiming::Immediate)
                    );
                });
            }
            KeyCode::Char('t') => {
                let Some(frame) = state.selected_frame() else {
                    return;
                };
                state.popup("Duration", "Frame duration in beats", frame.duration, |state, value| {
                    let value : f64 = value.into();
                    if value < 0.0 {
                        return;
                    }
                    let Some(frame) = state.selected_frame() else {
                        return;
                    };
                    let mut frame = frame.clone();
                    frame.duration = value;
                    state.events.send(
                        SchedulerMessage::SetFrames(vec![(state.selected.0, state.selected.1, frame)], ActionTiming::Immediate)
                    );
                });
            }
            KeyCode::Char('e') => {
                let Some(frame) = state.selected_frame() else {
                    return;
                };
                let mut frame = frame.clone();
                frame.enabled = !frame.enabled;
                state.events.send(
                    SchedulerMessage::SetFrames(vec![(state.selected.0, state.selected.1, frame)], ActionTiming::Immediate)
                );
            }
            
            KeyCode::Delete if event.modifiers.contains(KeyModifiers::CONTROL) => {
                if state.selected_line().is_none() {
                    state.events.send(AppEvent::Negative("No line to delete !".to_string()));
                    return;
                }
                state.events.send(
                    SchedulerMessage::RemoveLine(state.selected.0, ActionTiming::Immediate)
                );
            }
            KeyCode::Delete => {
                if state.selected_frame().is_none() {
                    state.events.send(AppEvent::Negative("No frame to delete !".to_string()));
                    return;
                }
                state.events.send(
                    SchedulerMessage::RemoveFrame(state.selected.0, state.selected.1, ActionTiming::Immediate)
                );
            }

            KeyCode::Right => {
                let max_f = state.selected_line().map(|l| l.n_frames().saturating_sub(1)).unwrap_or(0);
                state.selected.1 = std::cmp::min(state.selected.1 + 1, max_f);
            }
            KeyCode::Left => {
                state.selected.1 = state.selected.1.saturating_sub(1)
            }
            KeyCode::Down => {
                let max_l = state.client_state.scene.n_lines().saturating_sub(1);
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
            KeyCode::PageUp => {
                state.go_to_previous_frame();
            }
            KeyCode::PageDown => {
                state.go_to_next_frame();
            }
            KeyCode::Enter => {
                state.go_to(AppPage::Edit);
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
        let scene_image = &state.client_state.scene;
        let mut max_frames = 0;
        for line in scene_image.lines.iter() {
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
            std::cmp::min(state.selected.0 - mid_h, state.client_state.scene.n_lines().saturating_sub(visible_lines))
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
        let mut y_scroll_state = ScrollbarState::new(scene_image.n_lines().saturating_sub(visible_lines) + 1)
            .position(first_line);
        for j in 0..max_frames {
            if j >= visible_frames {
                break;
            }
            let header_area = Rect::new(
                area.x + LINE_HEADER_WIDTH + 1 + (j as u16) * (FRAME_WIDTH + 1), 
                area.y, 
                FRAME_WIDTH, 
                FRAME_HEADER_HEIGHT
            );
            let f_j = j + first_frame;
            Self::render_frame_header(header_area, buf, f_j, f_j == state.selected.1, &state.palette);
        }
        for (i, line) in scene_image.lines[first_line..].iter().enumerate() {
            if i >= visible_lines {
                break;
            }
            let header_area = Rect::new(
                area.x, 
                area.y + FRAME_HEADER_HEIGHT + 1 + (i as u16) * (FRAME_HEIGHT + 1), 
                LINE_HEADER_WIDTH, 
                FRAME_HEIGHT
            );
            let f_i = i + first_line;
            Self::render_line_header(header_area, buf, f_i, line, f_i == state.selected.0, &state.palette);
            if line.n_frames() <= first_frame {
                continue;
            }

            let line_pos : BTreeMap<usize, usize> = state.client_state.positions
                .get(f_i)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect();

            for (j, frame) in line.frames[first_frame..].iter().enumerate() {
                if j >= visible_frames {
                    break;
                }
                let frame_area = Rect::new(
                    area.x + LINE_HEADER_WIDTH + 1 + (j as u16) * (FRAME_WIDTH + 1), 
                    area.y + FRAME_HEADER_HEIGHT + 1 + (i as u16) * (FRAME_HEIGHT + 1), 
                    FRAME_WIDTH, 
                    FRAME_HEIGHT
                );
                
                let f_j = j + first_frame;
                let selected = (f_i, f_j) == state.selected;
                let rep = line_pos.get(&f_j).copied();
                let color = state.palette.variations[f_i % state.palette.variations.len()];
                Self::render_frame(
                    frame_area, 
                    buf, 
                    color, 
                    frame, 
                    selected,
                    rep,
                    &state.palette,
                );
            }
        }
        if visible_lines < scene_image.n_lines() {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .thumb_style(state.palette.selection)
                .style(state.palette.surface)
                .end_symbol(None).begin_symbol(None);
            scrollbar.render(area.inner(Margin { horizontal: 0, vertical: 1 }), buf, &mut y_scroll_state);
        }
        if visible_frames < max_frames {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                .style(state.palette.surface)
                .thumb_style(state.palette.selection)
                .thumb_symbol("🬋")
                .end_symbol(None).begin_symbol(None);
            scrollbar.render(area.inner(Margin { horizontal: 1, vertical: 0 }), buf, &mut x_scroll_state);
        }
    }
}