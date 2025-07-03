use crate::dsp::oscillators::SineOscillator;
use crate::modules::{AudioModule, Frame, LocalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_FREQUENCY: &str = "ringfreq";
const PARAM_DEPTH: &str = "ringdepth";

const DEFAULT_FREQUENCY: f32 = 5.0;
const DEFAULT_DEPTH: f32 = 1.0;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_FREQUENCY,
        aliases: &["rfreq"],
        min_value: 0.01,
        max_value: 1000.0,
        default_value: DEFAULT_FREQUENCY,
        unit: "Hz",
        description: "Ring modulation frequency",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_DEPTH,
        aliases: &["rdepth"],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_DEPTH,
        unit: "",
        description: "Ring modulation depth",
        modulable: true,
    },
];

pub struct RingModulator {
    frequency: f32,
    depth: f32,
    oscillator: SineOscillator,
    sample_rate: f32,
    is_active: bool,
}

impl Default for RingModulator {
    fn default() -> Self {
        Self::new()
    }
}

impl RingModulator {
    pub fn new() -> Self {
        let mut oscillator = SineOscillator::new();
        oscillator.set_frequency(DEFAULT_FREQUENCY, 44100.0);

        Self {
            frequency: DEFAULT_FREQUENCY,
            depth: DEFAULT_DEPTH,
            oscillator,
            sample_rate: 44100.0,
            is_active: true,
        }
    }
}

impl AudioModule for RingModulator {
    fn get_name(&self) -> &'static str {
        "ringmod"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_FREQUENCY => {
                self.frequency = value.clamp(0.01, 1000.0);
                self.oscillator.set_frequency(self.frequency, self.sample_rate);
                true
            }
            PARAM_DEPTH => {
                self.depth = value.clamp(0.0, 1.0);
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl LocalEffect for RingModulator {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.oscillator.set_frequency(self.frequency, sample_rate);
        }

        for frame in buffer.iter_mut() {
            let carrier = self.oscillator.next_sample();
            
            let wet_left = frame.left * carrier;
            let wet_right = frame.right * carrier;
            
            frame.left = frame.left * (1.0 - self.depth) + wet_left * self.depth;
            frame.right = frame.right * (1.0 - self.depth) + wet_right * self.depth;
        }
    }
}

impl ModuleMetadata for RingModulator {
    fn get_static_name() -> &'static str {
        "ringmod"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_ring_modulator() -> Box<dyn LocalEffect> {
    Box::new(RingModulator::new())
}
