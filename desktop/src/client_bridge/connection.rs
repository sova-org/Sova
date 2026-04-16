use std::sync::mpsc;

use sova_server::{ClientMessage, ServerMessage, SovaClient};
use tokio::sync::mpsc as tokio_mpsc;

use super::{ClientBridge, ConnectionStatus, EventSender, OutgoingMessage};

impl ClientBridge {
    pub fn connect(&mut self, ip: &str, port: u16, username: &str, password: &str, feedback: bool) {
        if matches!(
            self.status,
            ConnectionStatus::Connecting | ConnectionStatus::Connected
        ) {
            return;
        }

        let ip = ip.to_owned();
        let username = username.to_owned();
        let password = if password.is_empty() {
            None
        } else {
            Some(password.to_owned())
        };
        let (send_tx, mut send_rx) = tokio_mpsc::unbounded_channel();
        let (raw_event_tx, event_rx) = mpsc::channel();
        let event_tx = EventSender(raw_event_tx);
        let ctx = self.ctx.clone();

        self.send_tx = Some(send_tx);
        self.event_rx = Some(event_rx);
        self.status = ConnectionStatus::Connecting;
        self.error_msg = None;

        self.runtime.spawn(async move {
            let mut client = SovaClient::new(ip, port);

            match tokio::time::timeout(std::time::Duration::from_secs(5), client.connect()).await {
                Ok(Err(e)) => {
                    let _ = event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                    ctx.request_repaint();
                    return;
                }
                Err(_) => {
                    let _ = event_tx.send(ServerMessage::ConnectionRefused(
                        "Connection timed out".to_string(),
                    ));
                    ctx.request_repaint();
                    return;
                }
                Ok(Ok(())) => {}
            }

            if let Err(e) = client
                .send(ClientMessage::SetName {
                    name: username,
                    password,
                })
                .await
            {
                let _ = event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                ctx.request_repaint();
                return;
            }

            match client.read().await {
                Ok(Some(msg @ ServerMessage::Hello { .. })) => {
                    let _ = event_tx.send(msg);
                    ctx.request_repaint();
                    if feedback && let Err(e) = client.send(ClientMessage::EnableFeedback).await {
                        let _ = event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                        ctx.request_repaint();
                        return;
                    }
                }
                Ok(Some(ServerMessage::ConnectionRefused(reason))) => {
                    let _ = event_tx.send(ServerMessage::ConnectionRefused(reason));
                    ctx.request_repaint();
                    let _ = client.disconnect().await;
                    return;
                }
                Ok(Some(_)) => {
                    let _ = event_tx.send(ServerMessage::ConnectionRefused(
                        "Unexpected server response".into(),
                    ));
                    ctx.request_repaint();
                    let _ = client.disconnect().await;
                    return;
                }
                Ok(None) => {
                    let _ = event_tx.send(ServerMessage::ConnectionRefused(
                        "Failed to deserialize handshake".into(),
                    ));
                    ctx.request_repaint();
                    let _ = client.disconnect().await;
                    return;
                }
                Err(e) => {
                    let _ = event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                    ctx.request_repaint();
                    return;
                }
            }

            let mut reader = client
                .take_reader()
                .expect("reader is always available immediately after successful connect");
            let read_event_tx = event_tx.clone();
            let read_ctx = ctx.clone();

            let read_task = tokio::spawn(async move {
                loop {
                    match sova_server::read_server_message(&mut reader).await {
                        Ok(msg) => {
                            if read_event_tx.send(msg).is_err() {
                                break;
                            }
                            read_ctx.request_repaint();
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => continue,
                        Err(e) => {
                            let _ =
                                read_event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                            read_ctx.request_repaint();
                            break;
                        }
                    }
                }
            });

            loop {
                match send_rx.recv().await {
                    Some(OutgoingMessage::Send(client_msg)) => {
                        if let Err(e) = client.send(*client_msg).await {
                            let _ = event_tx.send(ServerMessage::ConnectionRefused(e.to_string()));
                            ctx.request_repaint();
                            break;
                        }
                    }
                    Some(OutgoingMessage::Disconnect) | None => {
                        let _ = client.disconnect().await;
                        event_tx.send_local_disconnect();
                        ctx.request_repaint();
                        break;
                    }
                }
            }

            read_task.abort();
        });
    }

    pub fn disconnect(&mut self) {
        if let Some(tx) = &self.send_tx {
            let _ = tx.send(OutgoingMessage::Disconnect);
        }
        self.clear_state();
    }

    pub fn send<T: Into<ClientMessage>>(&self, msg: T) {
        if let Some(tx) = &self.send_tx {
            let _ = tx.send(OutgoingMessage::Send(Box::new(msg.into())));
        }
    }
}
