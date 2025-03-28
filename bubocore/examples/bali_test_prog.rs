use std::{sync::Arc, vec};

use bubocorelib::clock::ClockServer;
use bubocorelib::pattern::Pattern;
use bubocorelib::compiler::{
    bali::BaliCompiler,
    Compiler,
};
use bubocorelib::device_map::DeviceMap;
use bubocorelib::lang::Program;
use bubocorelib::pattern::Sequence;
use bubocorelib::protocol::midi::{MidiInterface, MidiOut};
use bubocorelib::schedule::{Scheduler, SchedulerMessage};
use bubocorelib::world::World;

/*
    */

fn main() {
    let bali = BaliCompiler;
    let bali_program: Program = bali.compile("
    (d note 20)
    (> (// 1 4) (n note 90 1 0 nimp))
    (> (// 1 2) (d note (+ note 13)))
    (@ (// 3 4) 
        (<< (n note 90 1 0 nimp))
        (d note (+ note 13))
        (>> (n note 90 1 0 nimp))
        (> (// 1 4) (n 100 90 1 0 nimp))
    )
    (@ (// 5 4)
        (< (// 1 8) (n 101 90 1 0 nimp))
        (> (// 1 8) (n 102 90 1 0 nimp))
    )
    ").unwrap();
    //print!("PROGRAM\n{:?}\nENDPROGRAM\n", bali_program);

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

    let mut sequence = Sequence::new(vec![1.0]);
    sequence.set_script(0, bali_program.clone().into());
    let pattern = Pattern::new(vec![sequence]);

    let message = SchedulerMessage::UploadPattern(pattern);
    let _ = sched_iface.send(message);

    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");
}