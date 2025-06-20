//! Cова Audio Engine
//!
//! High-performance real-time audio engine for live coding and performance.
//!
//! Features zero-allocation audio processing, pre-allocated memory pools,
//! polyphonic voice management, and OSC control interface.

use clap::Parser;
use engine::AudioEngine;
use memory::{MemoryPool, SampleLibrary, VoiceMemory};
use registry::ModuleRegistry;
use server::OscServer;
use std::sync::Arc;
use std::thread;
use crossbeam_channel::bounded;

pub mod audio_tools;
pub mod dsp;
pub mod engine;
pub mod memory;
pub mod modulation;
pub mod modules;
pub mod parser;
pub mod registry;
pub mod server;
pub mod timing;
pub mod track;
pub mod types;
pub mod voice;

/// Command line arguments for the Sova audio engine
#[derive(Parser)]
#[command(name = "Sova")]
#[command(about = "High-performance realtime audio engine for live coding and performance")]
struct Args {
    /// Audio sample rate in Hz
    #[arg(short, long, default_value_t = 44100)]
    sample_rate: u32,

    /// Audio processing block size in samples
    #[arg(short, long, default_value_t = 512)]
    block_size: u32,

    /// Audio buffer size per channel
    #[arg(short, long, default_value_t = 1024)]
    buffer_size: usize,

    /// Maximum number of audio buffers for sample storage
    #[arg(short, long, default_value_t = 2048)]
    max_audio_buffers: usize,

    /// Maximum number of simultaneous voices
    #[arg(short, long, default_value_t = 128)]
    max_voices: usize,

    /// Specific audio output device name
    #[arg(short, long)]
    output_device: Option<String>,

    /// OSC server port
    #[arg(long, default_value_t = 12345)]
    osc_port: u16,

    /// OSC server host address
    #[arg(long, default_value = "127.0.0.1")]
    osc_host: String,

    /// OSC timestamp tolerance in milliseconds
    #[arg(long, default_value_t = 1000)]
    timestamp_tolerance_ms: u64,

    /// Directory path for audio sample files
    #[arg(long, default_value = "./samples")]
    audio_files_location: String,
}

/// Prints startup banner with configuration details
fn print_banner(
    sample_rate: u32,
    buffer_size: usize,
    max_audio_buffers: usize,
    osc_host: &str,
    osc_port: u16,
) {
    println!("\n");
    println!(" ▗▄▄▖▄▄▄  ▗▖▗▞▀▜▌    Sample rate: {}", sample_rate);
    println!("▐▌  █   █ ▐▌▝▚▄▟▌    Buffer size: {}", buffer_size);
    println!(
        "▐▌  ▀▄▄▄▀ ▐▛▀▚▖      Max audio buffers: {}",
        max_audio_buffers
    );
    println!("▝▚▄▄▖     ▐▙▄▞▘      OSC server: {}:{}", osc_host, osc_port);
    println!("\n");
}

/// Main entry point for the Sova audio engine
///
/// Initializes memory pools, audio engine, OSC server, and starts processing threads
fn main() {
    let args = Args::parse();
    print_banner(
        args.sample_rate,
        args.buffer_size,
        args.max_audio_buffers,
        &args.osc_host,
        args.osc_port,
    );

    let memory_per_voice = args.buffer_size * 8 * 4;
    let sample_memory = args.max_audio_buffers * args.buffer_size * 8;
    let dsp_memory = args.max_voices * args.buffer_size * 16 * 4;
    let base_memory = 16 * 1024 * 1024;
    let available_memory =
        base_memory + (args.max_voices * memory_per_voice) + sample_memory + dsp_memory;

    let global_pool = Arc::new(MemoryPool::new(available_memory));
    let voice_memory = Arc::new(VoiceMemory::new());
    let sample_library = SampleLibrary::new(
        args.max_audio_buffers,
        &args.audio_files_location,
        args.sample_rate,
    );

    sample_library.preload_all_samples();
    let sample_library = Arc::new(sample_library);

    println!(
        "Memory allocation: {}MB total",
        available_memory / (1024 * 1024)
    );

    let mut registry = ModuleRegistry::new();
    registry.register_default_modules();
    registry.set_timestamp_tolerance(args.timestamp_tolerance_ms);

    print!("Engine config: {} voices", args.max_voices);
    if let Some(device) = &args.output_device {
        print!(" | Output: {}", device);
    }
    println!(" | Tolerance: {}ms", args.timestamp_tolerance_ms);

    let engine = AudioEngine::new_with_memory(
        args.sample_rate as f32,
        args.buffer_size,
        args.max_voices,
        args.block_size as usize,
        registry.clone(),
        Arc::clone(&global_pool),
        Arc::clone(&voice_memory),
        Arc::clone(&sample_library),
    );

    println!("Starting audio engine...");
    
    // Create bounded crossbeam channel for command communication
    let (engine_tx, engine_rx) = bounded(1024);
        
    let engine_tx_clone = engine_tx.clone();
    let registry_clone = registry.clone();
    let voice_memory_clone = Arc::clone(&voice_memory);
    let sample_library_clone = Arc::clone(&sample_library);
    let osc_host = args.osc_host.clone();
    let osc_port = args.osc_port;
    
    // Start OSC server thread
    let _osc_thread = thread::Builder::new()
        .name("osc_lockfree".to_string())
        .spawn(move || {
            let mut osc_server = match OscServer::new(
                &osc_host,
                osc_port,
                registry_clone,
                voice_memory_clone,
                sample_library_clone,
            ) {
                Ok(server) => server,
                Err(e) => {
                    eprintln!("Failed to create OSC server: {}", e);
                    return;
                }
            };
            osc_server.run_lockfree(engine_tx_clone);
        })
        .expect("Failed to spawn OSC thread");
        
    // Start audio thread
    let audio_thread = AudioEngine::start_audio_thread(
        engine,
        args.block_size,
        args.max_voices,
        args.sample_rate,
        args.buffer_size,
        args.output_device,
        engine_rx,
        None, // No status channel for standalone engine
    );

    println!("Ready ✓");

    let _ = audio_thread.join();
}
