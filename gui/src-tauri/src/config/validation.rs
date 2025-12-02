use super::types::{AppearanceConfig, Config, EditorConfig, ServerConfig};

pub trait Validate {
    fn validate(&mut self);
}

impl Validate for EditorConfig {
    fn validate(&mut self) {
        if self.font_size < 8.0 || self.font_size > 32.0 {
            sova_core::log_warn!(
                "Invalid font_size: {}. Using default: 14.0",
                self.font_size
            );
            self.font_size = 14.0;
        }

        if self.tab_size < 1 || self.tab_size > 16 {
            sova_core::log_warn!(
                "Invalid tab_size: {}. Using default: 4",
                self.tab_size
            );
            self.tab_size = 4;
        }

        if self.cursor_blink_rate > 3000 {
            sova_core::log_warn!(
                "Invalid cursor_blink_rate: {}. Using default: 1200",
                self.cursor_blink_rate
            );
            self.cursor_blink_rate = 1200;
        }

        if !self.indent_unit.chars().all(char::is_whitespace) || self.indent_unit.is_empty() {
            sova_core::log_warn!("Invalid indent_unit. Using default: two spaces");
            self.indent_unit = "  ".to_string();
        }

        if self.use_tabs {
            self.indent_unit = "\t".to_string();
        }
    }
}

impl Validate for AppearanceConfig {
    fn validate(&mut self) {
        if self.theme.trim().is_empty() {
            sova_core::log_warn!("Invalid theme: empty string. Using default: monokai");
            self.theme = "monokai".to_string();
        }

        if self.zoom < 0.125 || self.zoom > 3.0 {
            sova_core::log_warn!(
                "Invalid zoom: {}. Using default: 1.0",
                self.zoom
            );
            self.zoom = 1.0;
        }

        if self.hue > 360 {
            self.hue = self.hue % 360;
        }
    }
}

impl Validate for ServerConfig {
    fn validate(&mut self) {
        if self.port < 1024 {
            sova_core::log_warn!(
                "Invalid server port: {}. Using default: 8080",
                self.port
            );
            self.port = 8080;
        }

        if self.ip.trim().is_empty() {
            sova_core::log_warn!("Invalid server IP: empty string. Using default: 127.0.0.1");
            self.ip = "127.0.0.1".to_string();
        }
    }
}

impl Validate for Config {
    fn validate(&mut self) {
        self.editor.validate();
        self.appearance.validate();
        self.server.validate();
    }
}
