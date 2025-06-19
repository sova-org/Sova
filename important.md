# BuboCore Deterministic Scheduling Work Plan

## Executive Summary

Analysis of BuboCore's scheduling system reveals several critical timing precision issues that prevent achieving professional-grade deterministic sequencing. This work plan addresses timing domain coordination, real-time communication bottlenecks, and mathematical precision to achieve sub-millisecond accuracy.

## Current Critical Issues

### 1. Fixed Timing Drift (`bubocore/src/schedule.rs:35`)
- **Problem**: `SCHEDULED_DRIFT = 30_000` (30ms) creates unpredictable latency
- **Impact**: Timing inconsistencies under varying system loads
- **Priority**: CRITICAL

### 2. Scheduler State Machine (`bubocore/src/schedule.rs:960-1140`)
- **Problem**: 1ms/100ms polling intervals instead of event-driven timing
- **Impact**: Jitter accumulation and missed timing windows
- **Priority**: HIGH

### 3. AudioEngine Sample Precision (`engine/src/engine.rs:165-179`)
- **Problem**: Integer division loses sub-sample precision
- **Impact**: Timing drift in sample-accurate scheduling
- **Priority**: HIGH

### 4. Inter-Thread Communication
- **Problem**: Standard `mpsc` channels with unbounded latency
- **Impact**: Non-deterministic message delivery times
- **Priority**: HIGH

## Work Plan Phases

### Phase 1: Critical Timing Infrastructure (Week 1-2)

#### Task 1.1: Adaptive Timing Compensation
**File**: `bubocore/src/schedule.rs`
**Replace**: Fixed `SCHEDULED_DRIFT` constant

```rust
pub struct AdaptiveTimingCompensator {
    buffer_latency_micros: u64,
    system_jitter_estimate: f64,
    lookahead_window: u64,
}

impl AdaptiveTimingCompensator {
    fn calculate_schedule_offset(&self, current_load: f32) -> SyncTime {
        let base_latency = self.buffer_latency_micros;
        let jitter_compensation = (self.system_jitter_estimate * 2.0) as u64;
        let load_compensation = (current_load * 5000.0) as u64; // 5ms max
        base_latency + jitter_compensation + load_compensation
    }
}
```

**Acceptance Criteria**:
- [ ] Remove fixed 30ms drift
- [ ] Implement adaptive compensation based on buffer size
- [ ] Add system load monitoring
- [ ] Verify timing consistency under stress

#### Task 1.2: High-Precision Sample Timing
**File**: `engine/src/engine.rs`
**Replace**: `timestamp_to_sample_offset()` method

```rust
struct HighPrecisionTimer {
    fractional_samples: f64,
    sample_rate: f64,
    start_time_nanos: u128,
}

impl HighPrecisionTimer {
    fn timestamp_to_exact_sample_position(&mut self, timestamp_nanos: u128) -> (u64, f64) {
        let elapsed_nanos = timestamp_nanos - self.start_time_nanos;
        let exact_samples = (elapsed_nanos as f64 * self.sample_rate) / 1_000_000_000.0;
        let whole_samples = exact_samples.floor() as u64;
        let fractional_part = exact_samples.fract();
        (whole_samples, fractional_part)
    }
}
```

**Acceptance Criteria**:
- [ ] Replace integer division with high-precision calculation
- [ ] Maintain fractional sample accuracy
- [ ] Add nanosecond-precision timing
- [ ] Verify sub-sample accuracy in tests

### Phase 2: Real-Time Communication (Week 3)

#### Task 2.1: Lock-Free Message Queues
**Files**: `bubocore/src/schedule.rs`, `engine/src/engine.rs`
**Dependencies**: Add `crossbeam` crate

```rust
use crossbeam::queue::ArrayQueue;

pub struct RTSchedulerInterface {
    command_queue: ArrayQueue<SchedulerMessage>,
    status_queue: ArrayQueue<SchedulerNotification>,
    audio_events: ArrayQueue<(EngineMessage, u64)>, // (message, precise_timestamp)
}
```

