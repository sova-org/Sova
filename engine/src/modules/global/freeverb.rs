use crate::dsp::{all_pass_filter::AllPassFilter, comb_filter::CombFilter};
use crate::modules::{AudioModule, Frame, GlobalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_SIZE: &str = "size";
const PARAM_WIDTH: &str = "width";
const PARAM_FREEZE: &str = "freeze";

const DEFAULT_SIZE: f32 = 0.5;
const DEFAULT_WIDTH: f32 = 0.5;
const DEFAULT_FREEZE: f32 = 0.0;

pub static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_SIZE,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_SIZE,
        unit: "",
        description: "Room size",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_WIDTH,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_WIDTH,
        unit: "",
        description: "Stereo width",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_FREEZE,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_FREEZE,
        unit: "",
        description: "Freeze mode (0=off, 1=on)",
        modulable: true,
    },
];

// Tuning constants
const FIXED_GAIN: f32 = 0.015;
const SCALE_DAMPING: f32 = 0.4;
const SCALE_ROOM: f32 = 0.28;
const OFFSET_ROOM: f32 = 0.7;
const STEREO_SPREAD: usize = 23;

// Comb filter tuning values at 44100 Hz
const COMB_TUNING_L1: usize = 1116;
const COMB_TUNING_R1: usize = 1116 + STEREO_SPREAD;
const COMB_TUNING_L2: usize = 1188;
const COMB_TUNING_R2: usize = 1188 + STEREO_SPREAD;
const COMB_TUNING_L3: usize = 1277;
const COMB_TUNING_R3: usize = 1277 + STEREO_SPREAD;
const COMB_TUNING_L4: usize = 1356;
const COMB_TUNING_R4: usize = 1356 + STEREO_SPREAD;
const COMB_TUNING_L5: usize = 1422;
const COMB_TUNING_R5: usize = 1422 + STEREO_SPREAD;
const COMB_TUNING_L6: usize = 1491;
const COMB_TUNING_R6: usize = 1491 + STEREO_SPREAD;
const COMB_TUNING_L7: usize = 1557;
const COMB_TUNING_R7: usize = 1557 + STEREO_SPREAD;
const COMB_TUNING_L8: usize = 1617;
const COMB_TUNING_R8: usize = 1617 + STEREO_SPREAD;

// Allpass filter tuning values at 44100 Hz
const ALLPASS_TUNING_L1: usize = 556;
const ALLPASS_TUNING_R1: usize = 556 + STEREO_SPREAD;
const ALLPASS_TUNING_L2: usize = 441;
const ALLPASS_TUNING_R2: usize = 441 + STEREO_SPREAD;
const ALLPASS_TUNING_L3: usize = 341;
const ALLPASS_TUNING_R3: usize = 341 + STEREO_SPREAD;
const ALLPASS_TUNING_L4: usize = 225;
const ALLPASS_TUNING_R4: usize = 225 + STEREO_SPREAD;

pub struct Freeverb {
    // Parameters
    size: f32,
    width: f32,
    freeze: f32,

    // Internal state
    gain: f32,
    room_size1: f32,
    damp1: f32,
    wet: f32,
    wet1: f32,
    wet2: f32,

    // Comb filters - left channel
    comb_l1: CombFilter<COMB_TUNING_L1>,
    comb_l2: CombFilter<COMB_TUNING_L2>,
    comb_l3: CombFilter<COMB_TUNING_L3>,
    comb_l4: CombFilter<COMB_TUNING_L4>,
    comb_l5: CombFilter<COMB_TUNING_L5>,
    comb_l6: CombFilter<COMB_TUNING_L6>,
    comb_l7: CombFilter<COMB_TUNING_L7>,
    comb_l8: CombFilter<COMB_TUNING_L8>,

    // Comb filters - right channel
    comb_r1: CombFilter<COMB_TUNING_R1>,
    comb_r2: CombFilter<COMB_TUNING_R2>,
    comb_r3: CombFilter<COMB_TUNING_R3>,
    comb_r4: CombFilter<COMB_TUNING_R4>,
    comb_r5: CombFilter<COMB_TUNING_R5>,
    comb_r6: CombFilter<COMB_TUNING_R6>,
    comb_r7: CombFilter<COMB_TUNING_R7>,
    comb_r8: CombFilter<COMB_TUNING_R8>,

