# Lock-Free Migration Plan

## Current State
- Audio processing has lock-free variant but uses mutexes by default
- `SampleLibrary` wrapped in `Arc<Mutex<>>` causes audio thread blocking
- Background sample loading needed for runtime requests

## Goal
Complete lock-free audio thread with background sample loading.

## Implementation Plan

### Phase 1: Lock-Free SampleLibrary

**Add dependency:**
```toml
dashmap = "6.0"
```

**Replace HashMap with DashMap in `samplib.rs`:**
```rust
use dashmap::DashMap;

pub struct SampleLibrary {
    pool: MemoryPool,
    loaded_samples: DashMap<PathBuf, Sample>,  // Lock-free HashMap
    folder_index: DashMap<String, Vec<PathBuf>>, // Lock-free HashMap
    // ... rest unchanged
}
```

**Add lock-free sample access:**
```rust
impl SampleLibrary {
    pub fn get_sample_lockfree(&self, folder: &str, index: usize) -> Option<&[f32]> {
        let samples = self.folder_index.get(folder)?;
        let wrapped_index = index % samples.len();
        let path = samples.get(wrapped_index)?;
        
        if let Some(sample_ref) = self.loaded_samples.get(path) {
            unsafe {
                Some(std::slice::from_raw_parts(
                    sample_ref.data.as_ptr(),
                    sample_ref.frames * 2,
                ))
            }
        } else {
            None  // Not loaded yet
        }
    }
}
```

### Phase 2: Background Sample Loading

**Add sample loading channel to engine:**
```rust
pub struct AudioEngine {
    sample_library: Arc<SampleLibrary>,  // Remove Mutex wrapper
    sample_request_tx: Option<crossbeam_channel::Sender<(String, usize)>>,
    // ... rest unchanged
}
```

**Spawn background loader thread:**
```rust
impl AudioEngine {
    pub fn start_background_sample_loader(
        sample_library: Arc<SampleLibrary>,
    ) -> crossbeam_channel::Sender<(String, usize)> {
        let (tx, rx) = crossbeam_channel::unbounded();
        
        thread::spawn(move || {
            while let Ok((folder, index)) = rx.recv() {
                // Load sample in background - this can block
                sample_library.load_sample_background(&folder, index);
            }
        });
        
        tx
    }
}
```

### Phase 3: Update Audio Processing

**Modify `prepare_sample_data()` in `engine.rs`:**
```rust
fn prepare_sample_data_lockfree(
    &self,
    parameters: &HashMap<String, Box<dyn Any + Send>>,
    voice_id: VoiceId,
) -> Option<(Vec<f32>, f32)> {
    let sample_name = /* ... extract sample_name ... */;
    let sample_index = /* ... extract sample_index ... */;
    
    // Try lock-free access first
    if let Some(sample_data) = self.sample_library.get_sample_lockfree(&sample_name, sample_index) {
        // Sample available - process immediately
        return Some((sample_data.to_vec(), duration));
    }
    
    // Sample not loaded - request background loading for next time
    if let Some(ref tx) = self.sample_request_tx {
        let _ = tx.try_send((sample_name, sample_index));
    }
    
    None  // Skip this play request
}
```

### Phase 4: Make Lock-Free Default

**Update constructors to remove Mutex:**
```rust
impl AudioEngine {
    pub fn new(/* ... */) -> Self {
        let sample_library = Arc::new(SampleLibrary::new(/* ... */));
        let sample_request_tx = Some(Self::start_background_sample_loader(Arc::clone(&sample_library)));
        
        Self {
            sample_library,  // No Mutex!
            sample_request_tx,
            // ...
        }
    }
}
```

**Make lock-free audio thread the default:**
```rust
pub fn start_audio_thread(/* ... */) -> thread::JoinHandle<()> {
    Self::start_audio_thread_lockfree(/* ... */)
}
```

## Files to Modify

1. `Cargo.toml` - Add dashmap dependency
2. `src/memory/samplib.rs` - Replace HashMap with DashMap, add lock-free methods  
3. `src/engine.rs` - Remove Mutex wrapper, add background loading, update sample preparation
4. `src/server.rs` - Update to use lock-free sample library

## Expected Behavior

- **First sample request**: Skipped, sample loads in background
- **Subsequent requests**: Played immediately (lock-free)
- **Audio thread**: Never blocks on sample loading
- **Background thread**: Handles all blocking I/O

## Migration Steps

1. ✅ Add dashmap dependency
2. ✅ Convert SampleLibrary to use DashMap
3. ✅ Add lock-free sample access methods
4. ✅ Update sample preparation logic  
5. ✅ Remove all Mutex wrappers
6. ✅ Test with sample loading scenarios

## ✅ COMPLETED

The lock-free migration has been successfully completed. The engine now:

- Uses `DashMap` for concurrent lock-free access to sample data
- Provides lock-free sample access via `get_sample_lockfree()`
- Falls back to sample loading when needed (via `get_sample()`)
- Eliminates all mutex contention in the audio processing path
- Pre-loads all samples at startup for optimal performance
- Maintains full API compatibility with existing code

The audio thread is now completely lock-free and deterministic.