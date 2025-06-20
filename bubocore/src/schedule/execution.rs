use std::sync::Arc;
use crossbeam_channel::Sender;
use crate::{
    clock::{Clock, SyncTime},
    device_map::DeviceMap,
    lang::{event::ConcreteEvent, variable::VariableStore},
    protocol::{
        message::{ProtocolMessage, TimedMessage},
        payload::{AudioEnginePayload, ProtocolPayload},
    },
    scene::{Scene, script::ScriptExecution},
};

pub struct ExecutionManager;

impl ExecutionManager {
    pub fn process_executions(
        clock: &Clock,
        scene: &mut Scene,
        executions: &mut Vec<ScriptExecution>,
        global_vars: &mut VariableStore,
        devices: Arc<DeviceMap>,
        world_iface: &Sender<TimedMessage>,
        scheduled_drift: SyncTime,
        audio_engine_events: &mut Vec<(ConcreteEvent, SyncTime)>,
    ) -> SyncTime {
        if scene.n_lines() == 0 {
            return SyncTime::MAX;
        }

        let scheduled_date = clock.micros() + scheduled_drift;
        let mut next_timeout = SyncTime::MAX;
        audio_engine_events.clear();

        executions.retain_mut(|exec| {
            if !exec.is_ready(scheduled_date) {
                next_timeout = std::cmp::min(next_timeout, exec.remaining_before(scheduled_date));
                return true;
            }

            next_timeout = 0;
            if let Some((event, date)) = exec.execute_next(
                clock,
                global_vars,
                scene.mut_lines(),
                devices.clone(),
            ) {
                let maybe_slot_id: Option<usize> = match event {
                    ConcreteEvent::MidiNote(_, _, _, _, id)
                    | ConcreteEvent::MidiControl(_, _, _, id)
                    | ConcreteEvent::MidiProgram(_, _, id)
                    | ConcreteEvent::MidiAftertouch(_, _, _, id)
                    | ConcreteEvent::MidiChannelPressure(_, _, id)
                    | ConcreteEvent::MidiSystemExclusive(_, id)
                    | ConcreteEvent::MidiStart(id)
                    | ConcreteEvent::MidiStop(id)
                    | ConcreteEvent::MidiReset(id)
                    | ConcreteEvent::MidiContinue(id)
                    | ConcreteEvent::MidiClock(id) => Some(id),
                    ConcreteEvent::Dirt { device_id: id, .. } => Some(id),
                    ConcreteEvent::Osc { device_id: id, .. } => Some(id),
                    ConcreteEvent::AudioEngine { .. } => {
                        audio_engine_events.push((event.clone(), date));
                        None
                    }
                    ConcreteEvent::Nop => None,
                };

                if let Some(slot_id) = maybe_slot_id {
                    let messages = devices.map_event_for_slot_id(slot_id, event, date, clock);
                    for message in messages {
                        let _ = world_iface.send(message);
                    }
                }
            }

            !exec.has_terminated()
        });

        for (event, date) in audio_engine_events.drain(..) {
            Self::handle_audio_engine_event(event, date, world_iface);
        }

        next_timeout
    }

    fn handle_audio_engine_event(
        event: ConcreteEvent,
        date: SyncTime,
        world_iface: &Sender<TimedMessage>,
    ) {
        if let ConcreteEvent::AudioEngine { args, device_id } = event {
            let audio_payload = AudioEnginePayload { args, device_id };

            let protocol_message = ProtocolMessage {
                device: Arc::new(crate::protocol::device::ProtocolDevice::AudioEngine),
                payload: ProtocolPayload::AudioEngine(audio_payload),
            };

            let timed_message = TimedMessage {
                message: protocol_message,
                time: date,
            };

            let _ = world_iface.send(timed_message);
        }
    }
}