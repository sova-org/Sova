use serde::{Deserialize, Serialize};
use sova_core::log_eprintln;
use std::path::PathBuf;

use crate::widgets::EditorSettings;

#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
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
    pub scene: SceneSettings,
    pub tools: ToolsSettings,
    pub recent_scenes: Vec<PathBuf>,
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

#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScopeSettings {
    pub persistence: f32,
    pub detached: bool,
}

impl Default for ScopeSettings {
    fn default() -> Self {
        Self {
            persistence: 0.65,
            detached: false,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpectrumSettings {
    pub smoothing: f32,
    pub peak_decay: f32,
    pub gradient_strength: f32,
    pub detached: bool,
}

impl Default for SpectrumSettings {
    fn default() -> Self {
        Self {
            smoothing: 0.85,
            peak_decay: 0.92,
            gradient_strength: 0.3,
            detached: false,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
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
    pub ui_font: String,
    pub editor_font: String,
    pub bg_brightness: u8,
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
            ui_font: String::new(),
            editor_font: String::new(),
            bg_brightness: 0,
        }
    }
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct VisualsSettings {
    pub code: String,
    pub shared: bool,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeBarMode {
    #[default]
    Scope,
    Spectrogram,
    Both,
}

impl ScopeBarMode {
    pub fn next(self) -> Self {
        match self {
            Self::Scope => Self::Spectrogram,
            Self::Spectrogram => Self::Both,
            Self::Both => Self::Scope,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowSettings {
    pub chat_detached: bool,
    pub sample_browser_detached: bool,
    pub scope_bar_height: f32,
    pub scope_bar_mode: ScopeBarMode,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            chat_detached: false,
            sample_browser_detached: false,
            scope_bar_height: 64.0,
            scope_bar_mode: ScopeBarMode::default(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolsTab {
    Chat,
    SampleBrowser,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolsSettings {
    pub open: bool,
    pub show_chat: bool,
    pub show_sample_browser: bool,
    pub active_tab: Option<ToolsTab>,
    pub width: f32,
}

impl Default for ToolsSettings {
    fn default() -> Self {
        Self {
            open: false,
            show_chat: false,
            show_sample_browser: false,
            active_tab: None,
            width: 360.0,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerSettings {
    pub ip: String,
    pub port: String,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            ip: "0.0.0.0".into(),
            port: "8080".into(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    pub host: String,
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
            host: String::new(),
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

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SceneSettings {
    pub prelude_collapsed: bool,
    pub prelude_col_width: f32,
    pub view_mode: u8,
    pub show_phase_bar: bool,
}

impl Default for SceneSettings {
    fn default() -> Self {
        Self {
            prelude_collapsed: true,
            prelude_col_width: 300.0,
            view_mode: 0,
            show_phase_bar: true,
        }
    }
}

pub fn load() -> AppSettings {
    confy::load("sova", None).unwrap_or_default()
}

pub fn save(settings: &AppSettings) {
    if let Err(e) = confy::store("sova", None, settings) {
        log_eprintln!("Failed to save settings: {e}");
    }
}
