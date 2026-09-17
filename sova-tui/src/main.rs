use std::sync::Arc;

use langs::create_language_center;
use sova_core::{clock::ClockServer, device_map::DeviceMap, init::start_scheduler_and_world};

use crate::app::App;

pub mod app;
pub mod event;
pub mod ui;

const DEFAULT_TEMPO : f64 = 120.0; 
const DEFAULT_QUANTUM : f64 = 4.0;

fn main() -> color_eyre::Result<()> {
    let clock_server = Arc::new(ClockServer::new(DEFAULT_TEMPO, DEFAULT_QUANTUM));
    let devices = Arc::new(DeviceMap::new());
    let languages = Arc::new(create_language_center());
    
    let (world_handle, sched_handle, sched_iface, sched_update) 
        = start_scheduler_and_world(clock_server, devices, languages);

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
