use crate::memory::MemoryPool;
use crate::modules::Frame;
use std::ptr::NonNull;
use std::sync::Arc;

pub struct AudioBuffer {
    frames: NonNull<Frame>,
    length: usize,
    capacity: usize,
    #[allow(dead_code)]
    memory_pool: Option<Arc<MemoryPool>>,
}

impl AudioBuffer {
    pub fn new(capacity: usize) -> Self {
        Self::with_pool(capacity, None)
    }

    pub fn with_pool(capacity: usize, pool: Option<Arc<MemoryPool>>) -> Self {
        let frames = if let Some(ref pool) = pool {
            if let Some(ptr) = pool.allocate(capacity * std::mem::size_of::<Frame>(), 16) {
                unsafe {
                    let frame_ptr = ptr.as_ptr() as *mut Frame;
                    std::slice::from_raw_parts_mut(frame_ptr, capacity).fill(Frame::ZERO);
                    NonNull::new_unchecked(frame_ptr)
                }
            } else {
                let mut vec = Vec::with_capacity(capacity);
                vec.resize(capacity, Frame::ZERO);
                NonNull::new(vec.leak().as_mut_ptr()).unwrap()
            }
        } else {
            let mut vec = Vec::with_capacity(capacity);
            vec.resize(capacity, Frame::ZERO);
            NonNull::new(vec.leak().as_mut_ptr()).unwrap()
        };

        Self {
            frames,
            length: 0,
            capacity,
            memory_pool: pool,
        }
    }

    pub fn with_length(capacity: usize, length: usize) -> Self {
        let mut buffer = Self::new(capacity);
        buffer.set_length(length.min(capacity));
        buffer
    }

    pub fn frames(&self) -> &[Frame] {
        unsafe { std::slice::from_raw_parts(self.frames.as_ptr(), self.length) }
    }

    pub fn frames_mut(&mut self) -> &mut [Frame] {
        unsafe { std::slice::from_raw_parts_mut(self.frames.as_ptr(), self.length) }
    }

    pub fn set_length(&mut self, length: usize) {
        self.length = length.min(self.capacity);
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        Frame::process_block_zero(self.frames_mut());
    }

    pub fn resize(&mut self, new_length: usize) {
        let old_length = self.length;
        self.set_length(new_length);

        if new_length > old_length {
            unsafe {
                let slice = std::slice::from_raw_parts_mut(self.frames.as_ptr(), self.capacity);
                slice[old_length..new_length].fill(Frame::ZERO);
            }
        }
    }

    pub fn copy_from(&mut self, source: &[Frame]) {
        let copy_len = source.len().min(self.capacity);
        if copy_len > 0 {
            unsafe {
                let slice = std::slice::from_raw_parts_mut(self.frames.as_ptr(), self.capacity);
                slice[..copy_len].copy_from_slice(&source[..copy_len]);
            }
            self.length = copy_len;
        }
    }

    pub fn add_from(&mut self, source: &[Frame]) {
        let add_len = source.len().min(self.length).min(self.capacity);
        if add_len > 0 {
            Frame::process_block_add(self.frames_mut(), &source[..add_len]);
        }
    }

    pub fn multiply(&mut self, gain: f32) {
        Frame::process_block_mul_scalar(self.frames_mut(), gain);
    }

    pub fn multiply_from(&mut self, source: &[Frame]) {
        let mul_len = source.len().min(self.length);
        let frames = self.frames_mut();
        for (dest, src) in frames[..mul_len].iter_mut().zip(source.iter()) {
            dest.left *= src.left;
            dest.right *= src.right;
        }
    }
}

unsafe impl Send for AudioBuffer {}
unsafe impl Sync for AudioBuffer {}

pub struct BlockProcessor {
    block_size: usize,
    sample_rate: f32,
    input_buffer: AudioBuffer,
    output_buffer: AudioBuffer,
    temp_buffer: AudioBuffer,
}

