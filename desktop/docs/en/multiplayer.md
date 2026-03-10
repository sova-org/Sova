# Multiplayer

Sova is multiplayer by default. Connect to a server, see where others are in the
grid, edit code simultaneously, and communicate through the built-in chat. Live
coding is inherently a practice of sharing: code is visible, ideas flow freely,
and music is built collectively.

## Hosting a session

Two options.

The built-in server: open the Server panel and click Start. The app runs a
server internally and connects to it. Other musicians connect to your IP and
port.

The standalone server: run `sova-server` from the command line.

```
sova-server -p 8080
```

Better suited to dedicated hosting or headless machines. Same server, no GUI.

The server owns the scene, the clock, and all device routing. Clients are
lightweight: they send edits and receive state.

## Joining

Open the Server panel. Enter the host address, port, and a username. Click
Connect.

You receive the full scene, device map, and clock state immediately. Transport
syncs via Ableton Link — the beat is already locked by the time the grid
appears.

Usernames must be unique in the session. If yours is taken, pick another.

## What syncs

Everything scene-related goes through the server:

- Scene structure: lines, frames, durations, repetitions, scripts
- Transport state: play, stop, tempo, quantum
- Device assignments: which slot maps to which output
- Code evaluation: when you evaluate a frame, the server compiles and
  schedules it

When you disconnect and reconnect, you receive the current scene. No local
state is preserved.

## What does not sync

- Your panel layout and editor preferences stay local
- MIDI and OSC connections are per-machine (each musician configures their own
  outputs in the **Devices** panel)
- Visual scripts (Hydra) run client-side

## Collaborative editing

Each musician's position in the grid is visible to everyone. Colored indicators
appear on the cells others are viewing or editing.

When a musician opens a frame's editor, the grid signals it. This provides a
clear view of who is working where.

No locking. Two musicians can edit different frames simultaneously without
conflict. If two musicians edit the same frame, the last evaluation wins.

## Chat

The Chat panel sends text messages to everyone in the session. Useful for
coordinating transitions mid-performance: "stopping the bass at next quantum",
"switching to noise on line 3".

## Tips for collective play

Claim your own lines. If you stay on lines 1–2 and your partner on 3–4, you
avoid editing conflicts.

Agree on device slots before you start. Slot 1 for the synth, slot 3 for
drums — whatever suits your setup. If a musician reassigns a shared slot
mid-performance, everything routed there changes.

Ableton Link keeps the beat synchronized across machines on the same network.
Tempo changes propagate to all Link-enabled apps, not just Sova clients.

Use the quantum setting to coordinate transitions. A 4-beat quantum lands
changes on the next bar. An 8-beat quantum provides more breathing room.
