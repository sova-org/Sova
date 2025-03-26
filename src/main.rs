use std::{collections::HashMap, sync::Arc, thread, time::Duration, vec};
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
    variable::{Variable, VariableValue},
    Instruction, Program
};
use pattern::{script::Script, Sequence};
use protocol::midi::{MidiInterface, MidiOut};
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
    clock_server.link.enable(true);
    let devices = Arc::new(DeviceMap::new());

    let midi_name = "BuboCoreOut".to_owned();
    let mut midi_out = MidiOut::new(midi_name.clone()).unwrap();
    midi_out.connect_to_default(true).unwrap();
    devices.register_output_connection(midi_name.clone(), midi_out.into());

    let (world_handle, world_iface) = World::create(clock_server.clone());
    let (sched_handle, sched_iface) =
        Scheduler::create(clock_server.clone(), devices.clone(), world_iface.clone());

    let _bete = ExternalCompiler("bete".to_owned());
    let dummy = DummyCompiler;

    // This is a test program for the scheduler
    let var = Variable::Instance("A".to_owned());
    let crashtest_program: Program = vec![
        Instruction::Control(ControlASM::Mov(1.into(), var.clone())),
        Instruction::Effect(
            Event::MidiNote(60.into(), 90.into(), 1.into(), TimeSpan::Micros(500_000).into(), midi_name.clone().into()),
            TimeSpan::Micros(1_000_000).into(),
        ),
        Instruction::Control(ControlASM::Sub(var.clone(), 1.into(), var.clone())),
        Instruction::Control(ControlASM::JumpIfLess(
            (-1).into(),
            var.clone(),
            1,
        )),
    ];

    let kick: Program = vec![
        Instruction::Effect(
            Event::MidiNote(36.into(), 80.into(), 0.into(), TimeSpan::Beats(0.9).into(), midi_name.clone().into()),
            TimeSpan::Micros(1_000_000).into(),
        ),
    ];
    let tom: Program = vec![
        Instruction::Effect(
            Event::MidiNote(43.into(), 80.into(), 0.into(), TimeSpan::Beats(0.5).into(), midi_name.clone().into()),
            TimeSpan::Micros(1_000_000).into(),
        ),
    ];
    let hihats: Program = vec![
        Instruction::Effect(
            Event::MidiNote(72.into(), 80.into(), 0.into(), TimeSpan::Beats(0.1).into(), midi_name.clone().into()),
            TimeSpan::Micros(1_000_000).into(),
        ),
    ];

    let crashtest_program_with_calls: Program = vec![
        Instruction::Control(ControlASM::CallProcedure(6)),
        Instruction::Effect(
            Event::MidiNote(80.into(), 90.into(), 1.into(), TimeSpan::Micros(500_000).into(), midi_name.clone().into()),
            TimeSpan::Micros(100).into(),
        ),
        Instruction::Control(ControlASM::CallProcedure(6)),
        Instruction::Effect(
            Event::MidiNote(82.into(), 90.into(), 1.into(), TimeSpan::Micros(500_000).into(), midi_name.clone().into()),
            TimeSpan::Micros(100).into(),
        ),
        Instruction::Control(ControlASM::CallProcedure(9)),
        Instruction::Control(ControlASM::Return),
        Instruction::Effect(
            Event::MidiNote(84.into(), 90.into(), 1.into(), TimeSpan::Micros(500_000).into(), midi_name.clone().into()),
            TimeSpan::Micros(100).into(),
        ),
        Instruction::Control(ControlASM::CallProcedure(9)),
        Instruction::Control(ControlASM::Return),
        Instruction::Effect(
            Event::MidiNote(86.into(), 90.into(), 1.into(), TimeSpan::Micros(500_000).into(), midi_name.clone().into()),
            TimeSpan::Micros(100).into(),
        ),
        Instruction::Control(ControlASM::Return),
    ];

    let crashtest_func: Program = vec![
        Instruction::Effect(
            Event::MidiNote(40.into(), 90.into(), 1.into(), TimeSpan::Micros(500_000).into(), midi_name.clone().into()),
            TimeSpan::Micros(500000).into(),
        ),
        Instruction::Control(ControlASM::Return),
    ];

    let crashtest_program_with_function_calls: Program = vec![
        Instruction::Control(ControlASM::Mov(Variable::Constant(VariableValue::Func(crashtest_func.clone())), var.clone())),
        Instruction::Control(ControlASM::CallFunction(var.clone())),
        Instruction::Effect(
            Event::MidiNote(67.into(), 90.into(), 1.into(), TimeSpan::Micros(500_000).into(), midi_name.clone().into()),
            TimeSpan::Micros(100).into(),
        ),
        Instruction::Control(ControlASM::Return),
    ];


    // This is a test program obtained from a script
    let crashtest_parsed_program: Program = dummy
        .compile("N 100 2 1")
        .unwrap();
    print!("{:?}", crashtest_parsed_program);

    let mut sequence1 = Sequence::new(vec![1.0]);
    let mut sequence2 = Sequence::new(vec![1.0,1.0]);
    let mut sequence3 = Sequence::new(vec![0.25]);
    let mut sequence4 = Sequence::new(vec![1.0,1.0,1.0,1.0]);
    sequence1.set_script(0, kick.clone().into());
    sequence2.set_script(1, tom.clone().into());
    sequence3.set_script(0, hihats.clone().into());
    sequence4.set_script(0, crashtest_program.into());
    sequence4.set_script(1, crashtest_program_with_calls.into());
    sequence4.set_script(2, crashtest_program_with_function_calls.into());
    sequence4.set_script(3, crashtest_parsed_program.into());
    let pattern = Pattern::new(vec![sequence1]);

    let message = SchedulerMessage::UploadPattern(pattern);
    let _ = sched_iface.send(message);

    thread::sleep(Duration::from_millis(5000));
    let message2 = SchedulerMessage::AddSequence(sequence2);
    let _ = sched_iface.send(message2);
    thread::sleep(Duration::from_millis(5000));
    let message3 = SchedulerMessage::AddSequence(sequence3);
    let _ = sched_iface.send(message3);
    thread::sleep(Duration::from_millis(5000));
    let message4 = SchedulerMessage::AddSequence(sequence4);
    let _ = sched_iface.send(message4);

    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");
}
