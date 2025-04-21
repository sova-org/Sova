The `Grid` (also called `Scene`) is the most central view of `BuboCoreTUI`. It is used to view and manipulate the musical scene currently playing. By using the scene, you can create different tracks (called `Lines`) and create / delete / organise musical patterns (called `Frames`).


## Organisation

```
┌───────────────────────────────────────────┐
│ ┌───────────┐                       Scene │
│ │ Line ---- │                             │
│ │┌─────────┐│                             │
│ ││  Frame  ││                             │
│ ││         ││                             │
│ │└─────────┘│                             │
│ │┌─────────┐│                             │
│ ││  Frame  ││                             │
│ ││         ││                             │
│ │└─────────┘│                             │
│ └───────────┘                             │
└───────────────────────────────────────────┘
```

- `Scene` - The scene represents all `scripts` currently playing. It is composed of one or more `lines`. Each line is itself composed of one or more `frames`. A scene is playing back musical content for a specific duration -- in beats -- before looping.
    - `Line` - A line is a linear sequence of `frames`, of arbitrary duration.
    - `Frame` - A frame is a single unit of execution. It is the smallest unit that can be manipulated in the grid. A `Frame` is a `Script` spanning over a given duration (in beats).

## Keybindings

### Navigation & Selection

*   `↑` / `↓` / `←` / `→` : Move Cursor
*   `Shift` + `↑` / `↓` / `←` / `→` : Select Multiple Frames
*   `Esc` : Reset Selection to Cursor (if multiple cells selected)
*   `PgUp` / `PgDn` : Scroll Grid View

### Frame Editing

*   `Enter` : Edit selected `Frame Script`
*   `Space` : Enable/Disable selected `Frame(s)`
*   `l` : Set `Frame Length` (opens input prompt)
*   `n` : Set `Frame Name` (opens input prompt)
*   `b` : Toggle `Line` Start Marker at Cursor
*   `e` : Toggle `Line` End Marker at Cursor

### Frame Manipulation

*   `i` : Insert `Frame` After cursor (opens duration input prompt)
*   `Delete` / `Backspace` : Delete Selected `Frame(s)`
*   `a` : Duplicate Selection (inserting *before* cursor column)
*   `d` : Duplicate Selection (inserting *after* cursor column)
*   `c` : Copy Selected `Frame(s)` under cursor
*   `p` : Paste Copied `Frame(s)` under cursor

### Line Manipulation

*   `Shift` + `A` : Add New Line
*   `Shift` + `D` : Remove Last Line

### General

*   `?` : Toggle a small popup reminding you of the keybindings