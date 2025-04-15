use bubocorelib::{
    clock::{ClockServer, TimeSpan},
    device_map::DeviceMap,
    lang::{Instruction, Program, event::Event},
    scene::{Scene, Line},
    protocol::midi::{MidiInterface, MidiOut},
    schedule::{Scheduler, SchedulerMessage},
    world::World,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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

    let script_1: Program = vec![Instruction::Effect(
        Event::MidiNote(
            60.into(),
            80.into(),
            0.into(),
            TimeSpan::Beats(0.9).into(),
            midi_name.clone().into(),
        ),
        TimeSpan::Micros(1_000_000).into(),
    )];

    let script_2: Program = vec![Instruction::Effect(
        Event::MidiNote(
            72.into(),
            80.into(),
            1.into(),
            TimeSpan::Beats(0.9).into(),
            midi_name.clone().into(),
        ),
        TimeSpan::Micros(1_000_000).into(),
    )];

    let script_3: Program = vec![Instruction::Effect(
        Event::MidiNote(
            86.into(),
            80.into(),
            2.into(),
            TimeSpan::Beats(0.9).into(),
            midi_name.clone().into(),
        ),
        TimeSpan::Micros(1_000_000).into(),
    )];
    let mut line1 = Line::new(vec![1.0]);
    let mut line2 = Line::new(vec![0.5]);
    let mut line3 = Line::new(vec![0.25]);
    line1.set_script(0, script_1.clone().into());
    line2.set_script(0, script_2.clone().into());
    line3.set_script(0, script_3.clone().into());
    let scene = Scene::new(vec![line1]);
    let message = SchedulerMessage::UploadScene(scene);
    let _ = sched_iface.send(message);

    // Adding lines
    thread::sleep(Duration::from_millis(1000));
    println!("Adding line 2");
    let message2 = SchedulerMessage::AddLine(line2);
    let _ = sched_iface.send(message2);
    thread::sleep(Duration::from_millis(1000));
    println!("Adding line 3");
    let message3 = SchedulerMessage::AddLine(line3);
    let _ = sched_iface.send(message3);

    // Removing lines 
    thread::sleep(Duration::from_millis(4000));
    println!("Removing line 0");
    let message3 = SchedulerMessage::RemoveLine(0);
    let _ = sched_iface.send(message3);
    thread::sleep(Duration::from_millis(4000));
    println!("Removing line 1");
    let message2 = SchedulerMessage::RemoveLine(1);
    let _ = sched_iface.send(message2);
    thread::sleep(Duration::from_millis(4000));
    println!("Removing line 2");
    let message1 = SchedulerMessage::RemoveLine(2);
    let _ = sched_iface.send(message1);

    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");
}
