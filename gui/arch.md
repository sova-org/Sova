# BuboCore GUI Communication Architecture

## Overview

The BuboCore GUI is a Tauri-based application that provides a desktop interface for managing and connecting to BuboCore servers. The architecture consists of three main layers:

1. **Tauri Backend (Rust)**: Server instance management and low-level communication
2. **React Frontend (TypeScript)**: User interface and high-level state management
3. **BuboCore Server Communication**: TCP-based protocol for musical live coding

## Architecture Layers

### 1. Tauri Backend Layer (`src-tauri/`)

The Rust backend handles server instance management and provides a communication bridge between the React frontend and BuboCore servers.

#### Key Components:

**`lib.rs`** - Main Tauri command handlers:
- **Connection Management**: `connect_to_server()`, `disconnect_from_server()`
- **Message Handling**: `send_message()`, `get_messages()`
- **Server Management**: `start_server()`, `stop_server()`, `restart_server()`
- **State Management**: Client state, message state, server manager state

**`client.rs`** - BuboCore TCP client implementation:
- **BuboCoreClient**: Direct TCP connection to BuboCore server
- **Buffer Management**: Optimized buffer pool for message handling
- **Compression**: Intelligent message compression using zstd
- **ClientManager**: Async wrapper with mpsc channels for message handling

**`server_manager.rs`** - Local server instance management:
- **Process Management**: Spawns and manages BuboCore server processes
- **Configuration**: Handles server configuration and validation
- **Logging**: Captures and manages server logs
- **Device Management**: Audio device enumeration and validation

**`messages.rs`** - Protocol definitions:
- **Type Definitions**: Shared types between Rust and TypeScript
- **Compression Strategy**: Per-message compression optimization
- **Serialization**: MessagePack serialization for efficient transport

#### Communication Flow:

```
React Frontend → Tauri Commands → Rust Backend → BuboCore Server
                                      ↓
              ← Event Emissions ← Message Polling ←
```

### 2. React Frontend Layer (`src/`)

The TypeScript frontend provides the user interface and manages application state through a sophisticated store architecture.

#### Key Components:

**`client.ts`** - Frontend BuboCore client:
- **BuboCoreClient**: Wraps Tauri commands in a clean interface
- **Event Handling**: Listens for server messages via Tauri events
- **Message Dispatching**: Distributes messages to registered handlers

**`MainLayout.tsx`** - Main application container:
- **Connection Management**: Handles server connection lifecycle
- **Message Routing**: Routes server messages to appropriate stores
- **UI Orchestration**: Manages split view, panels, and global UI state

#### Store Architecture:

The application uses **Nanostores** with a focused store architecture:

**Core Data Stores:**
- **`sceneDataStore`**: Scene data and grid state (single source of truth)
- **`playbackStore`**: Transport control and playback state
- **`scriptEditorStore`**: Script editor content and compilation state
- **`peersStore`**: Peer collaboration and selections
- **`compilationStore`**: Script compilation tracking and errors

**UI State Stores:**
- **`gridUIStore`**: Grid selection and UI interactions
- **`optionsPanelStore`**: Panel visibility and sizing
- **`connectionStore`**: Connection settings (persistent)
- **`serverManagerStore`**: Server instance management

**Message Handling Pattern:**
```typescript
// Delegated message handling pattern
export const handleServerMessage = (message: ServerMessage): void => {
  handleSceneMessage(message);
  handlePlaybackMessage(message);
  handlePeerMessage(message);
  handleCompilationMessage(message);
  handleScriptEditorMessage(message);
};
```

### 3. Server Instance Management

The GUI includes a comprehensive server management system that allows users to spawn and manage local BuboCore instances.

#### Server Management Flow:

1. **Configuration**: Users configure server parameters through `ServerConfigForm`
2. **Process Control**: `ServerManager` spawns server processes with proper argument handling
3. **Status Monitoring**: Real-time polling of server state and logs
4. **Auto-Connection**: Seamless connection to newly started servers

#### Key Features:

