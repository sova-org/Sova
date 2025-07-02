use crate::audio_tools::midi;
use crate::constants::FM_AMPLITUDE_CALIBRATION;
use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};

const PARAM_FREQUENCY: &str = "frequency";
const PARAM_NOTE: &str = "note";
const PARAM_Z1: &str = "z1";
const PARAM_Z2: &str = "z2";
const PARAM_Z3: &str = "z3";
const PARAM_Z4: &str = "z4";

const DEFAULT_FREQUENCY: f32 = 440.0;
const DEFAULT_Z1: f32 = 0.0;
const DEFAULT_Z2: f32 = 0.0;
const DEFAULT_Z3: f32 = 0.0;
const DEFAULT_Z4: f32 = 0.0;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_FREQUENCY,
        aliases: &["freq"],
        min_value: 20.0,
        max_value: 20000.0,
        default_value: DEFAULT_FREQUENCY,
        unit: "Hz",
        description: "Carrier frequency",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_NOTE,
        aliases: &["n", "midi"],
        min_value: 0.0,
        max_value: 127.0,
        default_value: 69.0,
        unit: "MIDI",
        description: "MIDI note number (takes precedence over frequency)",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_Z1,
        aliases: &[],
        min_value: 0.0,
        max_value: 20.0,
        default_value: DEFAULT_Z1,
        unit: "",
        description: "Modulation index",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_Z2,
        aliases: &[],
        min_value: 0.0,
        max_value: 10.0,
        default_value: DEFAULT_Z2,
        unit: "",
        description: "Modulator ratio",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_Z3,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_Z3,
        unit: "",
        description: "Carrier detune",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_Z4,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_Z4,
        unit: "",
        description: "Feedback amount",
        modulable: true,
    },
];

faust_macro::dsp!(
    declare name "fm_oscillator";
    declare version "1.0";

    import("stdfaust.lib");

    freq = hslider("freq", 440, 20, 20000, 0.1);
    z1 = hslider("z1", 0.0, 0.0, 20.0, 0.01) : si.smoo;
    z2 = hslider("z2", 0.0, 0.0, 10.0, 0.01) : si.smoo;
    z3 = hslider("z3", 0.0, 0.0, 1.0, 0.01) : si.smoo;
    z4 = hslider("z4", 0.0, 0.0, 1.0, 0.01) : si.smoo;

    // Modulation index
    mod_index = z1;

    // Modulator frequency ratio (defaults to 1:1 when z2=0)
    mod_ratio = 1.0 + z2;
    mod_freq = freq * mod_ratio;

    // Feedback amount for self-modulation (map 0-1 to 0-0.6 range)
    feedback_amount = z4 * 0.6;

    // Carrier detune amount (-50 to +50 cents)
    detune_amount = (z3 - 0.5) * 0.1;

    // Detuned carrier frequency
    carrier_freq = freq * (1.0 + detune_amount);

    // Modulator oscillator
    modulator = os.osc(mod_freq) * mod_index * carrier_freq * 2 * ma.PI;

    // Feedback delay for self-modulation
    feedback_delay = _ <: _, mem : + : *(feedback_amount);

    // Carrier with modulation and feedback
    fm_core = modulator + feedback_delay ~ _ : os.osc(carrier_freq + _);

    // Output gain compensation based on modulation index
    output_gain = 1.0 / (1.0 + mod_index * 0.1);

    // Basic FM signal
    fm_signal = fm_core * output_gain;

    // Secondary detuned oscillator for stereo width
    detune_osc = (os.osc(mod_freq) * mod_index * (carrier_freq * 1.005) * 2 * ma.PI) :
                 os.osc((carrier_freq * 1.005) + _) * output_gain;

    // Stereo processing with detuned oscillator
    process = fm_signal <: left_channel, right_channel
    with {
        left_channel = _;
        right_channel = _ * 0.8 + detune_osc * 0.2;
    };
);

pub struct FmOscillator {
    frequency: f32,
    note: Option<f32>,
    z1: f32,
    z2: f32,
    z3: f32,
    z4: f32,
    faust_processor: fm_oscillator::FmOscillator,
    sample_rate: f32,
    is_active: bool,
    output: [f32; 2048],
}

impl Default for FmOscillator {
    fn default() -> Self {
        Self::new()
    }
}

impl FmOscillator {
    pub fn new() -> Self {
        let mut faust_processor = fm_oscillator::FmOscillator::new();
        faust_processor.init(44100);

        Self {
            frequency: DEFAULT_FREQUENCY,
            note: None,
            z1: DEFAULT_Z1,
            z2: DEFAULT_Z2,
            z3: DEFAULT_Z3,
            z4: DEFAULT_Z4,
            faust_processor,
            sample_rate: 44100.0,
            is_active: true,
            output: [0.0; 2048],
        }
    }

    fn update_faust_params(&mut self) {
        let effective_frequency = self
            .note
            .map(midi::note_to_frequency)
            .unwrap_or(self.frequency);

        self.faust_processor
            .set_param(faust_types::ParamIndex(0), effective_frequency);
        self.faust_processor
            .set_param(faust_types::ParamIndex(1), self.z1);
        self.faust_processor
            .set_param(faust_types::ParamIndex(2), self.z2);
        self.faust_processor
            .set_param(faust_types::ParamIndex(3), self.z3);
        self.faust_processor
            .set_param(faust_types::ParamIndex(4), self.z4);
    }
}

impl AudioModule for FmOscillator {
    fn get_name(&self) -> &'static str {
        "fm"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_FREQUENCY => {
                let new_value = value.clamp(20.0, 20000.0);
                if self.frequency != new_value {
                    self.frequency = new_value;
                    self.note = None;
                }
                true
            }
            PARAM_NOTE => {
                let new_value = value.clamp(0.0, 127.0);
                if self.note != Some(new_value) {
                    self.note = Some(new_value);
                }
                true
            }
            PARAM_Z1 => {
                self.z1 = value.clamp(0.0, 20.0);
                true
            }
            PARAM_Z2 => {
                self.z2 = value.clamp(0.0, 10.0);
                true
            }
            PARAM_Z3 => {
                self.z3 = value.clamp(0.0, 1.0);
                true
            }
            PARAM_Z4 => {
                self.z4 = value.clamp(0.0, 1.0);
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl Source for FmOscillator {
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
                *frame = Frame::new(
                    left_out[i] * FM_AMPLITUDE_CALIBRATION,
                    right_out[i] * FM_AMPLITUDE_CALIBRATION,
                );
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ModuleMetadata for FmOscillator {
    fn get_static_name() -> &'static str {
        "fm"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_fm_oscillator() -> Box<dyn Source> {
    Box::new(FmOscillator::new())
}
