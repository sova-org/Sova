use std::array;

use palette::{Hsl, ShiftHue, Srgb, convert::FromColor};
use ratatui::style::Color;
use ratatui_themes::{Style, Theme};
use sova_core::vm::language::TokenCategory;

pub const VARIATIONS : usize = 7;

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
    pub variations: [Color ; VARIATIONS]
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
            variations: Self::color_angles(base.accent, [
                90.0, 180.0, 270.0,
                45.0, 135.0, 225.0, 315.0
            ])
        }
    }
}

impl Palette {

    pub fn text(&self) -> Style {
        Style::default().fg(self.foreground)
    }

    pub fn color_angles<const N: usize>(color: Color, angles: [f32 ; N]) -> [Color ; N] {
        let Color::Rgb(r, g, b) = color else {
            return [color ; N];
        };
        let rgb : Srgb<f32> = Srgb::new(r, g, b).into_format();
        let hsl = Hsl::from_color(rgb);
        array::from_fn(|i| {
            let shifted = hsl.shift_hue(angles[i]);
            Color::from_hsl(shifted)
        })
    }

    pub fn token_color(&self, cat: TokenCategory) -> Color {
        match cat {
            TokenCategory::Keyword => self.variations[1],
            TokenCategory::Builtin => self.foreground,
            TokenCategory::Operator => self.accent,
            TokenCategory::Number => self.variations[0],
            TokenCategory::String => self.variations[0],
            TokenCategory::Comment => self.muted,
            TokenCategory::Variable => self.variations[2],
            TokenCategory::Symbol => self.variations[3],
            TokenCategory::Special => self.variations[4],
            TokenCategory::Punctuation => self.variations[5],
        }
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
    //((first as u16 + second as u16) * 2 / 5) as u8
    ((first as u16 + second as u16) / 2) as u8
}