use bubocorelib::{
    clock::{ClockServer, TimeSpan},
    device_map::DeviceMap,
    lang::{Instruction, Program, event::Event},
    protocol::midi::{MidiInterface, MidiOut},
    scene::{line::Line, Scene},
    schedule::{Scheduler, message::SchedulerMessage},
    world::World,
};
use std::sync::Arc;

fn main() {
    let clock_server = Arc::new(ClockServer::new(60.0, 4.0));
    clock_server.link.enable(true);
    let devices = Arc::new(DeviceMap::new());

    let midi_name = "BuboCoreOut".to_owned();
    let mut midi_out = MidiOut::new(midi_name.clone()).unwrap();
    midi_out.connect_to_default(true).unwrap();
    devices.register_output_connection(midi_name.clone(), midi_out.into());

    let (world_handle, world_iface) = World::create(clock_server.clone());
    let (sched_handle, sched_iface, _) =
        Scheduler::create(clock_server.clone(), devices.clone(), world_iface.clone());

    let aftertouch_script: Program = vec![Instruction::Effect(
        Event::MidiAftertouch(20.into(), 40.into(), 0.into(), midi_name.clone().into()),
        TimeSpan::Micros(1_000_000).into(),
    )];

    let mut line = Line::new(vec![0.25]);
    line.set_script(0, aftertouch_script.clone().into());
    let scene = Scene::new(vec![line]);

    let message = SchedulerMessage::UploadScene(scene);
    let _ = sched_iface.send(message);

    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");
}
