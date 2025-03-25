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
    event::{Event, EventPayload},
    variable::{Variable, VariableValue},
    Instruction, Program
};
use pattern::{script::Script, Sequence};
use protocol::{midi::{
    MIDIMessage, MIDIMessageType, MidiIn, MidiInterface, MidiOut
}, ProtocolDevice};
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
    let log_name = "log".to_owned();
    let mut midi_out = MidiOut::new(midi_name.clone()).unwrap();
    midi_out.connect_to_default(true).unwrap();
    devices.register_output_connection(midi_name.clone(), midi_out.into());

    devices.register_output_connection(log_name.clone(), ProtocolDevice::Log);

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
            Event {
                payload: EventPayload::Note(60.into(), TimeSpan::Micros(500_000).into(), None, None),
                device: midi_name.clone().into(),
            },
            TimeSpan::Micros(1_000_000),
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
            Event {
                payload: EventPayload::Note(0.into(), TimeSpan::Beats(0.5).into(), None, None),
                device: midi_name.clone().into(),
            },
            TimeSpan::Micros(1_000_000),
        ),
    ];

    let crashtest_program_with_calls: Program = vec![
        Instruction::Control(ControlASM::CallProcedure(6)),
        Instruction::Effect(
            Event {
                payload: EventPayload::Note(100.into(), TimeSpan::Micros(500_000).into(), None, None),
                device: midi_name.clone().into(),
            },
            TimeSpan::Micros(100),
        ),
        Instruction::Control(ControlASM::CallProcedure(6)),
        Instruction::Effect(
            Event {
                payload: EventPayload::Note(102.into(), TimeSpan::Micros(500_000).into(), None, None),
                device: midi_name.clone().into(),
            },
            TimeSpan::Micros(100),
        ),
        Instruction::Control(ControlASM::CallProcedure(9)),
        Instruction::Control(ControlASM::Return),
        Instruction::Effect(
            Event {
                payload: EventPayload::Note(104.into(), TimeSpan::Micros(500_000).into(), None, None),
                device: midi_name.clone().into(),
            },
            TimeSpan::Micros(100),
        ),
        Instruction::Control(ControlASM::CallProcedure(9)),
        Instruction::Control(ControlASM::Return),
        Instruction::Effect(
            Event {
                payload: EventPayload::Note(106.into(), TimeSpan::Micros(500_000).into(), None, None),
                device: midi_name.clone().into(),
            },
            TimeSpan::Micros(100),
        ),
        Instruction::Control(ControlASM::Return),
    ];

    let crashtest_func: Program = vec![
        Instruction::Effect(
            Event {
                payload: EventPayload::Note(40.into(), TimeSpan::Micros(500_000).into(), None, None),
                device: midi_name.clone().into(),
            },
            TimeSpan::Micros(500000),
        ),
        Instruction::Control(ControlASM::Return),
    ];

    let crashtest_program_with_function_calls: Program = vec![
        Instruction::Control(ControlASM::Mov(Variable::Constant(VariableValue::Func(crashtest_func.clone())), var.clone())),
        Instruction::Control(ControlASM::CallFunction(var.clone())),
        Instruction::Effect(
            Event {
                payload: EventPayload::Note(501.into(), TimeSpan::Micros(500_000).into(), None, None),
                device: midi_name.clone().into(),
            },
            TimeSpan::Micros(100),
        ),
        Instruction::Control(ControlASM::Return),
    ];

    // This is a test program obtained from a script
    let crashtest_parsed_program: Program = dummy
        .compile("N 5 2 1 C 3 7 100 4 5 A 1 3 5 8 6 3")
        .unwrap();
    print!("{:?}", crashtest_parsed_program);

    let sequence = Sequence {
        steps: vec![1.0, 1.0, 1.0, 1.0],
        sequence_vars:  HashMap::new(),
        scripts: vec![
            /*Arc::new(Script::from(crashtest_program)),
            Arc::new(Script::from(crashtest_parsed_program)),
            Arc::new(Script::from(crashtest_program_with_calls)),
            Arc::new(Script::from(crashtest_program_with_function_calls)),*/
            Arc::new(kick.clone().into()),
            Arc::new(kick.clone().into()),
            Arc::new(kick.clone().into()),
            Arc::new(kick.clone().into())
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
