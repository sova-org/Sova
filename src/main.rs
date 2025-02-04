use clock::MusicTime;
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

    let sender2 = message_sender.clone();

    let log0 = LogMessage::new(Severity::Debug, "Hello world !".to_owned());
    let log0 = ProtocolMessage::LOG(log0).timed(3 * 1000 * 1000);
    sender2.send(log0).unwrap();

    // This is a test program for the scheduler
    let crashtest_program: Program = vec![
        Instruction::Effect(
            Event::Note(60, MusicTime::Micros(1)),
            MusicTime::Micros(2)
        ),
        Instruction::Effect(
            Event::Exit,
            MusicTime::Micros(4)
        )
    ];

    handle.join().unwrap();
}