**Acceptance Criteria**:
- [ ] Replace all `mpsc` channels with lock-free queues
- [ ] Implement bounded queue sizes for deterministic behavior
- [ ] Add queue overflow handling
- [ ] Measure latency improvements

#### Task 2.2: Real-Time Thread Priorities
**File**: `bubocore/src/schedule.rs`
**Dependencies**: Configure `thread-priority` crate

```rust
use thread_priority::{ThreadBuilder, ThreadPriority, ThreadSchedulePolicy};

let handle = ThreadBuilder::default()
    .name("BuboCore-scheduler-RT")
    .priority(ThreadPriority::Max)
    .policy(ThreadSchedulePolicy::Realtime(RealtimeThreadSchedulePolicy::Fifo))
    .spawn(move |_| { /* scheduler logic */ })?;
```

**Acceptance Criteria**:
- [ ] Set FIFO scheduling for audio thread
- [ ] Configure appropriate priorities for all threads
- [ ] Add platform-specific implementations
- [ ] Verify thread priority effectiveness

### Phase 3: Scheduler State Machine Enhancement (Week 4)

#### Task 3.1: Event-Driven Timing
**File**: `bubocore/src/schedule.rs`
**Replace**: Polling-based scheduler loop

```rust
impl Scheduler {
    fn calculate_next_critical_timestamp(&self) -> Option<SyncTime> {
        let mut next_events = Vec::with_capacity(16);
        
        // Collect all pending timing events
        for execution in &self.executions {
            next_events.push(execution.next_event_time());
        }
        
        for line in self.scene.lines.iter() {
            let (_, _, _, _, next_frame_time) = Self::frame_index(
                &self.clock, self.scene.length(), line, self.theoretical_date()
            );
            if next_frame_time < SyncTime::MAX {
                next_events.push(self.theoretical_date() + next_frame_time);
            }
        }
        
        next_events.into_iter().min()
    }
}
```

**Acceptance Criteria**:
- [ ] Replace fixed polling intervals with event-driven timing
- [ ] Calculate precise next event timestamps
- [ ] Implement efficient event priority queue
- [ ] Reduce CPU usage in idle periods

#### Task 3.2: Precise Frame Mathematics
**File**: `bubocore/src/schedule.rs`
**Dependencies**: Use existing decimal operations

```rust
use crate::util::decimal_operations::{Decimal, DecimalOperations};

struct PreciseFrameTimer {
    accumulated_beats: Decimal,
    frame_length_beats: Decimal,
    tempo_bpm: Decimal,
}

impl PreciseFrameTimer {
    fn calculate_exact_frame_position(&self, current_beat: Decimal) -> (usize, Decimal) {
        let position_in_frames = current_beat.div(&self.frame_length_beats);
        let frame_index = position_in_frames.floor().to_usize();
        let beat_offset_in_frame = position_in_frames.fract().mul(&self.frame_length_beats);
        (frame_index, beat_offset_in_frame)
    }
}
```

**Acceptance Criteria**:
- [ ] Replace floating-point accumulation with rational arithmetic
- [ ] Eliminate timing drift from repeated calculations
- [ ] Maintain exact beat positions over long sequences
- [ ] Add comprehensive timing precision tests

### Phase 4: Audio Engine Optimization (Week 5)

#### Task 4.1: Sub-Sample Event Processing
**File**: `engine/src/engine.rs`
**Enhance**: `process()` method

```rust
impl AudioEngine {
    fn process_block_with_subsample_events(&mut self, output: &mut [Frame]) {
        let mut processed = 0;
        
        while processed < output.len() {
            // Find next event within this block
            let next_event_offset = self.find_next_event_in_block(processed);
            let process_length = next_event_offset.unwrap_or(output.len() - processed);
            
            // Process audio up to event
            self.process_audio_segment(&mut output[processed..processed + process_length]);
            
            // Process event at exact sample offset
            if let Some(offset) = next_event_offset {
                self.process_events_at_sample(processed + offset);
                processed += offset;
            } else {
                break;
            }
        }
    }
}
```

