use crossbeam_channel::{SendError, Sender};
use serde::{Deserialize, Serialize};

use crate::{
    clock::SyncTime,
    protocol::{error::ProtocolError, payload::ProtocolPayload},
    vm::{event::ConcreteEvent, variable::VariableValue},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostMessage {
    pub route: String,
    pub args: Vec<VariableValue>,
}

impl HostMessage {
    pub fn generate_messages(
        event: ConcreteEvent,
        date: SyncTime,
    ) -> Vec<(ProtocolPayload, SyncTime)> {
        match event {
            ConcreteEvent::Generic(value, _duration, route, _device_id) => {
                let args = match value {
                    VariableValue::Map(map) => {
                        let mut flat = Vec::with_capacity(map.len() * 2);
                        for (key, value) in map.into_iter() {
                            flat.push(VariableValue::Str(key));
                            flat.push(value);
                        }
                        flat
                    }
                    VariableValue::Vec(v) => v,
                    other => vec![other],
                };
                vec![(HostMessage { route, args }.into(), date)]
            }
            _ => Vec::new(),
        }
    }
}

pub struct HostProxy {
    pub name: String,
    pub tx: Sender<HostMessage>,
}

impl HostProxy {
    pub fn new(name: String, tx: Sender<HostMessage>) -> Self {
        HostProxy { name, tx }
    }

    pub fn send(&self, message: HostMessage) -> Result<(), ProtocolError> {
        match self.tx.send(message) {
            Ok(_) => Ok(()),
            Err(SendError(_)) => Err(ProtocolError(format!(
                "Host proxy '{}' is disconnected.",
                self.name
            ))),
        }
    }
}

impl std::fmt::Debug for HostProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostProxy").field("name", &self.name).finish()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crossbeam_channel::unbounded;

    use super::*;

    #[test]
    fn host_proxy_send_pushes_message() {
        let (tx, rx) = unbounded::<HostMessage>();
        let proxy = HostProxy::new("test".to_string(), tx);
        let msg = HostMessage {
            route: "hydra/eval".to_string(),
            args: vec![VariableValue::Str("osc().out()".to_string())],
        };
        proxy.send(msg.clone()).expect("send should succeed");
        let received = rx.recv().expect("receiver should get the message");
        assert_eq!(received, msg);
    }

    #[test]
    fn host_proxy_send_errors_when_receiver_dropped() {
        let (tx, rx) = unbounded::<HostMessage>();
        drop(rx);
        let proxy = HostProxy::new("test".to_string(), tx);
        let msg = HostMessage {
            route: "hydra/eval".to_string(),
            args: vec![],
        };
        let err = proxy.send(msg).expect_err("send should fail");
        assert!(err.0.contains("disconnected"));
    }

    #[test]
    fn generate_messages_flattens_string_arg() {
        let event = ConcreteEvent::Generic(
            VariableValue::Str("osc().out()".to_string()),
            10_000,
            "hydra/eval".to_string(),
            42,
        );
        let messages = HostMessage::generate_messages(event, 1234);
        assert_eq!(messages.len(), 1);
        let (payload, date) = &messages[0];
        assert_eq!(*date, 1234);
        let ProtocolPayload::Host(host_msg) = payload else {
            panic!("expected ProtocolPayload::Host");
        };
        assert_eq!(host_msg.route, "hydra/eval");
        assert_eq!(host_msg.args, vec![VariableValue::Str("osc().out()".to_string())]);
    }

    #[test]
    fn generate_messages_flattens_map_to_kv_pairs() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), VariableValue::Integer(7));
        let event = ConcreteEvent::Generic(
            VariableValue::Map(map),
            0,
            "host/test".to_string(),
            0,
        );
        let messages = HostMessage::generate_messages(event, 0);
        let ProtocolPayload::Host(host_msg) = &messages[0].0 else {
            panic!("expected ProtocolPayload::Host");
        };
        assert_eq!(host_msg.args.len(), 2);
        assert_eq!(host_msg.args[0], VariableValue::Str("key".to_string()));
        assert_eq!(host_msg.args[1], VariableValue::Integer(7));
    }

    #[test]
    fn generate_messages_drops_non_generic_events() {
        let event = ConcreteEvent::MidiNote(60, 100, 1, 1_000_000, 1);
        let messages = HostMessage::generate_messages(event, 0);
        assert!(messages.is_empty());
    }
}