impl BlockProcessor {
    pub fn new(block_size: usize, sample_rate: f32) -> Self {
        Self::with_pool(block_size, sample_rate, None)
    }

    pub fn with_pool(block_size: usize, sample_rate: f32, pool: Option<Arc<MemoryPool>>) -> Self {
        let effective_block_size = if block_size == 0 { 256 } else { block_size };
        Self {
            block_size: effective_block_size,
            sample_rate,
            input_buffer: AudioBuffer::with_pool(effective_block_size, pool.clone()),
            output_buffer: AudioBuffer::with_pool(effective_block_size, pool.clone()),
            temp_buffer: AudioBuffer::with_pool(effective_block_size, pool),
        }
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn input_buffer(&mut self) -> &mut AudioBuffer {
        &mut self.input_buffer
    }

    pub fn output_buffer(&mut self) -> &mut AudioBuffer {
        &mut self.output_buffer
    }

    pub fn temp_buffer(&mut self) -> &mut AudioBuffer {
        &mut self.temp_buffer
    }

    pub fn prepare_input(&mut self, input: &[Frame]) {
        self.input_buffer.copy_from(input);
    }

    pub fn get_output(&self) -> &[Frame] {
        self.output_buffer.frames()
    }

    pub fn clear_buffers(&mut self) {
        self.input_buffer.clear();
        self.output_buffer.clear();
        self.temp_buffer.clear();
    }
}

pub struct RingBuffer {
    buffer: NonNull<Frame>,
    capacity: usize,
    read_pos: usize,
    write_pos: usize,
    size: usize,
    #[allow(dead_code)]
    memory_pool: Option<Arc<MemoryPool>>,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self::with_pool(capacity, None)
    }

    pub fn with_pool(capacity: usize, pool: Option<Arc<MemoryPool>>) -> Self {
        let buffer = if let Some(ref pool) = pool {
            if let Some(ptr) = pool.allocate(capacity * std::mem::size_of::<Frame>(), 16) {
                unsafe {
                    let frame_ptr = ptr.as_ptr() as *mut Frame;
                    std::slice::from_raw_parts_mut(frame_ptr, capacity).fill(Frame::ZERO);
                    NonNull::new_unchecked(frame_ptr)
                }
            } else {
                let mut vec = Vec::with_capacity(capacity);
                vec.resize(capacity, Frame::ZERO);
                NonNull::new(vec.leak().as_mut_ptr()).unwrap()
            }
        } else {
            let mut vec = Vec::with_capacity(capacity);
            vec.resize(capacity, Frame::ZERO);
            NonNull::new(vec.leak().as_mut_ptr()).unwrap()
        };

        Self {
            buffer,
            capacity,
            read_pos: 0,
            write_pos: 0,
            size: 0,
            memory_pool: pool,
        }
    }

    pub fn write(&mut self, data: &[Frame]) -> usize {
        let available = self.capacity - self.size;
        let to_write = data.len().min(available);

        unsafe {
            let buffer_slice = std::slice::from_raw_parts_mut(self.buffer.as_ptr(), self.capacity);
            for &frame in data.iter().take(to_write) {
                buffer_slice[self.write_pos] = frame;
                self.write_pos = (self.write_pos + 1) % self.capacity;
            }
        }

        self.size += to_write;
        to_write
    }

    pub fn read(&mut self, data: &mut [Frame]) -> usize {
        let to_read = data.len().min(self.size);

        unsafe {
            let buffer_slice = std::slice::from_raw_parts(self.buffer.as_ptr(), self.capacity);
            for frame in data.iter_mut().take(to_read) {
                *frame = buffer_slice[self.read_pos];
                self.read_pos = (self.read_pos + 1) % self.capacity;
            }
        }

        self.size -= to_read;
        to_read
    }

