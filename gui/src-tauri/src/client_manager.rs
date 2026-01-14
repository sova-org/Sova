use anyhow::Result;
use serde::Serialize;
use sova_server::{ClientMessage, SovaClient, ServerMessage};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

#[derive(Clone, Serialize)]
struct ClientDisconnectEvent {
    reason: String,
}

pub struct ClientManager {
    app_handle: AppHandle,
    client: Option<SovaClient>,
    message_sender: Option<mpsc::UnboundedSender<ClientMessage>>,
    disconnect_sender: Option<mpsc::UnboundedSender<()>>,
}

impl ClientManager {
    pub fn new(app_handle: AppHandle) -> Self {
        ClientManager {
            app_handle,
            client: None,
            message_sender: None,
            disconnect_sender: None,
        }
    }

    pub async fn connect(&mut self, ip: String, port: u16) -> Result<()> {
        let mut client = SovaClient::new(ip, port);
        client.connect().await?;

        let (msg_tx, msg_rx) = mpsc::unbounded_channel();
        let (disconnect_tx, disconnect_rx) = mpsc::unbounded_channel();

        self.spawn_client_task(client, msg_rx, disconnect_rx, self.app_handle.clone()).await;

        self.message_sender = Some(msg_tx);
        self.disconnect_sender = Some(disconnect_tx);

        Ok(())
    }

