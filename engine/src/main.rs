//! Cова Audio Engine
//!
//! High-performance real-time audio engine for live coding and performance.
//!
//! Features zero-allocation audio processing, pre-allocated memory pools,
//! polyphonic voice management, and OSC control interface.

use clap::Parser;
use crossbeam_channel::bounded;
use engine::AudioEngine;
use memory::{MemoryPool, SampleLibrary, VoiceMemory};
use registry::ModuleRegistry;
use server::OscServer;
use types::LoggerHandle;
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;

mod constants;
use constants::{
    DEFAULT_AUDIO_PRIORITY, DEFAULT_BLOCK_SIZE, DEFAULT_BUFFER_SIZE, DEFAULT_MAX_AUDIO_BUFFERS,
    DEFAULT_MAX_VOICES, DEFAULT_OSC_PORT, DEFAULT_SAMPLE_DIR, DEFAULT_SAMPLE_RATE,
    ENGINE_TX_CHANNEL_BOUND,
};

pub mod audio_tools;
pub mod device_selector;
pub mod dsp;
pub mod effect_pool;
pub mod engine;
pub mod memory;
pub mod modulation;
pub mod modules;
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
    #[arg(short, long, default_value_t = DEFAULT_SAMPLE_RATE)]
    sample_rate: u32,

    /// Audio processing block size in samples
    #[arg(short, long, default_value_t = DEFAULT_BLOCK_SIZE)]
    block_size: u32,

    /// Audio buffer size per channel
    #[arg(long, default_value_t = DEFAULT_BUFFER_SIZE)]
    buffer_size: usize,

    /// Maximum number of audio buffers for sample storage
    #[arg(long, default_value_t = DEFAULT_MAX_AUDIO_BUFFERS)]
    max_audio_buffers: usize,

    /// Maximum number of simultaneous voices
    #[arg(short, long, default_value_t = DEFAULT_MAX_VOICES)]
    max_voices: usize,

    /// Specific audio output device name
    #[arg(long)]
    output_device: Option<String>,

    /// OSC server port
    #[arg(long, default_value_t = DEFAULT_OSC_PORT)]
    osc_port: u16,

    /// OSC server host address
    #[arg(long, default_value = "127.0.0.1")]
    osc_host: String,

    /// Directory path for audio sample files
    #[arg(long, default_value = DEFAULT_SAMPLE_DIR)]
    audio_files_location: String,

    /// Audio thread priority (0-99, higher = more priority, 0 = disable, auto-mapped to platform ranges)
    #[arg(long, default_value_t = DEFAULT_AUDIO_PRIORITY)]
    audio_priority: u8,

    /// List available audio output devices and exit
    #[arg(long)]
    list_devices: bool,
}

/// Prints startup banner with configuration details
fn print_banner(
    sample_rate: u32,
    buffer_size: usize,
    max_audio_buffers: usize,
    osc_host: &str,
    osc_port: u16,
    logger: &LoggerHandle,
) {
    logger.log_info("");
    logger.log_info(&format!(" ▗▄▄▖▄▄▄  ▗▖▗▞▀▜▌    Sample rate: {}", sample_rate));
    logger.log_info(&format!("▐▌  █   █ ▐▌▝▚▄▟▌    Buffer size: {}", buffer_size));
    logger.log_info(&format!(
        "▐▌  ▀▄▄▄▀ ▐▛▀▚▖      Max audio buffers: {}",
        max_audio_buffers
    ));
    logger.log_info(&format!("▝▚▄▄▖     ▐▙▄▞▘      OSC server: {}:{}", osc_host, osc_port));
    logger.log_info("");
}

