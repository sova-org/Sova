use std::{sync::Arc, vec};

use crate::clock::ClockServer;
use crate::pattern::Pattern;
use clock::TimeSpan;
use compiler::{dummylang::DummyCompiler, Compiler, ExternalCompiler};
use device_map::DeviceMap;
use lang::{control_asm::ControlASM, event::Event, variable::{Variable, VariableValue}, Instruction, Program};
use pattern::{script::Script, Track};
use schedule::{Scheduler, SchedulerMessage};
use world::World;
use protocol::{
    midi::init_default_midi_connection,
    midi::send,
    midi::MIDIMessage,
    midi::MIDIMessageType

};

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

    let clock_server = Arc::new(ClockServer::new(60.0, 4.0));
    let devices = Arc::new(DeviceMap::new());

    let (world_handle, world_iface) = World::create(clock_server.clone());
    let (sched_handle, sched_iface) = Scheduler::create(clock_server.clone(), devices.clone(), world_iface.clone());

    // Testing MIDI initialization
    let mut connection = protocol::midi::init_default_midi_connection(true)
        .expect("Failed to initialize MIDI connection");
    send(&mut connection, protocol::midi::MIDIMessage {
        // Basic information (channel and port)
        channel: 0, port: String::from("BuboCore"),
        // Message specific information
        payload: MIDIMessageType::NoteOn { note: 60, velocity: 100 }
    });
    send(&mut connection, protocol::midi::MIDIMessage {
        // Basic information (channel and port)
        channel: 0, port: String::from("BuboCore"),
        // Message specific information
        payload: MIDIMessageType::NoteOff { note: 60, velocity: 100 }
    });

    //let start = SystemTime::now();
    //let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backward");
    //let now = since_epoch.as_micros() as u64;

    //let sender2 = world_iface.clone();

    let bete = ExternalCompiler("bete".to_owned());
    let dummy = DummyCompiler;

    /*for i in 0..10 {
        let log0 = LogMessage::new(Severity::Debug, "Hello world !".to_owned());
        let log0 = ProtocolMessage::LOG(log0).timed(now + i * 1000 * 1000 * (i % 2));
        sender2.send(log0).unwrap();
    }*/

    // This is a test program for the scheduler
    let var = Variable::Ephemeral("A".to_owned());
    let crashtest_program: Program = vec![
        Instruction::Control(
            ControlASM::Mov(var.clone(), Variable::Constant(1.into()))
        ),
        Instruction::Effect(
            Event::Chord(vec![60], TimeSpan::Micros(100)),
            TimeSpan::Micros(1_000_000)
        ),
        Instruction::Control(
            ControlASM::Sub(var.clone(), Variable::Constant(1.into()))
        ),
        Instruction::Control(
            ControlASM::JumpIfLess(Variable::Constant((-1).into()), var.clone(), 1)
        ),
    ];

    // This is a test program obtained from a script
    let crashtest_parsed_program: Program = dummy.compile("N 5 2 1 C 3 7 100 4 5 A 1 3 5 8 6 3").unwrap();
    print!("{:?}", crashtest_parsed_program);

    let track = Track {
        steps: vec![1.0, 4.0],
        scripts: vec![
            Arc::new(Script::from(crashtest_program)),
            Arc::new(Script::from(crashtest_parsed_program))
        ],
        speed_factor: 1.0,
    };
    let pattern = Pattern {
        tracks: vec![track],
        track_index: 0,
    };
    let message = SchedulerMessage::UploadPattern(pattern);
    let _ = sched_iface.send(message);

    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");
}
