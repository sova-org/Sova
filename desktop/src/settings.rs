use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::widgets::EditorSettings;

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppSettings {
    pub windows: WindowSettings,
    pub editor: EditorSettings,
    pub server: ServerSettings,
    pub client: ClientSettings,
    pub audio: AudioSettings,
    pub appearance: AppearanceSettings,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum ThemePref {
    Dark,
    Light,
    #[default]
    System,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum SpacingPref {
    Compact,
    #[default]
    Normal,
    Comfortable,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceSettings {
    pub theme: ThemePref,
    pub zoom: f32,
    pub accent_color: [u8; 3],
    pub spacing: SpacingPref,
    pub window_shadows: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: ThemePref::default(),
            zoom: 1.25,
            accent_color: [0, 92, 128],
            spacing: SpacingPref::default(),
            window_shadows: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct WindowSettings {
    pub server: bool,
    pub audio: bool,
    pub devices: bool,
    pub logs_collapsed: bool,
    pub debug: bool,
    pub options: bool,
    pub scope: bool,
    pub spectrum: bool,
    pub vu_meter: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            server: true,
            audio: false,
            devices: false,
            logs_collapsed: true,
            debug: false,
            options: false,
            scope: false,
            spectrum: false,
            vu_meter: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct ServerSettings {
    pub ip: String,
    pub port: String,
    pub tempo: String,
    pub quantum: String,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".into(),
            port: "8080".into(),
            tempo: "120".into(),
            quantum: "4".into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct ClientSettings {
    pub ip: String,
    pub port: String,
    pub username: String,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".into(),
            port: "8080".into(),
            username: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    pub output_device: String,
    pub input_device: String,
    pub channels: u16,
    pub buffer_size: Option<u32>,
    pub max_voices: usize,
    pub sample_paths: Vec<PathBuf>,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            output_device: String::new(),
            input_device: String::new(),
            channels: 2,
            buffer_size: None,
            max_voices: 32,
            sample_paths: Vec::new(),
        }
    }
}

pub fn load() -> AppSettings {
    confy::load("sova", None).unwrap_or_default()
}

pub fn save(settings: &AppSettings) {
    if let Err(e) = confy::store("sova", None, settings) {
        eprintln!("Failed to save settings: {e}");
    }
}
