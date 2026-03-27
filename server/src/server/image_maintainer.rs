use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, thread};

use crossbeam_channel::Receiver;
use sova_core::{Scene, clock::Clock, schedule::{SovaNotification, playback::PlaybackState}};
use tokio::sync::Mutex;

use crate::{ClientRegistry, ServerMessage, server::{POSITION_BROADCAST_INTERVAL_MS, broadcast_raw}};

fn notification_to_server_message(
    notif: SovaNotification,
    clock: &mut Clock,
) -> ServerMessage {
    match notif {
        SovaNotification::Tick
        | SovaNotification::QuantumChanged(_)
        | SovaNotification::TempoChanged(_) => {
            clock.capture_app_state();
            ServerMessage::ClockState(clock.tempo(), clock.beat(), clock.micros(), clock.quantum())
        }
        notif => ServerMessage::Notification(notif)
        
    }
}

pub fn start_image_maintainer(
    scheduler_notifications: Receiver<SovaNotification>,
    scene_image: Arc<Mutex<Scene>>,
    client_registry: ClientRegistry,
    is_playing: Arc<AtomicBool>,
    mut clock: Clock
) {
    thread::spawn(move || {
        let position_broadcast_interval =
            std::time::Duration::from_millis(POSITION_BROADCAST_INTERVAL_MS);
        let mut last_position_broadcast = std::time::Instant::now();

        loop {
            match scheduler_notifications.recv() {
                Ok(p) => {
                    let mut guard = scene_image.blocking_lock();
                    match &p {
                        SovaNotification::UpdatedScene(scene) => {
                            *guard = scene.clone();
                        }
                        SovaNotification::UpdatedSceneMode(mode) => {
                            guard.mode = *mode;
                        }
                        SovaNotification::UpdatedScenePrelude(prelude) => {
                            guard.prelude = prelude.clone();
                        }
                        SovaNotification::UpdatedLines(lines) => {
                            for (i, line) in lines {
                                guard.set_line(*i, line.clone());
                            }
                        }
                        SovaNotification::AddedLine(i, line) => {
                            guard.insert_line(*i, line.clone());
                        }
                        SovaNotification::RemovedLine(index) => {
                            guard.remove_line(*index);
                        }
                        SovaNotification::UpdatedFrames(frames) => {
                            for (line_id, frame_id, frame) in frames.iter() {
                                guard.line_mut(*line_id).set_frame(*frame_id, frame.clone());
                            }
                        }
                        SovaNotification::AddedFrame(line_id, frame_id, frame) => {
                            guard
                                .line_mut(*line_id)
                                .insert_frame(*frame_id, frame.clone());
                        }
                        SovaNotification::RemovedFrame(line_id, frame_id) => {
                            guard.line_mut(*line_id).remove_frame(*frame_id);
                        }
                        SovaNotification::PlaybackStateChanged(state) => {
                            let playing = match state {
                                PlaybackState::Stopped => false,
                                PlaybackState::Starting(_) => false,
                                PlaybackState::Playing => true,
                            };
                            is_playing.store(playing, Ordering::Relaxed);
                        }
                        _ => (),
                    };
                    drop(guard);

                    let should_broadcast = match &p {
                        SovaNotification::FramePositionChanged(_) => {
                            let now = std::time::Instant::now();
                            if now.duration_since(last_position_broadcast)
                                >= position_broadcast_interval
                            {
                                last_position_broadcast = now;
                                true
                            } else {
                                false
                            }
                        }
                        _ => true,
                    };

                    if should_broadcast {
                        let msg = notification_to_server_message(p, &mut clock);
                        let droppable = matches!(
                            &msg, 
                            ServerMessage::Notification(SovaNotification::FramePositionChanged(_))
                            | ServerMessage::ClockState(..)
                        );
                        broadcast_raw(&client_registry, &msg, droppable);
                    }
                }
                Err(_) => break,
            }
        }
    });
}