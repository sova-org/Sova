use crate::dsp::all_pass_filter::AllPassFilter;
use crate::dsp::oscillators::TableLfo;
use crate::modules::{AudioModule, Frame, LocalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_DEPTH: &str = "pdepth";
const PARAM_RATE: &str = "prate";
const PARAM_FEEDBACK: &str = "pfeedback";
const PARAM_STAGES: &str = "pstages";
const PARAM_MIX: &str = "pmix";

const DEFAULT_DEPTH: f32 = 0.5;
const DEFAULT_RATE: f32 = 0.5;
const DEFAULT_FEEDBACK: f32 = 0.7;
const DEFAULT_STAGES: f32 = 4.0;
const DEFAULT_MIX: f32 = 0.5;

const MAX_STAGES: usize = 8;
const APF_SIZE: usize = 256;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_DEPTH,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_DEPTH,
        unit: "",
        description: "Phaser depth",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_RATE,
        aliases: &[],
        min_value: 0.01,
        max_value: 10.0,
        default_value: DEFAULT_RATE,
        unit: "Hz",
        description: "Phaser rate",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_FEEDBACK,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_FEEDBACK,
        unit: "",
        description: "Phaser feedback",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_STAGES,
        aliases: &[],
        min_value: 2.0,
        max_value: 8.0,
        default_value: DEFAULT_STAGES,
        unit: "",
        description: "Number of phaser stages",
        modulable: false,
    },
    ParameterDescriptor {
        name: PARAM_MIX,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_MIX,
        unit: "",
        description: "Phaser mix",
        modulable: true,
    },
];

pub struct Phaser {
    depth: f32,
    rate: f32,
    feedback: f32,
    stages: usize,
    mix: f32,
    lfo: TableLfo,
    apf_left: [AllPassFilter<APF_SIZE>; MAX_STAGES],
    apf_right: [AllPassFilter<APF_SIZE>; MAX_STAGES],
    sample_rate: f32,
    is_active: bool,
}

impl Default for Phaser {
    fn default() -> Self {
        Self::new()
    }
}

impl Phaser {
    pub fn new() -> Self {
        let mut lfo = TableLfo::new();
        lfo.set_frequency(DEFAULT_RATE, 44100.0);

        Self {
            depth: DEFAULT_DEPTH,
            rate: DEFAULT_RATE,
            feedback: DEFAULT_FEEDBACK * 0.7,
            stages: DEFAULT_STAGES as usize,
            mix: DEFAULT_MIX,
            lfo,
            apf_left: [
                AllPassFilter::new(),
                AllPassFilter::new(),
                AllPassFilter::new(),
                AllPassFilter::new(),
                AllPassFilter::new(),
                AllPassFilter::new(),
                AllPassFilter::new(),
                AllPassFilter::new(),
            ],
            apf_right: [
                AllPassFilter::new(),
                AllPassFilter::new(),
                AllPassFilter::new(),
                AllPassFilter::new(),
                AllPassFilter::new(),
                AllPassFilter::new(),
                AllPassFilter::new(),
                AllPassFilter::new(),
            ],
            sample_rate: 44100.0,
            is_active: true,
        }
    }
}

impl AudioModule for Phaser {
    fn get_name(&self) -> &'static str {
        "phaser"
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
                self.rate = value.clamp(0.01, 10.0);
                self.lfo.set_frequency(self.rate, self.sample_rate);
                true
            }
            PARAM_FEEDBACK => {
                self.feedback = value.clamp(0.0, 1.0) * 0.7;
                true
            }
            PARAM_STAGES => {
                self.stages = (value.clamp(2.0, 8.0) as usize).min(MAX_STAGES);
                true
            }
            PARAM_MIX => {
                self.mix = value.clamp(0.0, 1.0);
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl LocalEffect for Phaser {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.lfo.set_frequency(self.rate, sample_rate);
        }

        for frame in buffer.iter_mut() {
            let lfo_value = self.lfo.next_sample();
            let mod_value = 0.5 + 0.5 * lfo_value;
            let feedback_value = self.feedback * (0.5 + 0.4 * mod_value);

            let mut processed_left = frame.left;
            let mut processed_right = frame.right;

            for i in 0..self.stages {
                self.apf_left[i].set_feedback(feedback_value);
                self.apf_right[i].set_feedback(feedback_value);
                
                processed_left = self.apf_left[i].process(processed_left);
                processed_right = self.apf_right[i].process(processed_right);
            }

            // Apply gain compensation based on number of stages
            let gain_compensation = 1.0 / (1.0 + (self.stages as f32 - 2.0) * 0.25);
            processed_left *= gain_compensation;
            processed_right *= gain_compensation;

            frame.left = frame.left * (1.0 - self.mix) + processed_left * self.mix;
            frame.right = frame.right * (1.0 - self.mix) + processed_right * self.mix;
        }
    }
}

impl ModuleMetadata for Phaser {
    fn get_static_name() -> &'static str {
        "phaser"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_phaser() -> Box<dyn LocalEffect> {
    Box::new(Phaser::new())
}