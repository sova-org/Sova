use std::{thread::{self, JoinHandle}};

use crossbeam_channel::{Receiver, SendError, Sender};
use serde::{Serialize, Deserialize};

use crate::{LogMessage, clock::SyncTime, lang::{event::ConcreteEvent, variable::VariableValue}, log_eprintln, protocol::{error::ProtocolError, payload::ProtocolPayload}};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioEnginePayload {
    pub args: Vec<VariableValue>,
    pub timetag: Option<SyncTime>,
}

impl AudioEnginePayload {

    pub fn generate_messages(event: ConcreteEvent, date: SyncTime) -> Vec<(ProtocolPayload, SyncTime)> {
        if let ConcreteEvent::Dirt { args, device_id: _ } = event {
            let audio_payload = AudioEnginePayload {
                args,
                timetag: Some(date),
            };
            vec![(audio_payload.into(), date)]
        } else {
            Vec::new()
        }
    }

}

pub struct AudioEngineProxy {
    pub tx: Sender<AudioEnginePayload>,
    pub thread: Option<JoinHandle<()>>
}

impl AudioEngineProxy {

    pub fn new(tx: Sender<AudioEnginePayload>) -> Self {
        AudioEngineProxy { 
            tx, 
            thread: None
        }
    }

    pub fn log_callback<F>(&mut self, log_rx: Receiver<LogMessage>, callback: F) 
        where F: (Fn(LogMessage) -> ()) + Send + Sync + 'static
    {
        if self.thread.is_some() {
            log_eprintln!("Log handling thread is already started for audio engine !");
            return;
        }
        let handle = thread::spawn(move || {
            loop {
                match log_rx.recv() {
                    Ok(msg) => callback(msg),
                    Err(_) => break,
                }
            }
        });
        self.thread = Some(handle);
    }

    pub fn send(&self, message: AudioEnginePayload) -> Result<(), ProtocolError> {
        match self.tx.send(message) {
            Ok(_) => Ok(()),
            Err(SendError(_)) => Err(format!("Unable to send : audio engine is disconnected !").into()),
        }
    }

}