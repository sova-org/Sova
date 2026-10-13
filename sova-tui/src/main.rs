use ratatui_themes::{Theme, ThemeName};

use crate::{app::{App, AppPage, AppState}, event::EventHandler};

pub mod app;
pub mod event;
pub mod ui;
pub mod notification;
pub mod theme;
pub mod network;
pub mod widgets;
pub mod compiled_syntax;
pub mod flash;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {    
    color_eyre::install()?;
    let terminal = ratatui::init();

    let state = AppState {
        client_state: Default::default(),
        events: EventHandler::new(),
        selected: (0, 0),
        selected_prelude: 0,
        clipboard: None,
        page: AppPage::Connection,
        palette: Theme::new(ThemeName::Dracula).into(),
        client: None,
        server_task: None,
        compiled_syntax: Default::default(),
    };

    let result = App::new(state).run(terminal).await;
    ratatui::restore();
    result
}
