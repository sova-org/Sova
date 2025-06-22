pub mod global;
pub mod local;
pub mod source;

#[derive(Debug, Clone)]
pub struct ParameterDescriptor {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub min_value: f32,
    pub max_value: f32,
    pub default_value: f32,
    pub unit: &'static str,
    pub description: &'static str,
    pub modulable: bool,
}

impl ParameterDescriptor {
    // Fast parameter name matching - avoids string allocation and slice iteration
    #[inline]
    pub fn matches_name(&self, param: &str) -> bool {
        // Use pointer equality first for common case (faster than string comparison)
        if std::ptr::eq(self.name.as_ptr(), param.as_ptr()) && self.name.len() == param.len() {
            return true;
        }

        // Fast string comparison for main name
        if self.name == param {
            return true;
        }

        // Check aliases only if necessary - most parameters don't have aliases
        if !self.aliases.is_empty() {
            for &alias in self.aliases {
                if alias == param {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub left: f32,
    pub right: f32,
}

impl Frame {
    pub const ZERO: Frame = Frame {
        left: 0.0,
        right: 0.0,
    };

    pub fn new(left: f32, right: f32) -> Self {
        Self { left, right }
    }

    pub fn mono(value: f32) -> Self {
        Self {
            left: value,
            right: value,
        }
    }

    #[inline]
    pub fn add_assign(&mut self, other: &Frame) {
        self.left += other.left;
        self.right += other.right;
    }

    #[inline]
    pub fn mul_scalar(&self, scalar: f32) -> Frame {
        Frame {
            left: self.left * scalar,
            right: self.right * scalar,
        }
    }

    #[inline]
    pub fn mul_assign_scalar(&mut self, scalar: f32) {
        self.left *= scalar;
        self.right *= scalar;
    }

    #[inline]
    pub fn process_block_add(dest: &mut [Frame], src: &[Frame]) {
        let len = dest.len().min(src.len());
        let (dest_aligned, dest_rest) = dest[..len].split_at_mut(len & !3);
        let (src_aligned, src_rest) = src[..len].split_at(len & !3);

        for chunk in dest_aligned
            .chunks_exact_mut(4)
            .zip(src_aligned.chunks_exact(4))
        {
            let (dest_chunk, src_chunk) = chunk;
            dest_chunk[0].add_assign(&src_chunk[0]);
            dest_chunk[1].add_assign(&src_chunk[1]);
            dest_chunk[2].add_assign(&src_chunk[2]);
            dest_chunk[3].add_assign(&src_chunk[3]);
        }

        for (d, s) in dest_rest.iter_mut().zip(src_rest.iter()) {
            d.add_assign(s);
        }
    }

    #[inline]
    pub fn process_block_mul_scalar(dest: &mut [Frame], scalar: f32) {
        let (dest_aligned, dest_rest) = dest.split_at_mut(dest.len() & !3);

        for chunk in dest_aligned.chunks_exact_mut(4) {
            chunk[0].mul_assign_scalar(scalar);
            chunk[1].mul_assign_scalar(scalar);
            chunk[2].mul_assign_scalar(scalar);
            chunk[3].mul_assign_scalar(scalar);
        }

        for frame in dest_rest {
            frame.mul_assign_scalar(scalar);
        }
    }

    #[inline]
    pub fn process_block_zero(dest: &mut [Frame]) {
        let (dest_aligned, dest_rest) = dest.split_at_mut(dest.len() & !3);

        for chunk in dest_aligned.chunks_exact_mut(4) {
            chunk[0] = Frame::ZERO;
            chunk[1] = Frame::ZERO;
            chunk[2] = Frame::ZERO;
            chunk[3] = Frame::ZERO;
        }

        for frame in dest_rest {
            *frame = Frame::ZERO;
        }
    }
}

impl std::ops::Index<usize> for Frame {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.left,
            1 => &self.right,
            _ => &self.left,
        }
    }
}

impl std::ops::IndexMut<usize> for Frame {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.left,
            1 => &mut self.right,
            _ => &mut self.left,
        }
    }
}

impl From<[f32; 2]> for Frame {
    fn from(arr: [f32; 2]) -> Self {
        Self {
            left: arr[0],
            right: arr[1],
        }
    }
}

impl From<Frame> for [f32; 2] {
    fn from(frame: Frame) -> Self {
        [frame.left, frame.right]
    }
}

pub trait AudioModule: Send + Sync {
    fn get_name(&self) -> &'static str;
    fn get_parameter_descriptors(&self) -> &[ParameterDescriptor];
    fn set_parameter(&mut self, param: &str, value: f32) -> bool;
    fn is_active(&self) -> bool;
}

pub trait Source: AudioModule {
    fn generate(&mut self, buffer: &mut [Frame], sample_rate: f32);
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub trait LocalEffect: AudioModule {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32);
}

pub trait GlobalEffect: AudioModule {
    fn process(&mut self, buffer: &mut [Frame], sample_rate: f32);
}

pub trait ModuleMetadata {
    fn get_static_name() -> &'static str;
    fn get_static_parameter_descriptors() -> &'static [ParameterDescriptor];
}
