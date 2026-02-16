use std::{env, sync::Arc};

use crossbeam_channel::unbounded;
use doux_sova::{DouxConfig, DouxManager, audio};
use langs::{
    bali::BaliCompiler, bob::BobCompiler, boinx::BoinxInterpreterFactory,
    forth::ForthInterpreterFactory,
};
use sova_core::{
    Scene, clock::{Clock, ClockServer}, device_map::DeviceMap, init, scene::Line, schedule::{ActionTiming, SchedulerMessage}, vm::{LanguageCenter, Transcoder, interpreter::InterpreterDirectory}
};

use crate::app::App;

pub mod app;
pub mod event;
pub mod notification;
pub mod page;
pub mod popup;
pub mod ui;
pub mod widgets;

const DEFAULT_TEMPO: f64 = 120.0;
const DEFAULT_QUANTUM: f64 = 4.0;
const DEFAULT_MIDI_OUT: &str = "SovaOut";

fn create_language_center() -> Arc<LanguageCenter> {
    let mut transcoder = Transcoder::default();
    transcoder.add_compiler(BaliCompiler);
    transcoder.add_compiler(BobCompiler);
    let mut interpreters = InterpreterDirectory::new();
    interpreters.add_factory(BoinxInterpreterFactory);
    interpreters.add_factory(ForthInterpreterFactory);
    Arc::new(LanguageCenter {
        transcoder,
        interpreters,
    })
}

fn connect_engine(device_map: &DeviceMap, clock_server: &Arc<ClockServer>) -> DouxManager {
    let clock : Clock = clock_server.into();
    // 1. List available devices (for UI)
    let devices = audio::list_output_devices();
    let mut default = String::new();
    for dev in &devices {
        let default_marker = if dev.is_default { 
            default = dev.name.clone();
            " [default]" 
        } else { "" };
        println!("{}: {} (max {} ch){}", dev.index, dev.name, dev.max_channels, default_marker);
    }

    // 2. Create configuration
    let mut samples = env::home_dir().unwrap();
    samples.push("samples");
    let config = DouxConfig::default()
        .with_output_device(default)
        .with_channels(2)
        .with_buffer_size(512)
        .with_sample_path(samples);

    // 3. Create and start manager
    let mut manager = DouxManager::new(config).unwrap();
    let proxy = manager.start(clock.micros()).unwrap();

    // 4. Register with Sova
    let _ = device_map.connect_audio_engine("doux", proxy);

    manager
}

fn main() -> color_eyre::Result<()> {
    let (log_tx, log_rx) = unbounded();
    sova_core::logger::init_embedded(log_tx);

    let clock_server = Arc::new(ClockServer::new(DEFAULT_TEMPO, DEFAULT_QUANTUM));
    let languages = create_language_center();
    let devices = Arc::new(DeviceMap::new());

    let _ = devices.create_virtual_midi_port(DEFAULT_MIDI_OUT);
    let _ = devices.create_osc_output_device("SovaOSC", "127.0.0.1", 57110);
    let _ = devices.create_osc_output_device("Dirt", "127.0.0.1", 57120);

    let mut doux_manager = connect_engine(&devices, &clock_server);

    let _ = devices.assign_slot(1, "doux");
    let _ = devices.assign_slot(2, "Dirt");

    let (world_handle, sched_handle, sched_iface, sched_updates) =
        init::start_scheduler_and_world(clock_server.clone(), devices.clone(), languages.clone());

    let initial_scene = Scene::new(vec![Line::default()]);
    let _ = sched_iface.send(SchedulerMessage::SetScene(
        initial_scene,
        ActionTiming::Immediate,
    ));

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new(
        sched_iface.clone(),
        sched_updates,
        log_rx,
        clock_server,
        devices.clone(),
        languages.clone(),
    )
    .run(terminal);
    ratatui::restore();

    devices.panic_all_midi_outputs();
    let _ = sched_iface.send(SchedulerMessage::Shutdown);
    let _ = world_handle.join();
    let _ = sched_handle.join();
    devices.clear();

    doux_manager.stop();

    result
}
