#[derive(Clone)]
pub struct FeedbackDelay<const N: usize> {
    buffer: [f32; N],
    write_pos: usize,
}

impl<const N: usize> FeedbackDelay<N> {
    pub const fn new() -> Self {
        Self {
            buffer: [0.0; N],
            write_pos: 0,
        }
    }

    #[inline]
    pub fn process(&mut self, input: f32, delay_samples: usize, feedback: f32) -> f32 {
        // Clamp delay to valid range
        let delay = delay_samples.min(N - 1).max(1);

        // Calculate read position (going backwards from write position)
        let read_pos = if self.write_pos >= delay {
            self.write_pos - delay
        } else {
            N - (delay - self.write_pos)
        };

        // Read delayed sample
        let delayed = self.buffer[read_pos];

        // Write new sample with feedback
        self.buffer[self.write_pos] = input + delayed * feedback.clamp(0.0, 0.98);

        // Advance write position
        self.write_pos = (self.write_pos + 1) % N;

        delayed
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}
