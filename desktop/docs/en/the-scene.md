The scene is where the musical action takes place: it is a visual representation of what is currently playing, a live representation of the step sequencer and machine state. Think of the scene as a bird-eye view of all the scripts currently active and executed by the server. The scene also allows you to see other musicians working if you are playing in multiplayer. It is the working space / the jam session or whatever you imagine it to be! Learning how to manipulate the scene view is probably the most important thing that you need to know. Everything else can be learned by playing music.

You will soon figure out that there are multiple ways to approach the scene. It can be used to structure a song, to distribute space for improvisers, to store scripts and processes, etc. Its purpose is voluntarily left as a open space. However, it is robust and flexible enough to let people organize the code however they need it to be!

## Structure

A scene contains **lines** and **frames**. Lines run in parallel. Each produces its own stream of events. Inside a line, frames play in sequence. When a frame's duration elapses, the next one starts. There are multiple metaphors you can use to understand what a **line** is. It can be described as a _track_, as an _execution line_, etc.

## View modes

The scene can be displayed in two different ways. Switch between them in the Settings sidebar under the **Scene** section.

### Sequencer mode

The default view. Lines are horizontal rows, frames are compact tiles flowing left to right. The main work area switches between two presentations for the current selection:

- **Sequencer panel**: a compact overview of all lines and frames. Each frame is a small numbered tile. Tiles are grouped every four steps for readability. The grid shows playback state, selection, peer presence, and compilation flashes at a glance.
- **Editor panel**: the code editor for the currently selected frame or prelude script.

Press **Escape** to switch between the editor panel and the sequencer panel. This layout is designed for fast pattern editing without a floating popup.

### Classic mode

Lines are vertical columns with full inline code editors visible for every frame. This view shows all code simultaneously and is useful when you want to see multiple scripts side by side. Column widths and frame heights are adjustable by dragging the borders between them. The prelude appears as a collapsible column on the left.

## Prelude

The prelude is a list of scripts that execute **once** when playback starts. Use it to initialize [variables](variables), to configure things or to set up any state that other frames depend on. Prelude scripts are not part of any line and do not loop or repeat. Each can be written in any available language. Prelude scripts run in order, top to bottom. Here are a few things that you might be interested in doing using prelude scripts:

- writing functions or code snippets for re-use.
- define global variables.
- share global information with other people playing with you.

You can forcefully re-evaluate a prelude script if you need to update it while playing. In sequencer mode, prelude scripts appear as tiles in a dedicated row at the top of the grid. Select one, then press **Enter** or **Escape** to open it in the editor panel. In classic mode, the prelude is a collapsible column on the left side.

## Frames

Each frame holds a **script**. Think of a **frame** as a _script container_ that also comes with a set of properties:

- **Duration**: how long one execution of the frame lasts, in beats (see [Timing](timing)). Default is 1 beat. Fractional values work: 0.25 for a sixteenth note, 4 for a full bar at 4/4.
- **Repetitions**: how many times the frame plays before the line advances. Each repetition gets the full duration window. A frame with duration 1 and 4 repetitions runs its script four times, one beat each, occupying 4 beats total. Default is 1.
- **Enabled**: toggles the frame on or off. Disabled frames are skipped during playback but their code is preserved.
- **Script**: the code, along with the language it uses (Bob, Boinx, Cagire, or BaLi). See [Languages](languages).

Frame properties are edited directly in the frame header. We try to minimize hidden state in the graphical interface.

## Lines

Lines have their own controls, accessible in the line header:

- **Loop**: the line restarts after reaching the end of its playback range. Otherwise, it plays through once and stops.
- **Trailing**: when the line loops, previous script executions keep running alongside the new iteration. Otherwise, they are stopped when the line restarts. Trailing lets sounds ring out naturally across loop boundaries.
- **Manual**: whether the playback is automatic or manually scheduled. Useful to store code that you are going to trigger from another line.
- **Speed**: multiplier on the line's tempo. 2.0 for double time, 0.5 for half. One line at normal speed, another at half: polymetric structures emerge. See [Timing](timing) for details.
- **Start frame / End frame**: restricts playback to a range within the line. Useful for looping a section while building the next one.

## Execution modes

The execution mode controls how lines synchronize when the scene starts or restarts. Change it from the transport bar:

- **Free**: lines start immediately and loop at their own pace. Each line is independent.
- **AtQuantum**: lines wait for the next quantum boundary (bar line) before starting, so parts land on the downbeat.
- **LongestLine**: all lines wait for the longest one to finish its cycle before restarting. The scene loops as a single unit.

## Modal interaction

The scene view switches between two modes depending on how you interact with it:

