# BuboCore + Sova Audio Engine Integration Plan

## Overview

This document outlines the integration of the Sova audio engine as an internal target within BuboCore, bypassing the external OSC communication layer for optimal performance and tighter integration.

## Current Architecture

### BuboCore Event Flow
```
Script Execution → ConcreteEvent → World → Protocol Messages → External Targets
                                    ↓
                            [MIDI/OSC Devices]
```

### Sova Audio Engine (Standalone)
```
OSC Server → OSC Message Parsing → EngineMessage → Audio Thread
```

## Target Architecture

### Integrated Flow
```
Script Execution → ConcreteEvent::AudioEngine → World → Direct EngineMessage → Sova Audio Thread
                 → ConcreteEvent::Midi → World → MIDI Devices
                 → ConcreteEvent::Osc → World → OSC Devices
```

## Key Benefits of Direct Integration

1. **Zero Network Overhead**: No UDP/OSC parsing for internal audio engine
2. **Type Safety**: Direct Rust type conversion instead of string parsing
3. **Better Error Handling**: Compile-time guarantees vs runtime OSC parsing
4. **Tighter Timing**: Eliminate network latency and parsing overhead
5. **Shared Memory**: Potential for zero-copy sample sharing

## Implementation Plan

### Phase 1: Event System Extension

#### 1.1 Extend ConcreteEvent Enum
**File**: `bubocore/src/lang/event.rs`

Add new variant:
```rust
ConcreteEvent::AudioEngine {
    source_name: String,
    parameters: HashMap<String, AudioEngineValue>,
    voice_id: Option<u32>,          // For updates, None for new voices
    track_id: u8,                   // Audio track routing
}
```

#### 1.2 Define AudioEngineValue
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioEngineValue {
    Float(f32),
    Int(i32),
    String(String),
    Bool(bool),
}
```

#### 1.3 Extend Event Enum
Add corresponding high-level Event variant:
```rust
Event::AudioEngine {
    source: Variable,
    params: HashMap<String, Variable>,
    track_id: Variable,
}
```

### Phase 2: Direct EngineMessage Mapping

#### 2.1 Create Conversion Logic
**File**: `bubocore/src/audio_engine/mod.rs` (new module)

```rust
impl From<ConcreteEvent> for Option<EngineMessage> {
    fn from(event: ConcreteEvent) -> Self {
        match event {
            ConcreteEvent::AudioEngine { source_name, parameters, voice_id, track_id } => {
                // Convert HashMap<String, AudioEngineValue> to HashMap<String, Box<dyn Any + Send>>
                let engine_params = convert_parameters(parameters);

                match voice_id {
                    None => Some(EngineMessage::Play {
                        voice_id: generate_voice_id(),
                        track_id,
                        source_name,
                        parameters: engine_params,
                    }),
                    Some(id) => Some(EngineMessage::Update {
                        voice_id: id,
                        track_id,
                        parameters: engine_params,
                    }),
                }
            }
            _ => None,
        }
    }
}
```

### Phase 3: World Integration

#### 3.1 Extend World Structure
**File**: `bubocore/src/world.rs`

Add optional audio engine channel:
```rust
pub struct World {
    // ... existing fields
    audio_engine_tx: Option<mpsc::Sender<ScheduledEngineMessage>>,
}
```

#### 3.2 Update execute_message Method
```rust
pub fn execute_message(&self, msg: TimedMessage) {
    let TimedMessage { message, time } = msg;

    match message.payload {
        ProtocolPayload::LOG(log_message) => {
            // ... existing log handling
        }
        ProtocolPayload::AudioEngine(engine_message) => {
            if let Some(ref tx) = self.audio_engine_tx {
                let scheduled_msg = ScheduledEngineMessage::Immediate(engine_message);
                let _ = tx.send(scheduled_msg);
            }
        }
        _ => {
            // ... existing MIDI/OSC handling
        }
    }
}
```

### Phase 4: Main Application Integration

#### 4.1 Conditional Sova Initialization
**File**: `bubocore/src/main.rs`

```rust
// After parsing CLI arguments
let audio_engine_channel = if cli.audio_engine {
    println!("[+] Initializing internal audio engine (Sova)...");

    // Initialize Sova components
    let (engine_tx, engine_rx) = mpsc::channel();
    let audio_engine = initialize_sova_engine(&cli);
    let audio_thread = spawn_audio_thread(audio_engine, engine_rx, &cli);

    Some((engine_tx, audio_thread))
} else {
    None
};

