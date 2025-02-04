use std::time::{SystemTime, UNIX_EPOCH};

use clock::TimeSpan;
use lang::{Event, Instruction, Program};
use protocol::{log::{LogMessage, Severity}, ProtocolMessage};
use world::World;

pub mod schedule;
pub mod clock;
pub mod io;
pub mod world;
pub mod protocol;
pub mod lang;

fn main() {
    let (handle, message_sender) = World::create();

    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backward");
    let now = since_epoch.as_micros() as u64;

    let sender2 = message_sender.clone();

    let log0 = LogMessage::new(Severity::Debug, "Hello world !".to_owned());
    let log0 = ProtocolMessage::LOG(log0).timed(now + 3 * 1000 * 1000);
    sender2.send(log0).unwrap();

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

    handle.join();
}
