use crate::dsp::interpolating_delay::InterpolatingDelay;
use crate::dsp::oscillators::TableLfo;
use crate::modules::{AudioModule, Frame, LocalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_DEPTH: &str = "fdepth";
const PARAM_RATE: &str = "frate";
const PARAM_FEEDBACK: &str = "ffeedback";
const PARAM_MIX: &str = "fmix";

const DEFAULT_DEPTH: f32 = 0.003;
const DEFAULT_RATE: f32 = 0.5;
const DEFAULT_FEEDBACK: f32 = 0.5;
const DEFAULT_MIX: f32 = 0.5;

const MAX_DELAY_SAMPLES: usize = 1024;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_DEPTH,
        aliases: &[],
        min_value: 0.0,
        max_value: 0.01,
        default_value: DEFAULT_DEPTH,
        unit: "s",
        description: "Flanger depth",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_RATE,
        aliases: &[],
        min_value: 0.01,
        max_value: 10.0,
        default_value: DEFAULT_RATE,
        unit: "Hz",
        description: "Flanger rate",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_FEEDBACK,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_FEEDBACK,
        unit: "",
        description: "Flanger feedback",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_MIX,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_MIX,
        unit: "",
        description: "Flanger mix",
        modulable: true,
    },
];

pub struct Flanger {
    depth: f32,
    rate: f32,
    feedback: f32,
    mix: f32,
    lfo: TableLfo,
    delay_left: InterpolatingDelay<MAX_DELAY_SAMPLES>,
    delay_right: InterpolatingDelay<MAX_DELAY_SAMPLES>,
    sample_rate: f32,
    is_active: bool,
}

impl Default for Flanger {
    fn default() -> Self {
        Self::new()
    }
}

impl Flanger {
    pub fn new() -> Self {
        let mut lfo = TableLfo::new();
        lfo.set_frequency(DEFAULT_RATE, 44100.0);

        Self {
            depth: DEFAULT_DEPTH,
            rate: DEFAULT_RATE,
            feedback: DEFAULT_FEEDBACK,
            mix: DEFAULT_MIX,
            lfo,
            delay_left: InterpolatingDelay::new(),
            delay_right: InterpolatingDelay::new(),
            sample_rate: 44100.0,
            is_active: true,
        }
    }
}

impl AudioModule for Flanger {
    fn get_name(&self) -> &'static str {
        "flanger"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_DEPTH => {
                self.depth = value.clamp(0.0, 0.01);
                true
            }
            PARAM_RATE => {
                self.rate = value.clamp(0.01, 10.0);
                self.lfo.set_frequency(self.rate, self.sample_rate);
                true
            }
            PARAM_FEEDBACK => {
                self.feedback = value.clamp(0.0, 1.0) * 0.4;
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

impl LocalEffect for Flanger {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.lfo.set_frequency(self.rate, sample_rate);
        }

        let depth_samples = self.depth * sample_rate;
        let base_delay = depth_samples + 1.0;

        for frame in buffer.iter_mut() {
            let lfo_value = self.lfo.next_sample();
            let delay_time = base_delay + depth_samples * lfo_value;

            let delayed_left = self.delay_left.read_interpolated(delay_time);
            let delayed_right = self.delay_right.read_interpolated(delay_time);

            let input_left = frame.left + delayed_left * self.feedback;
            let input_right = frame.right + delayed_right * self.feedback;

            self.delay_left.write(input_left);
            self.delay_right.write(input_right);

            frame.left = frame.left * (1.0 - self.mix) + delayed_left * self.mix;
            frame.right = frame.right * (1.0 - self.mix) + delayed_right * self.mix;
        }
    }
}

impl ModuleMetadata for Flanger {
    fn get_static_name() -> &'static str {
        "flanger"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_flanger() -> Box<dyn LocalEffect> {
    Box::new(Flanger::new())
}