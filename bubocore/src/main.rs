use crate::clock::ClockServer;
use std::{sync::Arc, thread};

use device_map::DeviceMap;

use pattern::Pattern;
use protocol::midi::{MidiInterface, MidiOut};
use schedule::{Scheduler, SchedulerNotification};
use server::{
    BuboCoreServer, ServerState,
};
use tokio::sync::{watch, Mutex};
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

pub const DEFAULT_MIDI_OUTPUT: &str = "BuboCoreOut";
pub const DEFAULT_TEMPO: f64 = 80.0;
pub const DEFAULT_QUANTUM: f64 = 4.0;

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
    let (sched_handle, sched_iface, sched_update) =
        Scheduler::create(clock_server.clone(), devices.clone(), world_iface.clone());

    let (updater, update_notifier) = watch::channel(SchedulerNotification::default());
    let pattern_image : Arc<Mutex<Pattern>> = Default::default();
    let pattern_image_maintainer = Arc::clone(&pattern_image);
    thread::spawn(move || {
        loop {
            match sched_update.recv() {
                Ok(p) => {
                    let mut guard = pattern_image_maintainer.blocking_lock();
                    match &p {
                        SchedulerNotification::UpdatedPattern(pattern) => *guard = pattern.clone(),
                        SchedulerNotification::UpdatedSequence(i, sequence) => *guard.mut_sequence(*i) = sequence.clone(),
                        SchedulerNotification::ToggledStep(s, i, b) => todo!(),
                        SchedulerNotification::UploadedScript(_, _, script) => todo!(),
                        SchedulerNotification::UpdatedSequenceSteps(_, items) => todo!(),
                        SchedulerNotification::AddedSequence(sequence) => todo!(),
                        SchedulerNotification::RemovedSequence(_) => todo!(),
                        _ => ()
                    };
                    let _ = updater.send(p);
                }
                Err(_) => break,
            }
        }
    });

    let server_state = ServerState::new(
        pattern_image,
        clock_server,
        devices,
        world_iface,
        sched_iface,
        update_notifier,
    );
    let server = BuboCoreServer::new("127.0.0.1".to_owned(), 8080);
    server
        .start(server_state)
        .await
        .expect("Server internal error");

    println!("\n[-] Stopping BuboCore...");
    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");
}
