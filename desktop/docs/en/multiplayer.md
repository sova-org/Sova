# Multiplayer

Sova follows a client/server architecture where the server owns everything: the scene, the clock, the execution engine. Clients send edits and receive state. This holds whether one musician is connected or ten — the workflow is the same. There is no "single-player mode" to switch away from. You start a server, you connect, you play. When multiple musicians connect to the same server, they share the same scene, the same clock, the same global variables. Everyone sees the same grid and can modify it in real time. Even without sharing a scene, musicians on the same local network stay synchronized through Ableton Link — tempo and beat position are locked across all Link-enabled applications.

## Hosting a session

Two options.

**Embedded server.** Click Start Server on the splash screen (or in the Server panel). The desktop app launches a server inside the same process. You then connect to it like any other server — enter the address, port, and username, then click Connect. Other musicians on the network do the same using your machine's IP address. The embedded server creates a virtual MIDI port named "Sova" on slot 1, enables Ableton Link, and starts with the tempo and quantum configured in the Server panel. A password can be set to restrict access.

**Standalone server.** Run `sova-server` from the command line. Same server code, no GUI. Better suited to dedicated hosting, headless machines, or situations where you want the server running independently of any client.

```
sova-server -p 8080
```

CLI flags:

| Flag | Description | Default |
|------|-------------|---------|
| `-i` / `--ip` | Bind address | `0.0.0.0` |
| `-p` / `--port` | Listen port | `8080` |
| `-t` / `--tempo` | Initial tempo (BPM) | `120` |
| `-q` / `--quantum` | Quantum (beats) | `4` |
| `--password` | Require a password to connect | none (open) |
| `--no-audio` | Disable the Doux audio engine | |
| `--audio-host` | Audio driver (coreaudio, jack, alsa, wasapi) | system default |
| `--audio-device` | Output device (name or index) | system default |
| `--audio-input-device` | Input device (name or index) | system default |
| `--audio-channels` | Output channels | `2` |
| `--audio-buffer-size` | Buffer size in samples | |
| `--sample-path` | Sample directory (repeatable) | |
| `--max-voices` | Maximum polyphony | `32` |

## Joining

On the connection screen, enter the host address, port, and a username. If the server requires a password, enter it as well. Click `Connect`. On success, the client receives the full scene, device list, peer list, clock state, available languages, and audio engine state. Transport syncs via Ableton Link — the beat is already locked by the time the grid appears. On failure, the server refuses the connection with a reason. Three rules apply to usernames:

- Must not be empty.
- Must not be "Unknown musician" (reserved).
- Must be unique in the session — no two musicians can share a name.

If the password is wrong or missing when the server requires one, the connection is refused as well.

## What syncs

Everything scene-related goes through the server:

- **Scene structure**: lines, frames, durations, repetitions, scripts, scene execution mode, prelude.
- **Transport**: play, stop, tempo, quantum. When one musician presses Play, everyone's sequencer starts. Hush (emergency silence) is also shared.
- **Device assignments**: slot mappings (see [Devices](devices)).
- **Code evaluation**: the server compiles and schedules scripts; compilation results (success or error) are sent back per frame.
- **Global variables**: shared across the session (see [Variables](variables)).
- **Frame positions**: playback cursor updated at ~30 Hz.
- **Peer presence**: cursor positions and editing indicators (see below).
- **Chat messages**: text sent through the Chat panel.

If a client falls behind, the server automatically sends a full snapshot to resynchronize it. This happens transparently; no action is needed.

## What does not sync

- Panel layout and editor preferences, local to each client.
- MIDI and OSC connections, per-machine. Each musician configures their own outputs in the [Devices](devices) panel.

## Collaborative editing

Three types of peer awareness in the grid:

1. **Cursor position.** A colored bar on the left edge of the frame cell shows where another musician's cursor sits.
2. **Editing indicator.** Colored dots in the top-right corner of frame cells show who is currently editing that frame.
3. **Peer list.** The bottom bar shows the number of connected musicians. Hover to see their names. The list updates on join and leave.

Colors are deterministic per username — the same name always produces the same color, across sessions and machines. That color appears in the grid, the editor, and the chat. There is no locking. Two musicians can edit the same frame simultaneously. The last evaluation wins: whichever musician presses Cmd+Enter last determines the running script. Editing different frames produces no conflict at all. When a peer disconnects, their cursor and editing indicators are cleaned up automatically.

## Disconnection

Sova does not auto-reconnect. If the connection drops because of a network failure, server restart, timeout, you must reconnect manually from the connection screen. On reconnect, the server sends the current scene from scratch. Nothing is remembered from the previous session. Other musicians see you leave immediately: the peer list updates and a system message appears in the chat.

## Clock and Ableton Link

The server runs an Ableton Link session. Link synchronizes tempo and beat position across all Link-enabled applications on the same network — not just Sova clients, but also Ableton Live, SuperCollider, and anything else that speaks Link. When someone changes the tempo in any Link peer, all peers follow. A tempo change in Ableton Live propagates to Sova, and vice versa. The number of Link peers (Sova clients and external apps combined) is visible in the transport bar.

Start/stop synchronization can be enabled so that pressing Play in one app starts playback in all others. This is controlled per-client and can be toggled independently of the tempo sync.

## Chat

The Chat panel sends text messages to all session members. System messages appear automatically when a musician joins or leaves. Message history is capped at 500 on the client and is not persisted. Restarting the client clears the history. The panel can be detached into its own window.