// Pass audio engine channel to World
let (world_handle, world_iface) = World::create(
    clock_server.clone(),
    audio_engine_channel.map(|(tx, _)| tx)
);
```

#### 4.2 Sova Initialization Function
```rust
fn initialize_sova_engine(cli: &Cli) -> AudioEngine {
    // Memory allocation
    let memory_per_voice = cli.buffer_size * 8 * 4;
    let sample_memory = cli.max_audio_buffers * cli.buffer_size * 8;
    let dsp_memory = cli.max_voices * cli.buffer_size * 16 * 4;
    let base_memory = 16 * 1024 * 1024;
    let available_memory = base_memory +
        (cli.max_voices * memory_per_voice) +
        sample_memory +
        dsp_memory;

    let global_pool = Arc::new(MemoryPool::new(available_memory));
    let voice_memory = Arc::new(VoiceMemory::new());

    // Sample library
    let mut sample_library = SampleLibrary::new(
        cli.max_audio_buffers,
        &cli.audio_files_location,
        cli.sample_rate
    );
    sample_library.preload_all_samples();
    let sample_library = Arc::new(std::sync::Mutex::new(sample_library));

    // Module registry
    let mut registry = ModuleRegistry::new();
    registry.register_default_modules();
    registry.set_timestamp_tolerance(cli.timestamp_tolerance_ms);

    // Create audio engine
    AudioEngine::new_with_memory(
        cli.sample_rate as f32,
        cli.buffer_size,
        cli.max_voices,
        cli.block_size as usize,
        registry,
        global_pool,
        voice_memory,
        sample_library,
    )
}
```

### Phase 5: Protocol Layer Updates

#### 5.1 New ProtocolPayload Variant
**File**: `bubocore/src/protocol/payload.rs`

```rust
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum ProtocolPayload {
    OSC(OSCMessage),
    MIDI(MIDIMessage),
    LOG(LogMessage),
    AudioEngine(EngineMessage),  // New variant
}
```

## Voice Management Strategy

### Voice ID Generation
- BuboCore manages voice IDs internally
- Sequential ID generation with overflow handling
- Optional voice ID tracking for updates

### Track Routing
- Map BuboCore "slots" to Sova tracks
- Default track assignment based on line index
- Explicit track specification in language

## Error Handling

### Audio Engine Errors
1. **Invalid Parameters**: Log warnings, use defaults
2. **Voice Limit Exceeded**: Queue voices or drop oldest
3. **Sample Loading Errors**: Log errors, continue with synthesis
4. **Audio Device Errors**: Graceful fallback or panic

### Integration Points
1. **Channel Disconnection**: Audio engine thread cleanup
2. **Memory Allocation**: Pre-allocation with bounded limits
3. **Real-time Violations**: Logging without blocking audio thread

## Performance Considerations

### Memory Management
- Pre-allocate all audio buffers at startup
- Zero-allocation event conversion in hot paths
- Bounded parameter conversion pools

### Threading Model
```
Main Thread: BuboCore server, networking, UI
Scheduler Thread: Scene management, timing, event generation
World Thread: Protocol routing, device management
Audio Thread: Real-time audio processing (Sova)
```

### Timing Coordination
- BuboCore scheduler provides precise timing
- Convert BuboCore SyncTime to Sova timestamps
- Maintain sample-accurate synchronization

## Migration Path

### Phase 1: Core Integration (This Plan)
- Basic Play/Update message support
- Simple parameter mapping
- Single track routing

### Phase 2: Advanced Features
- Multi-track routing and mixing
- Advanced parameter modulation
- Sample library integration

### Phase 3: Optimization
- Zero-copy parameter sharing
- SIMD-optimized conversions
- Lock-free data structures

## Dependencies

### New Dependencies for BuboCore
```toml
[dependencies]
# Audio engine integration
engine = { path = "../engine" }  # Sova crate
```

### Shared Types
- Move common types to shared crate if needed
- Or use feature flags for conditional compilation

## Configuration

### CLI Arguments (Already Added)
```bash
bubocore --audio-engine \
         --sample-rate 48000 \
         --buffer-size 1024 \
         --max-voices 128 \
         --audio-files-location ./samples