    async fn spawn_client_task(
        &self,
        mut client: SovaClient,
        mut message_receiver: mpsc::UnboundedReceiver<ClientMessage>,
        mut disconnect_receiver: mpsc::UnboundedReceiver<()>,
        app_handle: AppHandle,
    ) {
        tauri::async_runtime::spawn(async move {
            let mut consecutive_failures = 0;
            let mut consecutive_emit_failures = 0;
            let mut last_message = std::time::Instant::now();
            const MESSAGE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
            loop {
                tokio::select! {
                    Some(message) = message_receiver.recv() => {
                        if let Err(e) = client.send(message).await {
                            sova_core::log_error!("Failed to send message: {}", e);
                            let _ = app_handle.emit("client-disconnected", ClientDisconnectEvent {
                                reason: "send_error".to_string(),
                            });
                            return;
                        }
                    }
                    Some(_) = disconnect_receiver.recv() => {
                        sova_core::log_info!("Disconnect signal received, closing connection");
                        if let Err(e) = client.disconnect().await {
                            sova_core::log_error!("Failed to disconnect client: {}", e);
                        }
                        let _ = app_handle.emit("client-disconnected", ClientDisconnectEvent {
                            reason: "manual_disconnect".to_string(),
                        });
                        return;
                    }
                    read_result = async {
                        // Timeout ready() check to prevent blocking forever on dead connections
                        match tokio::time::timeout(
                            tokio::time::Duration::from_millis(100),
                            client.ready()
                        ).await {
                            Ok(true) => {
                                // Data is available - read it with timeout
                                match tokio::time::timeout(
                                    tokio::time::Duration::from_secs(1),
                                    client.read()
                                ).await {
                                    Ok(result) => result,
                                    Err(_) => Err(std::io::Error::new(
                                        std::io::ErrorKind::TimedOut,
                                        "Read timeout after ready"
                                    ))
                                }
                            }
                            Ok(false) => {
                                // ready() returned false - connection closed by peer
                                Err(std::io::Error::new(
                                    std::io::ErrorKind::ConnectionReset,
                                    "Connection closed"
                                ))
                            }
                            Err(_) => {
                                // ready() timed out - no data available yet (NORMAL during idle)
                                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                                Err(std::io::Error::new(
                                    std::io::ErrorKind::WouldBlock,
                                    "No data available"
                                ))
                            }
                        }
                    } => {
                        match read_result {
                            Ok(message) => {
                                consecutive_failures = 0;
                                last_message = std::time::Instant::now();

                                if let Err(e) = Self::handle_server_message(&app_handle, message) {
                                    sova_core::log_error!("Failed to handle server message: {}", e);
                                    consecutive_emit_failures += 1;
                                    if consecutive_emit_failures > 5 {
                                        sova_core::log_error!("Too many emit failures ({}), disconnecting", consecutive_emit_failures);
                                        let _ = app_handle.emit("client-disconnected", ClientDisconnectEvent {
                                            reason: "emit_failures".to_string(),
                                        });
                                        return;
                                    }
                                } else {
                                    consecutive_emit_failures = 0;
                                }
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                // No data available - NOT a failure, this is normal during idle
                                // Check message timeout (clock ticks serve as implicit keep-alive)
                                if last_message.elapsed() > MESSAGE_TIMEOUT {
                                    sova_core::log_error!("No messages for {:?}, disconnecting", MESSAGE_TIMEOUT);
                                    let _ = app_handle.emit("client-disconnected", ClientDisconnectEvent {
                                        reason: "message_timeout".to_string(),
                                    });
                                    return;
                                }
                            }
                            Err(_) => {
                                // Real error - increment failures
                                consecutive_failures += 1;
                                if consecutive_failures > 100 {
                                    sova_core::log_error!("Connection dead after {} failures, disconnecting", consecutive_failures);
                                    if let Err(e) = client.disconnect().await {
                                        sova_core::log_error!("Failed to disconnect client: {}", e);
                                    }
                                    let _ = app_handle.emit("client-disconnected", ClientDisconnectEvent {
                                        reason: "connection_lost".to_string(),
                                    });
                                    return;
                                }
                                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn send_message(&self, message: ClientMessage) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            sender.send(message)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Not connected"))
        }
    }

    pub fn is_connected(&self) -> bool {
        if let Some(sender) = &self.message_sender {
            // Check if the channel is still open (task is still running)
            !sender.is_closed()
        } else {
            false
        }
    }

    fn handle_server_message(app_handle: &AppHandle, message: ServerMessage) -> Result<()> {
        use ServerMessage::*;

        match message {
            Hello { username, scene, devices, peers, link_state, is_playing, available_languages } => {
                app_handle.emit("server:hello", serde_json::json!({
                    "username": username,
                    "scene": scene,
                    "devices": devices,
                    "peers": peers,
                    "linkState": {
                        "tempo": link_state.0,
                        "beat": link_state.1,
                        "phase": link_state.2,
                        "numPeers": link_state.3,
                        "isEnabled": link_state.4,
                    },
                    "isPlaying": is_playing,
                    "availableLanguages": available_languages,
                }))?;
            }

            PeersUpdated(peers) => {
                app_handle.emit("server:peers-updated", peers)?;
            }

            PeerStartedEditing(user, line_id, frame_id) => {
                app_handle.emit("server:peer-started-editing", serde_json::json!({
                    "user": user,
                    "lineId": line_id,
                    "frameId": frame_id,
                }))?;
            }

            PeerStoppedEditing(user, line_id, frame_id) => {
                app_handle.emit("server:peer-stopped-editing", serde_json::json!({
                    "user": user,
                    "lineId": line_id,
                    "frameId": frame_id,
                }))?;
            }

            PlaybackStateChanged(state) => {
                app_handle.emit("server:playback-state-changed", state)?;
            }

            Log(log_message) => {
                app_handle.emit("server:log", log_message)?;
            }

            Chat(user, msg) => {
                app_handle.emit("server:chat", serde_json::json!({
                    "user": user,
                    "message": msg,
                }))?;
            }

            Success => {
                app_handle.emit("server:success", ())?;
            }

            InternalError(msg) => {
                app_handle.emit("server:error", msg)?;
            }

            ConnectionRefused(reason) => {
                app_handle.emit("server:connection-refused", reason)?;
            }

            Snapshot(snapshot) => {
                app_handle.emit("server:snapshot", snapshot)?;
            }

            DeviceList(devices) => {
                app_handle.emit("server:device-list", devices)?;
            }

            ClockState(tempo, beat, micros, quantum) => {
                app_handle.emit("server:clock-state", serde_json::json!({
                    "tempo": tempo,
                    "beat": beat,
                    "micros": micros,
                    "quantum": quantum,
                }))?;
            }

            SceneValue(scene) => {
                app_handle.emit("server:scene", scene)?;
            }

            LineValues(lines) => {
                app_handle.emit("server:line-values", lines)?;
            }

            LineConfigurations(lines) => {
                app_handle.emit("server:line-configurations", lines)?;
            }

            AddLine(idx, line) => {
                app_handle.emit("server:add-line", serde_json::json!({
                    "index": idx,
                    "line": line,
                }))?;
            }

            RemoveLine(idx) => {
                app_handle.emit("server:remove-line", idx)?;
            }

            FrameValues(frames) => {
                app_handle.emit("server:frame-values", frames)?;
            }

            AddFrame(line_id, frame_id, frame) => {
                app_handle.emit("server:add-frame", serde_json::json!({
                    "lineId": line_id,
                    "frameId": frame_id,
                    "frame": frame,
                }))?;
            }

            RemoveFrame(line_id, frame_id) => {
                app_handle.emit("server:remove-frame", serde_json::json!({
                    "lineId": line_id,
                    "frameId": frame_id,
                }))?;
            }

            FramePosition(positions) => {
                app_handle.emit("server:frame-position", positions)?;
            }

            GlobalVariablesUpdate(vars) => {
                app_handle.emit("server:global-variables", vars)?;
            }

            CompilationUpdate(line_id, frame_id, script_id, state) => {
                sova_core::log_info!("[CompilationUpdate] Received: line={}, frame={}, scriptId={}, state={:?}", line_id, frame_id, script_id, state);
                app_handle.emit("server:compilation-update", serde_json::json!({
                    "lineId": line_id,
                    "frameId": frame_id,
                    "scriptId": script_id.to_string(),
                    "state": state,
                }))?;
            }

            DevicesRestored { missing_devices } => {
                app_handle.emit("server:devices-restored", serde_json::json!({
                    "missingDevices": missing_devices,
                }))?;
            }
        }

        Ok(())
    }

    pub fn disconnect(&mut self) {
        // Send disconnect signal to the task
        if let Some(disconnect_sender) = &self.disconnect_sender {
            let _ = disconnect_sender.send(());
        }

        // Clear all channels
        self.message_sender = None;
        self.disconnect_sender = None;
        self.client = None;
    }
}