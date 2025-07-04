use crate::modules::{AudioModule, Frame, GlobalEffect, ModuleMetadata, ParameterDescriptor};

const PARAM_DUR: &str = "echodur";
const PARAM_FEEDBACK: &str = "echofb";
const PARAM_CUTOFF: &str = "echolpf";

const DEFAULT_DUR: f32 = 0.25;
const DEFAULT_FEEDBACK: f32 = 0.3;
const DEFAULT_CUTOFF: f32 = 4000.0;

pub static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_DUR,
        aliases: &[],
        min_value: 0.01,
        max_value: 2.0,
        default_value: DEFAULT_DUR,
        unit: "s",
        description: "Echo delay time",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_FEEDBACK,
        aliases: &[],
        min_value: 0.0,
        max_value: 0.98,
        default_value: DEFAULT_FEEDBACK,
        unit: "",
        description: "Echo feedback amount",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_CUTOFF,
        aliases: &[],
        min_value: 100.0,
        max_value: 15000.0,
        default_value: DEFAULT_CUTOFF,
        unit: "Hz",
        description: "Echo lowpass filter cutoff",
        modulable: true,
    },
];

struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
    max_delay_samples: usize,
}

impl DelayLine {
    fn new(max_delay_seconds: f32, sample_rate: f32) -> Self {
        let max_delay_samples = (max_delay_seconds * sample_rate) as usize;
        Self {
            buffer: vec![0.0; max_delay_samples],
            write_pos: 0,
            max_delay_samples,
        }
    }

    fn process_with_feedback(&mut self, input: f32, feedback: f32, delay_samples: usize) -> f32 {
        let delay_samples = delay_samples.min(self.max_delay_samples - 1);
        let read_pos = if self.write_pos >= delay_samples {
            self.write_pos - delay_samples
        } else {
            self.max_delay_samples - (delay_samples - self.write_pos)
        };

        let output = self.buffer[read_pos];
        // Input + feedback creates the classic delay effect
        // Limit the feedback signal to prevent runaway feedback
        let feedback_signal = (output * feedback).clamp(-0.95, 0.95);
        self.buffer[self.write_pos] = input + feedback_signal;
        self.write_pos = (self.write_pos + 1) % self.max_delay_samples;
        output
    }

    fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

struct OnePoleFilter {
    y1: f32,
    coefficient: f32,
}

impl OnePoleFilter {
    fn new() -> Self {
        Self {
            y1: 0.0,
            coefficient: 0.5,
        }
    }

    fn set_cutoff(&mut self, cutoff_hz: f32, sample_rate: f32) {
        let omega = 2.0 * std::f32::consts::PI * cutoff_hz / sample_rate;
        self.coefficient = (-omega).exp();
    }

    fn process(&mut self, input: f32) -> f32 {
        self.y1 = input * (1.0 - self.coefficient) + self.y1 * self.coefficient;
        self.y1
    }

    fn clear(&mut self) {
        self.y1 = 0.0;
    }
}

struct DcBlocker {
    x1: f32,
    y1: f32,
}

impl DcBlocker {
    fn new() -> Self {
        Self { x1: 0.0, y1: 0.0 }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = input - self.x1 + 0.995 * self.y1;
        self.x1 = input;
        self.y1 = output;
        output
    }

    fn clear(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }
}

pub struct EchoEffect {
    dur: f32,
    feedback: f32,
    cutoff: f32,
    sample_rate: f32,
    is_active: bool,

    left_delay: DelayLine,
    right_delay: DelayLine,
    left_filter: OnePoleFilter,
    right_filter: OnePoleFilter,
    left_dc_blocker: DcBlocker,
    right_dc_blocker: DcBlocker,

