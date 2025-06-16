use crate::modules::Frame;

pub struct DcBlocker {
    x1_left: f32,
    x1_right: f32,
    y1_left: f32,
    y1_right: f32,
    pole: f32,
}

impl Default for DcBlocker {
    fn default() -> Self {
        Self::new()
    }
}

impl DcBlocker {
    pub fn new() -> Self {
        Self {
            x1_left: 0.0,
            x1_right: 0.0,
            y1_left: 0.0,
            y1_right: 0.0,
            pole: 0.995,
        }
    }

    #[inline]
    pub fn process_frame(&mut self, frame: &mut Frame) {
        let left_input = frame.left;
        let right_input = frame.right;

        let left_output = left_input - self.x1_left + self.pole * self.y1_left;
        self.x1_left = left_input;
        self.y1_left = left_output;

        let right_output = right_input - self.x1_right + self.pole * self.y1_right;
        self.x1_right = right_input;
        self.y1_right = right_output;

        frame.left = left_output;
        frame.right = right_output;
    }

    #[inline]
    pub fn process_buffer(&mut self, buffer: &mut [Frame]) {
        for frame in buffer.iter_mut() {
            self.process_frame(frame);
        }
    }

    #[inline]
    pub fn process_block_optimized(&mut self, buffer: &mut [Frame]) {
        let (aligned, rest) = buffer.split_at_mut(buffer.len() & !3);

        for chunk in aligned.chunks_exact_mut(4) {
            for frame in chunk {
                self.process_frame(frame);
            }
        }

        for frame in rest {
            self.process_frame(frame);
        }
    }

    pub fn reset(&mut self) {
        self.x1_left = 0.0;
        self.x1_right = 0.0;
        self.y1_left = 0.0;
        self.y1_right = 0.0;
    }
}
