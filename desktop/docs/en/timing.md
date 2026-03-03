# Timing

Everything in Sova runs on a shared clock. Tempo, beats, and synchronization
are handled by Ableton Link, which keeps all connected applications and devices
locked to the same timeline.

## Beats and tempo

Sova measures time in **beats**. A beat is a musical pulse whose actual duration
depends on the tempo (BPM). At 120 BPM, one beat lasts 500 milliseconds. At
60 BPM, one beat lasts a full second.

Frame durations, wait commands, and note lengths are all expressed in beats.
This means your patterns automatically speed up or slow down when the tempo
changes — you don't need to recalculate anything.

## Ableton Link

Ableton Link is a protocol for synchronizing tempo, beat, and phase across
applications and devices on the same network. Sova uses Link as its master
clock.

When you change the tempo in Sova, any Link-enabled application (Ableton Live,
other Sova instances, mobile apps, etc.) sees the change instantly. Conversely,
if another app changes the tempo, Sova follows.

Link also synchronizes **start/stop** state. When you press play in Sova,
other Link peers can start too (if they opt in to start/stop sync).

You don't need to configure Link — it works automatically over your local
network. If no other Link peers are present, Sova simply runs its own clock.

## Quantum and phase

The **quantum** defines how many beats make up a phrase or bar. At quantum 4
(the default), the timeline is divided into groups of 4 beats. The **phase** is
where you are within the current quantum — beat 0, 1, 2, or 3.

Quantum matters for synchronization:

- In **AtQuantum** execution mode, lines wait for the next quantum boundary
  (the next "beat 0" of a bar) before starting.
- You can schedule events to fire at the next phase reset using timing controls
  in your code.

Changing the quantum doesn't change the tempo — it changes how the beat grid is
grouped.

## The transport bar

The transport bar at the top of the screen shows:

- **Play / Stop** — start or stop playback. Synchronized via Link.
- **Tempo** (BPM) — click to edit. Minimum 20 BPM. Shared across all Link
  peers.
- **Quantum** — the beats-per-phrase value.
- **Beat counter** — the current beat and phase position.

## Timing in code

Your scripts can control when events happen within a frame:

- **Wait** — pause execution for a number of beats before continuing. This is
  how you space events apart in time.
- **Frame duration** — the total time a frame plays. A frame with duration 2
  gives your script 2 beats to fill with events.
- **Repetitions** — how many times the script runs within the frame's duration.
  A duration of 4 with 4 repetitions means the script runs 4 times, once per
  beat.

The exact syntax for waits and timing varies by language — see each language's
reference for specifics.

## Timing guarantees

Sova's two-thread architecture is designed for tight timing:

- The **scheduler** runs ~30ms ahead of real time, compiling and preparing
  events in advance.
- The **world thread** runs at real-time priority, dispatching events to MIDI
  (2ms lookahead) and OSC (20ms lookahead) with sub-millisecond precision.

This means your events arrive on time even under CPU load, as long as the
scheduler can keep up.

## Tips

- Frame duration × repetitions = total frame time. Use repetitions to create
  rhythmic subdivisions without writing explicit loops.
- Speed factor on a line multiplies its tempo relative to the global BPM. A
  line at speed 2.0 plays double-time; 0.5 plays half-time.
- Use **AtQuantum** execution mode when you want all lines to stay phrase-
  aligned after changes. Use **Free** when you want polyrhythmic independence.
