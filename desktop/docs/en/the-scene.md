# The Scene

The **scene** is Sova's central structure: it's what you manipulate live. It
organizes your code, timing, and musical structure into a simple hierarchy
built for improvisation.

## Scene hierarchy

A scene contains **lines**, which run in parallel — each one is an independent
track producing its own stream of events. Each line contains **frames** that
run in sequence: when one finishes, the next one starts. Each frame holds a
**script** written in one of Sova's languages.

On the grid, lines are columns and frames are rows. All columns play at once;
within each column, rows play one after another.

## Frame properties

Every frame has these properties:

- **Duration** (beats) — how long the frame plays before advancing to the next
  one. Default: 1 beat. You can use fractional values (0.25, 0.5, 2.5, etc.).
- **Repetitions** — how many times the frame's script runs within its duration.
  Default: 1. A frame with duration 4 and repetitions 4 runs its script once
  per beat for four beats.
- **Enabled** — whether the frame plays at all. Disabled frames are skipped
  during playback. Useful for muting parts without deleting them.
- **Name** — an optional label for the frame, displayed in the grid cell.
- **Script** — the code and its language (Bob, Boinx, Cagire, or BaLi).

The **effective duration** of a frame is `duration × repetitions`. A frame with
duration 0.5 and 8 repetitions occupies 4 beats total.

## Line properties

Each line has controls that shape how its frames are played:

- **Looping** — when enabled, the line restarts from the beginning after its
  last frame finishes. When off, the line plays once and stops.
- **Trailing** — when enabled, events from previous frames keep ringing while
  the next frame starts. When off, previous events are cut.
- **Speed factor** — a multiplier on the line's tempo. A speed of 2.0 means
  the line plays twice as fast; 0.5 means half speed. Only affects this line.
- **Start frame / End frame** — optionally restrict playback to a range of
  frames within the line. Useful for focusing on a section during performance.

## Execution modes

The scene's **execution mode** controls how lines synchronize when triggered:

- **Free** — lines start immediately when triggered, regardless of where other
  lines are. Each line loops at its own pace. This is the default.
- **AtQuantum** — lines wait for the next quantum boundary (bar line) before
  starting. This keeps everything aligned to the global phrase structure.
- **LongestLine** — all lines wait for the longest currently running line to
  finish its cycle before restarting. This creates a natural loop grid where
  everything resets together.

You can change the execution mode from the transport bar.

## Saving and loading

You can save and load scenes through the scene menu. The scene captures
everything: all lines, frames, scripts, variable stores, and configuration.
When you connect to a server, you receive its current scene automatically.