    // Allpass filters - left channel
    allpass_l1: AllPassFilter<ALLPASS_TUNING_L1>,
    allpass_l2: AllPassFilter<ALLPASS_TUNING_L2>,
    allpass_l3: AllPassFilter<ALLPASS_TUNING_L3>,
    allpass_l4: AllPassFilter<ALLPASS_TUNING_L4>,

    // Allpass filters - right channel
    allpass_r1: AllPassFilter<ALLPASS_TUNING_R1>,
    allpass_r2: AllPassFilter<ALLPASS_TUNING_R2>,
    allpass_r3: AllPassFilter<ALLPASS_TUNING_R3>,
    allpass_r4: AllPassFilter<ALLPASS_TUNING_R4>,
}

impl Default for Freeverb {
    fn default() -> Self {
        Self::new()
    }
}

impl Freeverb {
    pub fn new() -> Self {
        let mut reverb = Self {
            size: DEFAULT_SIZE,
            width: DEFAULT_WIDTH,
            freeze: DEFAULT_FREEZE,

            gain: FIXED_GAIN,
            room_size1: 0.0,
            damp1: 0.0,
            wet: 0.0,
            wet1: 0.0,
            wet2: 0.0,

            // Initialize all comb filters
            comb_l1: CombFilter::new(),
            comb_l2: CombFilter::new(),
            comb_l3: CombFilter::new(),
            comb_l4: CombFilter::new(),
            comb_l5: CombFilter::new(),
            comb_l6: CombFilter::new(),
            comb_l7: CombFilter::new(),
            comb_l8: CombFilter::new(),

            comb_r1: CombFilter::new(),
            comb_r2: CombFilter::new(),
            comb_r3: CombFilter::new(),
            comb_r4: CombFilter::new(),
            comb_r5: CombFilter::new(),
            comb_r6: CombFilter::new(),
            comb_r7: CombFilter::new(),
            comb_r8: CombFilter::new(),

            // Initialize all allpass filters
            allpass_l1: AllPassFilter::new(),
            allpass_l2: AllPassFilter::new(),
            allpass_l3: AllPassFilter::new(),
            allpass_l4: AllPassFilter::new(),

            allpass_r1: AllPassFilter::new(),
            allpass_r2: AllPassFilter::new(),
            allpass_r3: AllPassFilter::new(),
            allpass_r4: AllPassFilter::new(),
        };

        // Set initial feedback for allpass filters
        reverb.allpass_l1.set_feedback(0.5);
        reverb.allpass_l2.set_feedback(0.5);
        reverb.allpass_l3.set_feedback(0.5);
        reverb.allpass_l4.set_feedback(0.5);
        reverb.allpass_r1.set_feedback(0.5);
        reverb.allpass_r2.set_feedback(0.5);
        reverb.allpass_r3.set_feedback(0.5);
        reverb.allpass_r4.set_feedback(0.5);

        reverb.update_parameters();
        reverb
    }

