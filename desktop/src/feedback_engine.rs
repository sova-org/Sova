use std::sync::{Arc, Mutex as StdMutex, atomic::Ordering};

use crossbeam_channel::Sender;
use sova_core::{clock::ClockServer, device_map::DeviceMap, schedule::SchedulerMessage};
use sova_server::audio::{AudioThread, spawn_audio_thread};
use sova_server::{AudioEngineState, AudioRestartConfig, AudioRestartRequest, ClientRegistry};

pub struct FeedbackEngine {
    sched_iface: Sender<SchedulerMessage>,
    _sched_handle: std::thread::JoinHandle<()>,
    _world_handle: std::thread::JoinHandle<()>,
    devices: Arc<DeviceMap>,
    audio_thread: Option<AudioThread>,
    audio_engine_state: Arc<StdMutex<AudioEngineState>>,
    _notification_drainer: std::thread::JoinHandle<()>,
}

impl FeedbackEngine {
    pub fn start(audio_config: AudioRestartConfig) -> Result<Self, String> {
        let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
        clock_server.link.enable(true);

        let devices = Arc::new(DeviceMap::new());
        if let Err(e) = devices.create_virtual_midi_port("Sova (Local)") {
            eprintln!("Feedback: Failed to create virtual MIDI port: {}", e);
        } else if let Err(e) = devices.assign_slot(1, "Sova (Local)") {
            eprintln!("Feedback: Failed to assign Sova (Local) to Slot 1: {}", e);
        }

        let languages = Arc::new(langs::create_language_center());

        let (world_handle, sched_handle, sched_iface, sched_notifications) =
            sova_core::init::start_scheduler_and_world(
                clock_server.clone(),
                devices.clone(),
                languages,
            );

        let audio_engine_state = Arc::new(StdMutex::new(AudioEngineState::default()));
        let dummy_registry = ClientRegistry::new();
        let audio_thread = spawn_audio_thread(
            audio_config,
            Arc::clone(&audio_engine_state),
            devices.clone(),
            clock_server,
            dummy_registry,
        );

        let notification_drainer =
            std::thread::spawn(move || while sched_notifications.recv().is_ok() {});

        Ok(Self {
            sched_iface,
            _sched_handle: sched_handle,
            _world_handle: world_handle,
            devices,
            audio_thread: Some(audio_thread),
            audio_engine_state,
            _notification_drainer: notification_drainer,
        })
    }

    pub fn devices(&self) -> &Arc<DeviceMap> {
        &self.devices
    }

    pub fn send(&self, msg: SchedulerMessage) {
        let _ = self.sched_iface.send(msg);
    }

    pub fn audio_state(&self) -> AudioEngineState {
        self.audio_engine_state
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    pub fn restart_audio(&self, config: AudioRestartConfig) {
        if let Some(at) = &self.audio_thread {
            let (tx, _rx) = crossbeam_channel::bounded(1);
            let _ = at.restart_tx.send(AudioRestartRequest {
                config,
                response_tx: tx,
            });
        }
    }

    pub fn fill_scope_data(&self, buf: &mut Vec<f32>) {
        buf.clear();
        if let Some(at) = self.audio_thread.as_ref()
            && let Ok(guard) = at.scope.lock()
            && let Some(scope) = guard.as_ref()
        {
            buf.extend_from_slice(&scope.read_samples());
        }
    }

    pub fn fill_peak_data(&self, buf: &mut Vec<f32>) {
        buf.clear();
        if let Some(at) = self.audio_thread.as_ref()
            && let Ok(guard) = at.peaks.lock()
            && let Some(p) = guard.as_ref()
        {
            buf.extend_from_slice(&p.read_and_reset());
        }
    }
}

impl Drop for FeedbackEngine {
    fn drop(&mut self) {
        if let Some(at) = self.audio_thread.take() {
            at.running.store(false, Ordering::Relaxed);
            let _ = at.thread_handle.join();
        }
        let _ = self.sched_iface.send(SchedulerMessage::Shutdown);
    }
}
