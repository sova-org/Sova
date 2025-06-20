use crate::memory::pool::MemoryPool;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_SAMPLE_LENGTH: usize = 44100 * 10;

struct Sample {
    data: NonNull<f32>,
    frames: usize,
    last_used: u64,
}

pub struct SampleLibrary {
    pool: MemoryPool,
    loaded_samples: DashMap<PathBuf, Sample>,
    folder_index: DashMap<String, Vec<PathBuf>>,
    max_loaded: usize,
    access_counter: AtomicU64,
    root_path: PathBuf,
    target_sample_rate: u32,
}

impl SampleLibrary {
    pub fn new(max_loaded: usize, root_path: &str, target_sample_rate: u32) -> Self {
        let pool_size = max_loaded * MAX_SAMPLE_LENGTH * 2 * 4 + 1024 * 1024;
        let pool = MemoryPool::new(pool_size);
        let root = PathBuf::from(root_path);

        let library = Self {
            pool,
            loaded_samples: DashMap::new(),
            folder_index: DashMap::new(),
            max_loaded,
            access_counter: AtomicU64::new(0),
            root_path: root.clone(),
            target_sample_rate,
        };

        if root.exists() {
            library.scan_folders();
        }

        library
    }

    fn scan_folders(&self) {
        self.folder_index.clear();

        if let Ok(entries) = std::fs::read_dir(&self.root_path) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    let mut samples = Vec::new();

                    if let Ok(files) = std::fs::read_dir(entry.path()) {
                        for file in files.flatten() {
                            if let Some(ext) = file.path().extension() {
                                if ext == "wav" {
                                    samples.push(file.path());
                                }
                            }
                        }
                    }

                    samples.sort();
                    self.folder_index.insert(folder_name, samples);
                }
            }
        }
    }

    pub fn get_sample(&self, folder: &str, index: usize) -> Option<Vec<f32>> {
        let samples_ref = self.folder_index.get(folder)?;
        let wrapped_index = index % samples_ref.len();
        let path = samples_ref.get(wrapped_index)?.clone();
        drop(samples_ref);
        self.load_sample(&path)
    }

    pub fn load_sample(&self, path: &Path) -> Option<Vec<f32>> {
        let counter = self.access_counter.fetch_add(1, Ordering::Relaxed);

        if let Some(mut sample_ref) = self.loaded_samples.get_mut(path) {
            sample_ref.last_used = counter;
            let data = unsafe {
                std::slice::from_raw_parts(
                    sample_ref.data.as_ptr(),
                    sample_ref.frames * 2,
                )
            };
            return Some(data.to_vec());
        }

        if self.loaded_samples.len() >= self.max_loaded {
            self.evict_oldest();
        }

        let (audio_data, channels) = self.read_wav_with_channels(path)?;

        let stereo_data = self.to_stereo(&audio_data, channels);

        let frames = stereo_data.len() / 2;

        let data_ptr = self.pool.allocate(stereo_data.len() * 4, 16)?;

        let data_ptr = data_ptr.cast::<f32>();

        unsafe {
            std::ptr::copy_nonoverlapping(
                stereo_data.as_ptr(),
                data_ptr.as_ptr(),
                stereo_data.len(),
            );
        }

        let sample = Sample {
            data: data_ptr,
            frames,
            last_used: counter,
        };

        self.loaded_samples.insert(path.to_path_buf(), sample);

        Some(stereo_data)
    }

    fn evict_oldest(&self) {
        let oldest_path = self
            .loaded_samples
            .iter()
            .min_by_key(|entry| entry.value().last_used)
            .map(|entry| entry.key().clone());
        
        if let Some(path) = oldest_path {
            self.loaded_samples.remove(&path);
        }
    }

    fn read_wav_with_channels(&self, path: &Path) -> Option<(Vec<f32>, u16)> {
        let mut reader = hound::WavReader::open(path).ok()?;
        let spec = reader.spec();

        // Validate sample format
        if !self.is_supported_format(&spec) {
            eprintln!(
                "[ENGINE ERROR] Unsupported format in {}: {:?}",
                path.display(),
                spec
            );
            return None;
        }

        let mut samples = Vec::new();

        match spec.sample_format {
            hound::SampleFormat::Float => {
                for sample in reader.samples::<f32>() {
                    samples.push(sample.ok()?);
                }
            }
            hound::SampleFormat::Int => match spec.bits_per_sample {
                16 => {
                    for sample in reader.samples::<i16>() {
                        let raw = sample.ok()? as f32;
                        let normalized = raw / 32768.0;
                        samples.push(normalized);
                    }
                }
                24 => {
                    for sample in reader.samples::<i32>() {
                        let raw = sample.ok()? as f32;
                        let normalized = raw / 8388608.0;
                        samples.push(normalized);
                    }
                }
                32 => {
                    for sample in reader.samples::<i32>() {
                        let raw = sample.ok()? as f32;
                        let normalized = raw / 2147483648.0;
                        samples.push(normalized);
                    }
                }
                8 => {
                    for sample in reader.samples::<i8>() {
                        let raw = sample.ok()? as f32;
                        let normalized = raw / 128.0;
                        samples.push(normalized);
                    }
                }
                _ => return None,
            },
        }

        if samples.len() > MAX_SAMPLE_LENGTH * spec.channels as usize {
            samples.truncate(MAX_SAMPLE_LENGTH * spec.channels as usize);
        }

        if spec.sample_rate != self.target_sample_rate {
            samples = self.simple_resample(&samples, spec.sample_rate, spec.channels);
        }

        Some((samples, spec.channels))
    }

    fn to_stereo(&self, audio: &[f32], channels: u16) -> Vec<f32> {
        match channels {
            1 => {
                let mut stereo = Vec::with_capacity(audio.len() * 2);
                for &sample in audio {
                    stereo.push(sample);
                    stereo.push(sample);
                }
                stereo
            }
            2 => audio.to_vec(),
            _ => {
                let mut stereo = Vec::with_capacity((audio.len() / channels as usize) * 2);
                for chunk in audio.chunks_exact(channels as usize) {
                    let left = chunk[0];
                    let right = if channels > 1 { chunk[1] } else { chunk[0] };
                    stereo.push(left);
                    stereo.push(right);
                }
                stereo
            }
        }
    }

    fn simple_resample(&self, samples: &[f32], from_rate: u32, channels: u16) -> Vec<f32> {
        if from_rate == self.target_sample_rate {
            return samples.to_vec();
        }

        let ratio = self.target_sample_rate as f32 / from_rate as f32;
        let frames_in = samples.len() / channels as usize;
        let frames_out = (frames_in as f32 * ratio) as usize;
        let mut output = Vec::with_capacity(frames_out * channels as usize);

        for frame_out in 0..frames_out {
            let src_frame = frame_out as f32 / ratio;
            let frame_idx = src_frame.floor() as usize;
            let frac = src_frame - frame_idx as f32;

            for ch in 0..channels {
                let ch_idx = ch as usize;
                let idx1 = frame_idx * channels as usize + ch_idx;
                let idx2 = ((frame_idx + 1).min(frames_in - 1)) * channels as usize + ch_idx;

                let sample1 = samples.get(idx1).copied().unwrap_or(0.0);
                let sample2 = samples.get(idx2).copied().unwrap_or(sample1);

                let interpolated = sample1 + (sample2 - sample1) * frac;
                output.push(interpolated);
            }
        }

        output
    }

    fn is_supported_format(&self, spec: &hound::WavSpec) -> bool {
        // Check sample rate is reasonable
        if spec.sample_rate < 8000 || spec.sample_rate > 192000 {
            return false;
        }

        // Check channel count
        if spec.channels == 0 || spec.channels > 8 {
            return false;
        }

        // Check bit depth
        match spec.sample_format {
            hound::SampleFormat::Float => spec.bits_per_sample == 32 || spec.bits_per_sample == 64,
            hound::SampleFormat::Int => {
                matches!(spec.bits_per_sample, 8 | 16 | 24 | 32)
            }
        }
    }

    pub fn get_folders(&self) -> Vec<String> {
        self.folder_index.iter().map(|entry| entry.key().clone()).collect()
    }

    pub fn get_folder_size(&self, folder: &str) -> usize {
        self.folder_index.get(folder).map(|v| v.len()).unwrap_or(0)
    }

    pub fn is_loaded(&self, path: &Path) -> bool {
        self.loaded_samples.contains_key(path)
    }

    pub fn reset(&self) {
        self.loaded_samples.clear();
        self.pool.reset();
        self.access_counter.store(0, Ordering::Relaxed);
    }

    pub fn get_all_folders(&self) -> Vec<(String, usize, usize)> {
        self.folder_index
            .iter()
            .map(|entry| {
                let name = entry.key().clone();
                let samples = entry.value();
                let total = samples.len();
                let loaded = samples
                    .iter()
                    .filter(|path| self.loaded_samples.contains_key(*path))
                    .count();
                (name, total, loaded)
            })
            .collect()
    }

    pub fn get_folder_contents(&self, folder_name: &str) -> Vec<String> {
        self.folder_index
            .get(folder_name)
            .map(|samples| {
                samples
                    .iter()
                    .filter_map(|path| path.file_name()?.to_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Lock-free sample access for real-time audio processing.
    /// Returns sample data if already loaded, None if not yet available.
    pub fn get_sample_lockfree(&self, folder: &str, index: usize) -> Option<&[f32]> {
        let samples_ref = self.folder_index.get(folder)?;
        let wrapped_index = index % samples_ref.len();
        let path = samples_ref.get(wrapped_index)?;
        
        if let Some(sample_ref) = self.loaded_samples.get(path) {
            unsafe {
                Some(std::slice::from_raw_parts(
                    sample_ref.data.as_ptr(),
                    sample_ref.frames * 2,
                ))
            }
        } else {
            None
        }
    }

    pub fn preload_samples(&self) {
        let folder_index = self.folder_index.clone();
        let folder_count = folder_index.len();
        for entry in folder_index.iter() {
            let samples = entry.value();
            let samples_to_load = (self.max_loaded / folder_count.max(1)).max(1);

            for sample_path in samples.iter().take(samples_to_load) {
                if self.loaded_samples.len() >= self.max_loaded {
                    break;
                }
                let _ = self.load_sample(sample_path);
            }
        }
    }

    /// Preload ALL samples from all folders to eliminate runtime loading.
    /// This ensures no mutex contention during audio processing.
    pub fn preload_all_samples(&self) {
        println!("Pre-loading all samples to eliminate runtime mutex usage...");
        let folder_index = self.folder_index.clone();
        let mut total_loaded = 0;

        for entry in folder_index.iter() {
            let folder_name = entry.key();
            let samples = entry.value();
            let mut folder_loaded = 0;
            for sample_path in samples.iter() {
                if self.loaded_samples.len() >= self.max_loaded {
                    println!(
                        "WARNING: Reached max sample limit ({}) - some samples not loaded",
                        self.max_loaded
                    );
                    break;
                }

                if self.load_sample(sample_path).is_some() {
                    folder_loaded += 1;
                    total_loaded += 1;
                }
            }
            println!(
                "  Loaded {} samples from folder '{}'",
                folder_loaded, folder_name
            );
        }

        println!("Successfully pre-loaded {} total samples", total_loaded);
    }
}

unsafe impl Send for SampleLibrary {}
unsafe impl Sync for SampleLibrary {}
