use crate::modules::{AudioModule, Frame, ParameterDescriptor, Source};

const PARAM_FREQUENCY: &str = "frequency";
const PARAM_Z1: &str = "z1";
const PARAM_SPARKLE: &str = "z2";
const PARAM_DRIFT: &str = "z3";
const PARAM_WOBBLE: &str = "z4";

const DEFAULT_FREQUENCY: f32 = 220.0;
const DEFAULT_Z1: f32 = 0.0;
const DEFAULT_SPARKLE: f32 = 0.0;
const DEFAULT_DRIFT: f32 = 0.0;
const DEFAULT_WOBBLE: f32 = 0.0;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_FREQUENCY,
        aliases: &["freq"],
        min_value: 20.0,
        max_value: 20000.0,
        default_value: DEFAULT_FREQUENCY,
        unit: "Hz",
        description: "Oscillator frequency",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_Z1,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_Z1,
        unit: "",
        description: "Wavefolder amount",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_SPARKLE,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_SPARKLE,
        unit: "",
        description: "High frequency sparkle",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_DRIFT,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_DRIFT,
        unit: "",
        description: "Slow frequency drift",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_WOBBLE,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_WOBBLE,
        unit: "",
        description: "Frequency wobble modulation",
        modulable: true,
    },
];

faust_macro::dsp!(
    declare name "triangle_oscillator";
    declare version "1.0";

    import("stdfaust.lib");

    freq = hslider("freq", 220, 20, 20000, 0.1);
    z1 = hslider("z1", 0.0, 0.0, 1.0, 0.01) : si.smoo;
    z2 = hslider("z2", 0.0, 0.0, 1.0, 0.01) : si.smoo;
    z3 = hslider("z3", 0.0, 0.0, 1.0, 0.01) : si.smoo;
    z4 = hslider("z4", 0.0, 0.0, 1.0, 0.01) : si.smoo;

    drift_noise = no.noise : fi.lowpass(1, 0.5) : *(z3 * freq * 0.05);
    wobble_lfo = os.lf_triangle(0.2 + z4 * 3.8) * z4 * freq * 0.01;
    
    modulated_freq = freq + drift_noise + wobble_lfo;
    
    triangle_osc = os.triangle(modulated_freq);

    wavefolder = _ : *(1 + z1 * 5) : wavefold_process
    with {
        wavefold_process = _ : max(-1) : min(1) : atan : *(2/ma.PI);
    };

    sparkle_harmonics = z2 * (
        (os.triangle(modulated_freq * 3) * 0.15) +
        (os.triangle(modulated_freq * 5) * 0.08) +
        (os.triangle(modulated_freq * 7) * 0.04)
    );

    process = triangle_osc : wavefolder : +(sparkle_harmonics);
);

pub struct TriangleOscillator {
    frequency: f32,
    z1: f32,
    sparkle: f32,
    drift: f32,
    wobble: f32,
    faust_processor: triangle_oscillator::TriangleOscillator,
    sample_rate: f32,
    is_active: bool,
    output: [f32; 1024],
}

impl Default for TriangleOscillator {
    fn default() -> Self {
        Self::new()
    }
}

impl TriangleOscillator {
    pub fn new() -> Self {
        let mut faust_processor = triangle_oscillator::TriangleOscillator::new();
        faust_processor.init(44100);

        Self {
            frequency: DEFAULT_FREQUENCY,
            z1: DEFAULT_Z1,
            sparkle: DEFAULT_SPARKLE,
            drift: DEFAULT_DRIFT,
            wobble: DEFAULT_WOBBLE,
            faust_processor,
            sample_rate: 44100.0,
            is_active: true,
            output: [0.0; 1024],
        }
    }

    fn update_faust_params(&mut self) {
        self.faust_processor
            .set_param(faust_types::ParamIndex(0), self.frequency);
        self.faust_processor
            .set_param(faust_types::ParamIndex(1), self.z1);
        self.faust_processor
            .set_param(faust_types::ParamIndex(2), self.sparkle);
        self.faust_processor
            .set_param(faust_types::ParamIndex(3), self.drift);
        self.faust_processor
            .set_param(faust_types::ParamIndex(4), self.wobble);
    }
}

impl AudioModule for TriangleOscillator {
    fn get_name(&self) -> &'static str {
        "triangle"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_FREQUENCY => {
                self.frequency = value.clamp(20.0, 20000.0);
                true
            }
            PARAM_Z1 => {
                self.z1 = value.clamp(0.0, 1.0);
                true
            }
            PARAM_SPARKLE => {
                self.sparkle = value.clamp(0.0, 1.0);
                true
            }
            PARAM_DRIFT => {
                self.drift = value.clamp(0.0, 1.0);
                true
            }
            PARAM_WOBBLE => {
                self.wobble = value.clamp(0.0, 1.0);
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl Source for TriangleOscillator {
    fn generate(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.faust_processor.init(sample_rate as i32);
            self.update_faust_params();
        }

        for chunk in buffer.chunks_mut(256) {
            let chunk_size = chunk.len();

            for i in 0..chunk_size {
                self.output[i] = 0.0;
            }

            self.update_faust_params();

            let inputs: [&[f32]; 0] = [];
            let mut outputs = [&mut self.output[..chunk_size]];

            self.faust_processor
                .compute(chunk_size, &inputs, &mut outputs);

            for (i, frame) in chunk.iter_mut().enumerate() {
                *frame = Frame::mono(self.output[i]);
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub fn create_triangle_oscillator() -> Box<dyn Source> {
    Box::new(TriangleOscillator::new())
}