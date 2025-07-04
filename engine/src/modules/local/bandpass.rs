use crate::dsp::biquad::{FilterType, StereoBiquadFilter};
use crate::modules::{AudioModule, Frame, LocalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_FREQUENCY: &str = "bpf";
const PARAM_RESONANCE: &str = "bpq";

const DEFAULT_FREQUENCY: f32 = 1000.0;
const DEFAULT_RESONANCE: f32 = 0.0;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_FREQUENCY,
        aliases: &[],
        min_value: 20.0,
        max_value: 20000.0,
        default_value: DEFAULT_FREQUENCY,
        unit: "Hz",
        description: "Band-pass filter center frequency",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_RESONANCE,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_RESONANCE,
        unit: "",
        description: "Filter resonance/bandwidth",
        modulable: true,
    },
];

pub struct BandPass {
    frequency: f32,
    resonance: f32,
    filter: StereoBiquadFilter,
    sample_rate: f32,
    is_active: bool,
}

impl Default for BandPass {
    fn default() -> Self {
        Self::new()
    }
}

impl BandPass {
    pub fn new() -> Self {
        let mut filter = StereoBiquadFilter::new();
        // Set initial filter parameters
        filter.set_filter(FilterType::BandPass, DEFAULT_FREQUENCY, 0.707, 0.0, 44100.0);

        Self {
            frequency: DEFAULT_FREQUENCY,
            resonance: DEFAULT_RESONANCE,
            filter,
            sample_rate: 44100.0,
            is_active: true,
        }
    }

    fn update_filter(&mut self) {
        // Scale 0.0-1.0 user range to 0.707-30.0 Q range for bandpass
        // Higher Q values make a narrower bandpass
        let q = 0.707 + self.resonance * self.resonance * 29.3;
        self.filter.set_filter(
            FilterType::BandPass,
            self.frequency,
            q,
            0.0,
            self.sample_rate,
        );
    }
}

impl AudioModule for BandPass {
    fn get_name(&self) -> &'static str {
        "bandpass"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_FREQUENCY => {
                self.frequency = value.clamp(20.0, 20000.0);
                self.update_filter();
                true
            }
            PARAM_RESONANCE => {
                self.resonance = value.clamp(0.0, 1.0);
                self.update_filter();
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl LocalEffect for BandPass {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.update_filter();
        }

        for frame in buffer.iter_mut() {
            let (left, right) = self.filter.process(frame.left, frame.right);
            frame.left = left;
            frame.right = right;
        }
    }
}

impl ModuleMetadata for BandPass {
    fn get_static_name() -> &'static str {
        "bandpass"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_bandpass() -> Box<dyn LocalEffect> {
    Box::new(BandPass::new())
}
