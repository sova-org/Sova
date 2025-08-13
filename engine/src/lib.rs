pub mod audio_tools;
pub mod constants;
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

use types::LoggerHandle;

pub use modules::Frame;

/// Lists all available audio output devices with validation status
pub fn list_audio_devices(logger: &LoggerHandle) {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();

    logger.log_info("Available audio output devices:");
    logger.log_info("(Devices marked with ✓ support 44.1kHz stereo output)\n");

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

                    logger.log_info(&format!("  {} {}{}", validation_mark, name, default_mark));

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
                                logger.log_info(&format!("      Supported rates: {}", rates.join(", ")));
                            }
                        }
                    }
                }
            }

            if !found_devices {
                logger.log_warning("  No audio output devices found");
            }
        }
        Err(e) => {
            logger.log_error(&format!("Error listing audio devices: {}", e));
            std::process::exit(1);
        }
    }

    logger.log_info("\nDevice selection will automatically try multiple strategies:");
    logger.log_info("  1. Specified device (--output-device)");
    logger.log_info("  2. System default device");
    logger.log_info("  3. First available device");
    logger.log_info("  4. Platform-specific fallbacks");

    if cfg!(target_os = "linux") {
        logger.log_info("\nLinux-specific devices that will be tried:");
        logger.log_info("  - pulse (PulseAudio)");
        logger.log_info("  - default (ALSA default)");
        logger.log_info("  - pipewire (PipeWire)");
        logger.log_info("  - hw:0,0 (Hardware device)");
    }

    logger.log_info("");
}
