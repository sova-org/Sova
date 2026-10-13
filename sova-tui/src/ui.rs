use ratatui::{
    buffer::Buffer, layout::{Constraint, Layout, Rect}, style::Stylize, widgets::{Fill, StatefulWidget, Widget},
};

use crate::{app::{App, AppPage, connecting_view::ConnectingView, scene_view::SceneView}, widgets::{Footer, Header, Scope, VUMeter}};

impl Widget for &mut App {
    /// Renders the user interface widgets.
    fn render(self, area: Rect, buf: &mut Buffer) {
        Fill::new(" ").bg(self.state.palette.background).render(area, buf);

        let main_area = if self.state.page.is_in_session() {

            let layout = Layout::vertical([
                Constraint::Length(4),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1)
            ]).split(area);

            Header.render(layout[0], buf, &mut self.state);
            Footer.render(layout[3], buf, &mut self.state);

            let layout = Layout::horizontal([Constraint::Length(2), Constraint::Min(0)])
                .spacing(1)
                .split(layout[2]);

            VUMeter(2).render(layout[0], buf, &mut self.state);

            let layout = Layout::vertical([Constraint::Min(0), Constraint::Length(5)])
                //.spacing(1)
                .split(layout[1]);

            Scope.render(layout[1], buf, &mut self.state);

            layout[0]
        } else {
            let layout = Layout::vertical([
                Constraint::Min(0),
                Constraint::Length(1)
            ]).spacing(1).split(area);

            Footer.render(layout[1], buf, &mut self.state);

            layout[0]
        };

        match self.state.page {
            AppPage::Connection => self.connection_view.render(main_area, buf, &mut self.state),
            AppPage::Connecting => ConnectingView.render(main_area, buf, &mut self.state),
            AppPage::Scene => SceneView.render(main_area, buf, &mut self.state),
            AppPage::Edit => self.editor.render(main_area, buf, &mut self.state),
            AppPage::Prelude => self.prelude_view.render(main_area, buf, &mut self.state),
            _ => ()
        }

        self.floating.render(area, buf, &mut self.state);
        self.popup.render(area, buf, &mut self.state);
        self.notification.render(area, buf);
    }
}
