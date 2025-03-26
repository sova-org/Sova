use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::{
    clock::{Clock, SyncTime}, lang::event::ConcreteEvent, protocol::{
        log::{LogMessage, Severity}, midi::{MIDIMessage, MIDIMessageType}, ProtocolDevice, ProtocolMessage, TimedMessage
    }
};

pub type DeviceItem = (String, Arc<ProtocolDevice>);

pub struct DeviceMap {
    pub input_connections : Mutex<HashMap<String, DeviceItem>>,
    pub output_connections : Mutex<HashMap<String, DeviceItem>>
}

pub const LOG_NAME: &str = "log";

impl DeviceMap {

    pub fn new() -> Self {
        let devices = DeviceMap {
            input_connections : Default::default(),
            output_connections : Default::default()
        };
        devices.register_output_connection(LOG_NAME.to_owned(), ProtocolDevice::Log);
        devices
    }

    pub fn register_input_connection(&self, name : String, device : ProtocolDevice) {
        let address = device.address().to_owned();
        let item = (name, Arc::new(device));
        self.input_connections.lock().unwrap().insert(address, item);
    }

    pub fn register_output_connection(&self, name : String, device : ProtocolDevice) {
        let address = device.address().to_owned();
        let item = (name, Arc::new(device));
        self.output_connections.lock().unwrap().insert(address, item);
    }

    fn generate_midi_message(&self,
        payload : ConcreteEvent,
        date : SyncTime,
        device : Arc<ProtocolDevice>,
        clock : &Clock
    )
        -> Vec<TimedMessage>
    {
        match payload {
            ConcreteEvent::MidiNote(note, vel, chan, dur, _) => {
                //let chan = chan.unwrap_or(0);
                //let vel = vel.unwrap_or(90);
                vec![
                    ProtocolMessage {
                        payload : MIDIMessage {
                            payload : MIDIMessageType::NoteOn { note: note as u8, velocity: vel as u8 },
                            channel : chan as u8
                        }.into(),
                        device : Arc::clone(&device)
                    }.timed(date),
                    ProtocolMessage {
                        payload : MIDIMessage {
                            payload : MIDIMessageType::NoteOff { note: note as u8, velocity: 0 },
                            channel : chan as u8
                        }.into(),
                        device : Arc::clone(&device)
                    }.timed(date + dur.as_micros(clock))
                ]
                /*notes.iter().map(|n|
                .chain(notes.iter().map(|n|
                )).collect()*/
            },
            _ => Vec::new()
        }
    }

    fn generate_log_message(&self,
        payload : ConcreteEvent,
        date : SyncTime,
        device : Arc<ProtocolDevice>,
    )
        -> Vec<TimedMessage>
    {
        vec![
            ProtocolMessage {
                payload : LogMessage { level : Severity::Info, msg : format!("{:?}", payload) }.into(),
                device : Arc::clone(&device)
            }.timed(date)
        ]
    }

    pub fn map_event(&self,
        event : ConcreteEvent,
        date : SyncTime,
        clock : &Clock
    )
        -> Vec<TimedMessage>
    {
        let (dev_name, opt_device) = self.find_device(&event);
        let Some(device) = opt_device else {
            return vec![
                ProtocolMessage {
                    payload : LogMessage { level : Severity::Error, msg : format!("Unable to find device {:?}", dev_name) }.into(),
                    device : Arc::new(ProtocolDevice::Log)
                }.timed(date)
            ];
        };
        match &*device {
            ProtocolDevice::OSCOutDevice => todo!(),
            ProtocolDevice::MIDIOutDevice(_) => self.generate_midi_message(event, date, device, clock),
            ProtocolDevice::Log => self.generate_log_message(event, date, device),
            _ => Vec::new()
        }
    }

    pub fn find_device(&self, event : &ConcreteEvent) -> (String, Option<Arc<ProtocolDevice>>) {
        let cons = self.output_connections.lock().unwrap();
        match event {
            ConcreteEvent::Nop => (LOG_NAME.to_string(), cons.get(LOG_NAME).map(|x| Arc::clone(&x.1))),
            ConcreteEvent::MidiNote(_, _, _, _, dev) => (dev.to_string(), cons.get(dev).map(|x| Arc::clone(&x.1))),
        }
    }

}
