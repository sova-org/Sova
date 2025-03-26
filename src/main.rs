use std::sync::Arc;
use crate::clock::ClockServer;

use device_map::DeviceMap;

use protocol::midi::{MidiInterface, MidiOut};
use schedule::Scheduler;
use server::BuboCoreServer;
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

#[tokio::main]
async fn main() {

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

    let mut server = BuboCoreServer { ip: "127.0.0.1".to_owned(), port: 8080 };
    server.start().await.expect("Server failed");

    println!("\n[-] Stopping BuboCore...");
    drop(sched_iface);
    drop(world_iface);
    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");

}
