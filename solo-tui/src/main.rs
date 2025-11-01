use std::sync::Arc;

use sova_core::{clock::ClockServer, compiler::{bali::BaliCompiler, dummylang::DummyCompiler, ExternalCompiler}, device_map::DeviceMap, init, lang::{interpreter::{boinx::BoinxInterpreterFactory, external::ExternalInterpreterFactory, InterpreterDirectory}, LanguageCenter, Transcoder}, schedule::SchedulerMessage};

use crate::app::App;

pub mod app;
pub mod page;
pub mod event;
pub mod ui;
pub mod widgets;

const DEFAULT_TEMPO : f64 = 120.0;
const DEFAULT_QUANTUM : f64 = 4.0;
const DEFAULT_MIDI_OUT : &str = "SovaOut";

fn create_language_center() -> Arc<LanguageCenter> {
    let mut transcoder = Transcoder::default();
    transcoder.add_compiler(BaliCompiler);
    transcoder.add_compiler(DummyCompiler);
    transcoder.add_compiler(ExternalCompiler);
    let mut interpreters = InterpreterDirectory::new();
    interpreters.add_factory(BoinxInterpreterFactory);
    interpreters.add_factory(ExternalInterpreterFactory);
    Arc::new(LanguageCenter { transcoder, interpreters })
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let clock_server = Arc::new(ClockServer::new(DEFAULT_TEMPO, DEFAULT_QUANTUM));
    let languages = create_language_center();
    let devices = Arc::new(DeviceMap::new());

    let _ = devices.create_virtual_midi_port(DEFAULT_MIDI_OUT);

    let (world_handle, sched_handle, sched_iface, sched_updates) = 
        init::start_scheduler_and_world(clock_server.clone(), devices.clone(), languages);

    let result = App::new(sched_iface.clone(), sched_updates, clock_server).run(terminal);
    ratatui::restore();

    devices.panic_all_midi_outputs();
    let _ = sched_iface.send(SchedulerMessage::Shutdown);
    let _ = world_handle.join();
    let _ = sched_handle.join();

    result
}
