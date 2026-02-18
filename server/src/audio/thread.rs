use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};

use crossbeam_channel::Sender;
use sova_core::clock::{Clock, ClockServer};
use sova_core::device_map::DeviceMap;
use sova_core::schedule::SovaNotification;
use tokio::sync::broadcast;

use super::{AudioEngineState, DouxConfig, DouxManager, ScopeCapture};
use crate::server::AudioRestartConfig;
use crate::AudioRestartRequest;

pub struct AudioThread {
    pub restart_tx: Sender<AudioRestartRequest>,
    pub thread_handle: std::thread::JoinHandle<()>,
    pub running: Arc<AtomicBool>,
    pub scope: Arc<StdMutex<Option<Arc<ScopeCapture>>>>,
}

pub fn spawn_audio_thread(
    initial_config: AudioRestartConfig,
    state_cache: Arc<StdMutex<AudioEngineState>>,
    devices: Arc<DeviceMap>,
    clock_server: Arc<ClockServer>,
    scope_sender: broadcast::Sender<SovaNotification>,
) -> AudioThread {
    let (restart_tx, restart_rx) = crossbeam_channel::unbounded::<AudioRestartRequest>();
    let running = Arc::new(AtomicBool::new(true));
    let running_flag = Arc::clone(&running);
    let scope_slot: Arc<StdMutex<Option<Arc<ScopeCapture>>>> = Arc::new(StdMutex::new(None));
    let scope_slot_inner = Arc::clone(&scope_slot);

    let thread_handle = std::thread::spawn(move || {
        let doux_config = build_doux_config(&initial_config);
        let mut manager: Option<DouxManager> = match DouxManager::new(doux_config) {
            Ok(mut mgr) => {
                let sync_time = Clock::from(&clock_server).micros();
                match mgr.start(sync_time) {
                    Ok(proxy) => {
                        if let Err(e) = devices.connect_audio_engine("Doux", proxy) {
                            eprintln!("Failed to register Doux engine: {}", e);
                            if let Ok(mut state) = state_cache.lock() {
                                state.error = Some(format!("Failed to register: {}", e));
                            }
                            None
                        } else {
                            println!("Doux audio engine started successfully.");
                            if let Err(e) = devices.assign_slot(2, "Doux") {
                                eprintln!("Failed to assign Doux to Slot 2: {}", e);
                            }
                            if let Ok(mut state) = state_cache.lock() {
                                *state = mgr.state();
                            }
                            if let Ok(mut slot) = scope_slot_inner.lock() {
                                *slot = mgr.scope_capture();
                            }
                            Some(mgr)
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to start Doux audio engine: {:?}", e);
                        if let Ok(mut state) = state_cache.lock() {
                            state.error = Some(format!("{:?}", e));
                        }
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to create Doux manager: {:?}", e);
                if let Ok(mut state) = state_cache.lock() {
                    state.error = Some(format!("{:?}", e));
                }
                None
            }
        };

        let mut frame_counter = 0u32;

        while running_flag.load(Ordering::Relaxed) {
            if let Ok(request) = restart_rx.try_recv() {
                println!("[ audio ] Received restart request");

                if let Some(ref mut mgr) = manager {
                    mgr.hush();
                    let _ = devices.remove_output_device("Doux");
                    mgr.stop();
                }

                let new_config = build_doux_config(&request.config);
                let result = match DouxManager::new(new_config) {
                    Ok(mut new_mgr) => {
                        let sync_time = Clock::from(&clock_server).micros();
                        match new_mgr.start(sync_time) {
                            Ok(proxy) => {
                                if let Err(e) = devices.connect_audio_engine("Doux", proxy) {
                                    manager = None;
                                    if let Ok(mut state) = state_cache.lock() {
                                        state.running = false;
                                        state.error =
                                            Some(format!("Failed to register: {}", e));
                                    }
                                    Err(format!("Failed to register audio engine: {}", e))
                                } else {
                                    if let Err(e) = devices.assign_slot(2, "Doux") {
                                        eprintln!("Failed to assign Doux to Slot 2: {}", e);
                                    }
                                    let new_state = new_mgr.state();
                                    if let Ok(mut state) = state_cache.lock() {
                                        *state = new_state.clone();
                                    }
                                    if let Ok(mut slot) = scope_slot_inner.lock() {
                                        *slot = new_mgr.scope_capture();
                                    }
                                    manager = Some(new_mgr);
                                    println!("[ audio ] Restart successful");
                                    Ok(new_state)
                                }
                            }
                            Err(e) => {
                                manager = None;
                                if let Ok(mut state) = state_cache.lock() {
                                    state.running = false;
                                    state.error = Some(format!("{:?}", e));
                                }
                                Err(format!("Failed to start audio engine: {:?}", e))
                            }
                        }
                    }
                    Err(e) => {
                        manager = None;
                        if let Ok(mut state) = state_cache.lock() {
                            state.running = false;
                            state.error = Some(format!("{:?}", e));
                        }
                        Err(format!("Failed to create audio manager: {:?}", e))
                    }
                };

                let _ = request.response_tx.send(result);
            }

            std::thread::sleep(std::time::Duration::from_millis(16));
            frame_counter += 1;

            if let Some(ref mgr) = manager {
                if let Some(scope) = mgr.scope_capture() {
                    let samples = scope.read_samples();
                    let _ = scope_sender.send(SovaNotification::ScopeData(samples));
                }

                if frame_counter.is_multiple_of(6)
                    && let Ok(engine) = mgr.engine_handle().lock()
                    && let Ok(mut cache) = state_cache.lock()
                {
                    cache.cpu_load = engine.metrics.load.get_load();
                    cache.active_voices = engine.active_voices;
                    cache.peak_voices =
                        engine.metrics.peak_voices.load(Ordering::Relaxed) as usize;
                    cache.schedule_depth =
                        engine.metrics.schedule_depth.load(Ordering::Relaxed) as usize;
                    cache.sample_pool_mb = engine.metrics.sample_pool_mb();
                }
            }
        }

        if let Some(mut mgr) = manager {
            mgr.hush();
            let _ = devices.remove_output_device("Doux");
            mgr.stop();
        }
    });

    AudioThread {
        restart_tx,
        thread_handle,
        running,
        scope: scope_slot,
    }
}

fn build_doux_config(cfg: &AudioRestartConfig) -> DouxConfig {
    let mut config = DouxConfig::default()
        .with_channels(cfg.channels)
        .with_max_voices(cfg.max_voices);
    if let Some(ref device) = cfg.device {
        config = config.with_output_device(device);
    }
    if let Some(ref device) = cfg.input_device {
        config = config.with_input_device(device);
    }
    for path in &cfg.sample_paths {
        config = config.with_sample_path(path);
    }
    if let Some(size) = cfg.buffer_size {
        config = config.with_buffer_size(size);
    }
    config
}

