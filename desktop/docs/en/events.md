# Events

Events are the messages your code produces — MIDI notes, control changes, OSC
messages, and more. Understanding how events work helps you write code that does
exactly what you intend.

## MIDI note events

The most common event is a MIDI note. A note event has these parameters:

- **Note** (0–127) — the pitch. 60 = middle C.
- **Velocity** (0–127) — how hard the note is struck. 0 usually means note-off.
- **Channel** (1–16) — the MIDI channel. Default: 1.
- **Duration** (beats) — how long the note rings before a note-off is sent.
- **Device** (1–16) — which device slot receives the event. Default: 1.

When a note event fires, Sova sends a MIDI Note On immediately and schedules a
corresponding Note Off after the specified duration. You don't need to manage
note-offs manually.

## MIDI control events

Beyond notes, MIDI offers several control messages:

- **CC (Control Change)** — continuous controllers (mod wheel, expression,
  custom knobs). Specify a CC number (0–127) and a value (0–127).
- **Program Change** — switch patches/presets on a synthesizer. Specify a
  program number (0–127).
- **Aftertouch** — pressure-sensitive expression. Can be per-channel or
  per-note (polyphonic aftertouch).
- **Pitch Bend** — pitch wheel position. Range depends on the receiving synth.

All MIDI control events take a channel and device slot, just like notes.

## OSC messages

OSC (Open Sound Control) events send messages to external software over UDP.
An OSC event has:

- **Address** — an OSC address pattern (e.g. `/synth/freq`).
- **Arguments** — a list of values (integers, floats, strings).
- **Device** — the device slot of an OSC output endpoint.

OSC is useful for communicating with SuperCollider, Max/MSP, Pure Data, visual
software, or any application that speaks OSC.

## How events are emitted

The exact syntax for creating events differs by language, but the general
pattern is:

1. **Set context**: choose a device slot and MIDI channel. These become the
   default for subsequent events until changed.
2. **Emit the event**: use the language's event syntax to fire a note, CC, or
   OSC message.
3. **Wait**: pause for a number of beats before the next event. Without waits,
   all events fire simultaneously at the start of the frame.

Each language has its own syntax — see the per-language tabs for details:

- **Bob** uses event maps: `>> [note: 60 vel: 100 dur: 0.5]`
- **Boinx** uses pattern notation for rhythmic sequences.
- **Cagire** uses stack-based words to push and emit events.
- **BaLi** uses expression-based event construction.

## Channel and device routing

Every event carries a channel and device value. You set these before emitting:

- **Device** selects the output slot (1–16). Slot 0 is the log console.
- **Channel** selects the MIDI channel (1–16). Ignored for OSC events.

You can change device and channel mid-script to route different events to
different outputs within a single frame. For example, you might send melody
notes to device 1 / channel 1 and bass notes to device 2 / channel 3.

## Event timing

Events are dispatched with precise timing by the world thread:

- MIDI events are sent with a 2ms lookahead for tight synchronization.
- OSC events are sent with a 20ms lookahead.

The scheduler prepares events ~30ms ahead of real time. This means events are
queued and dispatched at exactly the right moment, not fired and hoped for.

## Tips

- Use the **Log** device (slot 0) to inspect what events your code produces
  before routing them to a real output.
- A note with velocity 0 is treated as note-off by most synths.
- OSC lets you control anything — lights, visuals, other software — not just
  sound.
- Keep duration in mind: overlapping notes (long durations + short waits) can
  produce chords. Non-overlapping notes (duration ≤ wait) produce staccato.
