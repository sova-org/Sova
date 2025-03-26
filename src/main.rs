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
use tokio::signal;

pub mod clock;
pub mod compiler;
pub mod device_map;
pub mod io;
pub mod lang;
pub mod pattern;
pub mod protocol;
pub mod schedule;
pub mod world;

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

    let _bete = ExternalCompiler("bete".to_owned());
    let dummy = DummyCompiler;

    println!("Waiting for interrupt");
    signal::ctrl_c().await.expect("failed to listen for event");;
    println!("\n[-] Stopping BuboCore...");
    drop(sched_iface);
    drop(world_iface);
    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
