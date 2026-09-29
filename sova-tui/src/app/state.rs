use std::sync::Arc;

use arboard::Clipboard;
use sova_core::{Scene, clock::Clock, device_map::DeviceMap, protocol::DeviceInfo, scene::{Frame, Line}, schedule::playback::PlaybackState, vm::LanguageCenter};

use crate::{app::page::AppPage, event::EventHandler};

pub struct AppState {
    pub scene_image: Scene,
    pub playing: PlaybackState,
    pub positions: Vec<Vec<(usize, usize)>>,
    pub events: EventHandler,
    pub selected: (usize, usize),
    pub clipboard: Option<Clipboard>,
    pub page: AppPage,
    pub clock: Clock,
    pub devices: Vec<DeviceInfo>,
    pub device_map: Arc<DeviceMap>,
    pub languages: Arc<LanguageCenter>
}

impl AppState {
    pub fn selected_frame(&self) -> Option<&Frame> {
        self.scene_image.frame(self.selected.0, self.selected.1)
    }

    pub fn selected_line(&self) -> Option<&Line> {
        self.scene_image.line(self.selected.0)
    }

    pub fn refresh_devices(&mut self) {
        self.devices = self.device_map.device_list();
    }
}