**Acceptance Criteria**:
- [ ] Implement sample-accurate event scheduling within blocks
- [ ] Add fractional sample interpolation for voices
- [ ] Optimize block processing for minimal latency
- [ ] Verify timing accuracy with oscilloscope measurements

#### Task 4.2: Enhanced Clock Precision
**File**: `bubocore/src/clock.rs`
**Enhance**: Timing calculations

```rust
impl Clock {
    pub fn beats_to_micros_exact(&self, beats: Decimal) -> u64 {
        let tempo = Decimal::from_f64(self.session_state.tempo());
        let beat_duration_micros = Decimal::from_u64(60_000_000).div(&tempo);
        beats.mul(&beat_duration_micros).round().to_u64()
    }
}
```

**Acceptance Criteria**:
- [ ] Add decimal-precision tempo calculations
- [ ] Implement exact beat-to-time conversions
- [ ] Maintain clock stability over extended periods
- [ ] Add clock drift monitoring and compensation

### Phase 5: System-Level Optimizations (Week 6)

#### Task 5.1: Memory and Process Optimization
**File**: `bubocore/src/main.rs`
**Add**: System-level real-time configuration

```rust
#[cfg(target_os = "linux")]
fn configure_realtime_system() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Lock all memory to prevent page faults
        libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE);
        
        // Set scheduler policy for audio thread
        let mut param: libc::sched_param = std::mem::zeroed();
        param.sched_priority = 80; // High priority for audio
        libc::sched_setscheduler(0, libc::SCHED_FIFO, &param);
    }
    Ok(())
}
```

**Acceptance Criteria**:
- [ ] Add memory locking to prevent page faults
- [ ] Configure CPU isolation for audio threads
- [ ] Implement platform-specific optimizations
- [ ] Add runtime performance monitoring

#### Task 5.2: Comprehensive Testing
**Files**: Throughout codebase
**Add**: Timing precision test suite

**Acceptance Criteria**:
- [ ] Create microsecond-precision timing tests
- [ ] Add stress testing under various loads
- [ ] Implement automated latency measurements
- [ ] Verify against professional DAW timing standards

## Success Metrics

### Timing Precision Goals
- [ ] **Jitter**: < 1ms standard deviation under normal load
- [ ] **Latency**: < 5ms deterministic scheduling latency
- [ ] **Accuracy**: Sample-accurate event timing (±1 sample)
- [ ] **Stability**: No timing drift over 1-hour sessions

### Performance Goals
- [ ] **CPU Usage**: < 10% increase from optimizations
- [ ] **Memory**: Zero real-time allocations maintained
- [ ] **Throughput**: Support 64+ simultaneous voices
- [ ] **Reliability**: 99.9% uptime during live performance

## Dependencies and Considerations

### Crate Dependencies
- [ ] `crossbeam` for lock-free data structures
- [ ] `thread-priority` for real-time scheduling
- [ ] Platform-specific libraries for memory locking

### Testing Requirements
- [ ] Professional audio interface for accurate measurements
- [ ] Oscilloscope or equivalent for timing verification
- [ ] Stress testing framework for load simulation
- [ ] Automated CI/CD integration for performance regression detection

### Platform Support
- [ ] Linux: Full real-time optimization support
- [ ] macOS: Core Audio integration enhancements
- [ ] Windows: MMCSS and high-precision timer support

## Implementation Notes

1. **Incremental Deployment**: Each phase can be implemented and tested independently
2. **Backward Compatibility**: Maintain existing API during transition
3. **Performance Monitoring**: Add telemetry for timing precision measurements
4. **Graceful Degradation**: Fallback to current behavior if real-time features unavailable

---

**Next Steps**: Begin with Phase 1, Task 1.1 (Adaptive Timing Compensation) as it provides immediate timing improvements with minimal risk.