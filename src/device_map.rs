use crate::{clock::{Clock, SyncTime}, lang::event::Event, protocol::{ProtocolMessage, TimedMessage}};

pub struct DeviceMap;

impl DeviceMap {

    pub fn new() -> Self {
        DeviceMap
    }

    pub fn map_event(&self, event : Event, date : SyncTime, clock : &Clock) -> Vec<TimedMessage> {
        match event {
            Event::Nop => todo!(),
            Event::Chord(_,_) => todo!(),
            _ => todo!()
        }
    }

}