use crate::{
    clock::{Clock, SyncTime},
    lang::event::{ConcreteEvent, ConcreteEventPayload},
    protocol::{
        self, ProtocolMessage, TimedMessage
    }
};

use protocol::log::LogMessage;

pub struct DeviceMap;

impl DeviceMap {

    pub fn new() -> Self {
        DeviceMap
    }

    pub fn map_event(&self,
        event : ConcreteEvent,
        date : SyncTime,
        clock : &Clock
    ) -> Vec<TimedMessage> {
        match event.payload {
            ConcreteEventPayload::Nop => Vec::new(),
            ConcreteEventPayload::Chord(_, _) => {
                let msg = serde_json::to_string(&event).unwrap();
                vec![ProtocolMessage::LOG(LogMessage::info(msg)).timed(date)]
            },
            //_ => todo!()
        }
    }

}
