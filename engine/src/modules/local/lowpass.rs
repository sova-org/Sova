use crate::dsp::moog_ladder::StereoMoogLadder;
use crate::modules::{AudioModule, Frame, LocalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_CUTOFF: &str = "lpf";
const PARAM_RESONANCE: &str = "lpq";

const DEFAULT_CUTOFF: f32 = 1000.0;
const DEFAULT_RESONANCE: f32 = 0.0;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_CUTOFF,
        aliases: &[],
        min_value: 20.0,
        max_value: 20000.0,
        default_value: DEFAULT_CUTOFF,
        unit: "Hz",
        description: "Low-pass filter cutoff frequency",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_RESONANCE,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_RESONANCE,
        unit: "",
        description: "Filter resonance",
        modulable: true,
    },
];

pub struct LowPass {
    cutoff: f32,
    resonance: f32,
    filter: StereoMoogLadder,
    sample_rate: f32,
    is_active: bool,
}

impl Default for LowPass {
    fn default() -> Self {
        Self::new()
    }
}

impl LowPass {
    pub fn new() -> Self {
        let mut filter = StereoMoogLadder::new();
        filter.init(44100.0);
        filter.set_cutoff(DEFAULT_CUTOFF);
        filter.set_resonance(DEFAULT_RESONANCE);
        
        Self {
            cutoff: DEFAULT_CUTOFF,
            resonance: DEFAULT_RESONANCE,
            filter,
            sample_rate: 44100.0,
            is_active: true,
        }
    }
    
    fn update_filter(&mut self) {
        self.filter.set_cutoff(self.cutoff);
        // Scale 0.0-1.0 user range to 0.0-4.0 internal range for self-oscillation
        let scaled_resonance = self.resonance * 4.0;
        self.filter.set_resonance(scaled_resonance);
    }
}

impl AudioModule for LowPass {
    fn get_name(&self) -> &'static str {
        "lowpass"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_CUTOFF => {
                self.cutoff = value.clamp(20.0, 20000.0);
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

impl LocalEffect for LowPass {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.filter.init(sample_rate);
            self.update_filter();
        }

        for frame in buffer.iter_mut() {
            let (left, right) = self.filter.process(frame.left, frame.right);
            frame.left = left;
            frame.right = right;
        }
    }
}

impl ModuleMetadata for LowPass {
    fn get_static_name() -> &'static str {
        "lowpass"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_lowpass() -> Box<dyn LocalEffect> {
    Box::new(LowPass::new())
}