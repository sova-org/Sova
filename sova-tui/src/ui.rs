use ratatui::{
    buffer::Buffer, layout::{Alignment, Rect}, style::{Color, Stylize}, widgets::{Block, BorderType, Paragraph, StatefulWidget, Widget},
};

use crate::app::{App, AppPage, scene_view::SceneView};

impl Widget for &mut App {
    /// Renders the user interface widgets.
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.state.page {
            AppPage::Scene => SceneView.render(area, buf, &mut self.state),
            _ => ()
        }

        self.popup.render(area, buf, &mut self.state);
        self.notification.render(area, buf);
    }
}
