use crate::dsp::oscillators::{TableLfo, TriangleOscillator};
use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};
use std::any::Any;

/// Pure Rust implementation of detuned triangle oscillator
///
/// Features:
/// - Triangle wave with smooth edges for warm tonal character
/// - Efficient table-based LFO for wobble modulation
/// - True stereo output with detuned oscillator pair
/// - Zero-allocation real-time processing
/// - Automatic sample rate and block size detection
/// - Optimized to avoid unnecessary recalculations
pub struct DTriangleOscillator {
    /// Left channel triangle oscillator
    osc_left: TriangleOscillator,

    /// Right channel detuned triangle oscillator
    osc_right: TriangleOscillator,

    /// LFO for wobble modulation
    lfo: TableLfo,

    /// Current sample rate (detected from engine)
    sample_rate: f32,

    /// Current block size (detected from engine)
    block_size: usize,

    /// Parameters
    base_frequency: f32,
    detune_amount: f32,
    wobble_amount: f32,
    note: Option<f32>,

    /// LFO state for efficient frequency updates
    current_lfo: f32,
    last_lfo: f32,
    lfo_update_counter: u32,

    /// Initialization and dirty state
    initialized: bool,
    params_dirty: bool,
}

impl DTriangleOscillator {
    /// Create new pure Rust detuned triangle oscillator
    pub fn new() -> Self {
        Self {
            osc_left: TriangleOscillator::new(),
            osc_right: TriangleOscillator::new(),
            lfo: TableLfo::new(),
            sample_rate: 0.0,
            block_size: 0,
            base_frequency: 440.0,
            detune_amount: 1.0,
            wobble_amount: 0.3,
            note: None,
            current_lfo: 0.0,
            last_lfo: 0.0,
            lfo_update_counter: 0,
            initialized: false,
            params_dirty: true,
        }
    }

    /// Initialize oscillator with engine parameters
    fn initialize(&mut self, sample_rate: f32, block_size: usize) {
        if !self.initialized || self.sample_rate != sample_rate || self.block_size != block_size {
            self.sample_rate = sample_rate;
            self.block_size = block_size;
            self.lfo.set_frequency(0.5, sample_rate); // 0.5 Hz LFO
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

        // Calculate modulated detune
        let modulated_detune =
            self.detune_amount + (self.current_lfo * self.detune_amount * 0.5 * self.wobble_amount);

        // Update oscillators only when frequencies actually change
        self.osc_left.set_frequency(frequency, self.sample_rate);
        self.osc_right
            .set_frequency(frequency + modulated_detune, self.sample_rate);
    }

    /// Update parameters and LFO efficiently
    fn update_params(&mut self) {
        if self.params_dirty {
            self.update_frequencies();
            self.params_dirty = false;
        }

        // Update LFO only every 4 samples for efficiency
        self.lfo_update_counter += 1;
        if self.lfo_update_counter >= 4 {
            self.lfo_update_counter = 0;
            self.current_lfo = self.lfo.next_sample();

            // Only update frequencies if LFO changed significantly
            let lfo_diff = (self.current_lfo - self.last_lfo).abs();
            if lfo_diff > 0.001 && self.wobble_amount > 0.0 {
                self.update_frequencies();
                self.last_lfo = self.current_lfo;
            }
        }
    }
}

impl AudioModule for DTriangleOscillator {
    fn get_name(&self) -> &'static str {
        "dtriangle"
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
            "z1" | "detune" => {
                let new_detune = value.clamp(0.0, 10.0);
                if self.detune_amount != new_detune {
                    self.detune_amount = new_detune;
                    self.params_dirty = true;
                }
                true
            }
            "z2" | "wobble" => {
                let new_wobble = value.clamp(0.0, 1.0);
                if self.wobble_amount != new_wobble {
                    self.wobble_amount = new_wobble;
                    // No need to set params_dirty for wobble - it just changes LFO effect
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

impl Source for DTriangleOscillator {
    fn generate(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        // Auto-detect and initialize with engine parameters
        self.initialize(sample_rate, buffer.len());

        // Generate audio samples
        for frame in buffer.iter_mut() {
            // Update parameters and LFO efficiently (not every sample)
            self.update_params();

            // Generate oscillator outputs
            let left_osc = self.osc_left.next_sample();
            let right_osc = self.osc_right.next_sample();

            // Mix for stereo width (80% main osc, 20% other osc per channel)
            let gain = crate::dsp::math::stereo_mix_gain();
            frame.left = (left_osc * 0.8 + right_osc * 0.2) * gain;
            frame.right = (right_osc * 0.8 + left_osc * 0.2) * gain;
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ModuleMetadata for DTriangleOscillator {
    fn get_static_name() -> &'static str {
        "dtriangle"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        &PARAMETER_DESCRIPTORS
    }
}

/// Parameter descriptors for dtriangle oscillator
static PARAMETER_DESCRIPTORS: [ParameterDescriptor; 4] = [
    ParameterDescriptor {
        name: "frequency",
        aliases: &["freq"],
        min_value: 20.0,
        max_value: 20000.0,
        default_value: 440.0,
        unit: "Hz",
        description: "Oscillator frequency in Hz",
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
        aliases: &["detune"],
        min_value: 0.0,
        max_value: 10.0,
        default_value: 1.0,
        unit: "",
        description: "Detune amount",
        modulable: true,
    },
    ParameterDescriptor {
        name: "z2",
        aliases: &["wobble"],
        min_value: 0.0,
        max_value: 1.0,
        default_value: 0.3,
        unit: "",
        description: "Detune wobble amount",
        modulable: true,
    },
];

/// Creates a new detuned triangle oscillator instance
pub fn create_dtriangle_oscillator() -> Box<dyn Source> {
    Box::new(DTriangleOscillator::new())
}

impl Default for DTriangleOscillator {
    fn default() -> Self {
        Self::new()
    }
}
