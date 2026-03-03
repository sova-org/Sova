# The Grid

The scene grid is the main workspace in Sova. Lines are displayed as columns
and frames as rows within each column. You navigate, edit, and organize your
musical material here.

## Layout

Each column is a **line**. The column header shows the line number and controls
(looping, trailing, speed). Below the header, each cell is a **frame** — it
displays the frame's name (if any), duration, repetitions, and a preview of the
script code.

The currently playing frame is highlighted. If other players are editing a
frame, you'll see their cursor indicator on the cell.

## Navigation

| Key | Action |
|-----|--------|
| Arrow Up / Down | Move cursor between frames in the current line |
| Arrow Left / Right | Move cursor between lines (same frame index) |
| Click | Select a cell |
| Shift + Click | Extend selection from anchor to clicked cell |
| Shift + Arrow Up/Down | Extend selection vertically |
| Double-click | Open the step editor for a cell |
| Escape | Clear selection |

## Editing frame properties

With a cell selected, press a key to start editing a property inline:

| Key | Edit |
|-----|------|
| Enter or D | Duration |
| R | Repetitions |
| N | Name |

Inside an edit field:

| Key | Action |
|-----|--------|
| Enter | Commit the edit |
| Tab | Commit and move to the next field |
| Shift+Tab | Commit and move to the previous field |
| Escape | Cancel the edit |

To edit the **code** inside a frame, double-click the cell or press Enter when
the step editor is configured to open that way. The step editor is a full code
editor with syntax highlighting for the frame's language.

## Line controls

| Key | Action |
|-----|--------|
| S | Edit the line's speed factor |
| L | Toggle looping |
| T | Toggle trailing |

You can also adjust start frame and end frame from the line header. Tab moves
between Start Frame and End Frame fields.

## Frame operations

| Key | Action |
|-----|--------|
| Delete / Backspace | Delete selected frame(s) |
| Cmd+D | Duplicate selected frame(s) |
| Cmd+C | Copy selected frame(s) |
| Cmd+X | Cut selected frame(s) |
| Cmd+V | Paste frames after cursor |
| Alt+Up | Move selected frame(s) up |
| Alt+Down | Move selected frame(s) down |

## Line operations

| Key | Action |
|-----|--------|
| Cmd+Shift+D | Duplicate the current line |
| Cmd+Delete | Remove the current line |
| Alt+Left | Move line one position left |
| Alt+Right | Move line one position right |

## Selection

| Key | Action |
|-----|--------|
| Cmd+A | Select all frames in the current line |
| Escape | Clear selection |

You can select multiple frames and apply operations (delete, duplicate, copy,
cut, move) to all of them at once.

## Context menu

Right-click on a frame cell to access additional options: adding frames,
inserting lines, toggling panel visibility, and more.

## Tips

- Use **Name** (N) to label sections of your arrangement — it makes the grid
  much easier to read at a glance.
- **Duplicate** (Cmd+D) is the fastest way to build variations: copy a frame,
  then tweak the code.
- **Alt+Up/Down** lets you reorder frames on the fly during a performance.
- Disabled frames (toggle via context menu) stay visible but don't play — handy
  for keeping alternate ideas around.
