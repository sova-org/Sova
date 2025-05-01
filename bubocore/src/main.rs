use crate::clock::ClockServer;
use crate::compiler::{Compiler, CompilerCollection, bali::BaliCompiler, dummylang::DummyCompiler};
use clap::Parser;
use device_map::DeviceMap;
use scene::line::Line;
use scene::Scene;
use schedule::{Scheduler, SchedulerMessage, SchedulerNotification};
use server::{BuboCoreServer, ServerState};
use std::io::ErrorKind;
use std::sync::atomic::AtomicBool;
use std::{collections::HashMap, sync::Arc, thread};
use tokio::sync::{Mutex, watch};
use transcoder::Transcoder;
use world::World;

// Déclaration des modules
pub mod clock;
pub mod compiler;
pub mod device_map;
pub mod lang;
pub mod protocol;
pub mod scene;
pub mod schedule;
pub mod server;
pub mod shared_types;
pub mod transcoder;
pub mod world;

pub const DEFAULT_MIDI_OUTPUT: &str = "BuboCore";
pub const DEFAULT_TEMPO: f64 = 120.0;
pub const DEFAULT_QUANTUM: f64 = 4.0;
pub const GREETER_LOGO: &str = "
▗▄▄▖ █  ▐▌▗▖    ▄▄▄   ▗▄▄▖▄▄▄   ▄▄▄ ▗▞▀▚▖
▐▌ ▐▌▀▄▄▞▘▐▌   █   █ ▐▌  █   █ █    ▐▛▀▀▘
▐▛▀▚▖     ▐▛▀▚▖▀▄▄▄▀ ▐▌  ▀▄▄▄▀ █    ▝▚▄▄▖
▐▙▄▞▘     ▐▙▄▞▘      ▝▚▄▄▖               
                                         
";

fn greeter() {
    print!("{}", GREETER_LOGO);
    println!("Version: {}\n", env!("CARGO_PKG_VERSION"));
}

// Define the CLI arguments struct
#[derive(Parser, Debug)]
#[clap(author = "Raphaël Forment <raphael.forment@gmail.com>")]
#[clap(author = "Loïg Jezequel <loig.jezequel@univ-nantes.fr>")]
#[clap(author = "Tanguy Dubois <email@address.com>")]
#[command(
    version = "0.0.1",
    about = "BuboCore: A live coding environment server.",
    long_about = "BuboCore acts as the central server for a collaborative live coding environment.\n
    It manages connections from clients (like bubocoretui), handles MIDI devices,
    \nsynchronizes state, and processes scenes."
)]
struct Cli {
    /// IP address to bind the server to
    #[arg(short, long, value_name = "IP_ADDRESS", default_value = "127.0.0.1")]
    ip: String,

    /// Port to bind the server to
    #[arg(short, long, value_name = "PORT", default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() {
    // ======================================================================
    // Splash screen
    greeter();

    // ======================================================================
    // Parse CLI arguments
    let cli = Cli::parse();

    // ======================================================================
    // Initialize the clock
    let clock_server = Arc::new(ClockServer::new(DEFAULT_TEMPO, DEFAULT_QUANTUM));
    clock_server.link.enable(true);

    // ======================================================================
    // Initialize the list of devices
    let devices = Arc::new(DeviceMap::new());
    let midi_name = DEFAULT_MIDI_OUTPUT.to_owned();
    // Create the default virtual port
    if let Err(e) = devices.create_virtual_midi_port(&midi_name) {
        eprintln!(
            "[!] Failed to create default virtual MIDI port '{}': {}",
            midi_name, e
        );
    } else {
        println!(
            "[+] Default virtual MIDI port '{}' created successfully.",
            midi_name
        );
        // Assign default MIDI port to Slot 1
        if let Err(e) = devices.assign_slot(1, &midi_name) {
            eprintln!("[!] Failed to assign '{}' to Slot 1: {}", midi_name, e);
        }
    }

    // Create and assign default OSC device (SuperDirt) to Slot 2
    let osc_name = "SuperDirt";
    let osc_ip = "127.0.0.1";
    let osc_port = 57120;
    if let Err(e) = devices.create_osc_output_device(osc_name, osc_ip, osc_port) {
        eprintln!(
            "[!] Failed to create default OSC device '{}': {}",
            osc_name, e
        );
    } else {
        println!(
            "[+] Default OSC device '{}' created successfully ({}:{}).",
            osc_name, osc_ip, osc_port
        );
        // Assign SuperDirt to Slot 2
        if let Err(e) = devices.assign_slot(2, osc_name) {
            eprintln!("[!] Failed to assign '{}' to Slot 2: {}", osc_name, e);
        }
    }

