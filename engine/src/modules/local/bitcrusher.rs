use crate::modules::{AudioModule, Frame, LocalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_BITS: &str = "bits";
const PARAM_RATE: &str = "rate";

const DEFAULT_BITS: f32 = 16.0;
const DEFAULT_RATE: f32 = 1.0;

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_BITS,
        aliases: &[],
        min_value: 2.0,
        max_value: 32.0,
        default_value: DEFAULT_BITS,
        unit: "bits",
        description: "Bit depth reduction",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_RATE,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: DEFAULT_RATE,
        unit: "",
        description: "Sample rate reduction factor",
        modulable: true,
    },
];

pub struct BitCrusher {
    bits: f32,
    rate: f32,
    is_active: bool,

    left_hold_sample: f32,
    right_hold_sample: f32,
    accumulator: f32,
    sample_rate: f32,
}

impl Default for BitCrusher {
    fn default() -> Self {
        Self::new()
    }
}

impl BitCrusher {
    pub fn new() -> Self {
        Self {
            bits: DEFAULT_BITS,
            rate: DEFAULT_RATE,
            is_active: true,
            left_hold_sample: 0.0,
            right_hold_sample: 0.0,
            accumulator: 0.0,
            sample_rate: 44100.0,
        }
    }

    #[inline]
    fn crush_bits(&self, input: f32) -> f32 {
        if self.bits >= 32.0 {
            return input;
        }

        let levels = (1 << (self.bits as u32)).min(16777216) as f32;
        let step = 2.0 / levels;

        let quantized = ((input + 1.0) / step).floor() * step - 1.0;
        quantized.clamp(-1.0, 1.0)
    }

    #[inline]
    fn should_update_sample(&mut self) -> bool {
        if self.rate >= 1.0 {
            return true;
        }

        let rate_hz = self.rate * self.sample_rate;
        let samples_per_hold = self.sample_rate / rate_hz;

        self.accumulator += 1.0;
        if self.accumulator >= samples_per_hold {
            self.accumulator -= samples_per_hold;
            true
        } else {
            false
        }
    }
}

impl AudioModule for BitCrusher {
    fn get_name(&self) -> &'static str {
        "bitcrusher"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_BITS => {
                self.bits = value.clamp(2.0, 32.0);
                true
            }
            PARAM_RATE => {
                self.rate = value.clamp(0.0, 1.0);
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl LocalEffect for BitCrusher {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        self.sample_rate = sample_rate;

        for frame in buffer.iter_mut() {
            if self.should_update_sample() {
                self.left_hold_sample = self.crush_bits(frame.left);
                self.right_hold_sample = self.crush_bits(frame.right);
            }

            frame.left = self.left_hold_sample;
            frame.right = self.right_hold_sample;
        }
    }
}

impl ModuleMetadata for BitCrusher {
    fn get_static_name() -> &'static str {
        "bitcrusher"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_bitcrusher() -> Box<dyn LocalEffect> {
    Box::new(BitCrusher::new())
}
