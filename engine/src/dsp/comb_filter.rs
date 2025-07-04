use super::delay_line::DelayLine;

#[derive(Clone)]
pub struct CombFilter<const N: usize> {
    delay_line: DelayLine<N>,
    feedback: f32,
    filter_store: f32,
    damp1: f32,
    damp2: f32,
}

impl<const N: usize> CombFilter<N> {
    pub const fn new() -> Self {
        Self {
            delay_line: DelayLine::new(),
            feedback: 0.0,
            filter_store: 0.0,
            damp1: 0.0,
            damp2: 0.0,
        }
    }

    #[inline]
    pub fn set_damp(&mut self, value: f32) {
        self.damp1 = value;
        self.damp2 = 1.0 - value;
    }

    #[inline]
    pub fn set_feedback(&mut self, value: f32) {
        self.feedback = value;
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.delay_line.read();

        self.filter_store = (output * self.damp2) + (self.filter_store * self.damp1);

        self.delay_line
            .write(input + (self.filter_store * self.feedback));

        output
    }

    pub fn clear(&mut self) {
        self.delay_line.clear();
        self.filter_store = 0.0;
    }
}