    // ======================================================================
    // Initialize the world (side effect performer)
    let (world_handle, world_iface) = World::create(clock_server.clone());

    // ======================================================================
    // Initialize the transcoder (list of available compilers)
    let mut compilers: CompilerCollection = HashMap::new();
    // 1) The BaLi compiler
    let bali_compiler = BaliCompiler;
    let dummy_compiler = DummyCompiler;
    compilers.insert(bali_compiler.name(), Box::new(bali_compiler));
    compilers.insert(dummy_compiler.name(), Box::new(dummy_compiler));
    let transcoder = Arc::new(tokio::sync::Mutex::new(Transcoder::new(
        compilers,
        Some("bali".to_string()),
    )));

    // Shared flag for transport state (playing/stopped)
    let shared_atomic_is_playing = Arc::new(AtomicBool::new(false));

    // ======================================================================
    // Initialize the scheduler (scene manager)
    let (sched_handle, sched_iface, sched_update) = Scheduler::create(
        clock_server.clone(),
        devices.clone(),
        world_iface.clone(),
        shared_atomic_is_playing.clone(),
    );
    let (updater, update_notifier) = watch::channel(SchedulerNotification::default());

    // ======================================================================
    // Initialize the default scene loaded when the server starts
    let initial_scene = Scene::new(vec![
        Line::new(vec![1.0]),
    ]);
    let scene_image: Arc<Mutex<Scene>> = Arc::new(Mutex::new(initial_scene.clone()));
    let scene_image_maintainer = Arc::clone(&scene_image);
    let updater_clone = updater.clone();

    thread::spawn(move || {
        loop {
            match sched_update.recv() {
                Ok(p) => {
                    let mut guard = scene_image_maintainer.blocking_lock();
                    match &p {
                        SchedulerNotification::UpdatedScene(scene) => {
                            *guard = scene.clone();
                        }
                        SchedulerNotification::UpdatedLine(i, line) => {
                            *guard.mut_line(*i) = line.clone()
                        }
                        SchedulerNotification::FramePositionChanged(_positions) => {
                            // No update to scene needed for this notification
                        }
                        SchedulerNotification::EnableFrames(line_index, frame_indices) => {
                            guard.mut_line(*line_index).enable_frames(frame_indices);
                        }
                        SchedulerNotification::DisableFrames(line_index, frame_indices) => {
                            guard.mut_line(*line_index).disable_frames(frame_indices);
                        }
                        SchedulerNotification::UploadedScript(_, _, _script) => {}
                        SchedulerNotification::UpdatedLineFrames(frame_index, items) => {
                            guard.mut_line(*frame_index).set_frames(items.clone());
                        }
                        SchedulerNotification::AddedLine(line) => {
                            guard.add_line(line.clone());
                        }
                        SchedulerNotification::RemovedLine(index) => {
                            guard.remove_line(*index);
                        }
                        SchedulerNotification::SceneLengthChanged(length) => {
                            guard.set_length(*length);
                        }
                        _ => (),
                    };
                    drop(guard);
                    let _ = updater_clone.send(p);
                }
                Err(_) => break,
            }
        }
    });

    if let Err(e) = sched_iface.send(SchedulerMessage::UploadScene(initial_scene)) {
        eprintln!("[!] Failed to send initial scene to scheduler: {}", e);
        std::process::exit(1);
    }

    let server_state = ServerState::new(
        scene_image,
        clock_server,
        devices.clone(),
        world_iface,
        sched_iface,
        updater,
        update_notifier,
        transcoder,
        shared_atomic_is_playing.clone(),
    );

    // Use parsed arguments
    let server = BuboCoreServer::new(cli.ip, cli.port);
    println!(
        "[+] Starting BuboCore server on {}:{}...",
        server.ip, server.port
    );
    // Handle potential errors during server start
    match server.start(server_state).await {
        Ok(_) => {}
        Err(e) => {
            if e.kind() == ErrorKind::AddrInUse {
                eprintln!(
                    "[!] Error: Address {}:{} is already in use.",
                    server.ip, server.port
                );
                eprintln!(
                    "    Please check if another BuboCore instance or application is running on this port."
                );
                std::process::exit(1); // Exit with a non-zero code to indicate failure
            } else {
                // For other errors, print a generic message and the error details
                eprintln!("[!] Server failed to start: {}", e);
                std::process::exit(1);
            }
        }
    }

    println!("\n[-] Stopping BuboCore...");
    // Send MIDI Panic (All Notes Off) before shutting down completely
    devices.panic_all_midi_outputs();
    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");
}
