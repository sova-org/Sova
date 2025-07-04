use super::delay_line::DelayLine;

#[derive(Clone)]
pub struct AllPassFilter<const N: usize> {
    delay_line: DelayLine<N>,
    feedback: f32,
}

impl<const N: usize> AllPassFilter<N> {
    pub const fn new() -> Self {
        Self {
            delay_line: DelayLine::new(),
            feedback: 0.5,
        }
    }

    #[inline]
    pub fn set_feedback(&mut self, value: f32) {
        self.feedback = value;
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let delayed = self.delay_line.read();
        let output = -input + delayed;
        self.delay_line.write(input + (delayed * self.feedback));
        output
    }

    pub fn clear(&mut self) {
        self.delay_line.clear();
    }
}
