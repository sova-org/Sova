use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};
use std::any::Any;

/// Amplitude calibration constant for FM oscillator
const AMP_CALIBRATION: f32 = 0.5;

faust_macro::dsp!(
    declare name "sinefm_oscillator";
    declare version "1.0";

    import("stdfaust.lib");

    freq = hslider("freq", 440.0, 20.0, 20000.0, 0.01);
    index = hslider("index", 1.0, 0.0, 10.0, 0.01);
    ratio = hslider("ratio", 1.0, 0.25, 8.0, 0.01);

    modulator = os.osc(freq * ratio) * index * freq;
    carrier = os.osc(freq + modulator);

    process = carrier;
);

/// 2-operator FM synthesis oscillator
pub struct SineFmOscillator {
    dsp: Box<sinefm_oscillator::SinefmOscillator>,
    frequency: f32,
    note: Option<f32>,
    fm_index: f32,
    fm_ratio: f32,
    params_dirty: bool,
    sample_rate: f32,
    output: [f32; 1024],
}

impl SineFmOscillator {
    /// Creates a new FM oscillator
    pub fn new() -> Self {
        let dsp = Box::new(sinefm_oscillator::SinefmOscillator::new());
        Self {
            dsp,
            frequency: 440.0,
            note: None,
            fm_index: 1.0,
            fm_ratio: 1.0,
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
            self.dsp
                .set_param(faust_types::ParamIndex(1), self.fm_index);
            self.dsp
                .set_param(faust_types::ParamIndex(2), self.fm_ratio);
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
                self.fm_index = value.clamp(0.0, 10.0);
                self.params_dirty = true;
                true
            }
            "z2" => {
                self.fm_ratio = value.clamp(0.25, 8.0);
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

impl Source for SineFmOscillator {
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

impl ModuleMetadata for SineFmOscillator {
    fn get_static_name() -> &'static str {
        "sinefm"
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
