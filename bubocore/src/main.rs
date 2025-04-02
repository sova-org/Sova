use crate::clock::ClockServer;
use std::{sync::Arc, thread, collections::HashMap};
use clap::Parser;
use std::io::ErrorKind;
use device_map::DeviceMap;
use pattern::{Pattern, Sequence};
use protocol::midi::{MidiInterface, MidiOut};
use schedule::{Scheduler, SchedulerNotification, SchedulerMessage};
use server::{
    BuboCoreServer, ServerState,
};
use transcoder::Transcoder;
use tokio::sync::{watch, Mutex};
use world::World;
use crate::compiler::{Compiler, bali::BaliCompiler, CompilerCollection};

// Déclaration des modules
pub mod transcoder;
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
    \nsynchronizes state, and processes patterns."
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
    greeter();
    // Parse CLI arguments
    let cli = Cli::parse();

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
    let initial_pattern = Pattern::new(
        vec![
            Sequence::new(vec![1.0, 1.0, 1.0, 1.0]),
            Sequence::new(vec![0.25, 0.25, 0.25, 0.25]),
            Sequence::new(vec![0.5, 0.5, 0.5, 0.5]),
            Sequence::new(vec![4.0, 4.0, 2.0, 2.0])
        ]
    );
    let pattern_image : Arc<Mutex<Pattern>> = Arc::new(Mutex::new(initial_pattern.clone()));
    let pattern_image_maintainer = Arc::clone(&pattern_image);
    let updater_clone = updater.clone();

    // Create the compiler map
    let mut compilers: CompilerCollection = HashMap::new();
    // Instantiate and insert the Bali compiler
    let bali_compiler = BaliCompiler;
    compilers.insert(bali_compiler.name(), Box::new(bali_compiler));

    // Now create the transcoder with the populated map
    let transcoder = Arc::new(tokio::sync::Mutex::new(Transcoder::new(
        compilers, // Use the map with BaliCompiler
        Some("bali".to_string())
    )));

    thread::spawn(move || {
        loop {
            match sched_update.recv() {
                Ok(p) => {
                    let mut guard = pattern_image_maintainer.blocking_lock();
                    match &p {
                        SchedulerNotification::UpdatedPattern(pattern) => *guard = pattern.clone(),
                        SchedulerNotification::UpdatedSequence(i, sequence) => *guard.mut_sequence(*i) = sequence.clone(),
                        SchedulerNotification::EnableStep(s, i) => todo!(),
                        SchedulerNotification::DisableStep(s, i) => todo!(),
                        SchedulerNotification::UploadedScript(_, _, script) => todo!(),
                        SchedulerNotification::UpdatedSequenceSteps(_, items) => todo!(),
                        SchedulerNotification::AddedSequence(sequence) => todo!(),
                        SchedulerNotification::RemovedSequence(_) => todo!(),
                        _ => ()
                    };
                    let _ = updater_clone.send(p);
                }
                Err(_) => break,
            }
        }
    });

    if let Err(e) = sched_iface.send(SchedulerMessage::UploadPattern(initial_pattern)) {
        eprintln!("[!] Failed to send initial pattern to scheduler: {}", e);
        std::process::exit(1);
    }

    let server_state = ServerState::new(
        pattern_image,
        clock_server,
        devices,
        world_iface,
        sched_iface,
        updater,
        update_notifier,
        transcoder,
    );

    // Use parsed arguments
    let server = BuboCoreServer::new(cli.ip, cli.port);
    println!("[+] Starting BuboCore server on {}:{}...", server.ip, server.port);
    // Handle potential errors during server start
    match server.start(server_state).await {
        Ok(_) => {}
        Err(e) => {
            if e.kind() == ErrorKind::AddrInUse {
                eprintln!(
                    "[!] Error: Address {}:{} is already in use.",
                    server.ip,
                    server.port
                );
                eprintln!("    Please check if another BuboCore instance or application is running on this port.");
                std::process::exit(1); // Exit with a non-zero code to indicate failure
            } else {
                // For other errors, print a generic message and the error details
                eprintln!("[!] Server failed to start: {}", e);
                std::process::exit(1);
            }
        }
    }

    println!("\n[-] Stopping BuboCore...");
    sched_handle.join().expect("Scheduler thread error");
    world_handle.join().expect("World thread error");
}
