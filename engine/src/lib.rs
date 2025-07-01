pub mod audio_tools;
pub mod constants;
pub mod dsp;
pub mod effect_pool;
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

pub use modules::Frame;

/// Lists all available audio output devices
pub fn list_audio_devices() {
    use cpal::traits::{DeviceTrait, HostTrait};
    
    let host = cpal::default_host();
    
    println!("Available audio output devices:\n");
    
    // Get default device for comparison
    let default_device = host.default_output_device();
    let default_name = default_device
        .as_ref()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();
    
    match host.output_devices() {
        Ok(devices) => {
            let mut found_devices = false;
            for device in devices {
                if let Ok(name) = device.name() {
                    found_devices = true;
                    if name == default_name {
                        println!("  [DEFAULT] {}", name);
                    } else {
                        println!("  {}", name);
                    }
                }
            }
            
            if !found_devices {
                println!("  No audio output devices found");
            }
        }
        Err(e) => {
            eprintln!("Error listing audio devices: {}", e);
            std::process::exit(1);
        }
    }
    
    println!();
}
