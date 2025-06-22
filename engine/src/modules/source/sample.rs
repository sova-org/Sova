//! This file is a stereo sampler usable as a source for synthesis voices.
//! Samples are preloaded in the engine using the memory/samplib.rs file.
//! /play s sample fd kick -> plays the first sample in the kick folder
//! /play s sample fd kick nb 2 -> plays the third sample in the kick folder
//! /play s sample fd bob nb 4.3 -> plays the fourth sample in the bob folder, mixed with the third one
//! /play s sample fd bass freq 300 -> plays the bass sample at 300Hz

use crate::modules::{AudioModule, Frame, ModuleMetadata, ParameterDescriptor, Source};

const PARAM_SAMPLE_NAME: &str = "sample_name";
const PARAM_SAMPLE_NUMBER: &str = "sample_number";
const PARAM_SPEED: &str = "speed";
const PARAM_BEGIN: &str = "begin";
const PARAM_END: &str = "end";
const PARAM_LOOP: &str = "loop";

static PARAMETER_DESCRIPTORS: &[ParameterDescriptor] = &[
    ParameterDescriptor {
        name: PARAM_SAMPLE_NAME,
        aliases: &["sn", "folder", "fd"],
        min_value: 0.0,
        max_value: 0.0,
        default_value: 0.0,
        unit: "Sample Name",
        description: "Name of the sample to play, like 'kick' or 'bass'.",
        modulable: false,
    },
    ParameterDescriptor {
        name: PARAM_SAMPLE_NUMBER,
        aliases: &["sp", "nb"],
        min_value: 0.0,
        max_value: 9999.0,
        default_value: 0.0,
        unit: "Sample Number",
        description: "Sample number to play, where 0 is the first sample in the folder, 1 the second one. If a float is provided, we mix the sample with the previous one, so 0.5 will mix the first and second sample.",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_SPEED,
        aliases: &[],
        min_value: -999.0,
        max_value: 999.0,
        default_value: 1.0,
        unit: "Absolute",
        description: "Playback speed of the sample, where 1.0 is normal speed, -1.0 reversed normal speed and 2.0 twice the speed.",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_BEGIN,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: 0.0,
        unit: "Absolute",
        description: "Start position of the sample, where 0.0 is the beginning of the sample and 1.0 is the end.",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_END,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: 1.0,
        unit: "Absolute",
        description: "End position of the sample, where 0.0 is the beginning of the sample and 1.0 is the end.",
        modulable: true,
    },
    ParameterDescriptor {
        name: PARAM_LOOP,
        aliases: &[],
        min_value: 0.0,
        max_value: 1.0,
        default_value: 0.0,
        unit: "Absolute",
        description: "Loop the sample, where 1.0 is looping and 0.0 is not looping.",
        modulable: true,
    },
];

pub struct StereoSampler {
    sample_name_index: f32,
    sample_number: f32,
    speed: f32,
    begin: f32,
    end: f32,
    loop_sample: f32,
    is_active: bool,
    playback_position: f32,
    sample_data: Option<Vec<f32>>,
}

impl Default for StereoSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl StereoSampler {
    pub fn new() -> Self {
        Self {
            sample_name_index: 0.0,
            sample_number: 0.0,
            speed: 1.0,
            begin: 0.0,
            end: 1.0,
            loop_sample: 0.0,
            is_active: true,
            playback_position: 0.0,
            sample_data: None,
        }
    }

    pub fn load_sample_data(&mut self, data: Vec<f32>) {
        self.sample_data = Some(data);
        self.playback_position = 0.0;
    }

    pub fn trigger(&mut self) {
        self.playback_position = if self.speed >= 0.0 { 0.0 } else { 1.0 };
        self.is_active = true;
    }

    pub fn stop(&mut self) {
        self.is_active = false;
    }

    fn calculate_playback_speed(&self) -> f32 {
        self.speed
    }

    fn get_sample_bounds(&self, sample_length: usize) -> (usize, usize) {
        let begin_sample = (self.begin * sample_length as f32) as usize;
        let end_sample = (self.end * sample_length as f32).min(sample_length as f32) as usize;
        (begin_sample, end_sample)
    }

