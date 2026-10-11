use ratatui::{
    buffer::Buffer, layout::{Constraint, Layout, Margin, Rect}, style::Stylize, widgets::{Fill, StatefulWidget, Widget},
};

use crate::{app::{App, AppPage, connecting_view::ConnectingView, scene_view::SceneView}, widgets::{Footer, Header}};

impl Widget for &mut App {
    /// Renders the user interface widgets.
    fn render(self, area: Rect, buf: &mut Buffer) {
        Fill::new(" ").bg(self.state.palette.background).render(area, buf);

        let full_ui = match self.state.page {
            AppPage::Connection | AppPage::Connecting => false,
            _ => true
        };

        let mut main_area = area;

        if full_ui {
            let layout = Layout::vertical([
                Constraint::Length(5),
                Constraint::Min(0),
                Constraint::Length(1)
            ]).spacing(1).split(area);

            Header.render(layout[0], buf, &mut self.state);
            Footer.render(layout[2], buf, &mut self.state);

            main_area = layout[1];
        }

        match self.state.page {
            AppPage::Connection => self.connection_view.render(main_area, buf, &mut self.state),
            AppPage::Connecting => ConnectingView.render(main_area, buf, &mut self.state),
            AppPage::Scene => SceneView.render(main_area.inner(Margin::new(1, 0)), buf, &mut self.state),
            _ => ()
        }

        self.popup.render(area, buf, &mut self.state);
        self.notification.render(area, buf);
    }
}
