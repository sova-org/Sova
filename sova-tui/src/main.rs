use ratatui_themes::{Theme, ThemeName};

use crate::{app::{App, AppPage, AppState}, event::EventHandler};

pub mod app;
pub mod event;
pub mod ui;
pub mod notification;
pub mod popup;
pub mod theme;
pub mod network;
pub mod widgets;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {    
    color_eyre::install()?;
    let terminal = ratatui::init();

    let state = AppState {
        client_state: Default::default(),
        events: EventHandler::new(),
        selected: (0, 0),
        clipboard: None,
        page: AppPage::Connection,
        palette: Theme::new(ThemeName::GruvboxDark).into(),
        client: None,
        server_task: None,
        logs: Default::default()
    };

    let result = App::new(state).run(terminal).await;
    ratatui::restore();
    result
}
