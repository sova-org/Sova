use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};
use std::any::Any;

/// Amplitude calibration constant for detuned saw oscillator
const AMP_CALIBRATION: f32 = 0.4;

faust_macro::dsp!(
    declare name "dsaw_oscillator";
    declare version "1.0";

    import("stdfaust.lib");

    freq = hslider("freq", 440.0, 20.0, 20000.0, 0.01);
    detune = hslider("detune", 1.0, 0.0, 10.0, 0.01);
    wobble = hslider("wobble", 0.3, 0.0, 1.0, 0.01);

    // LFO for detune modulation (0.5 Hz)
    lfo = os.osc(0.5) * wobble;

    // Two oscillators: one at base freq, one slightly detuned
    osc1 = os.sawtooth(freq);
    osc2 = os.sawtooth(freq + detune + (lfo * detune * 0.5));

    // Stereo output: osc1 left, osc2 right, with slight mixing
    left = (osc1 * 0.8 + osc2 * 0.2) * 0.5;
    right = (osc2 * 0.8 + osc1 * 0.2) * 0.5;

    process = left, right;
);

/// Detuned saw oscillator with two voices
pub struct DSawOscillator {
    dsp: Box<dsaw_oscillator::DsawOscillator>,
    frequency: f32,
    note: Option<f32>,
    detune_cents: f32,
    wobble: f32,
    params_dirty: bool,
    sample_rate: f32,
    left_output: [f32; 1024],
    right_output: [f32; 1024],
    initialized: bool,
}

impl DSawOscillator {
    /// Creates a new detuned saw oscillator
    pub fn new() -> Self {
        let dsp = Box::new(dsaw_oscillator::DsawOscillator::new());
        Self {
            dsp,
            frequency: 440.0,
            note: None,
            detune_cents: 1.0,
            wobble: 0.3,
            params_dirty: true,
            sample_rate: 0.0,
            left_output: [0.0; 1024],
            right_output: [0.0; 1024],
            initialized: false,
        }
    }

    /// Updates the internal Faust parameters
    fn update_params(&mut self) {
        if self.params_dirty {
            let freq = if let Some(note) = self.note {
                440.0 * 2.0_f32.powf((note - 69.0) / 12.0)
            } else {
                self.frequency
            };

            self.dsp.set_param(faust_types::ParamIndex(0), freq);
            self.dsp
                .set_param(faust_types::ParamIndex(1), self.detune_cents);
            self.dsp.set_param(faust_types::ParamIndex(2), self.wobble);
            self.params_dirty = false;
        }
    }
}

impl AudioModule for DSawOscillator {
    fn get_name(&self) -> &'static str {
        "dsaw"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        &PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, name: &str, value: f32) -> bool {
        match name {
            "frequency" | "freq" => {
                self.frequency = value.clamp(20.0, 20000.0);
                self.note = None;
                self.params_dirty = true;
                true
            }
            "note" => {
                self.note = Some(value);
                self.params_dirty = true;
                true
            }
            "z1" => {
                self.detune_cents = value.clamp(0.0, 10.0);
                self.params_dirty = true;
                true
            }
            "z2" => {
                self.wobble = value.clamp(0.0, 1.0);
                self.params_dirty = true;
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        true
    }
}

impl Source for DSawOscillator {
    fn generate(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if !self.initialized {
            self.dsp.init(sample_rate as i32);
            self.sample_rate = sample_rate;
            self.initialized = true;
            self.update_params();
        }

        if self.params_dirty {
            self.update_params();
            self.params_dirty = false;
        }

        let max_chunk_size = 512;
        let chunk_size = buffer.len().min(max_chunk_size);

        for chunk in buffer.chunks_mut(chunk_size) {
            let actual_chunk_size = chunk.len();

            for i in 0..actual_chunk_size {
                self.left_output[i] = 0.0;
                self.right_output[i] = 0.0;
            }

            let inputs: [&[f32]; 0] = [];
            let mut outputs = [
                &mut self.left_output[..actual_chunk_size],
                &mut self.right_output[..actual_chunk_size],
            ];

            self.dsp.compute(actual_chunk_size, &inputs, &mut outputs);

            for (i, frame) in chunk.iter_mut().enumerate() {
                frame.left = self.left_output[i] * AMP_CALIBRATION;
                frame.right = self.right_output[i] * AMP_CALIBRATION;
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ModuleMetadata for DSawOscillator {
    fn get_static_name() -> &'static str {
        "dsaw"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        &PARAMETER_DESCRIPTORS
    }
}

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

/// Creates a new detuned saw oscillator instance
pub fn create_dsaw_oscillator() -> Box<dyn Source> {
    Box::new(DSawOscillator::new())
}
