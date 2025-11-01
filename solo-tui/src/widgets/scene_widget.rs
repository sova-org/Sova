use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

use crate::app::AppState;

pub struct SceneWidget {}

impl StatefulWidget for &SceneWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let structure = state.scene_image.structure();

        let cols = state.scene_image.n_lines();

        todo!()
    }
}