```

### Runtime Configuration
- Audio engine parameters exposed via server API
- Real-time parameter updates
- Performance monitoring endpoints

---

## Implementation Status

### ✅ COMPLETED PHASES

#### Phase 1: Event System Extension ✅ 
**Status**: COMPLETE
- ✅ Added `AudioEngineValue` enum for type-safe parameters
- ✅ Added `ConcreteEvent::AudioEngine` variant with source_name, parameters, voice_id, track_id
- ✅ Added `Event::AudioEngine` high-level variant 
- ✅ Implemented conversion logic in `make_concrete()` method
- ✅ Fixed pattern matching in scheduler for AudioEngine events

#### Phase 2: Protocol Layer Integration ✅
**Status**: COMPLETE  
- ✅ Created `AudioEnginePayload` struct (serializable alternative to `EngineMessage`)
- ✅ Added `ProtocolPayload::AudioEngine(AudioEnginePayload)` variant
- ✅ Implemented conversion from `ConcreteEvent::AudioEngine` to `AudioEnginePayload`
- ✅ Added proper From trait implementations
- ✅ Resolved serialization issues by avoiding `dyn Any + Send` in protocol layer

#### Phase 3: World Integration ✅
**Status**: COMPLETE
- ✅ Extended `World` struct with optional `audio_engine_tx: Option<Sender<ScheduledEngineMessage>>`
- ✅ Updated `World::create()` to accept audio engine channel parameter
- ✅ Added voice ID counter with overflow handling (`voice_id_counter: u32`)
- ✅ Implemented `convert_audio_engine_payload_to_engine_message()` function
- ✅ Updated `execute_message()` to handle `ProtocolPayload::AudioEngine` 
- ✅ Added direct routing to Sova bypassing DeviceMap

#### Phase 4: Sova Initialization ✅
**Status**: COMPLETE
- ✅ Added conditional initialization with `--audio-engine` CLI flag
- ✅ Implemented `initialize_sova_engine()` function with memory management
- ✅ Created proper channel setup: `(engine_tx, engine_rx)`
- ✅ Started audio thread with `AudioEngine::start_audio_thread()`
- ✅ Added clean shutdown with audio thread joins
- ✅ Integrated memory pools, sample library, and module registry
- ✅ Added informative logging and status messages

#### Phase 5: Scheduler Direct Interface ✅  
**Status**: COMPLETE
- ✅ Added `ProtocolDevice::AudioEngine` variant for uniform protocol handling
- ✅ Extended all device methods (connect, send, flush, address, Debug, Display)
- ✅ Implemented `handle_audio_engine_event()` method in Scheduler
- ✅ Added direct routing: `AudioEngine events → ProtocolMessage → World → Sova`
- ✅ Resolved borrowing conflicts by collecting events outside closure
- ✅ Maintained protocol uniformity while bypassing device registration

### 🔄 REMAINING WORK

#### Phase 6: Error Handling and Status Reporting
**Status**: PENDING
- [ ] Add audio engine status reporting and error handling
- [ ] Implement proper error propagation from Sova to BuboCore
- [ ] Add monitoring for audio thread health
- [ ] Handle audio device errors gracefully

---

## Current Integration Flow

The integration is now **FULLY FUNCTIONAL** with the following data flow:

```
BuboCore Script → Event::AudioEngine → ConcreteEvent::AudioEngine 
→ Scheduler.handle_audio_engine_event() → ProtocolMessage(AudioEngine device + AudioEnginePayload)
→ World.execute_message() → convert_to_EngineMessage() → Sova Audio Thread
```

### Key Achievements

1. **Zero OSC Overhead**: Direct Rust type conversion from BuboCore events to Sova EngineMessage
2. **Clean Architecture**: AudioEngine treated as internal protocol device, no slot assignment needed
3. **Type Safety**: Compile-time guarantees for audio engine communication  
4. **Conditional Integration**: Only initialized when `--audio-engine` flag provided
5. **Proper Resource Management**: Clean initialization and shutdown of audio threads
6. **Voice Management**: Sequential voice ID assignment with overflow handling

### Files Modified

- `bubocore/src/lang/event.rs` - Added AudioEngine event types
- `bubocore/src/protocol/payload.rs` - Added AudioEnginePayload  
- `bubocore/src/protocol/device.rs` - Added ProtocolDevice::AudioEngine
- `bubocore/src/world.rs` - Added audio engine routing and conversion
- `bubocore/src/schedule.rs` - Added direct audio engine handling
- `bubocore/src/main.rs` - Added conditional Sova initialization
- `bubocore/Cargo.toml` - Audio engine dependency already present

### Next Steps

The integration is **production-ready** for basic audio engine functionality. The only remaining work is:

1. **Error Handling**: Proper error propagation and recovery
2. **Status Reporting**: Audio engine health monitoring and reporting

The core integration is **COMPLETE** and **FUNCTIONAL**.
