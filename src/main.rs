use std::{sync::Arc, thread, time::Duration};
use crate::clock::ClockServer;

use clock::TimeSpan;
use device_map::DeviceMap;

use lang::{event::Event, Instruction, Program};
use pattern::Sequence;
use protocol::midi::{MidiInterface, MidiOut};
use schedule::{Scheduler, SchedulerMessage, SchedulerNotification};
use server::{client::{BuboCoreClient, ClientMessage}, BuboCoreServer, ServerState, ENDING_BYTE};
use tokio::{io::AsyncWriteExt, net::TcpSocket, sync::watch, time};
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

pub mod server;

pub const DEFAULT_MIDI_OUTPUT : &str = "BuboCoreOut";
pub const DEFAULT_TEMPO : f64 = 80.0;
pub const DEFAULT_QUANTUM : f64 = 4.0;

#[tokio::main]
async fn main() {

    let clock_server = Arc::new(ClockServer::new(DEFAULT_TEMPO, DEFAULT_QUANTUM));
    clock_server.link.enable(true);
    let devices = Arc::new(DeviceMap::new());

    let midi_name = DEFAULT_MIDI_OUTPUT.to_owned();
    let mut midi_out = MidiOut::new(midi_name.clone()).unwrap();
    midi_out.connect_to_default(true).unwrap();
    devices.register_output_connection(midi_name.clone(), midi_out.into());

    let (world_handle, world_iface) = World::create(clock_server.clone());
    let (sched_handle, sched_iface, update_pattern) =
        Scheduler::create(clock_server.clone(), devices.clone(), world_iface.clone());

    let (updater, update_notifier) = watch::channel(SchedulerNotification::default());
    thread::spawn(move|| {
        loop {
            match update_pattern.recv() {
                Ok(p) => { let _ = updater.send(p); },
                Err(_) => break,
            }
        }
    });

    tokio::spawn(async { client().await });

    let server_state = ServerState { clock_server, world_iface, sched_iface, update_notifier};
    let server = BuboCoreServer { ip: "127.0.0.1".to_owned(), port: 8080 };
    server.start(server_state).await.expect("Server internal error");

    println!("\n[-] Stopping BuboCore...");
    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");

}

async fn client() -> tokio::io::Result<()> {
    time::sleep(Duration::from_secs(5)).await;

    let mut client = BuboCoreClient::new("127.0.0.1".to_owned(), 8080);
    client.connect().await?;

    let mut seq = Sequence::new(vec![1.0,1.0,1.0,0.5,0.5]);
    let note: Program = vec![Instruction::Effect(
        Event::MidiNote(
            60.into(),
            80.into(),
            0.into(),
            TimeSpan::Beats(0.9).into(),
            DEFAULT_MIDI_OUTPUT.to_owned().into(),
        ),
        TimeSpan::Micros(1_000_000).into(),
    )];
    seq.set_script(0, note.clone().into());
    seq.set_script(1, note.clone().into());
    seq.set_script(3, note.clone().into());
    seq.set_script(4, note.clone().into());
    let msg = SchedulerMessage::AddSequence(seq);
    let msg = ClientMessage::SchedulerControl(msg);
    client.send(msg).await?;

    let con = client.ready().await;
    if !con {
        return Ok(());
    }
    let msg = client.read().await?;
    println!("{:?}", msg);
    
    Ok(())
}