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
    pub scope: ScopeSettings,
    pub spectrum: SpectrumSettings,
    pub visuals: VisualsSettings,
    pub recent_scenes: Vec<PathBuf>,
    pub dismissed_tips: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScopeSettings {
    pub smoothing: f32,
    pub stroke_width: f32,
    pub fill_alpha: f32,
    pub detached: bool,
}

impl Default for ScopeSettings {
    fn default() -> Self {
        Self {
            smoothing: 0.0,
            stroke_width: 1.0,
            fill_alpha: 0.35,
            detached: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SpectrumSettings {
    pub smoothing: f32,
    pub bar_gap: f32,
    pub gradient_strength: f32,
    pub detached: bool,
}

impl Default for SpectrumSettings {
    fn default() -> Self {
        Self {
            smoothing: 0.85,
            bar_gap: 1.0,
            gradient_strength: 0.3,
            detached: false,
        }
    }
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
    pub locale: String,
    pub visuals_enabled: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: ThemePref::default(),
            zoom: 1.25,
            accent_color: [0, 92, 128],
            spacing: SpacingPref::default(),
            window_shadows: true,
            locale: "en".into(),
            visuals_enabled: false,
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct VisualsSettings {
    pub code: String,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct WindowSettings {
    pub logs_collapsed: bool,
    pub log_panel_height: f32,
    pub chat_detached: bool,
    pub sample_browser_detached: bool,
    pub scope_bar: ScopeBarSettings,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            logs_collapsed: true,
            log_panel_height: 160.0,
            chat_detached: false,
            sample_browser_detached: false,
            scope_bar: ScopeBarSettings::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScopeBarSettings {
    pub height: f32,
    pub smoothing: f32,
}

impl Default for ScopeBarSettings {
    fn default() -> Self {
        Self {
            height: 64.0,
            smoothing: 0.3,
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
