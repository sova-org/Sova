use ratatui::{
    buffer::Buffer, layout::Rect, style::Stylize, widgets::{Fill, StatefulWidget, Widget},
};

use crate::app::{App, AppPage, connecting_view::ConnectingView, scene_view::SceneView};

impl Widget for &mut App {
    /// Renders the user interface widgets.
    fn render(self, area: Rect, buf: &mut Buffer) {
        Fill::new(" ").bg(self.state.palette.background).render(area, buf);

        match self.state.page {
            AppPage::Scene => SceneView.render(area, buf, &mut self.state),
            AppPage::Connection => self.connection_view.render(area, buf, &mut self.state),
            AppPage::Connecting => ConnectingView.render(area, buf, &mut self.state),
            _ => ()
        }

        self.popup.render(area, buf, &mut self.state);
        self.notification.render(area, buf);
    }
}
