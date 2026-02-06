# Sova Server

TCP server for Sova multiplayer coordination. Manages client connections, synchronizes scene state, and handles real-time collaboration between multiple Sova clients. This module is a good point of entry for developers interested in building multiplayer applications with Sova.

## Features

- Client session management with authentication
- Scene state synchronization across clients
- Transport control (play/pause, tempo, quantum)
- Device routing coordination
- Real-time chat relay
- Optional audio engine integration (Doux)

## Usage

```
cargo run -p sova-server --release -- [OPTIONS]
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `-i, --ip` | `0.0.0.0` | IP address to bind |
| `-p, --port` | `8080` | Port to listen on |
| `-t, --tempo` | `120.0` | Initial tempo (BPM) |
| `-q, --quantum` | `4.0` | Quantum (beats per cycle) |
| `--no-audio` | `false` | Disable audio engine |
| `--audio-device` | system default | Audio output device |
| `--audio-input-device` | system default | Audio input device |
| `--audio-channels` | `2` | Number of output channels |
| `--sample-path` | none | Sample directory (repeatable) |

## Building

```
cargo build -p sova-server --release
```
