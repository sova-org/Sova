use crate::modules::{AudioModule, Frame, ParameterDescriptor, Source};

const PARAM_FREQUENCY: &str = "frequency";
const PARAM_FOLD: &str = "z1";
const PARAM_HARMONICS: &str = "z2";
const PARAM_DRIFT: &str = "z3";
const PARAM_DRIFT_FREQ: &str = "z4";

const DEFAULT_FREQUENCY: f32 = 440.0;
const DEFAULT_FOLD: f32 = 0.0;
const DEFAULT_HARMONICS: f32 = 0.0;
const DEFAULT_DRIFT: f32 = 0.0;
const DEFAULT_DRIFT_FREQ: f32 = 0.3;

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
        name: PARAM_FOLD,
        aliases: &[],
        min_value: 0.0,
        max_value: 10.0,
        default_value: DEFAULT_FOLD,
        unit: "",
        description: "Wavefolder amount",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_HARMONICS,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_HARMONICS,
        unit: "",
        description: "Harmonics amount",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_DRIFT,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_DRIFT,
        unit: "",
        description: "Frequency drift amount",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_DRIFT_FREQ,
        aliases: &[],
        min_value: 0.01,
        max_value: 5.0,
        default_value: DEFAULT_DRIFT_FREQ,
        unit: "Hz",
        description: "Frequency drift rate",
        modulable: true,
    },
];

faust_macro::dsp!(
    declare name "sine_oscillator";
    declare version "1.0";

    import("stdfaust.lib");

    process = os.osc(freq_modulated) : wavefolding : harmonics_mix
    with {
        freq = hslider("frequency", 440, 20, 20000, 1);
        fold = hslider("sinfold", 0, 0, 10, 0.01);
        harm = hslider("sinharm", 0, 0, 1, 0.01);
        drift = hslider("sindrift", 0, 0, 1, 0.01);
        drift_freq = hslider("sindriftfreq", 0.3, 0.01, 5, 0.01);

        drift_lfo = os.osc(drift_freq) * drift * freq * 0.005;
        freq_modulated = freq + drift_lfo;

        wavefolding = _ : *(1 + fold) : fold_process
        with {
            fold_process = _ : max(-1) : min(1);
        };

        harmonics_mix = _ <: base, harmonics : +
        with {
            base = _ * (1 - harm);
            harmonics = harm * (
                (os.osc((freq_modulated * 0.5) + (drift_lfo * 0.7)) * 0.5) +
                (os.osc((freq_modulated * 2) + (drift_lfo * 1.3)) * 0.25) +
                (os.osc((freq_modulated * 4) + (drift_lfo * 0.9)) * 0.125)
            );
        };
    };
);

pub struct SineOscillator {
    frequency: f32,
    fold: f32,
    harmonics: f32,
    drift: f32,
    drift_freq: f32,
    faust_processor: sine_oscillator::SineOscillator,
    sample_rate: f32,
    is_active: bool,
    output: [f32; 1024],
}

impl Default for SineOscillator {
    fn default() -> Self {
        Self::new()
    }
}

impl SineOscillator {
    pub fn new() -> Self {
        let mut faust_processor = sine_oscillator::SineOscillator::new();
        faust_processor.init(44100);

        Self {
            frequency: DEFAULT_FREQUENCY,
            fold: DEFAULT_FOLD,
            harmonics: DEFAULT_HARMONICS,
            drift: DEFAULT_DRIFT,
            drift_freq: DEFAULT_DRIFT_FREQ,
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
            .set_param(faust_types::ParamIndex(1), self.fold);
        self.faust_processor
            .set_param(faust_types::ParamIndex(2), self.harmonics);
        self.faust_processor
            .set_param(faust_types::ParamIndex(3), self.drift);
        self.faust_processor
            .set_param(faust_types::ParamIndex(4), self.drift_freq);
    }
}

impl AudioModule for SineOscillator {
    fn get_name(&self) -> &'static str {
        "sine"
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
            PARAM_FOLD => {
                self.fold = value.clamp(0.0, 10.0);
                true
            }
            PARAM_HARMONICS => {
                self.harmonics = value.clamp(0.0, 1.0);
                true
            }
            PARAM_DRIFT => {
                self.drift = value.clamp(0.0, 1.0);
                true
            }
            PARAM_DRIFT_FREQ => {
                self.drift_freq = value.clamp(0.01, 5.0);
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl Source for SineOscillator {
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

pub fn create_sine_oscillator() -> Box<dyn Source> {
    Box::new(SineOscillator::new())
}
