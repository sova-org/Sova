use crate::modules::{AudioModule, Frame, LocalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_CUTOFF: &str = "mooglpf";
const PARAM_RESONANCE: &str = "moogres";

const DEFAULT_CUTOFF: f32 = 1000.0;
const DEFAULT_RESONANCE: f32 = 0.0;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_CUTOFF,
        aliases: &["cutoff"],
        min_value: 20.0,
        max_value: 20000.0,
        default_value: DEFAULT_CUTOFF,
        unit: "Hz",
        description: "Filter cutoff frequency",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_RESONANCE,
        aliases: &["res", "resonance"],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_RESONANCE,
        unit: "",
        description: "Filter resonance",
        modulable: true,
    },
];

faust_macro::dsp!(
    declare name "moog_vcf";
    declare version "1.0";

    import("stdfaust.lib");

    process = _,_ : ve.moog_vcf(res, freq),ve.moog_vcf(res, freq)
    with {
        freq = hslider("freq", 1000, 20, 20000, 1);
        res = hslider("res", 0, 0, 1, 0.01);
    };
);

pub struct MoogVcfFilter {
    cutoff: f32,
    resonance: f32,
    faust_processor: moog_vcf::MoogVcf,
    sample_rate: f32,
    is_active: bool,
    left_input: [f32; 1024],
    right_input: [f32; 1024],
    left_output: [f32; 1024],
    right_output: [f32; 1024],
    params_dirty: bool,
}

impl Default for MoogVcfFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl MoogVcfFilter {
    pub fn new() -> Self {
        let mut faust_processor = moog_vcf::MoogVcf::new();
        faust_processor.init(44100);

        Self {
            cutoff: DEFAULT_CUTOFF,
            resonance: DEFAULT_RESONANCE,
            faust_processor,
            sample_rate: 44100.0,
            is_active: true,
            left_input: [0.0; 1024],
            right_input: [0.0; 1024],
            left_output: [0.0; 1024],
            right_output: [0.0; 1024],
            params_dirty: true,
        }
    }

    fn update_faust_params(&mut self) {
        self.faust_processor
            .set_param(faust_types::ParamIndex(0), self.cutoff);
        self.faust_processor
            .set_param(faust_types::ParamIndex(1), self.resonance);
    }
}

impl AudioModule for MoogVcfFilter {
    fn get_name(&self) -> &'static str {
        "mooglpf"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_CUTOFF => {
                let new_value = value.clamp(20.0, 20000.0);
                if self.cutoff != new_value {
                    self.cutoff = new_value;
                    self.params_dirty = true;
                }
                true
            }
            PARAM_RESONANCE => {
                let new_value = value.clamp(0.0, 1.0);
                if self.resonance != new_value {
                    self.resonance = new_value;
                    self.params_dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl LocalEffect for MoogVcfFilter {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.faust_processor.init(sample_rate as i32);
            self.params_dirty = true;
        }

        // Only update parameters if they've changed
        if self.params_dirty {
            self.update_faust_params();
            self.params_dirty = false;
        }

        let max_chunk_size = 512;
        let chunk_size = buffer.len().min(max_chunk_size);

        for chunk in buffer.chunks_mut(chunk_size) {
            let actual_chunk_size = chunk.len();

            for (i, frame) in chunk.iter().enumerate() {
                self.left_input[i] = frame.left;
                self.right_input[i] = frame.right;
                self.left_output[i] = 0.0;
                self.right_output[i] = 0.0;
            }

            let inputs = [
                &self.left_input[..actual_chunk_size],
                &self.right_input[..actual_chunk_size],
            ];
            let mut outputs = [
                &mut self.left_output[..actual_chunk_size],
                &mut self.right_output[..actual_chunk_size],
            ];

            self.faust_processor
                .compute(actual_chunk_size, &inputs, &mut outputs);

            for (i, frame) in chunk.iter_mut().enumerate() {
                frame.left = self.left_output[i];
                frame.right = self.right_output[i];
            }
        }
    }
}

impl ModuleMetadata for MoogVcfFilter {
    fn get_static_name() -> &'static str {
        "mooglpf"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_mooglpf_filter() -> Box<dyn LocalEffect> {
    Box::new(MoogVcfFilter::new())
}
