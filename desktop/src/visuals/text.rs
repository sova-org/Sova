use std::sync::OnceLock;

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};

pub struct TextData {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

const TEX_SIZE: u32 = 512;

fn default_font() -> &'static FontArc {
    static FONT: OnceLock<FontArc> = OnceLock::new();
    FONT.get_or_init(|| {
        let defs = eframe::egui::FontDefinitions::default();
        let bytes = defs.font_data["Hack"].font.to_vec();
        FontArc::try_from_vec(bytes).expect("valid font")
    })
}

pub fn rasterize(text: &str) -> TextData {
    let font = default_font();
    let mut pixels = vec![0u8; (TEX_SIZE * TEX_SIZE * 4) as usize];

    let lines: Vec<&str> = text.lines().collect();
    let line_count = lines.len().max(1);

    let scale = auto_scale(font, &lines, line_count);
    let scaled = font.as_scaled(scale);
    let line_height = scaled.height();
    let ascent = scaled.ascent();

    let total_height = line_height * line_count as f32;
    let y_start = (TEX_SIZE as f32 - total_height) / 2.0 + ascent;

    for (li, line) in lines.iter().enumerate() {
        let y = y_start + li as f32 * line_height;
        let line_width: f32 = line
            .chars()
            .map(|c| scaled.h_advance(font.glyph_id(c)))
            .sum();
        let x_start = (TEX_SIZE as f32 - line_width) / 2.0;

        let mut x = x_start;
        for ch in line.chars() {
            let gid = font.glyph_id(ch);
            let glyph = gid.with_scale_and_position(scale, ab_glyph::point(x, y));
            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|px, py, cov| {
                    let a = (cov * 255.0) as u8;
                    if a == 0 {
                        return;
                    }
                    let gx = bounds.min.x as i32 + px as i32;
                    let gy = bounds.min.y as i32 + py as i32;
                    if gx >= 0
                        && gx < TEX_SIZE as i32
                        && gy >= 0
                        && gy < TEX_SIZE as i32
                    {
                        let i = ((gy as u32 * TEX_SIZE + gx as u32) * 4) as usize;
                        pixels[i] = 255;
                        pixels[i + 1] = 255;
                        pixels[i + 2] = 255;
                        pixels[i + 3] = pixels[i + 3].saturating_add(a);
                    }
                });
            }
            x += scaled.h_advance(gid);
        }
    }

    TextData { pixels, width: TEX_SIZE, height: TEX_SIZE }
}

fn auto_scale(font: &FontArc, lines: &[&str], line_count: usize) -> PxScale {
    let target_w = TEX_SIZE as f32 * 0.9;
    let max_h = TEX_SIZE as f32 / line_count as f32;
    let mut px = max_h * 0.85;

    for line in lines {
        if line.is_empty() {
            continue;
        }
        let scaled = font.as_scaled(PxScale::from(px));
        let w: f32 = line
            .chars()
            .map(|c| scaled.h_advance(font.glyph_id(c)))
            .sum();
        if w > target_w {
            px *= target_w / w;
        }
    }

    PxScale::from(px.min(max_h * 0.85))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterize_produces_visible_output() {
        let data = rasterize("SOVA");
        let non_zero: usize = data.pixels.chunks(4).filter(|px| px[3] > 0).count();
        assert!(non_zero > 100, "rasterizer must produce visible text");
    }
}
