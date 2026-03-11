# The Scene

The scene is your performance session. It holds every track, script, and timing
configuration. You edit it in real time.

Sova is a step sequencer where each step is defined by code. Step duration is
not fixed — a step can last a sixteenth note or a full measure (see [Timing](timing)).
Scripts can be modified during playback.

## Structure

A scene contains **lines** and **frames**. Lines are columns on the grid. They
run in parallel, each producing its own stream of events. Inside a line, frames
play in sequence. When a frame's duration elapses, the next one starts.

## Frames

Each frame holds a script and a set of properties:

- **Duration** — how long the frame lasts, in beats (see [Timing](timing)). Default is
  1 beat. Fractional values work: 0.25 for a sixteenth note, 4 for a full bar
  at 4/4.
- **Repetitions** — how many times the script runs within that duration. A frame
  with duration 4 and 4 repetitions runs its script once per beat. With 8
  repetitions, the script fires every eighth note. The total time the frame
  occupies remains 4 beats either way.
- **Enabled** — toggles the frame on or off. Disabled frames are skipped during
  playback. The code is preserved.
- **Name** — optional label shown on the grid cell.
- **Script** — the code, along with the language it uses (Bob, Boinx, Cagire,
  or BaLi).

### Editing frame properties

Select a cell, then press:

| Key | Action |
|-----|--------|
| Enter / D | Edit duration |
| R | Edit repetitions |
| N | Edit name |

Inside the edit field: Enter commits, Tab commits and moves to the next field,
Shift+Tab moves to the previous field, Escape cancels.

### Frame operations

| Shortcut | Action |
|----------|--------|
| Delete / Backspace | Delete selected frame(s) |
| Cmd+D | Duplicate selected frame(s) |
| Cmd+C / Cmd+X | Copy / Cut |
| Cmd+V | Paste after cursor |
| Alt+Up / Alt+Down | Move selected frame(s) up / down |

## Lines

Lines have their own controls:

- **Loop** — the line restarts from the top after its last frame. Otherwise, it
  plays once and stops.
- **Trailing** — events from previous frames keep ringing while the next frame
  starts. Otherwise, they are cut.
- **Speed** — multiplier on the line's tempo. 2.0 for double time, 0.5 for half.
  One line at normal speed, another at half — polymetric structures emerge.
- **Start frame / End frame** — restricts playback to a range within the line.
  Narrow the range to loop a section while you build the next one.

### Line shortcuts

| Key | Action |
|-----|--------|
| S | Edit speed factor |
| L | Toggle looping |
| T | Toggle trailing |
| Tab | Move between Start Frame and End Frame fields |
| Cmd+Shift+D | Duplicate line |
| Cmd+Delete | Remove line |
| Alt+Left / Alt+Right | Move line left / right |

## Grid navigation

| Input | Action |
|-------|--------|
| Arrow keys | Move between frames and lines |
| Click | Select a cell |
| Shift+Click | Extend selection to clicked cell |
| Shift+Arrow Up/Down | Extend selection vertically |
| Double-click | Open code editor for a frame |
| Cmd+A | Select all frames in current line |
| Escape | Clear selection |

Right-click a cell for a context menu: add frames, insert lines, toggle frames,
open panels.

## Execution modes

The execution mode controls how lines synchronize when the scene starts or
restarts. Change it from the transport bar.

**Free** — lines start immediately and loop at their own pace. Each line is
independent.

**AtQuantum** — lines wait for the next quantum boundary (bar line) before
starting. Parts land on the downbeat.

**LongestLine** — all lines wait for the longest one to finish its cycle before
restarting. The scene loops as a single unit.

## Saving and loading

Save and load scenes through the scene menu. The file captures lines, frames,
scripts, [Variables](variables), and configuration. Connecting to a server loads its
current scene automatically.

## Workflow tips

- **Name your frames** — press N on a selected cell. A grid of unnamed cells is
  unreadable during a performance.
- **Duplicate before modifying** — Cmd+D copies the frame. The original stays
  intact as a fallback.
- **Reorder on the fly** — Alt+Up/Down moves frames mid-performance without
  stopping playback.
- **Disable instead of deleting** — right-click a frame to toggle it off. The
  code stays visible but is skipped.
- **Isolate with ranges** — set start/end frame on a line to loop a section
  while you build the next one.
- **One role per line** — drums, bass, melody, effects. Easier to mute, isolate,
  or rearrange.
