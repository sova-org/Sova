# Getting Started with Sova

Sova is a live coding sequencer. You write code that generates musical events
in real time — notes, control changes, OSC messages — and Sova plays them
back on a shared timeline.

## Concepts

- **Scene** — the top-level container. A scene holds lines that run in parallel.
- **Line** — a single sequence of timed events, written in one of Sova's languages.
- **Frame** — a cell in the timeline grid. Each frame has a duration (in beats) and a number of repetitions.
- **Device** — a MIDI port, OSC endpoint, or audio output that receives events.

## Writing your first sequence

1. Connect to the Sova server (or start the built-in one).
2. Select a line in the scene grid.
3. Pick a language (Bob, Boinx, Cagire, or BaLi).
4. Type a short program and press **Enter** to evaluate.

The line starts producing events immediately.

## Execution modes

- **Free** — each line loops independently at its own pace.
- **AtQuantum** — lines re-sync at the global quantum boundary.
- **LongestLine** — all lines wait for the longest one before restarting.
