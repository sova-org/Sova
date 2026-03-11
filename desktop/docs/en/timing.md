# Timing

Sova measures time in beats. One beat at 120 BPM lasts 500 ms. At 60 BPM, one
full second. Frame durations, waits, and note lengths all use beats. Change the
tempo and every pattern follows.

## Tempo and sync

The clock runs on Ableton Link. Every Link-enabled app on the network shares
the same tempo and beat position. Change BPM in Sova and Ableton Live sees it.
Change it in Live and Sova follows. With no other peers on the network, Sova
runs its own clock. Link also shares start/stop state.

Under the hood, the scheduler prepares events ~30 ms ahead of real time; a
dedicated real-time thread dispatches them to devices with microsecond accuracy.

## Bars and phrases

The **quantum** sets how many beats make a bar. Default is 4 — standard 4/4.
The **phase** is your position inside that bar: beat 0, 1, 2, or 3.

This matters for launching lines. In AtQuantum mode, lines wait for the next
downbeat (phase 0) before starting. See **The Scene** for all execution modes.

## The transport bar

At the top of the screen: play/stop, BPM, quantum, and the current beat
position. Click the BPM to type a new value. Minimum 20 BPM.

## Spacing events in code

Without explicit timing, all events in a script fire at beat zero of the frame.
Each language provides its own mechanism for distributing events across the
frame's duration — fractional offsets, explicit clock advances, or other
approaches. The principle is the same: you place events at specific moments
within the time window the frame gives you. See the language tabs for syntax.

## Frame duration and repetitions

Frame duration determines the time window for events. Repetitions subdivide
that window — the script runs multiple times within the same duration. See
**The Scene** for frame properties.

## Line speed

The speed factor on a line multiplies tempo relative to the global BPM. At 2.0,
double time. At 0.5, half time. Combine with different quantum values across
lines for polymetric structures.

See **The Scene** for execution modes that control line synchronization.