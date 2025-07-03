use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};
use std::any::Any;

/// Amplitude calibration constant for sine oscillator
/// This value normalizes the output to approximately 0 dBFS
const AMP_CALIBRATION: f32 = 0.5;

faust_macro::dsp!(
    declare name "sine_oscillator";
    declare version "1.0";

    import("stdfaust.lib");

    freq = hslider("freq", 440.0, 20.0, 20000.0, 0.01);

    process = os.osc(freq);
);

/// Simple sine oscillator with frequency control
pub struct SineOscillator {
    dsp: Box<sine_oscillator::SineOscillator>,
    frequency: f32,
    note: Option<f32>,
    params_dirty: bool,
    sample_rate: f32,
    output: [f32; 1024],
}

impl SineOscillator {
    /// Creates a new sine oscillator
    pub fn new() -> Self {
        let dsp = Box::new(sine_oscillator::SineOscillator::new());
        Self {
            dsp,
            frequency: 440.0,
            note: None,
            params_dirty: true,
            sample_rate: 0.0,
            output: [0.0; 1024],
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
            self.params_dirty = false;
        }
    }
}

impl AudioModule for SineOscillator {
    fn get_name(&self) -> &'static str {
        "sine"
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
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        true
    }
}

impl Source for SineOscillator {
    fn generate(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.dsp.init(sample_rate as i32);
            self.params_dirty = true;
        }

        if self.params_dirty {
            self.update_params();
            self.params_dirty = false;
        }

        for chunk in buffer.chunks_mut(256) {
            let chunk_size = chunk.len();

            for i in 0..chunk_size {
                self.output[i] = 0.0;
            }

            let inputs: [&[f32]; 0] = [];
            let mut outputs = [&mut self.output[..chunk_size]];

            self.dsp.compute(chunk_size, &inputs, &mut outputs);

            for (i, frame) in chunk.iter_mut().enumerate() {
                let sample = self.output[i] * AMP_CALIBRATION;
                frame.left = sample;
                frame.right = sample;
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

static PARAMETER_DESCRIPTORS: [ParameterDescriptor; 2] = [
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
];

impl ModuleMetadata for SineOscillator {
    fn get_static_name() -> &'static str {
        "sine"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        &PARAMETER_DESCRIPTORS
    }
}

/// Creates a new sine oscillator instance
pub fn create_sine_oscillator() -> Box<dyn Source> {
    Box::new(SineOscillator::new())
}
