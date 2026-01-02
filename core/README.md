# Sova Core

`Core` implements the essential building blocks required for Sova to operate. The core is a tightly integrated system comprising a virtual machine, a scheduler, a server and some other components built upon that base: core languages, client/server communication protocol, etc.

Fundamentally, the `core` is meant to receive code from connected clients, to parse/interpreter/compile it, to execute code and schedule musical events with microsecond precision. Events are timestamped and dispatched to MIDI and OSC devices synchronized to a shared clock via Ableton Link. The server manages a **Scene** — a collection of parallel **Lines**, each containing a sequence of **Frames**. Each **Frame** holds a **Script** written in one of the supported languages. As the clock advances, the Scheduler evaluates scripts and queues timed messages for the World thread to dispatch at the exact right moment.

Sova includes a stack-based virtual machine where code execution is inherently temporal. Programs alternate between control instructions (arithmetic, jumps, stack operations) and effect instructions that produce events. Each effect carries a time offset specifying when it should fire relative to the current beat position. Languages can target the VM through a **Compiler** (source → bytecode), bypass it entirely through an **Interpreter** (source → events), or combine both. The `LanguageCenter` registers available compilers and interpreters, and the Scheduler selects the appropriate one based on each script's declared language.

## Architecture

There are three threads working together:

| Thread | Priority | Responsibility |
|--------|----------|----------------|
| **Server** | Normal | TCP connections, client I/O, message routing |
| **Scheduler** | Real-time | Scene management, script execution, beat timing |
| **World** | Real-time | Precision message dispatch to devices |

The Server accepts client connections and forwards commands to the Scheduler. The Scheduler maintains playback state, processes scripts, and calculates when each event should fire. The World receives timed messages and executes them with sub-millisecond accuracy, applying device-specific lookahead.

## Module Guide

| Module | Purpose | Start here for... |
|--------|---------|-------------------|
| `main.rs` | Entry point, initialization | Understanding startup flow |
| `server.rs` | TCP server, client handling | Client communication |
| `schedule.rs` | Scene playback, event timing | Sequencing logic |
| `world.rs` | Timed message execution | Real world timing |
| `clock.rs` | Tempo, beat sync (Link) | Timing and synchronization |
| `scene.rs` | Scene/Line/Frame data model | Sequencing data structures |
| `device_map.rs` | MIDI/OSC device registry | Device I/O |
| `vm.rs` | Language compilation/interpretation | Script processing |
| `lang/` | Bali, IMP, Boinx implementations | Adding and tweaking languages |
| `protocol/` | Wire format, message types | Client-server protocol |

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
| `-i, --ip <IP>` | Bind address | `0.0.0.0` |
| `-p, --port <PORT>` | Bind port | `8080` |
| `-t, --tempo <BPM>` | Initial tempo | `120.0` |
| `-q, --quantum <BEATS>` | Initial quantum | `4.0` |

### Defaults

- Virtual MIDI port: `Sova` (Slot 1)
- OSC output: `SuperDirt` at `127.0.0.1:57120` (Slot 2)
