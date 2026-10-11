use ratatui::{prelude::{Buffer, Rect}, widgets::StatefulWidget};

use crate::app::AppState;

pub struct Editor {

}

impl StatefulWidget for Editor {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        todo!()
    }
}