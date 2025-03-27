use std::{sync::Arc, vec};

use bubocoreserver::clock::ClockServer;
use bubocoreserver::pattern::Pattern;
use bubocoreserver::compiler::{
    bali::BaliCompiler,
    Compiler,
};
use bubocoreserver::device_map::DeviceMap;
use bubocoreserver::lang::Program;
use bubocoreserver::pattern::Sequence;
use bubocoreserver::protocol::midi::{MidiInterface, MidiOut};
use bubocoreserver::schedule::{Scheduler, SchedulerMessage};
use bubocoreserver::world::World;

/*
    */

fn main() {
    let bali = BaliCompiler;
    let bali_program: Program = bali.compile("
    (d test (+ 3 (+ 5 6)))
    (d bob 5)
    (@ (// 2 5) (d bob 6) (> (// 3 5) (n 4 5 12 94 out)))
    (@ (// 12 4) (n 5 5 12 34 out1) (<< (d test 120)))
    (> (// 3 9) (n 5 5 5 5 out))
    (n 1 2 3 4 out)
    (< (// 4 3) (d plop 3))
    ").unwrap();
    print!("PROGRAM\n{:?}\nENDPROGRAM\n", bali_program);

    let clock_server = Arc::new(ClockServer::new(60.0, 4.0));
    clock_server.link.enable(true);
    let devices = Arc::new(DeviceMap::new());

    let midi_name = "BuboCoreOut".to_owned();
    let mut midi_out = MidiOut::new(midi_name.clone()).unwrap();
    midi_out.connect_to_default(true).unwrap();
    devices.register_output_connection(midi_name.clone(), midi_out.into());

    let (world_handle, world_iface) = World::create(clock_server.clone());
    let (sched_handle, sched_iface) =
        Scheduler::create(clock_server.clone(), devices.clone(), world_iface.clone());

    let mut sequence = Sequence::new(vec![1.0]);
    sequence.set_script(0, bali_program.clone().into());
    let pattern = Pattern::new(vec![sequence]);

    let message = SchedulerMessage::UploadPattern(pattern);
    let _ = sched_iface.send(message);

    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");
}