- **Navigation mode** (default): arrow keys and vim keys move between frames and lines, single-key shortcuts operate on frames, and no typing goes to any editor.
- **Edit mode**: entered by pressing Enter on a frame in sequencer mode, or Enter / `i` in classic mode. The code editor receives focus and all typing goes to the editor. Clicking inside a code editor also enters Edit mode. In sequencer mode, **Escape** switches back to the sequencer panel for the current selection.

All shortcuts below use `Cmd` on macOS and `Ctrl` on other platforms.

## Keyboard shortcuts

### Moving around

In sequencer mode, Left/Right moves between frames (steps in a row) and Up/Down moves between lines (tracks). In classic mode the axes are swapped: Up/Down moves between frames within a line, Left/Right moves between lines. Vim keys (h/j/k/l) follow the same mapping as the arrow keys.

- **Arrow keys** or **h/j/k/l**: move cursor
- **Shift + arrow**: extend selection within the same line
- **Cmd+A**: select all frames in the current line
- **Escape**: in classic mode, exit focus mode, then clear selection, then clear cursor (one step per press)

### Entering and exiting edit mode

- **Enter**: enter edit mode (focus the code editor for the selected frame)
- In sequencer mode, when an inline duration or repetitions field is open, **Enter** confirms that field instead of opening the code editor
- **Escape**: in sequencer mode, toggle between the editor panel and the sequencer panel for the current selection
- **F**: toggle focus mode (full-screen editor for the selected frame)

### Frame operations

- **I** in sequencer mode, or **Shift+I** in classic mode: insert an empty frame after the cursor
- **Cmd+Shift+I**: insert an empty frame before the cursor
- **Cmd+D**: duplicate selected frame(s) and place after
- **Delete** or **Backspace**: delete selected frame(s)
- **Shift+J**: move selected frame(s) to a later position
- **Shift+K**: move selected frame(s) to an earlier position
- **E**: toggle enabled/disabled on the selected frame
- **T**: edit the selected frame duration inline in the sequencer grid
- **Y**: edit the selected frame repetitions inline in the sequencer grid
- **X**: clear the content of selected frame(s) (empties the script, keeps the frame)
- **P**: preview the selected frame (run it once as a snippet)

### Line operations

- **O**: add a new line below the current one
- **Shift+O**: add a new line above the current one
- **Cmd+Shift+D**: duplicate the entire current line
- **Cmd+Delete**: delete the entire current line
- **Alt+H**: move line left (swap with previous)
- **Alt+L**: move line right (swap with next)
- **Shift+E**: enable or disable the whole current line; if every frame is disabled it enables all frames, otherwise it disables only the enabled ones
- **Shift+S**: focus the line speed control in sequencer mode
- **R**: toggle looping
- **,** (comma): toggle trailing
- **M**: toggle manual

### Clipboard

- **Cmd+C**: copy selected frame(s)
- **Cmd+X**: cut selected frame(s)
- **Cmd+V**: paste after cursor

### Edit mode shortcuts

While the code editor has focus:

- **Escape**: in sequencer mode, switch back to the sequencer panel
- **Cmd+Enter**: evaluate script (send to server)
- **Cmd+L**: open language selector
- **Cmd+F**: search in editor
- **Ctrl+Space**: open code completion

Evaluating a script (`Cmd+Enter`) sends the code to the server for compilation and scheduling. The frame flashes white on success or red on error.

## Context menus

Right-click a frame to open its context menu: cut, copy, paste after, insert frame before, insert frame after, duplicate, move up, move down, toggle enabled, remove frame. Insert before and insert after only appear for single selections. Right-click a line header for line operations: insert line before, insert line after, duplicate line, move left, move right, toggle looping, toggle trailing, clear frame range, remove line.

There are a lot of actions that you can perform on the scene view and it is a bit difficult to fit them all in the interface. We have tried our best to make it very shallow, meaning that there is no menu diving. However, always try to right click on things, you might get surprises!

## Visual feedback

The currently playing frame shows a colored accent strip on its left edge. A progress fill tracks playback position within the frame's duration. Disabled frames remain visible but visually muted. Their code stays readable, they are simply skipped during playback. In a [multiplayer](multiplayer) session, colored bars indicate where other musicians' cursors are. Colors are derived from usernames and stay consistent across sessions.

## Saving and loading

Save and load scenes through the File menu (Cmd+S / Cmd+O). A scene file captures lines, frames, scripts, prelude, [variables](variables), and configuration. "Load at end" defers the load to the next downbeat, avoiding a mid-bar interruption. Recent files are listed in the File menu for quick access. Connecting to a server loads its current scene automatically.
