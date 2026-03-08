# Multiplayer

Connect to a server, see where other players are in the grid, edit code
simultaneously, chat, perform. Sova sessions are multiplayer by default.

## Hosting a session

Two options.

The built-in server: open the Server panel, click Start. The app runs a server
internally and connects to it. Other players connect to your IP and port.

The standalone server: run `sova-server` from the command line.

```
sova-server -p 8080
```

This is better for dedicated hosting or headless machines. Same server, no GUI.

The server owns the scene, the clock, and all device routing. Clients are thin:
they send edits and receive state.

## Joining

Open the Server panel. Enter the host address, port, and a username. Click
Connect.

You receive the full scene, device map, and clock state immediately. Your
transport syncs via Ableton Link -- the beat is already locked by the time you
see the grid.

Usernames must be unique in the session. If yours is taken, pick another.

## What syncs

Everything scene-related goes through the server:

- Scene structure: lines, frames, durations, repetitions, scripts
- Transport state: play, stop, tempo, quantum
- Device assignments: which slot maps to which output
- Code evaluation: when you evaluate a frame, the server compiles and schedules it

When you disconnect and reconnect, you get the current scene. There is no local
state memory.

## What doesn't sync

- Your panel layout and editor preferences stay local
- MIDI and OSC device connections are per-machine (each player configures their
  own outputs in the **Devices** panel)
- Visual scripts (Hydra) run client-side

## Peer editing

Each player's position in the grid is visible to everyone. You see colored
indicators on the cells other players are viewing or editing.

When someone opens a frame's editor, the grid shows it. This gives you a natural
sense of who is working where.

There is no locking. Two players can edit different frames at the same time
without conflict. If two players edit the same frame, the last evaluation wins.

## Chat

The Chat panel sends text messages to everyone in the session. Good for
coordinating transitions mid-performance: "dropping bass next quantum",
"switching to noise on line 3".

## Jamming tips

Claim your own lines. If you stay on lines 1-2 and your partner stays on 3-4,
you avoid stepping on each other's code.

Agree on device slots before you start. Slot 1 is synth, slot 3 is drums,
whatever works. If someone reassigns a shared slot mid-set, everything routed
there changes.

Ableton Link keeps the beat tight across machines on the same network. Tempo
changes propagate to all Link-enabled apps, not just Sova clients.

Use the quantum setting to coordinate transitions. A 4-beat quantum means
changes land on the next bar. An 8-beat quantum gives you more breathing room.
