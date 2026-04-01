use std::sync::atomic::Ordering;

use sova_core::{Scene, clock::Clock, schedule::{ActionTiming, SchedulerMessage, SovaNotification}};

use crate::{AudioCommand, AudioRestartRequest, BroadcastItem, ClientMessage, CoreRestartRequest, DEFAULT_CLIENT_NAME, ServerMessage, ServerState, Snapshot, server::broadcast_raw};

fn send_and_relay(state: &ServerState, msg: SchedulerMessage) -> ServerMessage {
    let iface = state.sched_iface.read().unwrap();
    if iface.send(msg.clone()).is_err() {
        return ServerMessage::InternalError("Scheduler communication error.".into());
    }
    drop(iface);
    state
        .client_registry
        .broadcast(BroadcastItem::Feedback(msg));
    ServerMessage::Success
}

pub async fn on_message(
    msg: ClientMessage,
    state: &ServerState,
    client_name: &mut String,
) -> ServerMessage {
    println!("[➡️ ] Client '{}' sent: {:?}", client_name, msg);

    match msg {
        ClientMessage::Chat(chat_msg) => {
            state.client_registry.broadcast(BroadcastItem::Filtered(
                client_name.clone(), ServerMessage::Chat(client_name.clone(), chat_msg),
            ));
            ServerMessage::Success
        }
        ClientMessage::SetName { name: new_name, .. } => {
            let mut clients_guard = state.clients.lock().await;
            let old_name = client_name.clone();
            let is_new_client = *client_name == DEFAULT_CLIENT_NAME;

            if is_new_client {
                println!("Client identified as: {}", new_name);
                clients_guard.push(new_name.clone());
            } else if let Some(i) = clients_guard.iter().position(|x| *x == old_name) {
                println!("Client {} changed name to {}", clients_guard[i], new_name);
                clients_guard[i] = new_name.clone();
            } else {
                eprintln!(
                    "Error: Could not find old name '{}' to replace. Adding '{}'.",
                    old_name, new_name
                );
                clients_guard.push(new_name.clone());
            }
            *client_name = new_name;

            let updated_clients = clients_guard.clone();
            drop(clients_guard);

            broadcast_raw(
                &state.client_registry,
                &ServerMessage::PeersUpdated(updated_clients),
                false,
            );

            ServerMessage::Success
        }
        ClientMessage::SchedulerControl(sched_msg) => send_and_relay(state, sched_msg),
        ClientMessage::GetClock => {
            let clock = Clock::from(&state.clock_server);
            ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum())
        }
        ClientMessage::GetScene => {
            ServerMessage::Notification(SovaNotification::UpdatedScene(state.scene_image.lock().await.clone()))
        }
        ClientMessage::GetPeers => ServerMessage::PeersUpdated(state.clients.lock().await.clone()),
        ClientMessage::GetSnapshot => {
            let scene = state.scene_image.lock().await.clone();
            let clock = Clock::from(&state.clock_server);
            let devices = state.devices.create_device_snapshot();
            let snapshot = Snapshot {
                scene,
                tempo: clock.tempo(),
                beat: clock.beat(),
                micros: clock.micros(),
                quantum: clock.quantum(),
                devices,
            };
            ServerMessage::Snapshot(snapshot)
        }
        ClientMessage::StartedEditingFrame(line_idx, frame_idx) => {
            state.client_registry.broadcast(BroadcastItem::Filtered(
                client_name.clone(), ServerMessage::PeerStartedEditing(client_name.clone(), line_idx, frame_idx),
            ));
            ServerMessage::Success
        }
        ClientMessage::StoppedEditingFrame(line_idx, frame_idx) => {
            state.client_registry.broadcast(BroadcastItem::Filtered(
                client_name.clone(), ServerMessage::PeerStoppedEditing(client_name.clone(), line_idx, frame_idx),
            ));
            ServerMessage::Success
        }
        ClientMessage::CursorPosition(line_idx, frame_idx, text_cursor) => {
            state.client_registry.broadcast(BroadcastItem::Filtered(
                client_name.clone(), ServerMessage::PeerCursorMoved(client_name.clone(), line_idx, frame_idx, text_cursor),
            ));
            ServerMessage::Success
        }
        ClientMessage::RequestDeviceList => {
            println!("[ info ] Client '{}' requested device list.", client_name);
            ServerMessage::Notification(SovaNotification::DeviceListChanged(state.devices.device_list()))
        }
        ClientMessage::ConnectMidiDeviceByName(device_name) => {
            match state.devices.connect_midi_by_name(&device_name) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    let msg = ServerMessage::Notification(SovaNotification::DeviceListChanged(updated_list));
                    broadcast_raw(
                        &state.client_registry,
                        &msg,
                        false,
                    );
                    msg
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to connect device '{}': {}",
                    device_name, e
                )),
            }
        }
        ClientMessage::DisconnectMidiDeviceByName(device_name) => {
            match state.devices.disconnect_midi_by_name(&device_name) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    let msg = ServerMessage::Notification(SovaNotification::DeviceListChanged(updated_list));
                    broadcast_raw(
                        &state.client_registry,
                        &msg,
                        false,
                    );
                    msg
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to disconnect device '{}': {}",
                    device_name, e
                )),
            }
        }
        ClientMessage::CreateVirtualMidiOutput(device_name) => {
            match state.devices.create_virtual_midi_port(&device_name) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    let msg = ServerMessage::Notification(SovaNotification::DeviceListChanged(updated_list));
                    broadcast_raw(
                        &state.client_registry,
                        &msg,
                        false,
                    );
                    msg
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to create virtual device '{}': {}",
                    device_name, e
                )),
            }
        }
        ClientMessage::AssignDeviceToSlot(slot_id, device_name) => {
            match state.devices.assign_slot(slot_id, &device_name) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    let msg = ServerMessage::Notification(SovaNotification::DeviceListChanged(updated_list));
                    broadcast_raw(
                        &state.client_registry,
                        &msg,
                        false,
                    );
                    msg
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to assign slot {}: {}",
                    slot_id, e
                )),
            }
        }
        ClientMessage::UnassignDeviceFromSlot(slot_id) => {
            match state.devices.unassign_slot(slot_id) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    let msg = ServerMessage::Notification(SovaNotification::DeviceListChanged(updated_list));
                    broadcast_raw(
                        &state.client_registry,
                        &msg,
                        false,
                    );
                    msg
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to unassign slot {}: {}",
                    slot_id, e
                )),
            }
        }
        ClientMessage::CreateOscDevice(name, ip, port) => {
            match state.devices.create_osc_output_device(&name, &ip, port) {
                Ok(_) => {
                    let updated_list = state.devices.device_list();
                    let msg = ServerMessage::Notification(SovaNotification::DeviceListChanged(updated_list));
                    broadcast_raw(
                        &state.client_registry,
                        &msg,
                        false,
                    );
                    msg
                }
                Err(e) => ServerMessage::InternalError(format!(
                    "Failed to create OSC device '{}': {}",
                    name, e
                )),
            }
        }
        ClientMessage::RemoveOscDevice(name) => match state.devices.remove_output_device(&name) {
            Ok(_) => {
                let updated_list = state.devices.device_list();
                let msg = ServerMessage::Notification(SovaNotification::DeviceListChanged(updated_list));
                    broadcast_raw(
                        &state.client_registry,
                        &msg,
                        false,
                    );
                    msg
            }
            Err(e) => ServerMessage::InternalError(format!(
                "Failed to remove OSC device '{}': {}",
                name, e
            )),
        },
        ClientMessage::SetDeviceLatency(name, latency) => {
            state.devices.set_latency(name, latency);
            let updated_list = state.devices.device_list();
            let msg = ServerMessage::Notification(SovaNotification::DeviceListChanged(updated_list));
                    broadcast_raw(
                        &state.client_registry,
                        &msg,
                        false,
                    );
                    msg
        }
        ClientMessage::GetLine(line_id) => {
            let scene = state.scene_image.lock().await;
            if let Some(line) = scene.line(line_id) {
                ServerMessage::Notification(SovaNotification::UpdatedLines(vec![(line_id, line.clone())]))
            } else {
                ServerMessage::InternalError(format!("No line at index {}", line_id))
            }
        }
        ClientMessage::GetFrame(line_id, frame_id) => {
            let scene = state.scene_image.lock().await;
            if let Some(frame) = scene.frame(line_id, frame_id) {
                ServerMessage::Notification(SovaNotification::UpdatedFrames(vec![(line_id, frame_id, frame.clone())]))
            } else {
                ServerMessage::InternalError(format!(
                    "Unable to get frame {} at line {}",
                    frame_id, line_id
                ))
            }
        }
        ClientMessage::RestoreDevices(devices) => {
            let missing_devices = state.devices.restore_from_snapshot(devices);
            let updated_list = state.devices.device_list();
            broadcast_raw(
                &state.client_registry,
                &ServerMessage::Notification(SovaNotification::DeviceListChanged(updated_list)),
                false,
            );
            ServerMessage::DevicesRestored { missing_devices }
        }
        ClientMessage::PreviewSample {
            folder,
            index,
            begin,
        } => {
            use sova_core::vm::event::ConcreteEvent;
            use sova_core::vm::variable::VariableValue;

            let mut args = std::collections::HashMap::new();
            args.insert("s".to_string(), VariableValue::Str(folder));
            args.insert("n".to_string(), VariableValue::Integer(index as i64));
            args.insert("gain".to_string(), VariableValue::Float(1.0));
            args.insert("gate".to_string(), VariableValue::Float(2.0));
            args.insert("begin".to_string(), VariableValue::Float(begin));

            let event = ConcreteEvent::Dirt { args, device_id: 0 };

            let clock = Clock::from(&state.clock_server);
            let time = clock.micros();
            let messages = state
                .devices
                .map_event_for_device_name("Doux", event, time, &clock);

            for timed in messages {
                let _ = timed.message.send();
            }

            ServerMessage::Success
        }
        ClientMessage::GetAudioEngineState => {
            ServerMessage::AudioEngineState(state.get_audio_engine_state())
        }
        ClientMessage::RestartAudioEngine(mut config) => {
            let Some(ref restart_tx) = state.audio_restart_tx else {
                return ServerMessage::InternalError("Audio engine not available".to_string());
            };

            #[cfg(feature = "default-samples")]
            {
                let default = crate::audio::default_samples::ensure_default_samples();
                if !config.sample_paths.contains(&default) {
                    config.sample_paths.insert(0, default);
                }
            }

            let (response_tx, response_rx) = crossbeam_channel::bounded(1);
            let request = AudioRestartRequest {
                config,
                response_tx,
            };

            if restart_tx.send(request).is_err() {
                return ServerMessage::InternalError("Failed to send restart request".to_string());
            }

            match response_rx.recv() {
                Ok(Ok(new_state)) => ServerMessage::AudioEngineState(new_state),
                Ok(Err(e)) => ServerMessage::InternalError(format!("Audio restart failed: {}", e)),
                Err(_) => ServerMessage::InternalError("Audio restart channel closed".to_string()),
            }
        }
        ClientMessage::ResetScene(timing) => {
            send_and_relay(state, SchedulerMessage::SetScene(Scene::default(), timing))
        }
        ClientMessage::RestartCore => {
            let restart_tx = state.core_restart_tx.clone();
            let (response_tx, mut response_rx) = tokio::sync::mpsc::channel(1);
            if restart_tx.send(CoreRestartRequest { response_tx }).await.is_err() {
                return ServerMessage::InternalError("Core restart channel closed".into());
            }
            match response_rx.recv().await {
                Some(Ok(())) => ServerMessage::Success,
                Some(Err(e)) => ServerMessage::InternalError(format!("Core restart failed: {e}")),
                None => ServerMessage::InternalError("Core restart channel closed".into()),
            }
        }
        ClientMessage::HydraCode(code) => {
            broadcast_raw(
                &state.client_registry,
                &ServerMessage::HydraCode(client_name.clone(), code),
                false,
            );
            ServerMessage::Success
        }
        ClientMessage::SetMasterVolume(vol) => {
            let clamped = vol.clamp(0.0, 1.0);
            state
                .master_gain
                .store(clamped.to_bits(), Ordering::Relaxed);
            ServerMessage::Success
        }
        ClientMessage::EnableFeedback => {
            let scene = state.scene_image.lock().await.clone();
            let clock = Clock::from(&state.clock_server);
            ServerMessage::FeedbackEnabled {
                scene,
                tempo: clock.tempo(),
                quantum: clock.quantum(),
                is_playing: state.is_playing.load(Ordering::Relaxed),
            }
        }
        ClientMessage::Hush => {
            let _ = state
                .sched_iface
                .read()
                .unwrap()
                .send(SchedulerMessage::TransportStop(ActionTiming::Immediate));
            if let Some(ref tx) = state.audio_cmd_tx {
                let _ = tx.send(AudioCommand::Hush);
            }
            state.devices.panic_all_midi_outputs();
            ServerMessage::Success
        }
        ClientMessage::ScriptEdit { li, fi, ops } => {
            broadcast_raw(
                &state.client_registry,
                &ServerMessage::ScriptEdit {
                    sender: client_name.clone(),
                    li,
                    fi,
                    ops,
                },
                true,
            );
            ServerMessage::Success
        }
        ClientMessage::SetLinkEnabled(enabled) => {
            state.clock_server.link.enable(enabled);
            broadcast_raw(
                &state.client_registry,
                &ServerMessage::LinkState {
                    enabled,
                    start_stop_sync: state.clock_server.link.is_start_stop_sync_enabled(),
                    num_peers: state.clock_server.link.num_peers() as u32,
                },
                false,
            );
            ServerMessage::Success
        }
        ClientMessage::SetStartStopSync(enabled) => {
            state.clock_server.link.enable_start_stop_sync(enabled);
            broadcast_raw(
                &state.client_registry,
                &ServerMessage::LinkState {
                    enabled: state.clock_server.link.is_enabled(),
                    start_stop_sync: enabled,
                    num_peers: state.clock_server.link.num_peers() as u32,
                },
                false,
            );
            ServerMessage::Success
        }
        ClientMessage::Panic => {
            let _ = state
                .sched_iface
                .read()
                .unwrap()
                .send(SchedulerMessage::TransportStop(ActionTiming::Immediate));
            if let Some(ref tx) = state.audio_cmd_tx {
                let _ = tx.send(AudioCommand::Panic);
            }
            state.devices.panic_all_midi_outputs();
            ServerMessage::Success
        }
    }
}