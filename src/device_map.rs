use crate::{clock::{Clock, SyncTime}, lang::event::Event, protocol::{log::LogMessage, ProtocolMessage, TimedMessage}};

pub struct DeviceMap;

impl DeviceMap {

    pub fn new() -> Self {
        DeviceMap
    }

    pub fn map_event(&self, event : Event, date : SyncTime, clock : &Clock) -> Vec<TimedMessage> {
        match event {
            Event::Nop => Vec::new(),
            Event::Chord(_, _) => {
                let msg = serde_json::to_string(&event).unwrap();
                vec![ProtocolMessage::LOG(LogMessage::info(msg)).timed(date)]
            },
            _ => todo!()
        }
    }

}