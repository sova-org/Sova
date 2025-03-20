use std::{sync::Arc, vec, collections::HashMap};
use crate::clock::ClockServer;
use crate::pattern::Pattern;
use clock::TimeSpan;
use compiler::{
    dummylang::DummyCompiler,
    Compiler,
    ExternalCompiler
};
use device_map::DeviceMap;
use lang::{
    control_asm::ControlASM,
    event::Event,
    variable::Variable,
    Instruction, Program
};
use pattern::{script::Script, Sequence};
use protocol::midi::{
    MidiOut,
    MidiIn,
    MidiInterface,
    MIDIMessage,
    MIDIMessageType
};
use schedule::{Scheduler, SchedulerMessage};
use world::World;

pub mod clock;
pub mod compiler;
pub mod device_map;
pub mod io;
pub mod lang;
pub mod pattern;
pub mod protocol;
pub mod schedule;
pub mod world;

fn main() {
    let clock_server = Arc::new(ClockServer::new(60.0, 4.0));
    let devices = Arc::new(DeviceMap::new());

    let (world_handle, world_iface) = World::create(clock_server.clone());
    let (sched_handle, sched_iface) =
        Scheduler::create(clock_server.clone(), devices.clone(), world_iface.clone());

    // Sending a few MIDI messages out
    let mut midi_out = MidiOut::new("BuboCoreOut").unwrap();
    midi_out.connect_to_default(true).unwrap();
    midi_out.send(MIDIMessage {
        payload: MIDIMessageType::NoteOn { note: 60, velocity: 100 },
        channel: 0,
        port: "default".to_string(),
    }).expect("Error sending MIDI message");
    midi_out.send(MIDIMessage {
        payload: MIDIMessageType::NoteOff { note: 60, velocity: 100 },
        channel: 0,
        port: "default".to_string(),
    }).expect("Error sending MIDI Message");

    // Test: receiving MIDI-In callback messages
    //let mut midi_in = MidiIn::new("BuboCoreIn").unwrap();
    //let _something = midi_in.connect("MIDI Bus 1").unwrap();


    //let start = SystemTime::now();
    //let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backward");
    //let now = since_epoch.as_micros() as u64;

    //let sender2 = world_iface.clone();

    let _bete = ExternalCompiler("bete".to_owned());
    let dummy = DummyCompiler;

    /*for i in 0..10 {
        let log0 = LogMessage::new(Severity::Debug, "Hello world !".to_owned());
        let log0 = ProtocolMessage::LOG(log0).timed(now + i * 1000 * 1000 * (i % 2));
        sender2.send(log0).unwrap();
    }*/

    // This is a test program for the scheduler
    let var = Variable::Instance("A".to_owned());
    let crashtest_program: Program = vec![
        Instruction::Control(ControlASM::Mov(Variable::Constant(1.into()), var.clone())),
        Instruction::Effect(
            Event::Chord(vec![60], TimeSpan::Micros(100)),
            TimeSpan::Micros(1_000_000),
        ),
        Instruction::Control(ControlASM::Sub(var.clone(), Variable::Constant(1.into()), var.clone())),
        Instruction::Control(ControlASM::JumpIfLess(
            Variable::Constant((-1).into()),
            var.clone(),
            1,
        )),
    ];

    // This is a test program obtained from a script
    let crashtest_parsed_program: Program = dummy
        .compile("N 5 2 1 C 3 7 100 4 5 A 1 3 5 8 6 3")
        .unwrap();
    print!("{:?}", crashtest_parsed_program);

    let sequence = Sequence {
        steps: vec![1.0, 4.0],
        sequence_vars:  HashMap::new(),
        scripts: vec![
            Arc::new(Script::from(crashtest_program)),
            Arc::new(Script::from(crashtest_parsed_program)),
        ],
        speed_factor: 1.0,
    };
    let pattern = Pattern {
        sequences: vec![sequence],
        sequence_index: 0,
    };
    let message = SchedulerMessage::UploadPattern(pattern);
    let _ = sched_iface.send(message);

    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");
}
