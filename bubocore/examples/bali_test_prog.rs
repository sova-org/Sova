use std::{sync::Arc, vec, thread, time::Duration};

use bubocorelib::clock::ClockServer;
use bubocorelib::pattern::Pattern;
use bubocorelib::compiler::{
    bali::BaliCompiler,
    Compiler,
};
use bubocorelib::device_map::DeviceMap;
use bubocorelib::lang::Program;
use bubocorelib::pattern::Sequence;
use bubocorelib::protocol::midi::{MidiInterface, MidiOut};
use bubocorelib::schedule::{Scheduler, SchedulerMessage, SchedulerNotification};
use bubocorelib::world::World;
use bubocorelib::server::{
    BuboCoreServer, ServerState,
    client::{BuboCoreClient, ClientMessage},
};
use tokio::{sync::watch, time};



pub const DEFAULT_MIDI_OUTPUT: &str = "BuboCoreOut";
pub const DEFAULT_TEMPO: f64 = 80.0;
pub const DEFAULT_QUANTUM: f64 = 4.0;

/*
    */

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
        thread::spawn(move || {
            loop {
                match sched_update.recv() {
                    Ok(p) => {
                        let _ = updater.send(p);
                    }
                    Err(_) => break,
                }
            }
        });
    
        tokio::spawn(async { client().await });
    
        let server_state = ServerState {
            clock_server,
            world_iface,
            sched_iface,
            update_notifier,
        };
        let server = BuboCoreServer {
            ip: "127.0.0.1".to_owned(),
            port: 8080,
        };
        server
            .start(server_state)
            .await
            .expect("Server internal error");
    
        println!("\n[-] Stopping BuboCore...");
        sched_handle.join().expect("Scheduler thread error");
        world_handle.join().expect("World thread error");
    }
    
    async fn client() -> tokio::io::Result<()> {
        time::sleep(Duration::from_secs(5)).await;
    
        let mut client = BuboCoreClient::new("127.0.0.1".to_owned(), 8080);
        client.connect().await?;

        let bali = BaliCompiler;
        let bali_program: Program = bali.compile("
        (d note 20)
        (> (// 1 4) (n note 90 1 0 nimp))
        (> (// 1 2) (d note (+ note 13)))
        (@ (// 3 4) 
            (<< (n note 90 1 0 nimp))
            (d note (+ note 13))
            (>> (n note 90 1 0 nimp))
            (> (// 1 4) (n 100 90 1 0 nimp))
        )
        (@ (// 5 4)
            (< (// 1 8) (n 101 90 1 0 nimp))
            (> (// 1 8) (n 102 90 1 0 nimp))
        )
        ").unwrap();
    
        let mut sequence = Sequence::new(vec![1.0]);
        sequence.set_script(0, bali_program.clone().into());
    
        let msg = SchedulerMessage::AddSequence(sequence);
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
    