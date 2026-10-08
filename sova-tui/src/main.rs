use std::sync::Arc;

use langs::create_language_center;
use ratatui_themes::{Theme, ThemeName};
use sova_core::{Scene, clock::ClockServer, device_map::DeviceMap, init::start_scheduler_and_world, scene::Line, schedule::playback::PlaybackState};

use crate::{app::{App, AppPage, AppState}, event::EventHandler};

pub mod app;
pub mod event;
pub mod ui;
pub mod notification;
pub mod popup;
pub mod theme;
pub mod network;

const DEFAULT_TEMPO : f64 = 120.0; 
const DEFAULT_QUANTUM : f64 = 4.0;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {    
    color_eyre::install()?;
    let terminal = ratatui::init();

    let mut scene = Scene::new(vec![
        Line::new(vec![1.4 ; 7]),
        Line::new(vec![1.0 ; 8]),
        Line::new(vec![12.4 ; 7]),
    ]);
    scene.line_mut(2).manual = true;
    scene.line_mut(2).looping = true;
    scene.line_mut(2).trailing = true;
    scene.line_mut(4).manual = true;
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
