use crate::dsp::biquad::{StereoBiquadFilter, FilterType};
use crate::modules::{AudioModule, Frame, LocalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_FREQUENCY: &str = "npf";
const PARAM_RESONANCE: &str = "npq";

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
        description: "Notch filter center frequency",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_RESONANCE,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_RESONANCE,
        unit: "",
        description: "Filter resonance/notch width",
        modulable: true,
    },
];

pub struct Notch {
    frequency: f32,
    resonance: f32,
    filter: StereoBiquadFilter,
    sample_rate: f32,
    is_active: bool,
}

impl Default for Notch {
    fn default() -> Self {
        Self::new()
    }
}

impl Notch {
    pub fn new() -> Self {
        let mut filter = StereoBiquadFilter::new();
        // Set initial filter parameters
        filter.set_filter(FilterType::Notch, DEFAULT_FREQUENCY, 0.707, 0.0, 44100.0);
        
        Self {
            frequency: DEFAULT_FREQUENCY,
            resonance: DEFAULT_RESONANCE,
            filter,
            sample_rate: 44100.0,
            is_active: true,
        }
    }
    
    fn update_filter(&mut self) {
        // Scale 0.0-1.0 user range to 0.707-30.0 Q range for notch
        // Higher Q values make a narrower notch (deeper, more precise cut)
        let q = 0.707 + self.resonance * self.resonance * 29.3;
        self.filter.set_filter(FilterType::Notch, self.frequency, q, 0.0, self.sample_rate);
    }
}

impl AudioModule for Notch {
    fn get_name(&self) -> &'static str {
        "notch"
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

impl LocalEffect for Notch {
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

impl ModuleMetadata for Notch {
    fn get_static_name() -> &'static str {
        "notch"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_notch() -> Box<dyn LocalEffect> {
    Box::new(Notch::new())
}