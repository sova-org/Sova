#[derive(Clone)]
pub struct InterpolatingDelay<const N: usize> {
    buffer: [f32; N],
    write_pos: usize,
}

impl<const N: usize> InterpolatingDelay<N> {
    pub const fn new() -> Self {
        Self {
            buffer: [0.0; N],
            write_pos: 0,
        }
    }

    #[inline]
    pub fn write(&mut self, input: f32) {
        self.buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % N;
    }

    #[inline]
    pub fn read_interpolated(&self, delay_samples: f32) -> f32 {
        // Clamp delay to valid range
        let delay = delay_samples.clamp(1.0, (N - 1) as f32);

        // Calculate read positions
        let delay_int = delay.floor() as usize;
        let delay_frac = delay - delay.floor();

        // Calculate read indices (going backwards from write position)
        let read_pos1 = if self.write_pos >= delay_int {
            self.write_pos - delay_int
        } else {
            N - (delay_int - self.write_pos)
        };

        let read_pos2 = if read_pos1 == 0 { N - 1 } else { read_pos1 - 1 };

        // Linear interpolation
        let sample1 = self.buffer[read_pos1];
        let sample2 = self.buffer[read_pos2];

        sample1 * (1.0 - delay_frac) + sample2 * delay_frac
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}
