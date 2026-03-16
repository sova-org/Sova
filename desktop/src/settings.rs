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
    pub doc: DocSettings,
    pub recent_scenes: Vec<PathBuf>,
    pub dismissed_tips: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocSide {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocTrigger {
    Click,
    Hover,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DocSettings {
    pub side: DocSide,
    pub collapsed: bool,
    pub pinned: bool,
    pub trigger: DocTrigger,
    pub width: f32,
    pub mode: u8,
    pub settings_tab: u8,
}

impl Default for DocSettings {
    fn default() -> Self {
        Self {
            side: DocSide::Right,
            collapsed: true,
            pinned: false,
            trigger: DocTrigger::Click,
            width: 400.0,
            mode: 0,
            settings_tab: 0,
        }
    }
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
            bar_gap: 0.0,
            gradient_strength: 0.3,
            detached: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceSettings {
    pub zoom: f32,
    pub accent_color: [u8; 3],
    pub window_shadows: bool,
    pub locale: String,
    pub visuals_enabled: bool,
    pub scene_opacity: f32,
    pub ui_font_size: f32,
    pub animation_time: f32,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            zoom: 1.25,
            accent_color: [0, 92, 128],
            window_shadows: true,
            locale: "en".into(),
            visuals_enabled: false,
            scene_opacity: 0.5,
            ui_font_size: 13.0,
            animation_time: 0.15,
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct VisualsSettings {
    pub code: String,
    pub shared: bool,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowSettings {
    pub chat_detached: bool,
    pub sample_browser_detached: bool,
    pub scope_bar: ScopeBarSettings,
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
    pub feedback: bool,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".into(),
            port: "8080".into(),
            username: String::new(),
            feedback: false,
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
    pub master_volume: f32,
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
            master_volume: 1.0,
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
