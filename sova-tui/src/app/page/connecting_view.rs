use ratatui::{buffer::Buffer, layout::{Constraint::Length, Flex, Layout, Rect}, style::Stylize, widgets::{Block, BorderType, Padding, Paragraph, StatefulWidget, Widget}};

use crate::{app::AppState, event::AppEvent};

pub struct ConnectingView;

impl ConnectingView {

    pub async fn update_state(state: &mut AppState) {
        let Some(task) = &state.connection_task else {
            return;
        };
        if task.is_finished() {
            let handle = state.connection_task.take().unwrap();
            let (client, res) = handle.await.unwrap();
            match res {
                Ok(_) => {
                    state.client = Some(client);
                    state.events.send(AppEvent::Connected);
                }
                Err(e) => {
                    state.events.send(AppEvent::ConnectionFailed(e.to_string()));
                }
            }
        }
    }

}

impl StatefulWidget for ConnectingView {
    type State = AppState;
    
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = Layout::vertical([Length(5)])
            .flex(Flex::Center)
            .split(area);
        let layout = Layout::horizontal([Length(20)])
            .flex(Flex::Center)
            .split(layout[0]);
        let text = Paragraph::new("Connecting...")
            .centered()
            .fg(state.palette.foreground)
            .block(Block::bordered()
                .border_type(BorderType::Rounded)
                .bg(state.palette.surface)
                .border_style(state.palette.foreground)
                .padding(Padding::new(0, 0, 1, 1))
            );
        text.render(layout[0], buf);
    }
    
}