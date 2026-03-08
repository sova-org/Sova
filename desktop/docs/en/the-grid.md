# The Grid

The grid is where you work. Lines are columns, frames are rows. You write code
here, rearrange parts, and steer the music during a performance.

## Layout

Each column is a line. The header shows the line number and controls for
looping, trailing, and speed. Below the header, each cell is a frame showing
its name, duration, repetitions, and a preview of the code.

The currently playing frame is highlighted. In multiplayer, you see other
players' cursors on the cells they are editing.

## Navigation

Move around with arrow keys or the mouse.

- Arrow Up / Down -- move between frames in the current line
- Arrow Left / Right -- move between lines
- Click -- select a cell
- Shift + Click -- extend selection from anchor to clicked cell
- Shift + Arrow Up/Down -- extend selection vertically
- Double-click -- open the code editor for a frame
- Escape -- clear selection

## Editing frame properties

Select a cell, then press a key to edit a property inline:

- Enter or D -- duration
- R -- repetitions
- N -- name

Inside the edit field:

- Enter -- commit
- Tab -- commit and move to the next field
- Shift+Tab -- commit and move to the previous field
- Escape -- cancel

To edit the code, double-click the cell. The code editor opens with syntax
highlighting for the frame's language.

## Line controls

- S -- edit the line's speed factor
- L -- toggle looping
- T -- toggle trailing

Tab moves between the Start Frame and End Frame fields in the line header.

## Frame operations

- Delete / Backspace -- delete selected frame(s)
- Cmd+D -- duplicate selected frame(s)
- Cmd+C -- copy
- Cmd+X -- cut
- Cmd+V -- paste after cursor
- Alt+Up -- move selected frame(s) up
- Alt+Down -- move selected frame(s) down

## Line operations

- Cmd+Shift+D -- duplicate the current line
- Cmd+Delete -- remove the current line
- Alt+Left -- move line left
- Alt+Right -- move line right

## Selection

- Cmd+A -- select all frames in the current line
- Escape -- clear selection

Multi-select works with all operations: delete, duplicate, copy, cut, move.

## Context menu

Right-click on a cell for additional options: adding frames, inserting lines,
toggling panels, enabling or disabling frames.

## Workflow tips

Name your frames (N). A grid full of unnamed cells becomes unreadable fast.
Label your sections: "intro", "drop", "breakdown". During a performance you
need to find things at a glance.

Duplicate before you modify (Cmd+D). Copy a working frame, then change one
thing. This keeps the original intact and gives you a fallback if the edit
doesn't land.

Reorder on the fly (Alt+Up/Down). Mid-performance, you can shuffle the
sequence of frames in a line without stopping playback. Move a fill before
the drop, push a transition earlier.

Disable frames instead of deleting them. Right-click a cell and toggle it off.
The code stays visible but the frame is skipped during playback. Bring it back
when you need it.

Use start/end frame ranges to isolate a section. Set a line to loop over just
frames 2-4 while you build frame 5. When it's ready, widen the range.

Keep lines focused. One line per musical role: drums, bass, melody, effects.
It is easier to mute, solo, or rearrange a line when it does one thing.

See **The Scene** for how lines, frames, and execution modes work.