    fn interpolate_sample(
        &self,
        sample_data: &[f32],
        pos: f32,
        begin_sample: usize,
        effective_length: usize,
    ) -> Frame {
        let sample_pos = pos * effective_length as f32;
        let pos_int = sample_pos.floor() as usize;
        let pos_frac = sample_pos - pos_int as f32;

        let current_idx =
            (begin_sample + pos_int).min(begin_sample + effective_length.saturating_sub(1));
        let next_idx = if pos_int + 1 < effective_length {
            (begin_sample + pos_int + 1).min(begin_sample + effective_length.saturating_sub(1))
        } else if self.should_loop() {
            begin_sample // Loop back to beginning
        } else {
            current_idx // Use current sample to avoid clicks
        };

        // Graceful boundary handling - avoid abrupt cutoffs
        let (left1, right1) = if current_idx * 2 + 1 < sample_data.len() {
            (
                sample_data[current_idx * 2],
                sample_data[current_idx * 2 + 1],
            )
        } else if current_idx * 2 < sample_data.len() {
            (sample_data[current_idx * 2], sample_data[current_idx * 2]) // Mono fallback
        } else {
            (0.0, 0.0) // Silence only if completely out of bounds
        };

        let (left2, right2) = if next_idx * 2 + 1 < sample_data.len() {
            (sample_data[next_idx * 2], sample_data[next_idx * 2 + 1])
        } else if next_idx * 2 < sample_data.len() {
            (sample_data[next_idx * 2], sample_data[next_idx * 2]) // Mono fallback
        } else {
            (left1, right1) // Use current sample to avoid clicks
        };

        let left = left1 + (left2 - left1) * pos_frac;
        let right = right1 + (right2 - right1) * pos_frac;

        Frame::new(left, right)
    }

    fn should_loop(&self) -> bool {
        self.loop_sample > 0.5
    }

    fn update_position(&mut self, speed: f32, _sample_rate: f32, effective_length: usize) {
        // At speed=1.0, we want to advance by 1 sample per audio frame
        // Position is normalized (0.0 to 1.0), so increment per frame = speed / effective_length
        // But we need to account for sample rate conversion if the sample has a different rate
        // For now, assume sample is at the same rate as the engine
        let position_increment = speed / effective_length as f32;

        self.playback_position += position_increment;

        // Don't immediately deactivate - let handle_looping deal with boundaries
        // This prevents abrupt clicks
    }

    fn handle_looping(&mut self, speed: f32, _effective_length: usize) {
        if self.should_loop() {
            // Use modulo for seamless looping to avoid clicks
            if speed >= 0.0 {
                while self.playback_position >= 1.0 {
                    self.playback_position -= 1.0;
                }
            } else {
                while self.playback_position < 0.0 {
                    self.playback_position += 1.0;
                }
            }
        } else {
            // Only deactivate if completely out of bounds
            if self.playback_position >= 1.0 || self.playback_position < 0.0 {
                self.is_active = false;
            }
        }
    }
}

impl AudioModule for StereoSampler {
    fn get_name(&self) -> &'static str {
        "sample"
    }

    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> bool {
        match param {
            PARAM_SAMPLE_NAME => {
                self.sample_name_index = value;
                true
            }
            PARAM_SAMPLE_NUMBER => {
                self.sample_number = value.clamp(0.0, 9999.0);
                true
            }
            PARAM_SPEED => {
                self.speed = value.clamp(-999.0, 999.0);
                true
            }
            PARAM_BEGIN => {
                self.begin = value.clamp(0.0, 1.0);
                true
            }
            PARAM_END => {
                self.end = value.clamp(0.0, 1.0);
                true
            }
            PARAM_LOOP => {
                self.loop_sample = value.clamp(0.0, 1.0);
                true
            }
            _ => false,
        }
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl Source for StereoSampler {
    fn generate(&mut self, buffer: &mut [Frame], sample_rate: f32) {
        if self.sample_data.is_none() {
            buffer.fill(Frame::ZERO);
            return;
        }

        let sample_length = self.sample_data.as_ref().unwrap().len() / 2;
        if sample_length == 0 {
            buffer.fill(Frame::ZERO);
            return;
        }

        let (begin_sample, end_sample) = self.get_sample_bounds(sample_length);
        let effective_length = end_sample.saturating_sub(begin_sample);

        if effective_length == 0 {
            buffer.fill(Frame::ZERO);
            return;
        }

        let playback_speed = self.calculate_playback_speed();

        for frame in buffer.iter_mut() {
            if !self.is_active {
                *frame = Frame::ZERO;
                continue;
            }

            // Handle position updates first
            self.update_position(playback_speed, sample_rate, effective_length);
            self.handle_looping(playback_speed, effective_length);

            // Continue processing even if position is slightly out of bounds
            // This prevents abrupt clicks
            let normalized_pos = if playback_speed >= 0.0 {
                self.playback_position.clamp(0.0, 1.0)
            } else {
                (1.0 - self.playback_position).clamp(0.0, 1.0)
            };

            let sample_data = self.sample_data.as_ref().unwrap();
            *frame = self.interpolate_sample(
                sample_data,
                normalized_pos,
                begin_sample,
                effective_length,
            );

            // Only stop if inactive after processing this frame
            if !self.is_active {
                // Continue to next frame to allow gradual fadeout
                continue;
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ModuleMetadata for StereoSampler {
    fn get_static_name() -> &'static str {
        "sample"
    }

    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor] {
        PARAMETER_DESCRIPTORS
    }
}

pub fn create_stereo_sampler() -> Box<dyn Source> {
    Box::new(StereoSampler::new())
}
