use crate::dsp::oscillators::TableLfo;
use crate::modules::{AudioModule, Frame, LocalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_DEPTH: &str = "tdepth";
const PARAM_RATE: &str = "trate";

const DEFAULT_DEPTH: f32 = 0.5;
const DEFAULT_RATE: f32 = 5.0;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_DEPTH,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_DEPTH,
        unit: "",
        description: "Tremolo depth",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_RATE,
        aliases: &[],
        min_value: 0.1,
        max_value: 20.0,
        default_value: DEFAULT_RATE,
        unit: "Hz",
        description: "Tremolo rate",
        modulable: true,
    },
];

pub struct Tremolo {
    depth: f32,
    rate: f32,
    lfo: TableLfo,
    sample_rate: f32,
    is_active: bool,
}

impl Default for Tremolo {
    fn default() -> Self {
        Self::new()
    }
}

impl Tremolo {
    pub fn new() -> Self {
        let mut lfo = TableLfo::new();
        lfo.set_frequency(DEFAULT_RATE, 44100.0);

        Self {
            depth: DEFAULT_DEPTH,
            rate: DEFAULT_RATE,
            lfo,
            sample_rate: 44100.0,
            is_active: true,
        }
    }
}

impl AudioModule for Tremolo {
    fn get_name(&self) -> &'static str {
        "tremolo"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_DEPTH => {
                self.depth = value.clamp(0.0, 1.0);
                true
            }
            PARAM_RATE => {
                self.rate = value.clamp(0.1, 20.0);
                self.lfo.set_frequency(self.rate, self.sample_rate);
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl LocalEffect for Tremolo {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.lfo.set_frequency(self.rate, sample_rate);
        }

        for frame in buffer.iter_mut() {
            let lfo_value = self.lfo.next_sample();
            let gain = 1.0 - self.depth * 0.5 * (1.0 - lfo_value);

            frame.left *= gain;
            frame.right *= gain;
        }
    }
}

impl ModuleMetadata for Tremolo {
    fn get_static_name() -> &'static str {
        "tremolo"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_tremolo() -> Box<dyn LocalEffect> {
    Box::new(Tremolo::new())
}
