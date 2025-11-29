use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    #[serde(default)]
    pub editor: EditorConfig,

    #[serde(default)]
    pub appearance: AppearanceConfig,

    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub client: ClientConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EditorConfig {
    #[serde(default = "default_editor_mode")]
    pub mode: EditorMode,

    #[serde(default = "default_font_size")]
    pub font_size: f32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,

    #[serde(default = "default_show_line_numbers")]
    pub show_line_numbers: bool,

    #[serde(default = "default_line_wrapping")]
    pub line_wrapping: bool,

    #[serde(default = "default_highlight_active_line")]
    pub highlight_active_line: bool,

    #[serde(default = "default_cursor_blink_rate")]
    pub cursor_blink_rate: u32,

    #[serde(default = "default_tab_size")]
    pub tab_size: u32,

    #[serde(default = "default_indent_unit")]
    pub indent_unit: String,

    #[serde(default = "default_use_tabs")]
    pub use_tabs: bool,

    #[serde(default = "default_close_brackets")]
    pub close_brackets: bool,

    #[serde(default = "default_bracket_matching")]
    pub bracket_matching: bool,

    #[serde(default = "default_autocomplete")]
    pub autocomplete: bool,

    #[serde(default = "default_rectangular_selection")]
    pub rectangular_selection: bool,

    #[serde(default = "default_fold_gutter")]
    pub fold_gutter: bool,

    #[serde(default = "default_match_highlighting")]
    pub match_highlighting: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EditorMode {
    Vim,
    Normal,
    Emacs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AppearanceConfig {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_transparency")]
    pub transparency: u8,

    #[serde(default = "default_font_family")]
    pub font_family: String,

    #[serde(default = "default_zoom")]
    pub zoom: f32,

    #[serde(default = "default_hue")]
    pub hue: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ServerConfig {
    #[serde(default = "default_server_enabled")]
    pub enabled: bool,

    #[serde(default = "default_server_port")]
    pub port: u16,

    #[serde(default = "default_server_ip")]
    pub ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ClientConfig {
    #[serde(default = "default_client_ip")]
    pub ip: String,

    #[serde(default = "default_client_port")]
    pub port: u16,

    #[serde(default = "default_client_nickname")]
    pub nickname: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigUpdateEvent {
    pub editor: EditorConfig,
    pub appearance: AppearanceConfig,
    pub server: ServerConfig,
    pub client: ClientConfig,
}

fn default_editor_mode() -> EditorMode {
    EditorMode::Normal
}

fn default_font_size() -> f32 {
    14.0
}

fn default_show_line_numbers() -> bool {
    true
}

fn default_line_wrapping() -> bool {
    false
}

fn default_highlight_active_line() -> bool {
    true
}

fn default_cursor_blink_rate() -> u32 {
    1200
}

fn default_tab_size() -> u32 {
    4
}

fn default_indent_unit() -> String {
    "  ".to_string()
}

fn default_use_tabs() -> bool {
    false
}

fn default_close_brackets() -> bool {
    true
}

fn default_bracket_matching() -> bool {
    true
}

fn default_autocomplete() -> bool {
    true
}

fn default_rectangular_selection() -> bool {
    true
}

fn default_fold_gutter() -> bool {
    true
}

fn default_match_highlighting() -> bool {
    true
}

fn default_theme() -> String {
    "monokai".to_string()
}

fn default_transparency() -> u8 {
    100
}

fn default_server_enabled() -> bool {
    false
}

fn default_server_port() -> u16 {
    8080
}

fn default_server_ip() -> String {
    "127.0.0.1".to_string()
}

fn default_client_ip() -> String {
    "127.0.0.1".to_string()
}

fn default_client_port() -> u16 {
    8080
}

fn default_client_nickname() -> String {
    String::new()
}

fn default_font_family() -> String {
    "monospace".to_string()
}

fn default_zoom() -> f32 {
    1.0
}

fn default_hue() -> u16 {
    0
}
