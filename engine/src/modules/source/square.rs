use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};

const PARAM_FREQUENCY: &str = "frequency";
const PARAM_Z1: &str = "z1";
const PARAM_Z2: &str = "z2";
const PARAM_Z3: &str = "z3";

const DEFAULT_FREQUENCY: f32 = 100.0;
const DEFAULT_Z1: f32 = 0.2;
const DEFAULT_Z2: f32 = 0.4;
const DEFAULT_Z3: f32 = 0.1;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_FREQUENCY,
        aliases: &["freq"],
        min_value: 20.0,
        max_value: 2000.0,
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
        description: "Parameter z1",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_Z2,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_Z2,
        unit: "",
        description: "Parameter z2",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_Z3,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_Z3,
        unit: "",
        description: "Parameter z3",
        modulable: true,
    },
];

faust_macro::dsp!(
    declare name "square_oscillator";
    declare version "1.0";

    import("stdfaust.lib");

    // Playful controls
    freq = hslider("freq", 100, 20, 2000, 0.1);
    z1 = hslider("z1", 0.2, 0.0, 1.0, 0.01) : si.smoo;
    z2 = hslider("z2", 0.4, 0.0, 1.0, 0.01) : si.smoo;
    z3 = hslider("z3", 0.1, 0.0, 1.0, 0.01) : si.smoo;

    // Internal parameter mapping
    base_freq = freq;
    detune_amount = z3 * 0.05; // 0% to 5% detune

    // PWM modulation
    pwm_rate = 0.1 + z1 * 2.0; // 0.1Hz to 2.1Hz
    pwm_depth = z1 * 0.3; // 0% to 30% depth
    duty_base = 0.5;
    pwm_lfo = os.lf_triangle(pwm_rate) * pwm_depth;
    modulated_duty = (duty_base + pwm_lfo) : max(0.1) : min(0.9);

    // Echo delay modulation
    delay_time = 50 + z2 * 200; // 50 to 250 samples
    delay_lfo_rate = 0.08 + z3 * 0.3; // Slightly different rate
    delay_lfo = os.lf_triangle(delay_lfo_rate) * z2 * 30;
    modulated_delay = (delay_time + delay_lfo) : max(1) : min(400);

    // Two detuned oscillators with DC blocking
    osc1 = os.pulsetrain(base_freq, modulated_duty) : fi.dcblocker;
    osc2 = os.pulsetrain(base_freq * (1.0 + detune_amount), modulated_duty) : fi.dcblocker;

    // Echo mix amount
    echo_mix = z2 * 0.6; // 0% to 60% wet signal

    // Process with stereo delay
    process = (osc1 + osc2) * 0.25 <: left_channel, right_channel
    with {
        left_channel = _ <: (_ * (1.0 - echo_mix)), (de.delay(512, modulated_delay) * echo_mix) :> _;
        right_channel = _ <: (_ * (1.0 - echo_mix)), (de.delay(512, modulated_delay) * echo_mix) :> _;
    };
);

pub struct SquareOscillator {
    frequency: f32,
    z1: f32,
    z2: f32,
    z3: f32,
    faust_processor: square_oscillator::SquareOscillator,
    sample_rate: f32,
    is_active: bool,
    output: [f32; 2048],
}

impl Default for SquareOscillator {
    fn default() -> Self {
        Self::new()
    }
}

impl SquareOscillator {
    pub fn new() -> Self {
        let mut faust_processor = square_oscillator::SquareOscillator::new();
        faust_processor.init(44100);

        Self {
            frequency: DEFAULT_FREQUENCY,
            z1: DEFAULT_Z1,
            z2: DEFAULT_Z2,
            z3: DEFAULT_Z3,
            faust_processor,
            sample_rate: 44100.0,
            is_active: true,
            output: [0.0; 2048],
        }
    }

    fn update_faust_params(&mut self) {
        self.faust_processor
            .set_param(faust_types::ParamIndex(0), self.frequency);
        self.faust_processor
            .set_param(faust_types::ParamIndex(1), self.z1);
        self.faust_processor
            .set_param(faust_types::ParamIndex(2), self.z2);
        self.faust_processor
            .set_param(faust_types::ParamIndex(3), self.z3);
    }
}

impl AudioModule for SquareOscillator {
    fn get_name(&self) -> &'static str {
        "square"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_FREQUENCY => {
                self.frequency = value.clamp(20.0, 2000.0);
                true
            }
            PARAM_Z1 => {
                self.z1 = value.clamp(0.0, 1.0);
                true
            }
            PARAM_Z2 => {
                self.z2 = value.clamp(0.0, 1.0);
                true
            }
            PARAM_Z3 => {
                self.z3 = value.clamp(0.0, 1.0);
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl Source for SquareOscillator {
    fn generate(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.faust_processor.init(sample_rate as i32);
            self.update_faust_params();
        }

        for chunk in buffer.chunks_mut(256) {
            let chunk_size = chunk.len();

            for i in 0..chunk_size * 2 {
                self.output[i] = 0.0;
            }

            self.update_faust_params();

            let inputs: [&[f32]; 0] = [];
            let (left_out, right_out) = self.output.split_at_mut(chunk_size);
            let mut outputs = [&mut left_out[..chunk_size], &mut right_out[..chunk_size]];

            self.faust_processor
                .compute(chunk_size, &inputs, &mut outputs);

            for (i, frame) in chunk.iter_mut().enumerate() {
                *frame = Frame::new(left_out[i], right_out[i]);
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ModuleMetadata for SquareOscillator {
    fn get_static_name() -> &'static str {
        "square"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_square_oscillator() -> Box<dyn Source> {
    Box::new(SquareOscillator::new())
}
