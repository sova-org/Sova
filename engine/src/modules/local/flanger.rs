use crate::modules::{AudioModule, Frame, LocalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_LENGTH: &str = "flanger_length";
const PARAM_DELAY: &str = "flanger_delay";
const PARAM_DEPTH: &str = "flanger_depth";
const PARAM_FEEDBACK: &str = "flanger_feedback";

const DEFAULT_LENGTH: f32 = 0.08;
const DEFAULT_DELAY: f32 = 0.8;
const DEFAULT_DEPTH: f32 = 0.7;
const DEFAULT_FEEDBACK: f32 = 0.4;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_LENGTH,
        aliases: &["fl"],
        min_value: 0.002,
        max_value: 0.1,
        default_value: DEFAULT_LENGTH,
        unit: "s",
        description: "Maximum delay line length",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_DELAY,
        aliases: &["fd"],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_DELAY,
        unit: "",
        description: "Delay modulation control",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_DEPTH,
        aliases: &["fdepth"],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_DEPTH,
        unit: "",
        description: "Effect depth/mix",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_FEEDBACK,
        aliases: &["ffb"],
        min_value: 0.0,
        max_value: 0.95,
        default_value: DEFAULT_FEEDBACK,
        unit: "",
        description: "Feedback amount",
        modulable: true,
    },
];

faust_macro::dsp!(
    declare name "flanger_stereo";
    declare version "1.0";

    import("stdfaust.lib");

    process = _,_ : pf.flanger_stereo(dmax, curdel1, curdel2, depth, fb, 0)
    with {
        dmax = hslider("length", 0.08, 0.002, 0.1, 0.001) * ma.SR : int;
        base_delay = hslider("delay", 0.8, 0.0, 1.0, 0.01);
        depth = hslider("depth", 0.7, 0.0, 1.0, 0.01);
        fb = hslider("feedback", 0.4, 0.0, 0.95, 0.01);

        lfo_freq = 0.5 + base_delay * 2.0;
        lfo_phase_offset = ma.PI * 0.5;

        lfo1 = os.osc(lfo_freq);
        lfo2 = os.osc(lfo_freq + 0.1) * sin(lfo_phase_offset);

        delay_range = dmax * 0.8;
        center_delay = dmax * 0.2;

        curdel1 = center_delay + lfo1 * delay_range * base_delay;
        curdel2 = center_delay + lfo2 * delay_range * base_delay;
    };
);

pub struct Flanger {
    length: f32,
    delay: f32,
    depth: f32,
    feedback: f32,
    faust_processor: flanger_stereo::FlangerStereo,
    sample_rate: f32,
    is_active: bool,
    left_input: [f32; 1024],
    right_input: [f32; 1024],
    left_output: [f32; 1024],
    right_output: [f32; 1024],
}

impl Default for Flanger {
    fn default() -> Self {
        Self::new()
    }
}

impl Flanger {
    pub fn new() -> Self {
        let mut faust_processor = flanger_stereo::FlangerStereo::new();
        faust_processor.init(44100);

        Self {
            length: DEFAULT_LENGTH,
            delay: DEFAULT_DELAY,
            depth: DEFAULT_DEPTH,
            feedback: DEFAULT_FEEDBACK,
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
            .set_param(faust_types::ParamIndex(0), self.length);
        self.faust_processor
            .set_param(faust_types::ParamIndex(1), self.delay);
        self.faust_processor
            .set_param(faust_types::ParamIndex(2), self.depth);
        self.faust_processor
            .set_param(faust_types::ParamIndex(3), self.feedback);
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
            PARAM_LENGTH => {
                self.length = value.clamp(0.002, 0.1);
                self.update_faust_params();
                true
            }
            PARAM_DELAY => {
                self.delay = value.clamp(0.0, 1.0);
                self.update_faust_params();
                true
            }
            PARAM_DEPTH => {
                self.depth = value.clamp(0.0, 1.0);
                self.update_faust_params();
                true
            }
            PARAM_FEEDBACK => {
                self.feedback = value.clamp(0.0, 0.95);
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

impl LocalEffect for Flanger {
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
