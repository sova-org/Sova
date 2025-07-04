#[derive(Clone)]
pub struct DelayLine<const N: usize> {
    buffer: [f32; N],
    write_pos: usize,
}

impl<const N: usize> DelayLine<N> {
    pub const fn new() -> Self {
        Self {
            buffer: [0.0; N],
            write_pos: 0,
        }
    }

    #[inline]
    pub fn read(&self) -> f32 {
        self.buffer[self.write_pos]
    }

    #[inline]
    pub fn write(&mut self, input: f32) {
        self.buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % N;
    }

    #[inline]
    pub fn read_write(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.write_pos];
        self.buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % N;
        output
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}
