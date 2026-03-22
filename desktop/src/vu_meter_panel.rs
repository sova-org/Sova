use eframe::egui;
use egui::containers::panel::Side;

use crate::widgets::VuMeter;

const ATTACK_COEFF: f32 = 0.15;
const RELEASE_COEFF: f32 = 0.05;

const PEAK_HOLD_FRAMES: u32 = 90;
const PEAK_DECAY_DB_PER_FRAME: f32 = 0.3;

const DB_FLOOR: f32 = -60.0;

const GAP: f32 = 3.0;
const PADDING: f32 = 2.0;
const DEFAULT_BAR_WIDTH: f32 = 20.0;
const MIN_BAR_WIDTH: f32 = 8.0;

struct ChannelMeter {
    rms_db: f32,
    peak_db: f32,
    peak_hold_frames: u32,
}

impl ChannelMeter {
    fn new() -> Self {
        Self {
            rms_db: DB_FLOOR,
            peak_db: DB_FLOOR,
            peak_hold_frames: 0,
        }
    }

    fn reset(&mut self) {
        self.rms_db = DB_FLOOR;
        self.peak_db = DB_FLOOR;
        self.peak_hold_frames = 0;
    }
}

pub struct VuMeterPanel {
    pub open: bool,
    channels: Vec<ChannelMeter>,
}

impl VuMeterPanel {
    pub fn new() -> Self {
        Self {
            open: false,
            channels: Vec::new(),
        }
    }

    pub fn show_side_panel(&mut self, ctx: &egui::Context, peak_data: &[f32], side: Side) {
        if !self.open {
            return;
        }

        if peak_data.is_empty() {
            for ch in &mut self.channels {
                ch.reset();
            }
            return;
        }

        let num_channels = peak_data.len();
        let default_width = num_channels as f32 * DEFAULT_BAR_WIDTH
            + (num_channels - 1) as f32 * GAP
            + 2.0 * PADDING;
        let min_width = num_channels as f32 * MIN_BAR_WIDTH
            + (num_channels - 1) as f32 * GAP
            + 2.0 * PADDING;

        egui::SidePanel::new(side, "vu_meter")
            .default_width(default_width)
            .width_range(min_width..=f32::INFINITY)
            .resizable(true)
            .show(ctx, |ui| {
                // Resize channel state to match incoming data
                self.channels.resize_with(peak_data.len(), ChannelMeter::new);
                self.channels.truncate(peak_data.len());

                for (ch, &peak) in self.channels.iter_mut().zip(peak_data.iter()) {
                    let raw_db = peak_to_db(peak);
                    let coeff = if raw_db > ch.rms_db { ATTACK_COEFF } else { RELEASE_COEFF };
                    ch.rms_db += coeff * (raw_db - ch.rms_db);

                    if raw_db >= ch.peak_db {
                        ch.peak_db = raw_db;
                        ch.peak_hold_frames = PEAK_HOLD_FRAMES;
                    } else if ch.peak_hold_frames > 0 {
                        ch.peak_hold_frames -= 1;
                    } else {
                        ch.peak_db -= PEAK_DECAY_DB_PER_FRAME;
                        if ch.peak_db < ch.rms_db {
                            ch.peak_db = ch.rms_db;
                        }
                    }
                }

                let avail = ui.available_size();
                let desired = egui::vec2(avail.x.max(24.0), avail.y.max(24.0));
                let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
                let painter = ui.painter_at(rect);

                let bg = ui.visuals().extreme_bg_color;
                painter.rect_filled(rect, 0.0, bg);

                let usable_width = rect.width() - 2.0 * PADDING;
                let n = self.channels.len().max(1) as f32;
                let bar_width = (usable_width - (n - 1.0) * GAP) / n;
                let bar_start_x = rect.left() + PADDING;
                for (i, ch) in self.channels.iter().enumerate() {
                    let x = bar_start_x + i as f32 * (bar_width + GAP);
                    let bar_rect = egui::Rect::from_min_max(
                        egui::pos2(x, rect.top()),
                        egui::pos2(x + bar_width, rect.bottom()),
                    );
                    VuMeter::new(ch.rms_db, ch.peak_db).paint_bar(&painter, bar_rect);
                }

                ctx.request_repaint();
            });
    }
}

fn peak_to_db(peak: f32) -> f32 {
    let v = peak.max(1e-10);
    (20.0 * v.log10()).clamp(DB_FLOOR, 0.0)
}
