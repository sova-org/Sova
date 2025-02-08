use std::{clone, collections::HashMap, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use crate::clock::ClockServer;
use clock::TimeSpan;
use compiler::{dummyast::DummyCompiler, Compiler, ExternalCompiler};
use device_map::DeviceMap;
use lang::{Event, Instruction, Program};
use protocol::{log::{LogMessage, Severity}, ProtocolMessage};
use schedule::Scheduler;
use world::World;

pub mod schedule;
pub mod clock;
pub mod io;
pub mod world;
pub mod protocol;
pub mod lang;
pub mod pattern;
pub mod compiler;
pub mod device_map;

fn main() {

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let devices = Arc::new(DeviceMap::new());

    let (world_handle, world_iface) = World::create(clock_server.clone());
    let (sched_handle, sched_iface) = Scheduler::create(clock_server.clone(), devices.clone(), world_iface.clone());

    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backward");
    let now = since_epoch.as_micros() as u64;

    let sender2 = world_iface.clone();

    let bete = ExternalCompiler("bete".to_owned());
    let dummy = DummyCompiler;

    for i in 0..10 {
        let log0 = LogMessage::new(Severity::Debug, "Hello world !".to_owned());
        let log0 = ProtocolMessage::LOG(log0).timed(now + i * 1000 * 1000 * (i % 2));
        sender2.send(log0).unwrap();
    }

    // This is a test program for the scheduler
    let crashtest_program: Program = vec![
        Instruction::Effect(
            Event::Note(60, TimeSpan::Micros(1)),
            TimeSpan::Micros(2)
        ),
        Instruction::Effect(
            Event::Exit,
            TimeSpan::Micros(4)
        )
    ];

    // This is a test program obtained from a script
    let crashtest_parsed_program: Program = dummy.compile("N 5 2 1 C 3 7 100 4 5").unwrap();

    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");
}
