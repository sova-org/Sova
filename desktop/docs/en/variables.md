# Variables

Variables let you store and share data between scripts. Sova's variable system
is organized by **scope** — where the variable lives determines who can see it
and how long it persists.

## Scopes

- **Global** — Entire session. Visible to all scripts in the scene. Use for shared state, master parameters.
- **Line** — Line lifetime. Visible to all frames in that line. Use for per-track state, counters.
- **Frame** — Frame lifetime. Visible to the script in that frame. Use for per-cell state, iteration data.
- **Instance** — Single execution. Visible to one run of the script. Use for temporary registers, local work.

### Global variables

Global variables are shared across the entire scene. Any script in any line and
frame can read and write them. They persist as long as the session is running.

Use globals for values that multiple lines need to agree on: a root note, a
scale, a probability threshold, a global transposition.

### Line variables

Line variables belong to a specific line. All frames within that line can access
them, but scripts in other lines cannot. They persist across frame changes
within the line.

Use line variables for per-track state: a step counter that advances each time
the line loops, or a melody array that frames read from.

### Frame variables

Frame variables belong to a specific frame. They persist across repetitions of
that frame but reset when the line moves to the next frame.

Use frame variables for state that should survive repetitions but not leak into
other frames.

### Instance variables

Instance variables exist only during a single execution of a script. They are
created fresh each time the frame plays and discarded afterward. These are the
most local scope — essentially temporary registers.

In compiled languages, instance variables like `Instance("0")` and
`Instance("1")` serve as working registers for the VM.

## How scopes relate to the scene

The scope hierarchy mirrors the scene hierarchy:

```
Scene ──── Global variables
 └─ Line ──── Line variables
     └─ Frame ──── Frame variables
         └─ Execution ──── Instance variables
```

Data flows naturally: a global variable set in one line is immediately visible
in another. A line variable set in frame 1 is visible in frame 2 when the line
advances. Instance variables are isolated to one execution and vanish after.

## Built-in read-only values

Each language exposes certain built-in values that you can read but not write.
These come from the **Environment** scope and provide context about the current
execution:

- Current beat position
- Current tempo
- Random number generation
- Frame index, line index
- Iteration counter (how many times the current frame has repeated)

The exact names and access syntax vary by language — check each language's
reference for the full list.

## Tips

- Keep globals to a minimum. If only one line needs a value, use a line
  variable instead.
- Use frame variables for accumulators that reset naturally when the line
  advances to the next section.
- The variable system is the primary way scripts communicate. Two frames in
  different lines can coordinate by reading and writing the same global
  variable.