/// Main entry point for the Sova audio engine
///
/// Initializes memory pools, audio engine, OSC server, and starts processing threads
fn main() {
    let args = Args::parse();

    // Handle --list-devices flag before initialization
    if args.list_devices {
        let console_logger = LoggerHandle::new_console();
        // For standalone mode, we need to call the function directly
        // since bubo_engine:: would have conflicting types
        let host = cpal::default_host();
        console_logger.log_info("Available audio output devices:");
        console_logger.log_info("(Devices marked with ✓ support 44.1kHz stereo output)\n");
        
        // Get default device for comparison
        let default_device = host.default_output_device();
        let default_name = default_device
            .as_ref()
            .and_then(|d| d.name().ok())
            .unwrap_or_default();
        
        match host.output_devices() {
            Ok(devices) => {
                let mut found_devices = false;
                let devices_vec: Vec<_> = devices.collect();
                
                for device in devices_vec {
                    if let Ok(name) = device.name() {
                        found_devices = true;
                        
                        // Check if device supports standard configuration
                        let validation = if let Ok(mut configs) = device.supported_output_configs() {
                            configs.any(|cfg| {
                                cfg.channels() == 2
                                    && cfg.min_sample_rate().0 <= 44100
                                    && cfg.max_sample_rate().0 >= 44100
                            })
                        } else {
                            false
                        };
                        
                        let validation_mark = if validation { "✓" } else { "✗" };
                        let default_mark = if name == default_name {
                            " [DEFAULT]"
                        } else {
                            ""
                        };
                        
                        console_logger.log_info(&format!("  {} {}{}", validation_mark, name, default_mark));
                        
                        // Show sample rates for devices that don't support 44.1kHz
                        if !validation {
                            if let Ok(configs) = device.supported_output_configs() {
                                let rates: Vec<_> = configs
                                    .filter(|cfg| cfg.channels() == 2)
                                    .map(|cfg| {
                                        format!(
                                            "{}-{}Hz",
                                            cfg.min_sample_rate().0,
                                            cfg.max_sample_rate().0
                                        )
                                    })
                                    .collect();
                                if !rates.is_empty() {
                                    console_logger.log_info(&format!("      Supported rates: {}", rates.join(", ")));
                                }
                            }
                        }
                    }
                }
                
                if !found_devices {
                    console_logger.log_warning("  No audio output devices found");
                }
            }
            Err(e) => {
                console_logger.log_error(&format!("Error listing audio devices: {}", e));
                std::process::exit(1);
            }
        }
        
        console_logger.log_info("\nDevice selection will automatically try multiple strategies:");
        console_logger.log_info("  1. Specified device (--output-device)");
        console_logger.log_info("  2. System default device");
        console_logger.log_info("  3. First available device");
        console_logger.log_info("  4. Platform-specific fallbacks");
        
        if cfg!(target_os = "linux") {
            console_logger.log_info("\nLinux-specific devices that will be tried:");
            console_logger.log_info("  - pulse (PulseAudio)");
            console_logger.log_info("  - default (ALSA default)");
            console_logger.log_info("  - pipewire (PipeWire)");
            console_logger.log_info("  - hw:0,0 (Hardware device)");
        }
        
        console_logger.log_info("");
        return;
    }

    let console_logger = LoggerHandle::new_console();
    print_banner(
        args.sample_rate,
        args.buffer_size,
        args.max_audio_buffers,
        &args.osc_host,
        args.osc_port,
        &console_logger,
    );

    let memory_per_voice = args.buffer_size * 8 * 4;
    let sample_memory = args.max_audio_buffers * args.buffer_size * 8;
    let dsp_memory = args.max_voices * args.buffer_size * 16 * 4;
    let base_memory = 16 * 1024 * 1024; // 16MB base memory
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

    console_logger.log_info(&format!(
        "Memory allocation: {}MB total",
        available_memory / (1024 * 1024)
    ));

    let mut registry = ModuleRegistry::new();
    registry.register_default_modules();

    let mut config_msg = format!("Engine config: {} voices", args.max_voices);
    if let Some(device) = &args.output_device {
        config_msg.push_str(&format!(" | Output: {}", device));
    }
    console_logger.log_info(&config_msg);

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

    console_logger.log_info("Starting audio engine...");

    // Create bounded crossbeam channel for command communication
    let (engine_tx, engine_rx) = bounded(ENGINE_TX_CHANNEL_BOUND);

    let engine_tx_clone = engine_tx.clone();
    let registry_clone = registry.clone();
    let voice_memory_clone = Arc::clone(&voice_memory);
    let sample_library_clone = Arc::clone(&sample_library);
    let osc_host = args.osc_host.clone();
    let osc_port = args.osc_port;
    let osc_shutdown = Arc::new(AtomicBool::new(false));
    let osc_shutdown_clone = osc_shutdown.clone();

    // Start OSC server thread
    let osc_thread = thread::Builder::new()
        .name("osc_lockfree".to_string())
        .spawn(move || {
            let osc_logger = LoggerHandle::new_console();
            let mut osc_server = match OscServer::new(
                &osc_host,
                osc_port,
                registry_clone,
                voice_memory_clone,
                sample_library_clone,
                osc_shutdown_clone,
                osc_logger,
            ) {
                Ok(server) => server,
                Err(e) => {
                    let err_logger = LoggerHandle::new_console();
                    err_logger.log_error(&format!("Failed to create OSC server: {}", e));
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
        // No status channel for standalone engine
        None,
        args.audio_priority,
    );

    console_logger.log_info("Ready ✓");

    // Wait for audio thread to exit (it will exit on Stop message or channel disconnect)
    match audio_thread.join() {
        Ok(_) => console_logger.log_info("Audio thread exited"),
        Err(_) => console_logger.log_error("Audio thread panicked"),
    }

    // Signal OSC thread to shutdown
    osc_shutdown.store(true, Ordering::Relaxed);

    // Wait for OSC thread
    match osc_thread.join() {
        Ok(_) => console_logger.log_info("OSC thread exited"),
        Err(_) => console_logger.log_error("OSC thread panicked"),
    }
}
