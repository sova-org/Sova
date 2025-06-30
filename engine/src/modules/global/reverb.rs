use crate::modules::{AudioModule, Frame, GlobalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_SIZE: &str = "size";
const PARAM_DAMPING: &str = "damping";

const DEFAULT_SIZE: f32 = 0.5;
const DEFAULT_DAMPING: f32 = 0.5;

pub static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
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

// Fixed-size delay line with compile-time size limits
struct FixedDelayLine<const N: usize> {
    buffer: [f32; N],
    write_pos: usize,
    delay_samples: usize,
}

impl<const N: usize> FixedDelayLine<N> {
    const fn new(delay_samples: usize) -> Self {
        let delay = if delay_samples < N - 1 {
            delay_samples
        } else {
            N - 1
        };
        Self {
            buffer: [0.0; N],
            write_pos: 0,
            delay_samples: delay,
        }
    }

    #[inline]
    fn process(&mut self, input: f32, feedback: f32) -> f32 {
        let read_pos = if self.write_pos >= self.delay_samples {
            self.write_pos - self.delay_samples
        } else {
            N - (self.delay_samples - self.write_pos)
        };

        let output = self.buffer[read_pos];
        self.buffer[self.write_pos] = input + output * feedback;
        self.write_pos = (self.write_pos + 1) % N;
        output
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }

    fn set_delay(&mut self, delay_samples: usize) {
        self.delay_samples = delay_samples.min(N - 1);
    }
}

// High-performance one-pole filter
#[derive(Clone, Copy)]
struct OnePoleFilter {
    y1: f32,
    coefficient: f32,
}

impl OnePoleFilter {
    const fn new() -> Self {
        Self {
            y1: 0.0,
            coefficient: 0.5,
        }
    }

    #[inline]
    fn set_cutoff(&mut self, cutoff_hz: f32, sample_rate: f32) {
        let omega = 2.0 * std::f32::consts::PI * cutoff_hz / sample_rate;
        self.coefficient = (-omega).exp();
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        self.y1 = input * (1.0 - self.coefficient) + self.y1 * self.coefficient;
        self.y1
    }
}

// Zero-allocation reverb using fixed arrays
pub struct ZeroAllocReverb {
    size: f32,
    damping: f32,
    sample_rate: f32,
    is_active: bool,

    // Fixed-size comb filters (max ~100ms at 48kHz)
    comb1: FixedDelayLine<2048>, // ~42ms at 48kHz
    comb2: FixedDelayLine<2197>, // ~45ms at 48kHz
    comb3: FixedDelayLine<2357>, // ~49ms at 48kHz
    comb4: FixedDelayLine<2579>, // ~53ms at 48kHz

    // Fixed-size allpass filters
    ap1: FixedDelayLine<347>, // ~7ms at 48kHz
    ap2: FixedDelayLine<113>, // ~2.3ms at 48kHz

    // Damping filters for each comb
    damping_filters: [OnePoleFilter; 4],

    // Cached delay times for different sample rates
    base_delays: [usize; 6],
}

impl Default for ZeroAllocReverb {
    fn default() -> Self {
        Self::new()
    }
}

impl ZeroAllocReverb {
    pub fn new() -> Self {
        let sample_rate = 44100.0;

        // Prime number delays for better diffusion (in samples at 44.1kHz)
        let base_delays = [
            1687, 1801, 1933, 2089, // Comb delays
            317, 97, // Allpass delays
        ];

        let mut reverb = Self {
            size: DEFAULT_SIZE,
            damping: DEFAULT_DAMPING,
            sample_rate,
            is_active: true,

            comb1: FixedDelayLine::new(base_delays[0]),
            comb2: FixedDelayLine::new(base_delays[1]),
            comb3: FixedDelayLine::new(base_delays[2]),
            comb4: FixedDelayLine::new(base_delays[3]),

            ap1: FixedDelayLine::new(base_delays[4]),
            ap2: FixedDelayLine::new(base_delays[5]),

            damping_filters: [OnePoleFilter::new(); 4],
            base_delays,
        };

        reverb.update_parameters();
        reverb
    }

    #[inline]
    fn update_parameters(&mut self) {
        // High-frequency damping based on damping parameter
        let cutoff = (1.0 - self.damping) * 8000.0 + 2000.0;
        for filter in &mut self.damping_filters {
            filter.set_cutoff(cutoff, self.sample_rate);
        }
    }

    fn update_sample_rate(&mut self, new_sample_rate: f32) {
        if self.sample_rate != new_sample_rate {
            self.sample_rate = new_sample_rate;

            // Scale delay times for new sample rate
            let scale = new_sample_rate / 44100.0;

            self.comb1
                .set_delay((self.base_delays[0] as f32 * scale) as usize);
            self.comb2
                .set_delay((self.base_delays[1] as f32 * scale) as usize);
            self.comb3
                .set_delay((self.base_delays[2] as f32 * scale) as usize);
            self.comb4
                .set_delay((self.base_delays[3] as f32 * scale) as usize);

            self.ap1
                .set_delay((self.base_delays[4] as f32 * scale) as usize);
            self.ap2
                .set_delay((self.base_delays[5] as f32 * scale) as usize);

            // Clear all buffers
            self.comb1.clear();
            self.comb2.clear();
            self.comb3.clear();
            self.comb4.clear();
            self.ap1.clear();
            self.ap2.clear();

            self.update_parameters();
        }
    }
}

impl AudioModule for ZeroAllocReverb {
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
                true
            }
            PARAM_DAMPING => {
                self.damping = value.clamp(0.0, 1.0);
                self.update_parameters();
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl GlobalEffect for ZeroAllocReverb {
    #[inline]
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        self.update_sample_rate(sample_rate);

        // Feedback amount based on size parameter
        let feedback = self.size * 0.7 + 0.1;
        let ap_feedback = 0.7;

        // Process each frame
        for frame in buffer.iter_mut() {
            // Convert stereo to mono for reverb input
            let input = (frame.left + frame.right) * 0.5;

            // Parallel comb filters with feedback and damping
            let comb1_out = self.damping_filters[0].process(self.comb1.process(input, feedback));
            let comb2_out = self.damping_filters[1].process(self.comb2.process(input, feedback));
            let comb3_out = self.damping_filters[2].process(self.comb3.process(input, feedback));
            let comb4_out = self.damping_filters[3].process(self.comb4.process(input, feedback));

            // Sum and average the comb outputs
            let comb_sum = (comb1_out + comb2_out + comb3_out + comb4_out) * 0.25;

            // Series allpass filters for diffusion
            let ap1_input = comb_sum;
            let ap1_delayed = self.ap1.process(ap1_input, ap_feedback);
            let ap1_out = -ap1_input + ap1_delayed;

            let ap2_delayed = self.ap2.process(ap1_out, ap_feedback);
            let final_out = -ap1_out + ap2_delayed;

            // Output as stereo with slight stereo width
            let left_mult = 1.0 + self.size * 0.1;
            let right_mult = 1.0 - self.size * 0.1;

            frame.left = final_out * left_mult;
            frame.right = final_out * right_mult;
        }
    }
}

impl ModuleMetadata for ZeroAllocReverb {
    fn get_static_name() -> &'static str {
        "reverb"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_simple_reverb() -> Box<dyn GlobalEffect> {
    Box::new(ZeroAllocReverb::new())
}

// Keep old ZitaReverb name for compatibility
pub type ZitaReverb = ZeroAllocReverb;
