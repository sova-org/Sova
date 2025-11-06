use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Padding, Paragraph, StatefulWidget, Widget},
};
use sova_core::compiler::CompilationState;

use crate::{app::AppState, page::Page, widgets::{edit_widget::EditWidget, scene_widget::SceneWidget}};

#[derive(Default)]
pub struct Footer;

fn map_style(state: &AppState, page: Page) -> Style {
    if state.page == page {
        Style::default().bold().fg(Color::LightMagenta)
    } else {
        Style::default()
    }
}

fn format_compilation_state(state: &CompilationState) -> &str {
    match state {
        CompilationState::NotCompiled => "_",
        CompilationState::Compiling => "...",
        CompilationState::Compiled(_) => "✓",
        CompilationState::Error(_) => "❌",
    }
}

impl StatefulWidget for Footer {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        use Constraint::*;

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(1));

        let inner = block.inner(area);
        block.render(area, buf);

        let lines = vec![
            Line::from(vec![
                Span::styled("T", map_style(state, Page::Time)),
                Span::from(" "),
                Span::styled("C", map_style(state, Page::Configure)),
                Span::from(" "),
                Span::styled(" ", Style::default()),
            ]),
            Line::from(vec![
                Span::styled("D", map_style(state, Page::Devices)),
                Span::from(" "),
                Span::styled("S", map_style(state, Page::Scene)),
                Span::from(" "),
                Span::styled("E", map_style(state, Page::Edit)),
            ]),
            Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::from(" "),
                Span::styled("L", map_style(state, Page::Logs)),
                Span::from(" "),
                Span::styled("V", map_style(state, Page::Vars)),
            ]),
        ];
        let map = Paragraph::new(Text::from(lines));

        let pos = Paragraph::new(format!(
            "{}:{}\n{}\n{}", 
            state.selected.0, 
            state.selected.1,
            state.selected_frame().map_or("", |f| f.script().lang()),
            state.selected_frame().map_or("", |f| 
                format_compilation_state(&f.script().compilation_state())
            )

        ));

        let [left, middle, right] = Layout::horizontal([Length(5), Min(0), Length(5)]).areas(inner);

        pos.render(left, buf);
        map.render(right, buf);

        let help = match state.page {
            Page::Scene => SceneWidget::get_help(),
            Page::Edit => EditWidget::get_help(),
            _ => ""
        };
        Paragraph::new(help).render(middle.inner(Margin {
            horizontal: 4, vertical: 0
        }), buf);
    }
}
