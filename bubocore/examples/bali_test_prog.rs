use std::{sync::Arc, vec, thread, collections::HashMap, time::Duration};
use std::io::ErrorKind;

use bubocorelib::clock::ClockServer;
use bubocorelib::compiler::{
    bali::BaliCompiler,
    Compiler,
    CompilerCollection,
};
use bubocorelib::device_map::DeviceMap;
use bubocorelib::scene::{Scene, Line};
use bubocorelib::protocol::midi::{MidiInterface, MidiOut};
use bubocorelib::schedule::{Scheduler, SchedulerMessage, SchedulerNotification, ActionTiming};
use bubocorelib::world::World;
use bubocorelib::server::{
    BuboCoreServer, ServerState,
    client::{BuboCoreClient, ClientMessage},
};
use bubocorelib::transcoder::Transcoder;
use tokio::{sync::{watch, Mutex}, time};
use std::sync::atomic::AtomicBool;



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


/*
    */

    #[tokio::main]
    async fn main() {
        greeter();
    
        let clock_server = Arc::new(ClockServer::new(DEFAULT_TEMPO, DEFAULT_QUANTUM));
        clock_server.link.enable(true);
        let devices = Arc::new(DeviceMap::new());
    
        let midi_name = DEFAULT_MIDI_OUTPUT.to_owned();
        let mut midi_out = MidiOut::new(midi_name.clone()).unwrap();
        midi_out.connect_to_default(true).unwrap();
        devices.register_output_connection(midi_name.clone(), midi_out.into());
    
        let (world_handle, world_iface) = World::create(clock_server.clone());

        let shared_atomic_is_playing = Arc::new(AtomicBool::new(false));
        let (sched_handle, sched_iface, sched_update) =
            Scheduler::create(clock_server.clone(), devices.clone(), world_iface.clone(), shared_atomic_is_playing.clone());
    
        let (updater, update_notifier) = watch::channel(SchedulerNotification::default());
        let initial_scene = Scene::new(
            vec![
                Line::new(vec![1.0]),
            ]
        );
        let scene_image : Arc<Mutex<Scene>> = Arc::new(Mutex::new(initial_scene.clone()));
        let scene_image_maintainer = Arc::clone(&scene_image);
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
                        let mut guard = scene_image_maintainer.blocking_lock();
                        match &p {
                            SchedulerNotification::UpdatedScene(scene) => {
                                *guard = scene.clone();
                            },
                            SchedulerNotification::UpdatedLine(i, line) => {
                                *guard.mut_line(*i) = line.clone()
                            },
                            SchedulerNotification::FramePositionChanged(_positions) => {
                                // No update to scene_image needed for this notification
                            },
                            SchedulerNotification::EnableFrames(line_index, frame_indices) => {
                                guard.mut_line(*line_index).enable_frames(frame_indices);
                            },
                            SchedulerNotification::DisableFrames(line_index, frame_indices) => {
                                guard.mut_line(*line_index).disable_frames(frame_indices);
                            },
                            SchedulerNotification::UploadedScript(_, _, _script) => { /* guard.mut_line...set_script...? */ },
                            SchedulerNotification::UpdatedLineFrames(line_index, items) => {
                                guard.mut_line(*line_index).set_frames(items.clone());
                            },
                            SchedulerNotification::AddedLine(line) => {
                                guard.add_line(line.clone());
                            },
                            SchedulerNotification::RemovedLine(index) => {
                                guard.remove_line(*index);
                            },
                            SchedulerNotification::SceneLengthChanged(length) => {
                                guard.set_length(*length);
                            },
                            _ => ()
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
            devices,
            world_iface,
            sched_iface,
            updater,
            update_notifier,
            transcoder,
            shared_atomic_is_playing.clone(),
        );
    
        tokio::spawn(async { client().await });

        // Use parsed arguments
        let server = BuboCoreServer::new("127.0.0.1".to_owned(), 8080);
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
    
    async fn client() -> tokio::io::Result<()> {
        time::sleep(Duration::from_secs(1)).await;
    
        let mut client = BuboCoreClient::new("127.0.0.1".to_owned(), 8080);
        client.connect().await?;

        print!("Plop\n");

        let bali = BaliCompiler;

        let bali_program: String = "
            (with ch:5 v:2
                (>> (note 52 dur:2))
                (with v:3
                    (>> (note 53 dur:2))
                )
                (with ch:3
                    (>> (note 32 dur:2))
                    (with v:5
                        (> 2 
                            (note 35 dur:2)
                            (note 87 ch:8 v:7)
                        )
                    )
                )
            )
            (with ch:4
                (> 2
                    (note 49 dur:2)
                    (with v:6
                        (note 46 dur:2)
                    )
                )
            )
        ".to_string();
    

        //client.send(ClientMessage::SchedulerControl(SchedulerMessage::AddLine)).await?;
        //client.send(ClientMessage::SchedulerControl(SchedulerMessage::InsertFrame(0, 0, 2.0, ActionTiming::Immediate))).await?;
        client.send(ClientMessage::SetScript(0, 0, bali_program, ActionTiming::Immediate)).await?;

        let con = client.ready().await;
        if !con {
            return Ok(());
        }
        let msg = client.read().await?;
        println!("{:?}", msg);
    
        time::sleep(Duration::from_secs(10)).await;
    
        Ok(())
    }

