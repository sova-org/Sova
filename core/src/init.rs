use std::{sync::Arc, thread::JoinHandle};

use crossbeam_channel::{Receiver, Sender};

use crate::{clock::ClockServer, device_map::DeviceMap, lang::LanguageCenter, schedule::{Scheduler, SchedulerMessage, SovaNotification}, world::World};

/// Starts both World and Scheduler, ensuring that Scheduler is connected to World
/// And returns handles to both threads, as well as scheduler communication channels
pub fn start_scheduler_and_world(
    clock_server: Arc<ClockServer>,
    devices: Arc<DeviceMap>,
    languages: Arc<LanguageCenter>,
) -> (
    JoinHandle<()>,
    JoinHandle<()>,
    Sender<SchedulerMessage>,
    Receiver<SovaNotification>,
) {
    let (world_handle, world_iface) = World::create(clock_server.clone());

    let (sched_handle, sched_iface, sched_update) = Scheduler::create(
        clock_server,
        devices,
        languages,
        world_iface
    );

    (world_handle, sched_handle, sched_iface, sched_update)
}