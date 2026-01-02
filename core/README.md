# Sova Core

## Building

```
cargo build --release
```

## Running

```
cargo run --release -- [OPTIONS]
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-i, --ip <IP_ADDRESS>` | IP address to bind | `0.0.0.0` |
| `-p, --port <PORT>` | Port to bind | `8080` |
| `-t, --tempo <BPM>` | Initial tempo | `120.0` |
| `-q, --quantum <BEATS>` | Initial quantum | `4.0` |

### Defaults

- Virtual MIDI port: `Sova` (Slot 1)
- OSC output: `SuperDirt` at `127.0.0.1:57120` (Slot 2)
