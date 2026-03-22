# Multiplayer

Sova is designed from the ground up to allow people to pair and jam together over the network. Sova is built following a client / server architecture, and all sessions are multiplayer by default, even when you play alone. Multiple users can join a Sova server and they can all edit the same scene, same data, etc. If you decide not to share the scene, you will stay synchronized to other folks through the network anyways thanks to the Ableton Link protocol that we use internally for timing!


## Hosting a session

Two options.

**Embedded server.** Click Start Server on the splash screen (or in the Server
panel). The desktop app runs a server locally. You then connect to it like any
other server — enter the address, port, username, and click Connect. Other
musicians on the network do the same.

**Standalone server.** Run `sova-server` from the command line. Same server, no GUI.
Better suited to dedicated hosting or headless machines.

```
sova-server -p 8080
```

CLI flags:

- `-i` / `--ip` — bind address (default `0.0.0.0`)
- `-p` / `--port` — listen port (default `8080`)
- `-t` / `--tempo` — initial tempo in BPM (default `120`)
- `-q` / `--quantum` — initial quantum in beats (default `4`)
- `--no-audio` — disable the Doux audio engine
- `--audio-device` — output device name or index (system default if omitted)
- `--audio-input-device` — input device name or index (system default if omitted)
- `--audio-channels` — number of output channels (default `2`)
- `--audio-buffer-size` — buffer size in samples
- `--sample-path` — sample directory (repeatable)
- `--max-voices` — maximum polyphony (default `32`)

The server owns the scene, the clock (Ableton Link), device routing, the scheduler
thread, and the world thread. Clients are lightweight: they send edits and receive
state.

## Joining

Open the Server panel. Enter the host address, port, and a username. Click
Connect. On success, the client receives the full scene, device list, peer list,
clock state (tempo, beat, quantum), available languages, and audio engine state.
Transport syncs via Ableton Link — the beat is already locked by the time the
grid appears.

On failure, the server sends a refusal with a reason. Username must not be
empty, must not be "Unknown musician" (reserved), and must be unique in the
session.

Reconnecting receives the current scene. No local state is preserved.

## What syncs

Everything scene-related goes through the server:

- **Scene structure** — lines, frames, durations, repetitions, scripts, scene
  execution mode, prelude
- **Transport** — play, stop, tempo, quantum
- **Device assignments** — slot mappings (see [Devices](devices))
- **Code evaluation** — the server compiles and schedules scripts; compilation
  results (success or error) are sent back per frame
- **Global variables** — shared across the session (see [Variables](variables))
- **Frame positions** — playback cursor updated at ~30 Hz
- **Peer presence** — cursor positions and editing indicators (see below)
- **Chat messages** — text sent through the Chat panel

If a client falls behind (its message buffer fills up), the server automatically
sends a full snapshot — scene, clock, and devices — to resynchronize it. This
happens transparently; no action is needed from the musician.

## What does not sync

- Panel layout and editor preferences — local to each client.
- MIDI and OSC connections — per-machine. Each musician configures their own
  outputs in the [Devices](devices) panel.
- Hydra visual scripts — rendered client-side.

## Collaborative editing

Three types of peer awareness in the grid:

1. **Cursor position.** A colored border appears around the frame cell where
   another musician's cursor sits, with their name shown as a tag.
2. **Editing indicator.** Colored dots in the top-right corner of frame cells
   show who is currently editing that frame (up to 3 dots per cell).
3. **Peer list.** Updated on join and leave. Visible in the Server panel.

Colors are deterministic per username — the same color appears in the grid, the
editor, and the chat.

No locking. Two musicians editing different frames: no conflict. Same frame: the
last evaluation wins.

When a peer disconnects, their cursor and editing indicators are cleaned up
automatically.

## Chat

The Chat panel sends text messages to all session members. System messages are
generated when a musician joins or leaves. Message history is capped at 500 on
the client. The panel supports a detached (pop-out) window.

## Tips for collective play

Claim your own lines. If you stay on lines 1–2 and your partner on 3–4, you
avoid editing conflicts.

Agree on device slots before you start. Slot 1 for the synth, slot 3 for drums —
whatever suits your setup. If a musician reassigns a shared slot mid-performance,
everything routed there changes.

Ableton Link keeps the beat synchronized across machines on the same network.
Tempo changes propagate to all Link-enabled apps, not just Sova clients.

Use the quantum setting to coordinate transitions. A 4-beat quantum lands changes
on the next bar. An 8-beat quantum provides more breathing room. See [Timing](timing).
