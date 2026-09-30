use ratatui::style::Color;
use ratatui_themes::{Style, Theme};

#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color,
    pub surface: Color,
    pub foreground: Color,
    pub muted: Color,
    pub accent: Color,
    pub secondary: Color,
    pub selection: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,
}

impl From<Theme> for Palette {
    fn from(theme: Theme) -> Self {
        let base = theme.palette();
        Self {
            background: base.bg,
            surface: surface(&theme),
            foreground: base.fg,
            muted: base.muted,
            accent: base.accent,
            secondary: base.secondary,
            selection: base.selection,
            error: base.error,
            warning: base.warning,
            success: base.success,
            info: base.info,
        }
    }
}

impl Palette {

    pub fn text(&self) -> Style {
        Style::default().fg(self.foreground)
    }

}

const fn surface(theme: &Theme) -> Color {
    let palette = theme.palette();
    match (palette.bg, palette.selection) {
        (Color::Rgb(bg_red, bg_green, bg_blue), Color::Rgb(selection_red, selection_green, selection_blue)) => {
            Color::Rgb(
                midpoint(bg_red, selection_red),
                midpoint(bg_green, selection_green),
                midpoint(bg_blue, selection_blue),
            )
        }
        _ => palette.bg,
    }
}

const fn midpoint(first: u8, second: u8) -> u8 {
    ((first as u16 + second as u16) / 2) as u8
}