//! Predictive Sample Loading System
//!
//! This module implements aggressive predictive loading to minimize sample loading
//! latency for large sample libraries (10GB+). It maintains real-time safety by
//! never performing I/O in the audio thread, using silence fallback with hot
//! sample replacement when loading completes.

use crate::memory::samplib::SampleLibrary;
use crate::types::VoiceId;
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use dashmap::DashMap;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Priority levels for sample loading requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadPriority {
    /// User just triggered this sample - highest priority
    Immediate = 0,
    /// Likely to be used soon based on patterns - high priority  
    Predicted = 1,
    /// Background preloading - medium priority
    Preload = 2,
    /// Fill cache when idle - lowest priority
    Background = 3,
}

/// Request to load a sample in the background
#[derive(Debug, Clone)]
pub struct LoadRequest {
    pub sample_name: String,
    pub sample_index: usize,
    pub priority: LoadPriority,
    pub requester: Option<VoiceId>,
    pub request_time: Instant,
}

impl PartialEq for LoadRequest {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.request_time == other.request_time
    }
}

impl Eq for LoadRequest {}

impl PartialOrd for LoadRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LoadRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First by priority, then by time (earlier requests first)
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => self.request_time.cmp(&other.request_time),
            other => other,
        }
    }
}

/// Message sent when a sample finishes loading
#[derive(Debug, Clone)]
pub struct LoadedSampleMessage {
    pub sample_name: String,
    pub sample_index: usize,
    pub sample_data: Option<Vec<f32>>,
    pub requester: Option<VoiceId>,
    pub load_result: LoadResult,
}

#[derive(Debug, Clone)]
pub enum LoadResult {
    Success,
    NotFound,
    IoError(String),
}

/// Usage pattern tracking for prediction
#[derive(Debug, Clone)]
pub struct UsagePattern {
    pub frequency: f32,
    pub last_used: Instant,
    pub co_occurrences: HashMap<String, f32>, // sample_name -> probability
    pub usage_count: u32,
}

impl Default for UsagePattern {
    fn default() -> Self {
        Self {
            frequency: 0.0,
            last_used: Instant::now(),
            co_occurrences: HashMap::new(),
            usage_count: 0,
        }
    }
}

/// Tracks usage patterns to predict what samples will be needed
pub struct SamplePredictor {
    /// Recent sample usage history (last 100 samples)
    recent_samples: VecDeque<(String, usize, Instant)>,

    /// Per-sample usage patterns
    usage_patterns: DashMap<String, UsagePattern>,

    /// Sequence patterns (what typically follows what)
    sequence_patterns: DashMap<String, HashMap<String, f32>>,

    /// Series detection (kick_001 -> kick_002, etc.)
    series_cache: DashMap<String, Vec<String>>,

    /// Folder affinity tracking
    folder_patterns: DashMap<String, f32>,

    /// Access counter for LRU
    access_counter: AtomicU64,
}

impl SamplePredictor {
    pub fn new() -> Self {
        Self {
            recent_samples: VecDeque::with_capacity(100),
            usage_patterns: DashMap::new(),
            sequence_patterns: DashMap::new(),
            series_cache: DashMap::new(),
            folder_patterns: DashMap::new(),
            access_counter: AtomicU64::new(0),
        }
    }

    /// Called when a sample is triggered - updates patterns and triggers predictions
    pub fn on_sample_triggered(
        &mut self,
        sample_name: &str,
        sample_index: usize,
        loader: &BackgroundSampleLoader,
    ) {
        let now = Instant::now();
        let counter = self.access_counter.fetch_add(1, Ordering::Relaxed);

        // Update usage pattern
        self.update_usage_pattern(sample_name, now);

        // Update sequence patterns based on recent history
        self.update_sequence_patterns(sample_name);

        // Add to recent samples
        let key = format!("{}:{}", sample_name, sample_index);
        self.recent_samples
            .push_back((key.clone(), counter as usize, now));
        if self.recent_samples.len() > 100 {
            self.recent_samples.pop_front();
        }

        // Trigger predictions
        self.predict_and_load_samples(sample_name, sample_index, loader);
    }

    fn update_usage_pattern(&self, sample_name: &str, now: Instant) {
        let mut pattern = self
            .usage_patterns
            .entry(sample_name.to_string())
            .or_insert_with(UsagePattern::default);

        pattern.usage_count += 1;
        pattern.last_used = now;

        // Simple frequency calculation (could be more sophisticated)
        pattern.frequency = pattern.usage_count as f32 / 100.0; // normalize
    }

