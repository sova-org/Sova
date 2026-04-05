The scene is where the musical action takes place: it is a visual representation of what is currently playing, a live representation of the step sequencer and machine state. Think of the scene as a bird-eye view of all the scripts currently active and executed by the server. The scene also allows you to see other musicians working if you are playing in multiplayer. It is the working space / the jam session or whatever you imagine it to be! Learning how to manipulate the scene view is probably the most important thing that you need to know. Everything else can be learned by playing music.

You will soon figure out that there are multiple ways to approach the scene. It can be used to structure a song, to distribute space for improvisers, to store scripts and processes, etc. Its purpose is voluntarily left as a open space. However, it is robust and flexible enough to let people organize the code however they need it to be!

## Structure

A scene contains **lines** and **frames**. Lines are columns on the grid. Think about it as a _line of execution_. Lines run in parallel. Each produces its own stream of events. Inside a line, frames are stacked vertically and play in sequence. When a frame's duration elapses, the next one starts. Add a frame with the "+" button at the bottom of a line, and add a line with the button to the right of the last column. Column widths and frame heights are adjustable by dragging the borders between them. There are multiple metaphors you can use to understand what a **line** is. It can be described as a _track_, as an _execution line_, etc.

## Prelude

The prelude is a list of scripts that execute **once** when playback starts. Prelude scripts are stored as a collapsible column on the left side of the scene view. Use it to initialize [variables](variables), to configure things or to set up any state that other frames depend on. Prelude scripts are not part of any line and do not loop or repeat. Each can be written in any available language. Prelude scripts run in order, top to bottom. Add a new script with the button in the prelude header. Here are a few things that you might be interested in doing using prelude scripts:

- writing functions or code snippets for re-use.
- define global variables.
- share global information with other people playing with you.

You can forcefully re-evaluate a prelude script if you need to update it while playing.

## Frames

Each frame holds a **script**. Think of a **frame** as a _script container_ that also comes with a set of properties:

- **Duration**: how long one execution of the frame lasts, in beats (see [Timing](timing)). Default is 1 beat. Fractional values work: 0.25 for a sixteenth note, 4 for a full bar at 4/4.
- **Repetitions**: how many times the frame plays before the line advances. Each repetition gets the full duration window. A frame with duration 1 and 4 repetitions runs its script four times, one beat each, occupying 4 beats total. Default is 1.
- **Enabled**: toggles the frame on or off. Disabled frames are skipped during playback but their code is preserved.
- **Script**: the code, along with the language it uses (Bob, Boinx, Cagire, or BaLi). See [Languages](languages).

Frame properties are edited directly in the frame header. Each frame also contains an inline code editor. Just by looking at the scene, you can see all the code that is currently running on the server. You can also visualize all the properties of all frames at all times. We try to minimize hidden state in the graphical interface.

## Lines

Lines have their own controls, accessible in the line header:

- **Loop**: the line restarts after reaching the end of its playback range. Otherwise, it plays through once and stops.
- **Trailing**: when the line loops, previous script executions keep running alongside the new iteration. Otherwise, they are stopped when the line restarts. Trailing lets sounds ring out naturally across loop boundaries.
- **Manual**: whether the playback is automatic or manually scheduled. Useful to store code that you are going to trigger from another line.
- **Speed**: multiplier on the line's tempo. 2.0 for double time, 0.5 for half. One line at normal speed, another at half — polymetric structures emerge. See [Timing](timing) for details.
- **Start frame / End frame**: restricts playback to a range within the line. Useful for looping a section while building the next one.

## Execution modes

The execution mode controls how lines synchronize when the scene starts or restarts. Change it from the transport bar:

- **Free**: lines start immediately and loop at their own pace. Each line is independent.
- **AtQuantum**: lines wait for the next quantum boundary (bar line) before starting, so parts land on the downbeat.
- **LongestLine**: all lines wait for the longest one to finish its cycle before restarting. The scene loops as a single unit.

## Modal interaction

The scene view can switch between two modes depending on how you interact with it with your keyboard:

- **Navigation mode** (default): arrow keys and vim keys move between frames and lines, single-key shortcuts operate on frames, and no typing goes to any editor.
- **Edit mode**: entered by pressing Enter or `i` on a frame. The code editor receives focus and all typing goes to the editor. Clicking inside a code editor also enters Edit mode. Press Escape to return to Navigation mode.

All shortcuts below use `Cmd` on macOS and `Ctrl` on other platforms.

### Navigation mode shortcuts

| Shortcut | Action |
|----------|--------|
| Arrow keys | Move between frames and lines |
| h / j / k / l | Move (Vim-style) |
| Shift+Up/Down | Extend selection vertically |
| Enter / i | Enter Edit mode |
| Escape | Clear cursor and selection |
| Cmd+D | Duplicate selected frame(s) after |
| Cmd+Shift+D | Duplicate frame before |
| Shift+I | Insert empty frame after |
| Cmd+Shift+I | Insert empty frame before |
| Delete | Delete selected frame(s) |
| Shift+J | Move frame(s) down |
| Shift+K | Move frame(s) up |
| e | Toggle enabled |
| . | Toggle looping on line |
| , | Toggle trailing on line |
| Alt+H | Move line left |
| Alt+L | Move line right |
| Cmd+C | Copy |
| Cmd+X | Cut |
| Cmd+V | Paste after cursor |
| Cmd+A | Select all frames in current line |
| Cmd+Delete | Remove entire line |

### Edit mode shortcuts

| Shortcut | Action |
|----------|--------|
| Escape | Exit to Navigation mode |
| Cmd+Enter | Evaluate script |
| Cmd+L | Open language selector |
| Cmd+F | Search in editor |
| Ctrl+Space | Open code completion |

Evaluating a script (`Cmd+Enter`) sends the code to the server for compilation and scheduling. The frame flashes white on success or red on error.

## Context menus

Right-click a frame to open its context menu: cut, copy, paste after, insert frame before, insert frame after, duplicate, move up, move down, toggle enabled, remove frame. Insert before and insert after only appear for single selections. Right-click a line header for line operations: insert line before, insert line after, duplicate line, move left, move right, toggle looping, toggle trailing, clear frame range, remove line.

There are a lot of actions that you can perform on the scene view and it is a bit difficult to fit them all in the interface. We have tried our best to make it very shallow, meaning that there is no menu diving. However, always try to right click on things, you might get surprises!

## Visual feedback

The currently playing frame shows a colored accent strip on its left edge. A progress fill sweeps across the frame header and body, tracking playback position within the frame's duration. Disabled frames remain visible but visually muted. Their code stays readable, they are simply skipped during playback. In a [multiplayer](multiplayer) session, colored bars on the left edge of frame cells indicate where other musicians' cursors are. Colors are derived from usernames and stay consistent across sessions.

## Saving and loading

Save and load scenes through the File menu (Cmd+S / Cmd+O). A scene file captures lines, frames, scripts, prelude, [variables](variables), and configuration. "Load at end" defers the load to the next downbeat, avoiding a mid-bar interruption. Recent files are listed in the File menu for quick access. Connecting to a server loads its current scene automatically.