    fn update_parameters(&mut self) {
        // Always use full wet signal since dry/wet is handled externally
        self.wet = 1.0;
        self.wet1 = self.wet * (self.width / 2.0 + 0.5);
        self.wet2 = self.wet * ((1.0 - self.width) / 2.0);

        if self.freeze > 0.5 {
            // Freeze mode
            self.room_size1 = 1.0;
            self.damp1 = 0.0;
            self.gain = 0.0;
        } else {
            // Normal mode
            self.room_size1 = self.size * SCALE_ROOM + OFFSET_ROOM;
            self.damp1 = 0.5 * SCALE_DAMPING;
            self.gain = FIXED_GAIN;
        }

        // Update comb filters
        self.comb_l1.set_feedback(self.room_size1);
        self.comb_l2.set_feedback(self.room_size1);
        self.comb_l3.set_feedback(self.room_size1);
        self.comb_l4.set_feedback(self.room_size1);
        self.comb_l5.set_feedback(self.room_size1);
        self.comb_l6.set_feedback(self.room_size1);
        self.comb_l7.set_feedback(self.room_size1);
        self.comb_l8.set_feedback(self.room_size1);

        self.comb_r1.set_feedback(self.room_size1);
        self.comb_r2.set_feedback(self.room_size1);
        self.comb_r3.set_feedback(self.room_size1);
        self.comb_r4.set_feedback(self.room_size1);
        self.comb_r5.set_feedback(self.room_size1);
        self.comb_r6.set_feedback(self.room_size1);
        self.comb_r7.set_feedback(self.room_size1);
        self.comb_r8.set_feedback(self.room_size1);

        self.comb_l1.set_damp(self.damp1);
        self.comb_l2.set_damp(self.damp1);
        self.comb_l3.set_damp(self.damp1);
        self.comb_l4.set_damp(self.damp1);
        self.comb_l5.set_damp(self.damp1);
        self.comb_l6.set_damp(self.damp1);
        self.comb_l7.set_damp(self.damp1);
        self.comb_l8.set_damp(self.damp1);

        self.comb_r1.set_damp(self.damp1);
        self.comb_r2.set_damp(self.damp1);
        self.comb_r3.set_damp(self.damp1);
        self.comb_r4.set_damp(self.damp1);
        self.comb_r5.set_damp(self.damp1);
        self.comb_r6.set_damp(self.damp1);
        self.comb_r7.set_damp(self.damp1);
        self.comb_r8.set_damp(self.damp1);
    }

    #[inline]
    fn process_mono(&mut self, input: f32) -> (f32, f32) {
        let scaled_input = input * self.gain;

        // Process comb filters in parallel
        let out_l = self.comb_l1.process(scaled_input)
            + self.comb_l2.process(scaled_input)
            + self.comb_l3.process(scaled_input)
            + self.comb_l4.process(scaled_input)
            + self.comb_l5.process(scaled_input)
            + self.comb_l6.process(scaled_input)
            + self.comb_l7.process(scaled_input)
            + self.comb_l8.process(scaled_input);

        let out_r = self.comb_r1.process(scaled_input)
            + self.comb_r2.process(scaled_input)
            + self.comb_r3.process(scaled_input)
            + self.comb_r4.process(scaled_input)
            + self.comb_r5.process(scaled_input)
            + self.comb_r6.process(scaled_input)
            + self.comb_r7.process(scaled_input)
            + self.comb_r8.process(scaled_input);

        // Process allpass filters in series
        let mut out_l = self.allpass_l1.process(out_l);
        out_l = self.allpass_l2.process(out_l);
        out_l = self.allpass_l3.process(out_l);
        out_l = self.allpass_l4.process(out_l);

        let mut out_r = self.allpass_r1.process(out_r);
        out_r = self.allpass_r2.process(out_r);
        out_r = self.allpass_r3.process(out_r);
        out_r = self.allpass_r4.process(out_r);

        // Apply stereo width
        let left = out_l * self.wet1 + out_r * self.wet2;
        let right = out_r * self.wet1 + out_l * self.wet2;

        (left, right)
    }
}

impl AudioModule for Freeverb {
    fn get_name(&self) -> &'static str {
        "freeverb"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_SIZE => {
                self.size = value.clamp(0.0, 1.0);
                self.update_parameters();
                true
            }
            PARAM_WIDTH => {
                self.width = value.clamp(0.0, 1.0);
                self.update_parameters();
                true
            }
            PARAM_FREEZE => {
                self.freeze = value.clamp(0.0, 1.0);
                self.update_parameters();
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        true
    }
}

impl GlobalEffect for Freeverb {
    #[inline]
    fn process(&mut self, buffer: &mut [Frame], _sample_rate: f32) {
        for frame in buffer.iter_mut() {
            // Mix to mono for reverb input
            let input = (frame.left + frame.right) * 0.5;

            // Process through reverb
            let (left, right) = self.process_mono(input);

            // Output wet signal only
            frame.left = left;
            frame.right = right;
        }
    }
}

impl ModuleMetadata for Freeverb {
    fn get_static_name() -> &'static str {
        "freeverb"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_freeverb() -> Box<dyn GlobalEffect> {
    Box::new(Freeverb::new())
}