    fn update_sequence_patterns(&self, current_sample: &str) {
        // Look at the last few samples to find patterns
        if let Some((prev_sample, _, _)) = self.recent_samples.back() {
            let mut patterns = self
                .sequence_patterns
                .entry(prev_sample.clone())
                .or_insert_with(HashMap::new);

            let count = patterns.entry(current_sample.to_string()).or_insert(0.0);
            *count += 1.0;

            // Normalize probabilities
            let total: f32 = patterns.values().sum();
            if total > 0.0 {
                for prob in patterns.values_mut() {
                    *prob /= total;
                }
            }
        }
    }

    fn predict_and_load_samples(
        &self,
        sample_name: &str,
        sample_index: usize,
        loader: &BackgroundSampleLoader,
    ) {
        // 1. Series prediction (kick_001 -> kick_002, kick_003...)
        self.predict_series(sample_name, sample_index, loader);

        // 2. Sequence prediction (what typically follows this sample)
        self.predict_sequences(sample_name, loader);

        // 3. Folder affinity (load more from same folder)
        self.predict_folder_siblings(sample_name, loader);
    }

    fn predict_series(
        &self,
        sample_name: &str,
        _sample_index: usize,
        loader: &BackgroundSampleLoader,
    ) {
        // Check if this looks like a numbered series
        if let Some((base, num)) = self.extract_numbered_pattern(sample_name) {
            let current_key = format!("{}{:03}", base, num);

            // Cache or generate series list
            let series = self.series_cache.entry(base.clone()).or_insert_with(|| {
                // Generate likely series members
                let mut candidates = Vec::new();
                for i in 0..=20 {
                    // Check for up to 20 variations
                    candidates.push(format!("{}{:03}", base, i));
                    candidates.push(format!("{}{:02}", base, i));
                    candidates.push(format!("{}{}", base, i));
                }
                candidates
            });

            // Load next few in series
            for next_candidate in series.iter().skip(num).take(5) {
                if next_candidate != &current_key {
                    loader.request_load(LoadRequest {
                        sample_name: next_candidate.clone(),
                        sample_index: 0, // Series typically uses index 0
                        priority: LoadPriority::Predicted,
                        requester: None,
                        request_time: Instant::now(),
                    });
                }
            }
        }
    }

    fn predict_sequences(&self, sample_name: &str, loader: &BackgroundSampleLoader) {
        // Look up what typically follows this sample
        if let Some(patterns) = self.sequence_patterns.get(sample_name) {
            for (next_sample, probability) in patterns.iter() {
                if *probability > 0.2 {
                    // 20% threshold for prediction
                    loader.request_load(LoadRequest {
                        sample_name: next_sample.clone(),
                        sample_index: 0,
                        priority: LoadPriority::Predicted,
                        requester: None,
                        request_time: Instant::now(),
                    });
                }
            }
        }
    }

    fn predict_folder_siblings(&self, sample_name: &str, loader: &BackgroundSampleLoader) {
        // This would need access to folder structure - simplified for now
        // In practice, would scan folder and load siblings

        // For now, just try common variations
        let variations = [
            format!("{}_002", sample_name.trim_end_matches("_001")),
            format!("{}_003", sample_name.trim_end_matches("_001")),
            format!("{}_b", sample_name.trim_end_matches("_a")),
            format!("{}_alt", sample_name),
        ];

        for variation in &variations {
            if variation != sample_name {
                loader.request_load(LoadRequest {
                    sample_name: variation.clone(),
                    sample_index: 0,
                    priority: LoadPriority::Predicted,
                    requester: None,
                    request_time: Instant::now(),
                });
            }
        }
    }

    fn extract_numbered_pattern(&self, sample_name: &str) -> Option<(String, usize)> {
        // Try to extract patterns like "kick_001" -> ("kick_", 1)
        if let Some(pos) = sample_name.rfind('_') {
            let (base, suffix) = sample_name.split_at(pos + 1);
            if let Ok(num) = suffix.parse::<usize>() {
                return Some((base.to_string(), num));
            }
        }

        // Try patterns like "kick001" -> ("kick", 1)
        for i in (1..sample_name.len()).rev() {
            if let Ok(num) = sample_name[i..].parse::<usize>() {
                return Some((sample_name[..i].to_string(), num));
            }
        }

        None
    }
}

/// Background thread worker for loading samples asynchronously
pub struct BackgroundSampleLoader {
    /// Request queue for loading samples
    load_requests: Receiver<LoadRequest>,

    /// Channel to send loaded samples back to audio thread
    loaded_samples: Sender<LoadedSampleMessage>,

    /// Reference to the sample library for actual loading
    sample_library: Arc<SampleLibrary>,

