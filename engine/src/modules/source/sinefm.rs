use crate::dsp::oscillators::SineOscillator;
use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};
use std::any::Any;

/// Pure Rust implementation of 2-operator FM synthesis oscillator
///
/// Features:
/// - Classic FM synthesis with carrier and modulator oscillators
/// - Configurable modulation index and frequency ratio
/// - Zero-allocation real-time processing
/// - Automatic sample rate detection
/// - Optimized to avoid unnecessary recalculations
pub struct SineFmOscillator {
    /// Carrier oscillator
    carrier: SineOscillator,

    /// Modulator oscillator
    modulator: SineOscillator,

    /// Current sample rate (detected from engine)
    sample_rate: f32,

    /// Parameters
    base_frequency: f32,
    fm_index: f32,
    fm_ratio: f32,
    note: Option<f32>,

    /// Initialization and dirty state
    initialized: bool,
    params_dirty: bool,
}

impl SineFmOscillator {
    /// Create new pure Rust FM oscillator
    pub fn new() -> Self {
        Self {
            carrier: SineOscillator::new(),
            modulator: SineOscillator::new(),
            sample_rate: 0.0,
            base_frequency: 440.0,
            fm_index: 1.0,
            fm_ratio: 1.0,
            note: None,
            initialized: false,
            params_dirty: true,
        }
    }

    /// Initialize oscillator with engine parameters
    fn initialize(&mut self, sample_rate: f32) {
        if !self.initialized || self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.initialized = true;
            self.params_dirty = true;
        }
    }

    /// Update oscillator frequencies (only called when necessary)
    fn update_frequencies(&mut self) {
        let frequency = if let Some(note) = self.note {
            crate::dsp::math::midi_to_freq(note)
        } else {
            self.base_frequency
        };

        // Set modulator frequency (freq * ratio)
        self.modulator
            .set_frequency(frequency * self.fm_ratio, self.sample_rate);

        // Carrier frequency will be modulated in real-time, so we set base frequency
        self.carrier.set_frequency(frequency, self.sample_rate);
    }

    /// Update parameters efficiently
    fn update_params(&mut self) {
        if self.params_dirty {
            self.update_frequencies();
            self.params_dirty = false;
        }
    }
}

impl AudioModule for SineFmOscillator {
    fn get_name(&self) -> &'static str {
        "sinefm"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        &PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, name: &str, value: f32) -> bool {
        match name {
            "frequency" | "freq" => {
                let new_freq = value.clamp(20.0, 20000.0);
                if self.base_frequency != new_freq {
                    self.base_frequency = new_freq;
                    self.note = None; // Clear note override
                    self.params_dirty = true;
                }
                true
            }
            "note" => {
                let new_note = value.clamp(0.0, 127.0);
                if self.note != Some(new_note) {
                    self.note = Some(new_note);
                    self.params_dirty = true;
                }
                true
            }
            "z1" | "index" => {
                let new_index = value.clamp(0.0, 10.0);
                if self.fm_index != new_index {
                    self.fm_index = new_index;
                    // No need to set params_dirty for index - it's applied in real-time
                }
                true
            }
            "z2" | "ratio" => {
                let new_ratio = value.clamp(0.25, 8.0);
                if self.fm_ratio != new_ratio {
                    self.fm_ratio = new_ratio;
                    self.params_dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        true
    }
}

impl Source for SineFmOscillator {
    fn generate(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        // Auto-detect and initialize with engine parameters
        self.initialize(sample_rate);

        // Update parameters if needed
        self.update_params();

        // Generate FM synthesis samples
        for frame in buffer.iter_mut() {
            // Generate modulator signal: modulator_osc * index * carrier_freq
            let modulator_sample = self.modulator.next_sample();
            let carrier_freq = if let Some(note) = self.note {
                crate::dsp::math::midi_to_freq(note)
            } else {
                self.base_frequency
            };

            // FM modulation: modulator * index * carrier_frequency
            let fm_modulation = modulator_sample * self.fm_index * carrier_freq;

            // Set instantaneous carrier frequency: base_freq + modulation
            let modulated_freq = carrier_freq + fm_modulation;
            self.carrier.set_frequency(modulated_freq, self.sample_rate);

            // Generate carrier with frequency modulation
            let carrier_sample = self.carrier.next_sample();

            // Apply amplitude calibration to match Faust output
            let output = carrier_sample * 0.5;

            frame.left = output;
            frame.right = output;
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ModuleMetadata for SineFmOscillator {
    fn get_static_name() -> &'static str {
        "sinefm"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        &PARAMETER_DESCRIPTORS
    }
}

/// Parameter descriptors for sinefm oscillator
static PARAMETER_DESCRIPTORS: [ParameterDescriptor; 4] = [
    ParameterDescriptor {
        name: "frequency",
        aliases: &["freq"],
        min_value: 20.0,
        max_value: 20000.0,
        default_value: 440.0,
        unit: "Hz",
        description: "Carrier frequency in Hz",
        modulable: true,
    },
    ParameterDescriptor {
        name: "note",
        aliases: &[],
        min_value: 0.0,
        max_value: 127.0,
        default_value: 69.0,
        unit: "",
        description: "MIDI note number (overrides frequency)",
        modulable: true,
    },
    ParameterDescriptor {
        name: "z1",
        aliases: &["index"],
        min_value: 0.0,
        max_value: 10.0,
        default_value: 1.0,
        unit: "",
        description: "FM modulation index",
        modulable: true,
    },
    ParameterDescriptor {
        name: "z2",
        aliases: &["ratio"],
        min_value: 0.25,
        max_value: 8.0,
        default_value: 1.0,
        unit: "",
        description: "Modulator frequency ratio",
        modulable: true,
    },
];

/// Creates a new FM oscillator instance
pub fn create_sinefm_oscillator() -> Box<dyn Source> {
    Box::new(SineFmOscillator::new())
}

impl Default for SineFmOscillator {
    fn default() -> Self {
        Self::new()
    }
}
