use crate::modules::{AudioModule, Frame, LocalEffect, ParameterDescriptor};

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

faust_macro::dsp!(
    declare name "ring_modulator";
    declare version "1.0";

    import("stdfaust.lib");

    process = _,_ : ringmod,ringmod
    with {
        freq = hslider("freq", 5, 0.01, 1000, 0.01) : si.smoo;
        depth = hslider("depth", 0, 0, 1, 0.01) : si.smoo;
        carrier = os.osc(freq);
        ringmod = _ * (1 - depth + depth * carrier);
    };
);

pub struct RingModulator {
    frequency: f32,
    depth: f32,
    faust_processor: ring_modulator::RingModulator,
    sample_rate: f32,
    is_active: bool,
    left_input: [f32; 1024],
    right_input: [f32; 1024],
    left_output: [f32; 1024],
    right_output: [f32; 1024],
}

impl Default for RingModulator {
    fn default() -> Self {
        Self::new()
    }
}

impl RingModulator {
    pub fn new() -> Self {
        let mut faust_processor = ring_modulator::RingModulator::new();
        faust_processor.init(44100);

        Self {
            frequency: DEFAULT_FREQUENCY,
            depth: DEFAULT_DEPTH,
            faust_processor,
            sample_rate: 44100.0,
            is_active: true,
            left_input: [0.0; 1024],
            right_input: [0.0; 1024],
            left_output: [0.0; 1024],
            right_output: [0.0; 1024],
        }
    }

    fn update_faust_params(&mut self) {
        self.faust_processor
            .set_param(faust_types::ParamIndex(0), self.frequency);
        self.faust_processor
            .set_param(faust_types::ParamIndex(1), self.depth);
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
                self.update_faust_params();
                true
            }
            PARAM_DEPTH => {
                self.depth = value.clamp(0.0, 1.0);
                self.update_faust_params();
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
            self.faust_processor.init(sample_rate as i32);
            self.update_faust_params();
        }

        for chunk in buffer.chunks_mut(256) {
            let chunk_size = chunk.len();

            for (i, frame) in chunk.iter().enumerate() {
                self.left_input[i] = frame.left;
                self.right_input[i] = frame.right;
                self.left_output[i] = 0.0;
                self.right_output[i] = 0.0;
            }

            let inputs = [
                &self.left_input[..chunk_size],
                &self.right_input[..chunk_size],
            ];
            let mut outputs = [
                &mut self.left_output[..chunk_size],
                &mut self.right_output[..chunk_size],
            ];

            self.faust_processor
                .compute(chunk_size, &inputs, &mut outputs);

            for (i, frame) in chunk.iter_mut().enumerate() {
                frame.left = self.left_output[i];
                frame.right = self.right_output[i];
            }
        }
    }
}

pub fn create_ring_modulator() -> Box<dyn LocalEffect> {
    Box::new(RingModulator::new())
}