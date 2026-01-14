# Sova Core

`Core` is a Rust library implementing the essential building blocks required for Sova to operate. It comprises a virtual machine, a scheduler, protocols, and languages. The TCP server for multiplayer coordination lives in the separate `server/` crate.

Fundamentally, `core` provides the machinery to parse/compile code, execute it, and schedule musical events with microsecond precision. Events are timestamped and dispatched to MIDI and OSC devices synchronized to a shared clock via Ableton Link. It manages a **Scene** — a collection of parallel **Lines**, each containing a sequence of **Frames**. Each **Frame** holds a **Script** written in one of the supported languages. As the clock advances, the Scheduler evaluates scripts and queues timed messages for the World thread to dispatch at the exact right moment.

Sova includes a stack-based virtual machine where code execution is inherently temporal. Programs alternate between control instructions (arithmetic, jumps, stack operations) and effect instructions that produce events. Each effect carries a time offset specifying when it should fire relative to the current beat position. Languages can be compiled to VM bytecode through a **Compiler** (source → bytecode) or manage their own code execution and interact with the VM to access the shared state through an **Interpreter** (source → events). The `LanguageCenter` registers available compilers and interpreters, and the Scheduler selects the appropriate one based on each script's declared language.

## Architecture

There are two core threads working together:

| Thread | Priority | Responsibility |
|--------|----------|----------------|
| **Scheduler** | Normal | Scene management, script execution, beat timing, logic time |
| **World** | Real-time | Precision message dispatch to devices, real time |

The Scheduler maintains playback state, processes scripts, and calculates when each event should fire, using a logic time a few milliseconds ahead of the current real time. The World receives timed messages and executes them with sub-millisecond accuracy, applying device-specific lookahead.

The TCP server (in `server/` crate) is optional. One can communicate with the Scheduler locally using channels; the `solo-tui/` crate does exactly this.

## Module Guide

| Module | Purpose | Start here for... |
|--------|---------|-------------------|
| `schedule.rs` | Scene playback, event timing | Sequencing logic |
| `world.rs` | Timed message execution | Real world timing |
| `clock.rs` | Tempo, beat sync (Link) | Timing and synchronization |
| `scene.rs` | Scene/Line/Frame data model | Sequencing data structures |
| `device_map.rs` | MIDI/OSC device registry | Device I/O |
| `vm.rs` | Language compilation/interpretation | Script processing |
| `lang/` | Bali, Bob, Boinx, Forth | Adding and tweaking languages |
| `protocol/` | Wire format, message types | Protocol types |
| `init.rs` | Scheduler + World startup | Embedding core |

## Building

```
cargo build -p core --release
```

## Usage

Core is a library crate. To run the server, see the `server/` crate:

```
cargo run -p sova-server --release -- [OPTIONS]
```

For embedded usage (like `solo-tui/`), use `init::start_scheduler_and_world()` to spin up the Scheduler and World threads.
