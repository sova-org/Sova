use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host};

pub struct DeviceInfo {
    pub device: Device,
    pub name: String,
    pub is_default: bool,
}

pub enum SelectionResult {
    Success(DeviceInfo),
    Fallback(DeviceInfo, String),
    Error(String),
}

pub struct DeviceSelector {
    host: Host,
    sample_rate: u32,
    channels: u16,
}

impl DeviceSelector {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            host: cpal::default_host(),
            sample_rate,
            channels: 2,
        }
    }

    pub fn select_output_device(&self, preferred_name: Option<String>) -> SelectionResult {
        println!("Audio device selection starting...");
        
        let strategies: Vec<Box<dyn DeviceStrategy>> = vec![
            Box::new(PreferredDeviceStrategy::new(preferred_name.clone())),
            Box::new(DefaultDeviceStrategy),
            Box::new(FirstAvailableStrategy),
            Box::new(PlatformSpecificStrategy::new()),
        ];

        for (i, strategy) in strategies.iter().enumerate() {
            println!("  Trying strategy {}: {}", i + 1, strategy.name());
            
            match strategy.select(&self.host) {
                Some(device) => {
                    let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
                    let is_default = self.is_default_device(&device);
                    
                    println!("  Found device: {}", device_name);
                    
                    if self.validate_device(&device) {
                        let info = DeviceInfo {
                            device,
                            name: device_name.clone(),
                            is_default,
                        };
                        
                        return if i == 0 && preferred_name.is_some() {
                            SelectionResult::Success(info)
                        } else {
                            let reason = format!("Using {} (strategy: {})", device_name, strategy.name());
                            SelectionResult::Fallback(info, reason)
                        };
                    } else {
                        println!("  Device validation failed for: {}", device_name);
                    }
                }
                None => {
                    println!("  No device found with this strategy");
                }
            }
        }
        
        SelectionResult::Error("No suitable audio output device found".to_string())
    }

    fn validate_device(&self, device: &Device) -> bool {
        match device.supported_output_configs() {
            Ok(mut configs) => {
                let has_compatible = configs.any(|cfg| {
                    cfg.channels() == self.channels &&
                    cfg.min_sample_rate().0 <= self.sample_rate &&
                    cfg.max_sample_rate().0 >= self.sample_rate
                });
                
                if !has_compatible {
                    println!("    No compatible configuration found");
                    return false;
                }
                
                true
            }
            Err(e) => {
                println!("    Failed to query device configurations: {}", e);
                false
            }
        }
    }

    fn is_default_device(&self, device: &Device) -> bool {
        if let Some(default) = self.host.default_output_device() {
            if let (Ok(name1), Ok(name2)) = (device.name(), default.name()) {
                return name1 == name2;
            }
        }
        false
    }
}

trait DeviceStrategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn select(&self, host: &Host) -> Option<Device>;
}

struct PreferredDeviceStrategy {
    device_name: Option<String>,
}

impl PreferredDeviceStrategy {
    fn new(device_name: Option<String>) -> Self {
        Self { device_name }
    }
}

impl DeviceStrategy for PreferredDeviceStrategy {
    fn name(&self) -> &'static str {
        "Preferred Device"
    }

    fn select(&self, host: &Host) -> Option<Device> {
        let name = self.device_name.as_ref()?;
        
        host.output_devices().ok()?.find(|d| {
            d.name().unwrap_or_default() == *name
        })
    }
}

struct DefaultDeviceStrategy;

impl DeviceStrategy for DefaultDeviceStrategy {
    fn name(&self) -> &'static str {
        "System Default"
    }

    fn select(&self, host: &Host) -> Option<Device> {
        host.default_output_device()
    }
}

struct FirstAvailableStrategy;

impl DeviceStrategy for FirstAvailableStrategy {
    fn name(&self) -> &'static str {
        "First Available"
    }

    fn select(&self, host: &Host) -> Option<Device> {
        host.output_devices().ok()?.next()
    }
}

struct PlatformSpecificStrategy {
    platform_devices: Vec<&'static str>,
}

impl PlatformSpecificStrategy {
    fn new() -> Self {
        let platform_devices = if cfg!(target_os = "linux") {
            vec!["pulse", "default", "pipewire", "hw:0,0", "plughw:0,0"]
        } else if cfg!(target_os = "macos") {
            vec!["Built-in Output", "Default"]
        } else if cfg!(target_os = "windows") {
            vec!["Default", "Speakers"]
        } else {
            vec![]
        };
        
        Self { platform_devices }
    }
}

impl DeviceStrategy for PlatformSpecificStrategy {
    fn name(&self) -> &'static str {
        "Platform Specific Fallbacks"
    }

    fn select(&self, host: &Host) -> Option<Device> {
        let devices: Vec<_> = host.output_devices().ok()?.collect();
        
        for &preferred_name in &self.platform_devices {
            if let Some(device) = devices.iter().find(|d| {
                d.name().unwrap_or_default().contains(preferred_name)
            }) {
                if let Ok(name) = device.name() {
                    println!("    Trying platform-specific device: {}", name);
                }
                return Some(device.clone());
            }
        }
        
        None
    }
}