    last_input_energy: f32,
    silence_counter: usize,
}

impl Default for EchoEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl EchoEffect {
    pub fn new() -> Self {
        let sample_rate = 44100.0;
        let max_delay = 2.0;

        let mut left_filter = OnePoleFilter::new();
        let mut right_filter = OnePoleFilter::new();
        left_filter.set_cutoff(DEFAULT_CUTOFF, sample_rate);
        right_filter.set_cutoff(DEFAULT_CUTOFF, sample_rate);

        Self {
            dur: DEFAULT_DUR,
            feedback: DEFAULT_FEEDBACK,
            cutoff: DEFAULT_CUTOFF,
            sample_rate,
            is_active: true,

            left_delay: DelayLine::new(max_delay, sample_rate),
            right_delay: DelayLine::new(max_delay, sample_rate),
            left_filter,
            right_filter,
            left_dc_blocker: DcBlocker::new(),
            right_dc_blocker: DcBlocker::new(),

            last_input_energy: 0.0,
            silence_counter: 0,
        }
    }

    #[inline]
    fn soft_limiter(x: f32) -> f32 {
        if x.abs() <= 0.7 {
            x
        } else {
            let sign = x.signum();
            let abs_x = x.abs();
            if abs_x <= 1.0 {
                // Soft compression between 0.7 and 1.0
                let compressed = 0.7 + (abs_x - 0.7) * 0.3 / 0.3;
                sign * compressed
            } else {
                // Hard limit at 1.0
                sign * 1.0
            }
        }
    }

    fn update_filters(&mut self) {
        self.left_filter.set_cutoff(self.cutoff, self.sample_rate);
        self.right_filter.set_cutoff(self.cutoff, self.sample_rate);
    }

    fn clear_delay_buffers(&mut self) {
        self.left_delay.clear();
        self.right_delay.clear();
        self.left_filter.clear();
        self.right_filter.clear();
        self.left_dc_blocker.clear();
        self.right_dc_blocker.clear();
    }
}

impl AudioModule for EchoEffect {
    fn get_name(&self) -> &'static str {
        "echo"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_DUR => {
                self.dur = value.clamp(0.01, 2.0);
                true
            }
            PARAM_FEEDBACK => {
                self.feedback = value.clamp(0.01, 0.99);
                true
            }
            PARAM_CUTOFF => {
                self.cutoff = value.clamp(50.0, 15000.0);
                self.update_filters();
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl GlobalEffect for EchoEffect {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.left_delay = DelayLine::new(2.0, sample_rate);
            self.right_delay = DelayLine::new(2.0, sample_rate);
            self.update_filters();
        }

        let delay_samples = (self.dur * sample_rate) as usize;

        let input_energy: f32 = buffer
            .iter()
            .map(|f| f.left.abs() + f.right.abs())
            .sum::<f32>()
            / buffer.len() as f32;

        const SILENCE_THRESHOLD: f32 = 0.0001;
        const SILENCE_TIMEOUT_BLOCKS: usize = 100;

        if input_energy < SILENCE_THRESHOLD {
            self.silence_counter += 1;
            if self.silence_counter > SILENCE_TIMEOUT_BLOCKS {
                self.clear_delay_buffers();
                self.silence_counter = 0;
            }
        } else {
            self.silence_counter = 0;
        }

        for frame in buffer.iter_mut() {
            // Process delay with feedback - this creates the repeating echoes
            let left_delayed =
                self.left_delay
                    .process_with_feedback(frame.left, self.feedback, delay_samples);
            let right_delayed =
                self.right_delay
                    .process_with_feedback(frame.right, self.feedback, delay_samples);

            // Filter the delayed signal
            let left_filtered = self.left_filter.process(left_delayed);
            let right_filtered = self.right_filter.process(right_delayed);

            // DC block the filtered signal
            let left_blocked = self.left_dc_blocker.process(left_filtered);
            let right_blocked = self.right_dc_blocker.process(right_filtered);

            // Apply soft limiter to prevent clipping and output 100% wet signal
            frame.left = Self::soft_limiter(left_blocked);
            frame.right = Self::soft_limiter(right_blocked);
        }

        self.last_input_energy = input_energy;
    }
}

impl ModuleMetadata for EchoEffect {
    fn get_static_name() -> &'static str {
        "echo"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_echo_effect() -> Box<dyn GlobalEffect> {
    Box::new(EchoEffect::new())
}
