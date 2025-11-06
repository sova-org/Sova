use std::sync::Arc;

use ratatui::widgets::{ScrollbarState, StatefulWidget, TableState};
use sova_core::device_map::DeviceMap;

pub struct DevicesWidget {
    state: TableState,
    scroll_state: ScrollbarState,
    devices: Arc<DeviceMap>
}

