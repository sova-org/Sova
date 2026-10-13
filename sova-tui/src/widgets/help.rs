use crossterm::event::KeyEvent;
use ratatui::{prelude::{Buffer, Rect}, text::{Line, Span, Text}, widgets::{Paragraph, Widget}};

use crate::{app::{AppPage, AppState}, widgets::InteractiveWidget};

pub struct Help;

impl InteractiveWidget for Help {
    fn on_key_event(&mut self, _event: KeyEvent, _state: &mut AppState) {
        ()
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let shortcuts = match state.page {
            AppPage::Scene => vec![
                ("Ctrl-T", "Edit tempo")
            ],
            AppPage::Prelude => todo!(),
            AppPage::Edit => vec![
                ("Ctrl-Enter", "Upload script")
            ],
            _ => return
        };
        let lines : Vec<Line> = shortcuts.into_iter().map(|(s, d)| {
            Line::from(vec![
                Span::styled(s, state.palette.accent),
                Span::raw("    "),
                Span::styled(d, state.palette.text())
            ])
        }).collect();
        let text = Text::from(lines);
        let p = Paragraph::new(text);
        p.render(area, buf);
    }
}