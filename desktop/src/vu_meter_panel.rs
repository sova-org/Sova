use eframe::egui;

use crate::widgets::VuMeter;

// ~300ms VU ballistics at 60fps
const ATTACK_COEFF: f32 = 0.15;
const RELEASE_COEFF: f32 = 0.05;

const PEAK_HOLD_FRAMES: u32 = 90; // ~1.5s at 60fps
const PEAK_DECAY_DB_PER_FRAME: f32 = 0.3;

const DB_FLOOR: f32 = -60.0;

pub struct VuMeterPanel {
    pub open: bool,
    rms_db: f32,
    peak_db: f32,
    peak_hold_frames: u32,
}

impl VuMeterPanel {
    pub fn new() -> Self {
        Self {
            open: false,
            rms_db: DB_FLOOR,
            peak_db: DB_FLOOR,
            peak_hold_frames: 0,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, scope_data: &[f32]) {
        let mut open = self.open;
        egui::Window::new("VU Meter")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_size([60.0, 200.0])
            .show(ctx, |ui| {
                if scope_data.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "No audio data");
                    self.rms_db = DB_FLOOR;
                    self.peak_db = DB_FLOOR;
                    self.peak_hold_frames = 0;
                } else {
                    let raw_db = compute_rms_db(scope_data);

                    // Exponential smoothing (VU ballistics)
                    let coeff = if raw_db > self.rms_db {
                        ATTACK_COEFF
                    } else {
                        RELEASE_COEFF
                    };
                    self.rms_db += coeff * (raw_db - self.rms_db);

                    // Peak hold + decay
                    if raw_db >= self.peak_db {
                        self.peak_db = raw_db;
                        self.peak_hold_frames = PEAK_HOLD_FRAMES;
                    } else if self.peak_hold_frames > 0 {
                        self.peak_hold_frames -= 1;
                    } else {
                        self.peak_db -= PEAK_DECAY_DB_PER_FRAME;
                        if self.peak_db < self.rms_db {
                            self.peak_db = self.rms_db;
                        }
                    }

                    VuMeter {
                        rms_db: self.rms_db,
                        peak_db: self.peak_db,
                    }
                    .show(ui);

                    ctx.request_repaint();
                }
            });
        self.open = open;
    }
}

fn compute_rms_db(samples: &[f32]) -> f32 {
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    let rms = (sum_sq / samples.len() as f32).sqrt();
    if rms <= 0.0 {
        return DB_FLOOR;
    }
    (20.0 * rms.log10()).clamp(DB_FLOOR, 0.0)
}