    /// Worker thread handles
    worker_threads: Vec<JoinHandle<()>>,

    /// Request sender (kept for shutdown)
    request_sender: Sender<LoadRequest>,

    /// Active requests to avoid duplicates
    active_requests: Arc<DashMap<String, Instant>>,
}

impl BackgroundSampleLoader {
    pub fn new(
        sample_library: Arc<SampleLibrary>,
        loaded_samples_sender: Sender<LoadedSampleMessage>,
        num_worker_threads: usize,
    ) -> Self {
        let (request_sender, load_requests) = bounded(1000); // Bounded to prevent memory bloat
        let active_requests = Arc::new(DashMap::new());

        // Spawn worker threads
        let mut worker_threads = Vec::new();
        for worker_id in 0..num_worker_threads {
            let requests = load_requests.clone();
            let loaded_tx = loaded_samples_sender.clone();
            let library = sample_library.clone();
            let active = active_requests.clone();

            let handle = thread::Builder::new()
                .name(format!("sample-loader-{}", worker_id))
                .spawn(move || {
                    Self::worker_loop(worker_id, requests, loaded_tx, library, active);
                })
                .expect("Failed to spawn sample loader worker thread");

            worker_threads.push(handle);
        }

        Self {
            load_requests,
            loaded_samples: loaded_samples_sender,
            sample_library,
            worker_threads,
            request_sender,
            active_requests,
        }
    }

    /// Request a sample to be loaded in the background
    pub fn request_load(&self, request: LoadRequest) {
        let key = format!("{}:{}", request.sample_name, request.sample_index);

        // Check if already loading or recently loaded
        if let Some(existing_time) = self.active_requests.get(&key) {
            if existing_time.elapsed() < Duration::from_secs(1) {
                return; // Skip duplicate requests within 1 second
            }
        }

        self.active_requests.insert(key, request.request_time);

        // Try to send request (non-blocking)
        if let Err(_) = self.request_sender.try_send(request) {
            // Queue is full - this is expected under heavy load
            eprintln!("Sample loader queue full - dropping request");
        }
    }

    fn worker_loop(
        worker_id: usize,
        requests: Receiver<LoadRequest>,
        loaded_tx: Sender<LoadedSampleMessage>,
        library: Arc<SampleLibrary>,
        active_requests: Arc<DashMap<String, Instant>>,
    ) {
        let mut request_queue = BinaryHeap::new();

        loop {
            // Collect pending requests with priority ordering
            while let Ok(request) = requests.try_recv() {
                request_queue.push(Reverse(request));
            }

            if let Some(Reverse(request)) = request_queue.pop() {
                let key = format!("{}:{}", request.sample_name, request.sample_index);

                // Load the sample
                let start_time = Instant::now();
                let sample_data = library.get_sample(&request.sample_name, request.sample_index);
                let load_time = start_time.elapsed();

                let result = if sample_data.is_some() {
                    LoadResult::Success
                } else {
                    LoadResult::NotFound
                };

                // Send result back to audio thread
                let message = LoadedSampleMessage {
                    sample_name: request.sample_name.clone(),
                    sample_index: request.sample_index,
                    sample_data,
                    requester: request.requester,
                    load_result: result,
                };

                if let Err(_) = loaded_tx.try_send(message) {
                    eprintln!("Failed to send loaded sample message - audio thread may be busy");
                }

                // Remove from active requests
                active_requests.remove(&key);

                // Log slow loads for debugging
                if load_time > Duration::from_millis(100) {
                    eprintln!(
                        "Worker {}: Slow sample load: {} took {:?}",
                        worker_id, key, load_time
                    );
                }
            } else {
                // No requests, sleep briefly to avoid busy-waiting
                thread::sleep(Duration::from_millis(1));
            }
        }
    }

