use crate::{lang::Event, protocol::ProtocolMessage};

pub struct DeviceMap;

impl DeviceMap {

    pub fn new() -> Self {
        DeviceMap
    }

    pub fn map_event(&self, event : Event) -> ProtocolMessage {
        todo!()
    }

}