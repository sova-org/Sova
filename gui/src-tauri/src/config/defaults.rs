use super::types::{AppearanceConfig, ClientConfig, Config, EditorConfig, EditorMode, ServerConfig};

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            mode: EditorMode::Normal,
            font_size: 14.0,
            font_family: None,
            show_line_numbers: true,
            line_wrapping: false,
            highlight_active_line: true,
            cursor_blink_rate: 1200,
            tab_size: 4,
            indent_unit: "  ".to_string(),
            use_tabs: false,
            close_brackets: true,
            bracket_matching: true,
            autocomplete: true,
            rectangular_selection: true,
            fold_gutter: true,
            match_highlighting: true,
        }
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "monokai".to_string(),
            font_family: "monospace".to_string(),
            zoom: 1.0,
            hue: 0,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 8080,
            ip: "127.0.0.1".to_string(),
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port: 8080,
            nickname: String::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            appearance: AppearanceConfig::default(),
            server: ServerConfig::default(),
            client: ClientConfig::default(),
        }
    }
}
