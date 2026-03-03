# The Scene

A **scene** is the top-level container for everything you're working on in Sova.
It holds all the musical material — the code, the timing, the structure — in a
hierarchy designed for live performance.

## Scene hierarchy

```
Scene
 └─ Line        (parallel tracks — columns in the grid)
     └─ Frame   (sequential steps — rows in the grid)
         └─ Script  (code + language identifier)
```

- A **scene** contains one or more **lines**.
- Lines run **in parallel** — they are independent tracks, each producing its
  own stream of events simultaneously.
- Each line contains one or more **frames**.
- Frames run **in sequence** — when one frame finishes, the next one starts.
- Each frame holds a **script**: a piece of code written in one of Sova's
  languages.

Think of it as a table: lines are columns, frames are rows. The scene plays all
columns at once, and within each column, rows play one after another.

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

Scenes are serialized as MessagePack data. You can save and load scenes through
the scene menu. The scene captures everything: all lines, frames, scripts,
variable stores, and configuration. When you connect to a server, you receive
its current scene automatically.
