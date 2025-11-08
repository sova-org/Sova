use std::sync::Arc;

use ratatui::{buffer::Buffer, layout::Rect, widgets::{ScrollbarState, StatefulWidget, Table, TableState}};
use sova_core::device_map::DeviceMap;

use crate::app::AppState;

pub struct DevicesWidget {
    state: TableState,
    scroll_state: ScrollbarState,
    devices: Arc<DeviceMap>
}

impl StatefulWidget for &DevicesWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header = [ "Name", "Address", ];
        
        todo!()
    }
}