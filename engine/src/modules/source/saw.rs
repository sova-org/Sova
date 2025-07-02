use crate::audio_tools::midi;
use crate::constants::SAW_AMPLITUDE_CALIBRATION;
use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};

const PARAM_FREQUENCY: &str = "frequency";
const PARAM_NOTE: &str = "note";
const PARAM_Z1: &str = "z1";
const PARAM_Z2: &str = "z2";
const PARAM_Z3: &str = "z3";
const PARAM_Z4: &str = "z4";

const DEFAULT_FREQUENCY: f32 = 220.0;
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
        description: "Oscillator frequency",
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
        max_value: 1.0,
        default_value: DEFAULT_Z1,
        unit: "",
        description: "Supersaw voices (1-7)",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_Z2,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_Z2,
        unit: "",
        description: "Detune spread",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_Z3,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_Z3,
        unit: "",
        description: "Octave layering",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_Z4,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_Z4,
        unit: "",
        description: "Phase spread",
        modulable: true,
    },
];

faust_macro::dsp!(
    declare name "saw_oscillator";
    declare version "1.0";

    import("stdfaust.lib");

    freq = hslider("freq", 220, 20, 20000, 0.1);
    z1 = hslider("z1", 0.0, 0.0, 1.0, 0.01) : si.smoo;
    z2 = hslider("z2", 0.0, 0.0, 1.0, 0.01) : si.smoo;
    z3 = hslider("z3", 0.0, 0.0, 1.0, 0.01) : si.smoo;
    z4 = hslider("z4", 0.0, 0.0, 1.0, 0.01) : si.smoo;

    // Map z1 to number of voices (1-7)
    num_voices = 1 + z1 * 6;

    // Detune spread - maximum 50 cents
    detune_spread = z2 * 0.05;

    // Octave layering mix
    octave_mix = z3;

    // Phase spread for stereo width
    phase_spread = z4 * ma.PI;

    // Basic saw oscillator
    saw_osc(f) = os.sawtooth(f);

    // Supersaw generator with variable voices
    supersaw = saw_main + saw_voices : *(1.0 / sqrt(num_voices))
    with {
        saw_main = saw_osc(freq);

        // Detuned voices with phase offset
        saw_voices = voice1 + voice2 + voice3 + voice4 + voice5 + voice6
        with {
            voice1 = select2(num_voices > 1.5, 0, saw_osc(freq * (1 + detune_spread * 0.2)));
            voice2 = select2(num_voices > 2.5, 0, saw_osc(freq * (1 - detune_spread * 0.15)));
            voice3 = select2(num_voices > 3.5, 0, saw_osc(freq * (1 + detune_spread * 0.35)));
            voice4 = select2(num_voices > 4.5, 0, saw_osc(freq * (1 - detune_spread * 0.3)));
            voice5 = select2(num_voices > 5.5, 0, saw_osc(freq * (1 + detune_spread * 0.5)));
            voice6 = select2(num_voices > 6.5, 0, saw_osc(freq * (1 - detune_spread * 0.45)));
        };
    };

    // Octave layering
    octave_layer = octave_mix * (
        (saw_osc(freq * 0.5) * 0.6) +     // Sub-octave
        (saw_osc(freq * 2.0) * 0.3) +     // Higher octave
        (saw_osc(freq * 4.0) * 0.15)      // Higher octave
    );

    // Stereo processing with phase spread
    process = (supersaw + octave_layer) <: left_channel, right_channel
    with {
        left_channel = _;
        right_channel = _ : *(cos(phase_spread)) + (saw_osc(freq * (1 + detune_spread * 0.1)) * sin(phase_spread) * 0.3);
    };
);

pub struct SawOscillator {
    frequency: f32,
    note: Option<f32>,
    z1: f32,
    z2: f32,
    z3: f32,
    z4: f32,
    faust_processor: saw_oscillator::SawOscillator,
    sample_rate: f32,
    is_active: bool,
    output: [f32; 2048],
    params_dirty: bool,
}

impl Default for SawOscillator {
    fn default() -> Self {
        Self::new()
    }
}

impl SawOscillator {
    pub fn new() -> Self {
        let mut faust_processor = saw_oscillator::SawOscillator::new();
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
            params_dirty: true,
        }
    }

    fn update_faust_params(&mut self) {
        let effective_frequency = self
            .note
            .map(|note| midi::note_to_frequency(note))
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

impl AudioModule for SawOscillator {
    fn get_name(&self) -> &'static str {
        "saw"
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
                    self.params_dirty = true;
                }
                true
            }
            PARAM_NOTE => {
                let new_value = value.clamp(0.0, 127.0);
                if self.note != Some(new_value) {
                    self.note = Some(new_value);
                    self.params_dirty = true;
                }
                true
            }
            PARAM_Z1 => {
                let new_value = value.clamp(0.0, 1.0);
                if self.z1 != new_value {
                    self.z1 = new_value;
                    self.params_dirty = true;
                }
                true
            }
            PARAM_Z2 => {
                let new_value = value.clamp(0.0, 1.0);
                if self.z2 != new_value {
                    self.z2 = new_value;
                    self.params_dirty = true;
                }
                true
            }
            PARAM_Z3 => {
                let new_value = value.clamp(0.0, 1.0);
                if self.z3 != new_value {
                    self.z3 = new_value;
                    self.params_dirty = true;
                }
                true
            }
            PARAM_Z4 => {
                let new_value = value.clamp(0.0, 1.0);
                if self.z4 != new_value {
                    self.z4 = new_value;
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

impl Source for SawOscillator {
    fn generate(&mut self, buffer: &mut [Frame], sample_rate: f32) {
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

        for chunk in buffer.chunks_mut(256) {
            let chunk_size = chunk.len();

            for i in 0..chunk_size * 2 {
                self.output[i] = 0.0;
            }

            let inputs: [&[f32]; 0] = [];
            let (left_out, right_out) = self.output.split_at_mut(chunk_size);
            let mut outputs = [&mut left_out[..chunk_size], &mut right_out[..chunk_size]];

            self.faust_processor
                .compute(chunk_size, &inputs, &mut outputs);

            for (i, frame) in chunk.iter_mut().enumerate() {
                *frame = Frame::new(
                    left_out[i] * SAW_AMPLITUDE_CALIBRATION,
                    right_out[i] * SAW_AMPLITUDE_CALIBRATION,
                );
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ModuleMetadata for SawOscillator {
    fn get_static_name() -> &'static str {
        "saw"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_saw_oscillator() -> Box<dyn Source> {
    Box::new(SawOscillator::new())
}