    pub fn peek(&self, data: &mut [Frame]) -> usize {
        let to_peek = data.len().min(self.size);
        let mut pos = self.read_pos;

        unsafe {
            let buffer_slice = std::slice::from_raw_parts(self.buffer.as_ptr(), self.capacity);
            for frame in data.iter_mut().take(to_peek) {
                *frame = buffer_slice[pos];
                pos = (pos + 1) % self.capacity;
            }
        }

        to_peek
    }

    pub fn available_read(&self) -> usize {
        self.size
    }

    pub fn available_write(&self) -> usize {
        self.capacity - self.size
    }

    pub fn clear(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
        self.size = 0;
    }
}

unsafe impl Send for RingBuffer {}
unsafe impl Sync for RingBuffer {}

pub struct DelayLine {
    buffer: NonNull<Frame>,
    capacity: usize,
    write_pos: usize,
    #[allow(dead_code)]
    memory_pool: Option<Arc<MemoryPool>>,
}

impl DelayLine {
    pub fn new(max_delay_samples: usize) -> Self {
        Self::with_pool(max_delay_samples, None)
    }

    pub fn with_pool(max_delay_samples: usize, pool: Option<Arc<MemoryPool>>) -> Self {
        let buffer = if let Some(ref pool) = pool {
            if let Some(ptr) = pool.allocate(max_delay_samples * std::mem::size_of::<Frame>(), 16) {
                unsafe {
                    let frame_ptr = ptr.as_ptr() as *mut Frame;
                    std::slice::from_raw_parts_mut(frame_ptr, max_delay_samples).fill(Frame::ZERO);
                    NonNull::new_unchecked(frame_ptr)
                }
            } else {
                let mut vec = Vec::with_capacity(max_delay_samples);
                vec.resize(max_delay_samples, Frame::ZERO);
                NonNull::new(vec.leak().as_mut_ptr()).unwrap()
            }
        } else {
            let mut vec = Vec::with_capacity(max_delay_samples);
            vec.resize(max_delay_samples, Frame::ZERO);
            NonNull::new(vec.leak().as_mut_ptr()).unwrap()
        };

        Self {
            buffer,
            capacity: max_delay_samples,
            write_pos: 0,
            memory_pool: pool,
        }
    }

    pub fn write(&mut self, sample: Frame) {
        unsafe {
            let buffer_slice = std::slice::from_raw_parts_mut(self.buffer.as_ptr(), self.capacity);
            buffer_slice[self.write_pos] = sample;
        }
        self.write_pos = (self.write_pos + 1) % self.capacity;
    }

    pub fn read(&self, delay_samples: usize) -> Frame {
        let delay = delay_samples.min(self.capacity - 1);
        let read_pos = (self.write_pos + self.capacity - delay - 1) % self.capacity;
        unsafe {
            let buffer_slice = std::slice::from_raw_parts(self.buffer.as_ptr(), self.capacity);
            buffer_slice[read_pos]
        }
    }

    pub fn read_interpolated(&self, delay_samples: f32) -> Frame {
        let delay = delay_samples.min(self.capacity as f32 - 1.0);
        let delay_int = delay as usize;
        let delay_frac = delay - delay_int as f32;

        let pos1 = (self.write_pos + self.capacity - delay_int - 1) % self.capacity;
        let pos2 = (self.write_pos + self.capacity - delay_int - 2) % self.capacity;

        unsafe {
            let buffer_slice = std::slice::from_raw_parts(self.buffer.as_ptr(), self.capacity);
            let sample1 = buffer_slice[pos1];
            let sample2 = buffer_slice[pos2];

            Frame::new(
                sample1.left + delay_frac * (sample2.left - sample1.left),
                sample1.right + delay_frac * (sample2.right - sample1.right),
            )
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            let buffer_slice = std::slice::from_raw_parts_mut(self.buffer.as_ptr(), self.capacity);
            buffer_slice.fill(Frame::ZERO);
        }
        self.write_pos = 0;
    }
}

unsafe impl Send for DelayLine {}
unsafe impl Sync for DelayLine {}
