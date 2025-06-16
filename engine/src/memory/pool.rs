use std::alloc::{Layout, alloc, dealloc};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MemoryPool {
    memory: NonNull<u8>,
    size: usize,
    offset: AtomicUsize,
}

impl MemoryPool {
    pub fn new(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 64).unwrap();
        let memory = unsafe { alloc(layout) };
        assert!(!memory.is_null(), "Failed to allocate memory pool");

        Self {
            memory: NonNull::new(memory).unwrap(),
            size,
            offset: AtomicUsize::new(0),
        }
    }

    pub fn allocate(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let align_mask = align - 1;

        loop {
            let current = self.offset.load(Ordering::Acquire);
            let aligned = (current + align_mask) & !align_mask;

            if aligned + size > self.size {
                return None;
            }

            match self.offset.compare_exchange_weak(
                current,
                aligned + size,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => unsafe {
                    return Some(NonNull::new_unchecked(self.memory.as_ptr().add(aligned)));
                },
                Err(_) => continue,
            }
        }
    }

    pub fn reset(&self) {
        self.offset.store(0, Ordering::Relaxed);
    }

    pub fn used(&self) -> usize {
        self.offset.load(Ordering::Relaxed)
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.size, 64).unwrap();
        unsafe {
            dealloc(self.memory.as_ptr(), layout);
        }
    }
}

unsafe impl Send for MemoryPool {}
unsafe impl Sync for MemoryPool {}
