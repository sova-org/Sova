use eframe::egui::{self, Color32, Context, FontFamily, FontId, Style, TextStyle};

use crate::settings::AppearanceSettings;

pub(crate) fn apply_appearance(ctx: &egui::Context, a: &AppearanceSettings) {
    ctx.set_theme(egui::ThemePreference::Dark);

    ctx.set_zoom_factor(a.zoom);

    ctx.all_styles_mut(|style| {
        let zero = egui::CornerRadius::ZERO;
        style.visuals.window_corner_radius = zero;
        style.visuals.menu_corner_radius = zero;
        style.visuals.widgets.noninteractive.corner_radius = zero;
        style.visuals.widgets.inactive.corner_radius = zero;
        style.visuals.widgets.hovered.corner_radius = zero;
        style.visuals.widgets.active.corner_radius = zero;
        style.visuals.widgets.open.corner_radius = zero;

        let accent =
            egui::Color32::from_rgb(a.accent_color[0], a.accent_color[1], a.accent_color[2]);
        style.visuals.selection.bg_fill = accent;

        if a.window_shadows {
            let defaults = if style.visuals.dark_mode {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            };
            style.visuals.window_shadow = defaults.window_shadow;
            style.visuals.popup_shadow = defaults.popup_shadow;
        } else {
            style.visuals.window_shadow = egui::Shadow::NONE;
            style.visuals.popup_shadow = egui::Shadow::NONE;
        }

        let bg = a.bg_brightness;
        style.visuals.extreme_bg_color = egui::Color32::from_gray(bg);
        style.visuals.panel_fill = egui::Color32::from_gray(bg.saturating_add(8));
        style.visuals.window_fill = egui::Color32::from_gray(bg.saturating_add(12));
        style.visuals.faint_bg_color = egui::Color32::from_gray(bg.saturating_add(20));

        style.spacing.button_padding = egui::vec2(5.0, 4.0);
        style.spacing.indent_ends_with_horizontal_line = true;

        style.spacing.scroll.bar_width = 0.0;
        style.spacing.scroll.floating_width = 0.0;
        style.spacing.scroll.floating_allocated_width = 0.0;
        style.spacing.scroll.bar_inner_margin = 0.0;
        style.spacing.scroll.bar_outer_margin = 0.0;

        apply_text_styles(style, a.ui_font_size);

        style.animation_time = a.animation_time;
    });
}

pub const COLOR_OK: Color32 = Color32::from_rgb(100, 200, 100);
pub const COLOR_ERROR: Color32 = Color32::from_rgb(200, 100, 100);
pub const COLOR_MUTED: Color32 = Color32::from_rgb(128, 128, 128);

pub const STROKE_HAIRLINE: f32 = 1.0;
pub const STROKE_NORMAL: f32 = 1.5;
pub const STROKE_EMPHASIS: f32 = 2.0;

pub fn accent_fill_strong(c: Color32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 120)
}

pub fn accent_fill_med(c: Color32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 70)
}

pub fn accent_fill_soft(c: Color32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 35)
}

pub fn tile_label_font(ctx: &Context) -> FontId {
    let size = ctx.style().text_styles[&TextStyle::Small].size;
    FontId::new(size, FontFamily::Monospace)
}

pub fn apply_text_styles(style: &mut Style, ui_size: f32) {
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(ui_size));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(ui_size));
    style.text_styles.insert(
        TextStyle::Small,
        FontId::proportional((ui_size - 2.0).max(8.0)),
    );
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(ui_size + 7.0));
}

pub fn username_color(name: &str) -> Color32 {
    let mut hash: u32 = 0;
    for b in name.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as u32);
    }
    let hue = (hash % 360) as f32;
    let (r, g, b) = hsl_to_rgb(hue, 0.40, 0.60);
    Color32::from_rgb(r, g, b)
}

pub fn cycled_accent(accent: Color32, index: usize) -> Color32 {
    const N: usize = 8;
    if index.is_multiple_of(N) {
        return accent;
    }
    let (h, s, l) = rgb_to_hsl(accent.r(), accent.g(), accent.b());
    let rotated = (h + (index % N) as f32 * (360.0 / N as f32)) % 360.0;
    let (r, g, b) = hsl_to_rgb(rotated, s, l);
    Color32::from_rgb(r, g, b)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match (h as u32) / 60 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    (h, s, l)
}
