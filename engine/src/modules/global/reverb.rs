use crate::modules::{AudioModule, Frame, GlobalEffect, ParameterDescriptor};

const PARAM_SIZE: &str = "size";
const PARAM_DAMPING: &str = "damping";

const DEFAULT_SIZE: f32 = 0.5;
const DEFAULT_DAMPING: f32 = 0.5;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_SIZE,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_SIZE,
        unit: "",
        description: "Reverb size",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_DAMPING,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_DAMPING,
        unit: "",
        description: "High frequency damping",
        modulable: true,
    },
];

faust_macro::dsp!(
    declare name "zita_reverb";
    declare version "1.0";
    import("stdfaust.lib");
    process = _ <: re.zita_rev_fdn(f1, f2, t60dc, t60m, fsmax) :> _,_
    with {
        f1 = hslider("f1", 200, 50, 1000, 1);
        f2 = hslider("f2", 6000, 1500, 20000, 1);
        t60dc = hslider("t60dc", 3, 1, 8, 0.1);
        t60m = hslider("t60m", 2, 1, 8, 0.1);
        fsmax = 48000*2;
    };
);

pub struct ZitaReverb {
    size: f32,
    damping: f32,
    faust_processor: zita_reverb::ZitaReverb,
    sample_rate: f32,
    is_active: bool,
    left_buffer: [f32; 1024],
    left_output: [f32; 1024],
    right_output: [f32; 1024],
}

impl Default for ZitaReverb {
    fn default() -> Self {
        Self::new()
    }
}

impl ZitaReverb {
    pub fn new() -> Self {
        let mut faust_processor = zita_reverb::ZitaReverb::new();
        faust_processor.init(44100);

        Self {
            size: DEFAULT_SIZE,
            damping: DEFAULT_DAMPING,
            faust_processor,
            sample_rate: 44100.0,
            is_active: true,
            left_buffer: [0.0; 1024],
            left_output: [0.0; 1024],
            right_output: [0.0; 1024],
        }
    }

    fn update_faust_params(&mut self) {
        let f1 = 50.0 + self.size * 950.0;
        let f2 = 1500.0 + self.size * 18500.0;
        let t60dc = 1.0 + self.size * 7.0;
        let t60m = 1.0 + self.size * 7.0 * self.damping;

        self.faust_processor
            .set_param(faust_types::ParamIndex(0), f1);
        self.faust_processor
            .set_param(faust_types::ParamIndex(1), f2);
        self.faust_processor
            .set_param(faust_types::ParamIndex(2), t60dc);
        self.faust_processor
            .set_param(faust_types::ParamIndex(3), t60m);
    }
}

impl AudioModule for ZitaReverb {
    fn get_name(&self) -> &'static str {
        "reverb"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_SIZE => {
                self.size = value.clamp(0.0, 1.0);
                self.update_faust_params();
                true
            }
            PARAM_DAMPING => {
                self.damping = value.clamp(0.0, 1.0);
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

impl GlobalEffect for ZitaReverb {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.faust_processor.init(sample_rate as i32);
            self.update_faust_params();
        }

        for chunk in buffer.chunks_mut(256) {
            let chunk_size = chunk.len();

            for (i, frame) in chunk.iter().enumerate() {
                self.left_buffer[i] = (frame.left + frame.right) * 0.5;
                self.left_output[i] = 0.0;
                self.right_output[i] = 0.0;
            }

            let inputs = [&self.left_buffer[..chunk_size]];
            let mut outputs = [
                &mut self.left_output[..chunk_size],
                &mut self.right_output[..chunk_size],
            ];

            self.faust_processor
                .compute(chunk_size, &inputs, &mut outputs);

            for (i, frame) in chunk.iter_mut().enumerate() {
                frame.left = self.left_output[i] * 0.1;
                frame.right = self.right_output[i] * 0.1;
            }
        }
    }
}

pub fn create_simple_reverb() -> Box<dyn GlobalEffect> {
    Box::new(ZitaReverb::new())
}