**Process Management:**
- Binary discovery (searches common build paths)
- Argument construction from configuration
- Graceful shutdown with SIGTERM → SIGKILL fallback
- Process ID tracking and status monitoring

**Configuration Management:**
- Network settings (IP, port)
- Audio engine configuration (sample rate, block size, devices)
- OSC settings
- Relay configuration
- Advanced settings (timestamps, file locations)

**Status Monitoring:**
- Real-time status updates via polling
- Log capture and display
- Error state handling
- Auto-refresh when server is running

## Communication Protocols

### 1. BuboCore Server Protocol

The GUI communicates with BuboCore servers using a custom TCP protocol:

#### Message Format:
```
[4 bytes: length + compression flag][variable: message data]
```

#### Message Types:

**Client Messages** (GUI → Server):
- **Scene Control**: `SetScript`, `SetScene`, `EnableFrames`, `DisableFrames`
- **Transport Control**: `TransportStart`, `TransportStop`, `SchedulerControl`
- **Collaboration**: `UpdateGridSelection`, `StartedEditingFrame`, `Chat`
- **Device Management**: `ConnectMidiDevice`, `CreateOscDevice`

**Server Messages** (Server → GUI):
- **State Updates**: `Hello`, `SceneValue`, `Snapshot`, `ClockState`
- **Transport Events**: `TransportStarted`, `TransportStopped`, `FramePosition`
- **Compilation**: `ScriptCompiled`, `CompilationErrorOccurred`
- **Collaboration**: `PeersUpdated`, `PeerGridSelectionUpdate`, `Chat`

#### Compression Strategy:
- **Never**: Quick messages (grid selections, get requests)
- **Always**: Large messages (scripts, scenes)
- **Adaptive**: Medium messages (based on size threshold)

### 2. Internal Communication

#### Tauri Command Pattern:
```typescript
// Frontend invokes Rust command
await invoke('send_message', { message: clientMessage });

// Rust processes and forwards to BuboCore server
client.send_message(message).await
```

#### Event System:
```typescript
// Rust backend emits events
app_handle.emit("server-message", &message);

// Frontend listens for events
listen<ServerMessage>('server-message', (event) => {
  handleServerMessage(event.payload);
});
```

### 3. State Synchronization

The application maintains several synchronized state layers:

#### Message Polling:
- **10ms interval** polling for server messages
- **Batch processing** of multiple messages per cycle
- **Store delegation** for message handling

#### State Persistence:
- **Connection settings** persist across sessions
- **UI preferences** (panel positions, editor settings)
- **Project data** through disk operations

## Current Architecture Assessment

### Strengths:

1. **Clean Separation**: Well-defined layers with clear responsibilities
2. **Focused Stores**: Each store handles specific domain concerns
3. **Efficient Communication**: Optimized message handling with compression
4. **Process Management**: Robust server instance lifecycle management
5. **Real-time Synchronization**: Proper handling of collaborative features

### Architectural Patterns:

1. **Command Pattern**: Tauri commands provide clean API boundaries
2. **Observer Pattern**: Event-driven message handling
3. **State Management**: Centralized state with focused stores
4. **Facade Pattern**: `sceneStore.ts` provides backward compatibility
5. **Delegation Pattern**: Message handling delegated to specialized stores

### Areas for Improvement:

1. **Connection Management**: Currently connection state is scattered across multiple layers
2. **Error Handling**: Some error states could be more consistently managed
3. **Message Buffering**: Could benefit from more sophisticated message queuing
4. **Server Discovery**: Currently relies on fixed paths for server binary discovery
5. **Configuration Persistence**: Server configurations could be saved/loaded

## Integration Points for New Features

When adding the ability to spawn core instances and connect to them:

1. **Server Manager Enhancement**: Extend `ServerManager` to handle multiple instances
2. **Connection Orchestration**: Coordinate between server startup and client connection
3. **UI Integration**: Seamlessly integrate server management into main workflow
4. **State Coordination**: Ensure proper state transitions during server lifecycle

The current architecture provides a solid foundation for these enhancements while maintaining clean separation of concerns and efficient communication patterns.