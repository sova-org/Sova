use std::sync::{
    Arc, Mutex as StdMutex,
    atomic::Ordering,
};

use crossbeam_channel::Sender;
use sova_core::{
    clock::ClockServer,
    device_map::DeviceMap,
    schedule::SchedulerMessage,
};
use sova_server::{AudioEngineState, AudioRestartConfig, BroadcastItem};
use sova_server::audio::{AudioThread, spawn_audio_thread};
use tokio::sync::broadcast;

pub struct FeedbackEngine {
    sched_iface: Sender<SchedulerMessage>,
    _sched_handle: std::thread::JoinHandle<()>,
    _world_handle: std::thread::JoinHandle<()>,
    devices: Arc<DeviceMap>,
    audio_thread: Option<AudioThread>,
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
        let (dummy_broadcast, _) = broadcast::channel::<BroadcastItem>(16);
        let audio_thread = spawn_audio_thread(
            audio_config,
            audio_engine_state,
            devices.clone(),
            clock_server,
            dummy_broadcast,
        );

        let notification_drainer = std::thread::spawn(move || {
            while sched_notifications.recv().is_ok() {}
        });

        Ok(Self {
            sched_iface,
            _sched_handle: sched_handle,
            _world_handle: world_handle,
            devices,
            audio_thread: Some(audio_thread),
            _notification_drainer: notification_drainer,
        })
    }

    pub fn devices(&self) -> &Arc<DeviceMap> {
        &self.devices
    }

    pub fn send(&self, msg: SchedulerMessage) {
        let _ = self.sched_iface.send(msg);
    }

    pub fn scope_data(&self) -> Vec<f32> {
        self.audio_thread
            .as_ref()
            .and_then(|at| at.scope.lock().ok())
            .and_then(|guard| guard.as_ref().map(|scope| scope.read_samples()))
            .unwrap_or_default()
    }
}

impl Drop for FeedbackEngine {
    fn drop(&mut self) {
        if let Some(at) = self.audio_thread.take() {
            at.running.store(false, Ordering::Relaxed);
            let _ = at.thread_handle.join();
        }
        let _ = self.sched_iface.send(SchedulerMessage::Shutdown);
        // Scheduler and world threads will terminate after receiving Shutdown.
        // The notification drainer will exit when the channel closes.
    }
}
