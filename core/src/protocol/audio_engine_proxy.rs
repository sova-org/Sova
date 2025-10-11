use std::{collections::HashMap, sync::atomic::{AtomicU32, Ordering}};

use bubo_engine::{registry::ModuleRegistry, server::ScheduledEngineMessage, types::{EngineMessage, ScheduledMessage}};
use crossbeam_channel::Sender;
use serde::{Serialize, Deserialize};

use crate::{clock::SyncTime, lang::event::ConcreteEvent, protocol::{error::ProtocolError, osc::Argument, payload::ProtocolPayload}};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioEnginePayload {
    pub args: Vec<Argument>,
    pub timetag: Option<SyncTime>
}

impl AudioEnginePayload {

    pub fn generate_messages(event: ConcreteEvent, date: SyncTime) -> Vec<(ProtocolPayload, SyncTime)> {
        if let ConcreteEvent::Dirt { args, device_id: _ } = event {
            let audio_payload = AudioEnginePayload {
                args,
                timetag: Some(date)
            };
            vec![(audio_payload.into(), date)]
        } else {
            Vec::new()
        }
    }

}

pub struct AudioEngineProxy {
    pub tx: Option<Sender<ScheduledEngineMessage>>,
    pub voice_id: AtomicU32,
    pub registry: ModuleRegistry
}

impl AudioEngineProxy {

    pub fn send(&self, message: AudioEnginePayload) -> Result<(), ProtocolError> {
        let voice_id_counter = self.voice_id.load(Ordering::Relaxed);
        let timetag = message.timetag;
        if let Some(tx) = &self.tx {
            let (engine_message, new_voice_id_counter) = self
                .convert_audio_engine_payload_to_engine_message(
                    message,
                    voice_id_counter,
                );
            self.voice_id.store(new_voice_id_counter, Ordering::Relaxed);

            let scheduled_msg = if let Some(timetag) = timetag {
                ScheduledEngineMessage::Scheduled(ScheduledMessage {
                    due_time_micros: timetag,
                    message: engine_message,
                })
            } else {
                ScheduledEngineMessage::Immediate(engine_message)
            };
            
            let _ = tx.send(scheduled_msg);
            Ok(())
        } else {
            Err(format!("No opened interface to an audio engine !").into())
        }
    }

    fn convert_audio_engine_payload_to_engine_message(
        &self,
        payload: AudioEnginePayload,
        voice_id_counter: u32,
    ) -> (EngineMessage, u32) {
        // Convert Argument array to string array for unified parser
        let mut string_args: Vec<String> = Vec::with_capacity(payload.args.len());
        
        for arg in payload.args {
            match arg {
                Argument::String(s) => string_args.push(s.clone()),
                Argument::Int(i) => string_args.push(i.to_string()),
                Argument::Float(f) => string_args.push(f.to_string()),
                Argument::Blob(_) | Argument::Timetag(_) => continue,
            }
        }
        
        // Convert to &str references
        let str_args: Vec<&str> = string_args.iter().map(|s| s.as_str()).collect();
        
        // Use the unified parser from registry
        if let Some((engine_msg, new_counter)) = self.registry.parse_unified_message(&str_args, voice_id_counter) {
            (engine_msg, new_counter)
        } else {
            // Fallback: create a no-op message if parsing fails
            (
                EngineMessage::Update {
                    voice_id: 0,
                    track_id: 0,
                    parameters: HashMap::new(),
                },
                voice_id_counter,
            )
        }
    }

}