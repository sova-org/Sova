use std::{sync::Arc, vec, thread, collections::HashMap, time::Duration};
use std::io::ErrorKind;

use bubocorelib::clock::ClockServer;
use bubocorelib::compiler::{
    bali::BaliCompiler,
    Compiler,
    CompilerCollection,
};
use bubocorelib::device_map::DeviceMap;
use bubocorelib::lang::Program;
use bubocorelib::pattern::{Pattern, Sequence};
use bubocorelib::protocol::midi::{MidiInterface, MidiOut};
use bubocorelib::schedule::{Scheduler, SchedulerMessage, SchedulerNotification};
use bubocorelib::world::World;
use bubocorelib::server::{
    BuboCoreServer, ServerState,
    client::{BuboCoreClient, ClientMessage},
};
use bubocorelib::transcoder::Transcoder;
use tokio::{sync::{watch, Mutex}, time};



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
        let (sched_handle, sched_iface, sched_update) =
            Scheduler::create(clock_server.clone(), devices.clone(), world_iface.clone());
    
        let (updater, update_notifier) = watch::channel(SchedulerNotification::default());
        let initial_pattern = Pattern::new(
            vec![
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
                            SchedulerNotification::UpdatedPattern(pattern) => {
                                *guard = pattern.clone();
                            },
                            SchedulerNotification::UpdatedSequence(i, sequence) => {
                                *guard.mut_sequence(*i) = sequence.clone()
                            },
                            SchedulerNotification::StepPositionChanged(positions) => {
                                // No update to pattern_image needed for this notification
                            },
                            SchedulerNotification::EnableSteps(sequence_index, step_indices) => {
                                guard.mut_sequence(*sequence_index).enable_steps(step_indices);
                            },
                            SchedulerNotification::DisableSteps(sequence_index, step_indices) => {
                                guard.mut_sequence(*sequence_index).disable_steps(step_indices);
                            },
                            SchedulerNotification::UploadedScript(_, _, _script) => { /* guard.mut_sequence...set_script...? */ },
                            SchedulerNotification::UpdatedSequenceSteps(sequence_index, items) => {
                                guard.mut_sequence(*sequence_index).set_steps(items.clone());
                            },
                            SchedulerNotification::AddedSequence(sequence) => {
                                guard.add_sequence(sequence.clone());
                            },
                            SchedulerNotification::RemovedSequence(index) => {
                                guard.remove_sequence(*index);
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
            (eucloop 3 8 (// 1 8) (note 50))
            (loop 8 (// 1 8) (>> (note 40)))
        ".to_string();
    

        client.send(ClientMessage::SchedulerControl(SchedulerMessage::AddSequence)).await?;
        client.send(ClientMessage::SchedulerControl(SchedulerMessage::InsertStep(0, 0, 2.0))).await?;
        client.send(ClientMessage::SetScript(0, 0, bali_program)).await?;

/*
        let mut sequence = Sequence::new(vec![2.0]);
        sequence.set_script(0, bali_program.clone().into());
    
        let msg = SchedulerMessage::AddSequence(sequence);
        let msg = ClientMessage::SchedulerControl(msg);
        client.send(msg).await?;
    
        let mut sequence = Sequence::new(vec![2.0, 1.0/6.0]);
        sequence.set_script(0, bali_program.clone().into());
    
        let msg = SchedulerMessage::AddSequence(sequence);
        let msg = ClientMessage::SchedulerControl(msg);
        client.send(msg).await?;
*/

        let con = client.ready().await;
        if !con {
            return Ok(());
        }
        let msg = client.read().await?;
        println!("{:?}", msg);
    
        time::sleep(Duration::from_secs(10)).await;
        Ok(())
    }