    /// Get statistics about loader performance
    pub fn get_stats(&self) -> LoaderStats {
        LoaderStats {
            active_requests: self.active_requests.len(),
            queue_size: self.load_requests.len(),
            worker_count: self.worker_threads.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoaderStats {
    pub active_requests: usize,
    pub queue_size: usize,
    pub worker_count: usize,
}

/// Result of attempting to get a sample immediately
#[derive(Debug)]
pub enum SampleResult {
    /// Sample is ready to use immediately
    Ready(Vec<f32>),
    /// Sample is being loaded, voice will be silent until ready
    Loading(String, usize), // sample_name, sample_index
    /// Sample not found in library
    NotFound,
}

/// Manages predictive sample loading with real-time safety
pub struct PredictiveSampleManager {
    /// Core sample library
    sample_library: Arc<SampleLibrary>,

    /// Background loader
    background_loader: BackgroundSampleLoader,

    /// Usage pattern predictor
    predictor: SamplePredictor,

    /// Channel to receive loaded samples
    loaded_sample_receiver: Receiver<LoadedSampleMessage>,

    /// Pending voice tracking
    pending_voices: DashMap<VoiceId, PendingVoice>,
}

#[derive(Debug, Clone)]
pub struct PendingVoice {
    pub voice_id: VoiceId,
    pub sample_name: String,
    pub sample_index: usize,
    pub start_time: Instant,
    pub parameters: HashMap<String, f32>,
}

impl PredictiveSampleManager {
    pub fn new(sample_library: Arc<SampleLibrary>, num_worker_threads: usize) -> Self {
        let (loaded_tx, loaded_rx) = unbounded();

        let background_loader =
            BackgroundSampleLoader::new(sample_library.clone(), loaded_tx, num_worker_threads);

        Self {
            sample_library,
            background_loader,
            predictor: SamplePredictor::new(),
            loaded_sample_receiver: loaded_rx,
            pending_voices: DashMap::new(),
        }
    }

    /// Get a sample for immediate use (real-time safe)
    pub fn get_sample_immediate(&mut self, sample_name: &str, sample_index: usize) -> SampleResult {
        // Try lock-free access first
        if let Some(sample_data) = self
            .sample_library
            .get_sample_lockfree(sample_name, sample_index)
        {
            // Update predictor and trigger background loading
            self.predictor
                .on_sample_triggered(sample_name, sample_index, &self.background_loader);
            return SampleResult::Ready(sample_data.to_vec());
        }

        // Sample not immediately available - request background loading
        self.background_loader.request_load(LoadRequest {
            sample_name: sample_name.to_string(),
            sample_index,
            priority: LoadPriority::Immediate,
            requester: None,
            request_time: Instant::now(),
        });

        // Trigger predictive loading
        self.predictor
            .on_sample_triggered(sample_name, sample_index, &self.background_loader);

        SampleResult::Loading(sample_name.to_string(), sample_index)
    }

    /// Register a voice as pending sample load
    pub fn register_pending_voice(
        &self,
        voice_id: VoiceId,
        sample_name: String,
        sample_index: usize,
    ) {
        self.pending_voices.insert(
            voice_id,
            PendingVoice {
                voice_id,
                sample_name: sample_name.clone(),
                sample_index,
                start_time: Instant::now(),
                parameters: HashMap::new(),
            },
        );

        // Request immediate loading for this voice
        self.background_loader.request_load(LoadRequest {
            sample_name,
            sample_index,
            priority: LoadPriority::Immediate,
            requester: Some(voice_id),
            request_time: Instant::now(),
        });
    }

    /// Process loaded samples and update pending voices (called from audio thread)
    pub fn update_pending_samples(&self) -> Vec<(VoiceId, Vec<f32>)> {
        let mut ready_samples = Vec::new();

        // Process all available loaded samples
        while let Ok(loaded_msg) = self.loaded_sample_receiver.try_recv() {
            if let LoadResult::Success = loaded_msg.load_result {
                if let Some(sample_data) = loaded_msg.sample_data {
                    // Find pending voices waiting for this sample
                    let mut voices_to_update = Vec::new();

                    for entry in self.pending_voices.iter() {
                        let pending = entry.value();
                        if pending.sample_name == loaded_msg.sample_name
                            && pending.sample_index == loaded_msg.sample_index
                        {
                            voices_to_update.push(pending.voice_id);
                        }
                    }

                    // Prepare updates
                    for voice_id in voices_to_update {
                        ready_samples.push((voice_id, sample_data.clone()));
                        self.pending_voices.remove(&voice_id);
                    }
                }
            }
        }

        ready_samples
    }

    /// Get loader statistics
    pub fn get_stats(&self) -> (LoaderStats, usize) {
        (
            self.background_loader.get_stats(),
            self.pending_voices.len(),
        )
    }

    /// Preload common samples on startup
    pub fn preload_common_samples(&self) {
        let common_patterns = [
            "kick", "snare", "hihat", "openhat", "crash", "ride", "808", "bass", "lead", "pad",
            "arp", "pluck", "break", "loop", "perc", "fx",
        ];

        for pattern in &common_patterns {
            // Try to load first few samples of each pattern
            for i in 0..3 {
                self.background_loader.request_load(LoadRequest {
                    sample_name: pattern.to_string(),
                    sample_index: i,
                    priority: LoadPriority::Preload,
                    requester: None,
                    request_time: Instant::now(),
                });
            }
        }
    }
}
