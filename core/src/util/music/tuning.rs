pub const NOTE_C : i16 = 0;
pub const NOTE_C_SHARP : i16 = 1;
pub const NOTE_D_FLAT : i16 = 1;
pub const NOTE_D : i16 = 2;
pub const NOTE_D_SHARP : i16 = 3;
pub const NOTE_E_FLAT : i16 = 3;
pub const NOTE_E : i16 = 4;
pub const NOTE_F : i16 = 5;
pub const NOTE_F_SHARP : i16 = 6;
pub const NOTE_G_FLAT : i16 = 6;
pub const NOTE_G : i16 = 7;
pub const NOTE_G_SHARP : i16 = 8;
pub const NOTE_A_FLAT : i16 = 8;
pub const NOTE_A : i16 = 9;
pub const NOTE_A_SHARP : i16 = 10;
pub const NOTE_B_FLAT : i16 = 10;
pub const NOTE_B : i16 = 11;

pub const DEFAULT_C_TUNING : f64 = 261.6255653006;

const TUNING_OCTAVE : i16 = 4;

pub fn note_freq_with_tune(note: i16, octave: i16, tuning: f64, tuning_octave: i16) -> f64 {
    let incr_factor = f64::powf(2.0, 1.0 / 12.0);
    let delta_octave = (octave - tuning_octave) as i32;
    tuning * incr_factor.powi(note as i32) * f64::powi(2.0, delta_octave)
}

/// Use default C4 tuning 261.6255... Hz to get note frequency
pub fn note_freq(note: i16, octave: i16) -> f64 {
    note_freq_with_tune(note, octave, DEFAULT_C_TUNING, TUNING_OCTAVE)
}
