# Getting Started with Sova

Sova is a polyglot live coding sequencer for real-time musical improvisation.
You write code that generates musical events — MIDI notes, control changes, OSC
messages — and Sova plays them back on a shared timeline synchronized via
Ableton Link. Multiple languages, multiple players, one beat.

## Running the app

When you launch Sova, you can either start a **built-in server** or connect to
a **remote server** that someone else is hosting.

- **Built-in server**: Open the Server panel and click Start. The app creates a
  local server and connects to it automatically. This is the simplest way to get
  going solo.
- **Remote server**: Enter the host address and port in the Server panel and
  click Connect. You'll join an existing session with other players.

Once connected, you'll see the scene grid — the heart of the interface.

## The interface at a glance

Sova's interface is built around panels that you can show, hide, and rearrange:

- **Scene grid** — the main workspace. Lines run left to right as columns,
  frames stack top to bottom as rows. This is where you write and organize code.
- **Transport bar** — play/stop, tempo, and quantum controls at the top.
- **Server panel** — connection settings and server status.
- **Devices panel** — manage MIDI ports, OSC endpoints, and audio outputs.
- **Audio panel** — configure the built-in audio engine (Doux).
- **Scope / Spectrum / VU Meter** — visualize audio output in real time.
- **Log panel** — see event output and debug messages.
- **Chat panel** — talk to other players in a multiplayer session.
- **Options panel** — editor theme, font size, and other preferences.
- **Documentation panel** — the panel you're reading right now.

Right-click on empty space in the grid to toggle panels on and off.

## Hearing sound

To hear anything, you need at least one output device:

- **MIDI output**: Open the Devices panel, connect a hardware MIDI port or
  create a virtual MIDI output. Assign it to a device slot (1–16). Your code
  sends events to a slot number, and the device in that slot plays them.
- **Audio engine**: If the server was started with audio support, the built-in
  synthesizer (Doux) is available on a device slot. Open the Audio panel to
  configure it.
- **OSC output**: Create an OSC endpoint in the Devices panel to send messages
  to external software (SuperCollider, Max/MSP, etc.).

See the **Devices** article for full setup details.
